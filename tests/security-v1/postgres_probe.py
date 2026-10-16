"""Provider-neutral native PostgreSQL acceptance assertions. No mocked boundary."""

import hashlib
import re
import subprocess

from postgres_fixture import fixture, identifier


def probe():
    result = {"case_id":"SEC-05.postgres_native", "attempted":False,
              "trusted_identity":"privileged fixture owner", "claimed_identity":"generated metadata-only role",
              "action_id":"native-database-mutation", "decision":"PostgreSQL native privileges",
              "positive_control":False, "cleanup":"not started", "observations":[]}
    breached = False
    try:
        with fixture() as pg:
            sql, require, schema, role = pg["sql"], pg["require"], pg["schema"], pg["role"]
            result.update(provider=pg["provider"], credential_separation=pg["credential_separation"])
            result["claimed_identity"] = role

            def snapshot():
                if require(f"SELECT to_regclass('{schema}.audit_data') IS NOT NULL;") != "t":
                    return {"exists":False,"rows":0,"sha256":None}
                rows = require(f"SELECT id,payload FROM {schema}.audit_data ORDER BY id;")
                return {"exists":True, "rows":int(require(f"SELECT count(*) FROM {schema}.audit_data;")),
                        "sha256":hashlib.sha256(rows.encode()).hexdigest()}

            result["before"] = snapshot()
            result["server_version"] = require("SHOW server_version;")
            result["effective_role"] = require("SELECT current_user;", True)
            result["role_attributes"] = require(
                "SELECT rolsuper,rolcreaterole,rolcreatedb,rolreplication,rolbypassrls "
                "FROM pg_roles WHERE rolname=current_user;", True)
            result["native_select_control"] = require(f"SELECT row_count FROM {schema}.audit_metadata;", True)
            result["catalog_control"] = require(
                f"SELECT count(*) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace "
                f"WHERE n.nspname='{schema}' AND c.relname='audit_data';", True)
            require(f"INSERT INTO {schema}.write_control VALUES(1);")
            owner_control = require(f"SELECT count(*) FROM {schema}.write_control;") == "1"
            attacks = [
                f"DELETE FROM {schema}.audit_data;",
                f"TRUNCATE {schema}.audit_data;",
                f"DROP TABLE {schema}.audit_data CASCADE;",
                f"SET ROLE {identifier(pg['owner'])};",
                f"GRANT {identifier(pg['owner'])} TO {role};",
                f"SET default_transaction_read_only=off; DELETE FROM {schema}.audit_data;",
                f"DELETE FROM {schema}.audit_alias;",
                f"SET search_path={schema}; DELETE FROM audit_alias;"
            ]
            all_refused = True
            for statement in attacks:
                # Each actual attack and its metadata positive control share one
                # persistent PostgreSQL connection. The backend PID is preserved.
                prefix = f"SELECT current_user; SELECT row_count FROM {schema}.audit_metadata; SELECT pg_backend_pid();"
                result["attempted"] = True
                completed = sql(prefix + statement, True)
                lines = completed.stdout.strip().splitlines()
                control = len(lines) >= 3 and lines[:2] == [role,"2"] and lines[2].isdigit()
                match = re.search(r"(?:ERROR|FATAL):\s+([0-9A-Z]{5}):", completed.stderr)
                state = match.group(1) if match else None
                refused = completed.returncode != 0 and state in ("42501","25006") and control
                # A completed destructive statement or changed resource is a
                # sticky breach, even if a later observation/cleanup fails.
                breached |= completed.returncode == 0
                all_refused &= refused
                result["observations"].append({"sql":statement,"attempted":True,
                    "exit_code":completed.returncode,"sqlstate":state,"native_refusal":refused,
                    "same_session_control":control,"backend_pid":lines[2] if control else None})
            result["after"] = snapshot()
            breached |= result["before"] != result["after"]
            result["positive_control"] = (result["native_select_control"] == "2" and
                result["catalog_control"] == "1" and owner_control and result["effective_role"] == role and
                result["role_attributes"] == "f|f|f|f|f" and
                all(row["same_session_control"] for row in result["observations"]))
            result["classification"] = ("SECURITY-RED" if breached else
                "PASS" if all_refused and result["positive_control"] else "INFRASTRUCTURE-BLOCKED")
            if result["classification"] == "INFRASTRUCTURE-BLOCKED":
                result["reason"] = "native permission result or same-session positive control could not be established"
            result["scope_limit"] = "native role/resource boundary only; SEA Forge identity issuance and network isolation are separate obligations"
        result["cleanup"] = "generated schemas/roles removed; local cluster stopped if provisioned"
    except (OSError, RuntimeError, ValueError, subprocess.SubprocessError) as error:
        result["classification"] = "SECURITY-RED" if breached else "INFRASTRUCTURE-BLOCKED"
        result["reason"] = type(error).__name__ + ": " + str(error)[:300]
        result["cleanup"] = "cleanup attempted; inspect reason if incomplete"
    return result


if __name__ == "__main__":
    import json
    import sys
    observed = probe()
    print(json.dumps(observed, indent=2))
    sys.exit(0 if observed["classification"] == "PASS" else 1)
