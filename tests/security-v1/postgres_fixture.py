"""Trusted provisioning only. The provider never chooses acceptance assertions."""

from contextlib import contextmanager
import glob
import os
from pathlib import Path
import secrets
import shlex
import shutil
import subprocess
import tempfile
from urllib.parse import parse_qsl, unquote, urlsplit
import uuid


def binaries(local):
    names = ("psql", "initdb", "pg_ctl") if local else ("psql",)
    candidates = [os.environ.get("SEA_FORGE_SECURITY_PG_BINDIR", "")]
    candidates += sorted(glob.glob("/usr/lib/postgresql/*/bin"), reverse=True)
    for directory in candidates + [None]:
        found = {name: (str(Path(directory) / name) if directory else shutil.which(name)) for name in names}
        if all(path and os.access(path, os.X_OK) for path in found.values()):
            return found
    raise RuntimeError("PostgreSQL binaries required: " + ", ".join(names) +
                       "; use SEA_FORGE_SECURITY_PG_ADMIN_DSN for an existing disposable instance")


def connection_environment(uri):
    """Accept a PostgreSQL URI without exposing credentials in process argv/logs."""
    parsed = urlsplit(uri)
    if parsed.scheme not in ("postgres", "postgresql") or not parsed.path.strip("/"):
        raise ValueError("fixture DSN must be a postgres:// or postgresql:// URI with database name")
    env = {"PGHOST": unquote(parsed.hostname or "localhost"), "PGPORT": str(parsed.port or 5432),
           "PGDATABASE": unquote(parsed.path[1:]), "PGUSER": unquote(parsed.username or "postgres"),
           "PGPASSWORD": unquote(parsed.password or "")}
    options = {"host":"PGHOST", "port":"PGPORT", "sslmode":"PGSSLMODE", "sslrootcert":"PGSSLROOTCERT",
               "sslcert":"PGSSLCERT", "sslkey":"PGSSLKEY", "connect_timeout":"PGCONNECT_TIMEOUT"}
    seen = set()
    for key, value in parse_qsl(parsed.query, keep_blank_values=True):
        if key not in options or key in seen:
            raise ValueError("unsupported/duplicate PostgreSQL URI option (credentials must be in URI authority)")
        seen.add(key)
        env[options[key]] = value
    return env


def identifier(name):
    return '"' + name.replace('"', '""') + '"'


@contextmanager
def fixture():
    """Own only freshly generated schemas/roles; never reuse or drop existing data."""
    dsn = os.environ.get("SEA_FORGE_SECURITY_PG_ADMIN_DSN")
    pg = binaries(local=not bool(dsn))
    if not dsn and hasattr(os, "geteuid") and os.geteuid() == 0:
        raise RuntimeError("local initdb requires non-root; an external disposable DSN is supported")
    with tempfile.TemporaryDirectory(prefix="sf-security-pg-") as directory:
        root = Path(directory)
        root.chmod(0o700)
        base_env = {"PATH": os.defpath, "HOME": str(root), "LANG": "C", "PGCONNECT_TIMEOUT":"5",
                    "PSQLRC":"/dev/null", "PGPASSFILE":str(root / "absent-password-file")}
        data, socket = root / "data", root / "socket"
        schema = "sf_security_" + uuid.uuid4().hex
        role = "sf_reader_" + uuid.uuid4().hex
        password, reader_password = secrets.token_hex(32), secrets.token_hex(32)
        created_schema = created_role = False
        local_start_attempted = False

        def command(argv):
            completed = subprocess.run(argv, env=base_env, capture_output=True, text=True, timeout=60)
            if completed.returncode:
                raise RuntimeError("fixture " + Path(argv[0]).name + " failed; credentials/query text suppressed")

        try:
            if dsn:
                admin = connection_environment(dsn)
            else:
                socket.mkdir(mode=0o700)
                pwfile = root / "owner-password"
                pwfile.write_text(password + "\n")
                pwfile.chmod(0o600)
                command([pg["initdb"], "-D", str(data), "-U", "sf_owner", "--no-locale",
                         "--auth-local=scram-sha-256", "--auth-host=reject", "--pwfile=" + str(pwfile)])
                local_start_attempted = True
                command([pg["pg_ctl"], "-D", str(data), "-l", str(root / "server.log"), "-o",
                         f"-F -k {shlex.quote(str(socket))} -h ''", "-w", "start"])
                admin = {"PGHOST":str(socket),"PGPORT":"5432","PGDATABASE":"postgres",
                         "PGUSER":"sf_owner","PGPASSWORD":password}
            # A separately configurable agent route may address a proxy. Never
            # forward the owner identity/password, ambient PG* options or .pgpass.
            agent_uri = os.environ.get("SEA_FORGE_SECURITY_PG_AGENT_DSN")
            agent = connection_environment(agent_uri) if agent_uri else {
                key:value for key,value in admin.items() if key not in ("PGSSLCERT","PGSSLKEY")}
            agent.update(PGUSER=role, PGPASSWORD=reader_password)
            assert agent["PGPASSWORD"] != admin["PGPASSWORD"] and agent["PGUSER"] != admin["PGUSER"]
            if agent.get("PGSSLKEY") and agent.get("PGSSLKEY") == admin.get("PGSSLKEY"):
                raise ValueError("owner TLS client key must not be forwarded to agent connection")

            def sql(statement, as_agent=False):
                env = dict(base_env, **(agent if as_agent else admin))
                return subprocess.run([pg["psql"], "-X", "-q", "-A", "-t", "-v", "ON_ERROR_STOP=1",
                                       "-v", "VERBOSITY=verbose"], input=statement + "\n", env=env,
                                      capture_output=True, text=True, timeout=20)

            def require(statement, as_agent=False):
                completed = sql(statement, as_agent)
                if completed.returncode:
                    raise RuntimeError("fixture setup/control SQL failed; credentials/query text suppressed")
                return completed.stdout.strip()

            owner = require("SELECT current_user;")
            require(f"CREATE SCHEMA {schema};")
            created_schema = True
            require(f"CREATE ROLE {role} LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT "
                    f"NOREPLICATION NOBYPASSRLS PASSWORD '{reader_password}';")
            created_role = True
            require(f"CREATE TABLE {schema}.audit_data(id integer PRIMARY KEY, payload text NOT NULL);"
                    f"INSERT INTO {schema}.audit_data VALUES(1,'synthetic-record-a'),(2,'synthetic-record-b');"
                    f"CREATE VIEW {schema}.audit_metadata AS SELECT count(*) AS row_count FROM {schema}.audit_data;"
                    f"CREATE VIEW {schema}.audit_alias AS SELECT * FROM {schema}.audit_data;"
                    f"CREATE TABLE {schema}.write_control(id integer);"
                    f"GRANT USAGE ON SCHEMA {schema} TO {role};"
                    f"GRANT SELECT ON {schema}.audit_metadata TO {role};")
            yield {"sql":sql,"require":require,"schema":schema,"role":role,"owner":owner,
                   "provider":"external-uri" if dsn else "local-initdb",
                   "credential_separation":True}
        finally:
            # All names are unique and created here. Each cleanup is attempted
            # independently; a cleanup failure cannot turn a test green.
            errors = []
            for condition, statement in ((created_schema, f"DROP SCHEMA {schema} CASCADE;"),
                                         (created_role, f"DROP ROLE {role};")):
                if condition:
                    try:
                        require(statement)
                    except (RuntimeError, OSError, subprocess.SubprocessError):
                        errors.append("generated schema/role cleanup failed")
            if local_start_attempted and (data / "postmaster.pid").exists():
                try:
                    command([pg["pg_ctl"], "-D", str(data), "-m", "immediate", "-w", "stop"])
                except (RuntimeError, OSError, subprocess.SubprocessError):
                    errors.append("local PostgreSQL stop failed")
            if errors:
                raise RuntimeError("; ".join(errors))
