# SEC-05 PostgreSQL native enforcement evidence

This supplementary resource-side run executed the frozen `tests/security-v1/postgres_probe.py` against a real local PostgreSQL 18.1 cluster. The fixture created and removed its own temporary cluster, schema, and roles. No product code or frozen test files were changed.

## Provisioner pin

- Provisioner: pg0 v0.15.2, official release asset `pg0-linux-x86_64-gnu`.
- Official release metadata: `https://api.github.com/repos/vectorize-io/pg0/releases/tags/v0.15.2`.
- Release metadata JSON SHA-256: `e9b7727e6f1d01fafcb8a8afed5fd3a5fb639c8729e330f37ed688e2133d970e`.
- Official asset digest in that metadata: `sha256:06c4678ff7b461dba8ed1f95c5e202ead3b57b70f549a22c17a94985121c0bf5`.
- Downloaded binary SHA-256 independently matched: `06c4678ff7b461dba8ed1f95c5e202ead3b57b70f549a22c17a94985121c0bf5`.
- pg0 extracted bundled PostgreSQL 18.1.0 and pgvector 0.8.5. The probe observed server version `18.1`.

## Reproduction

Run from the repository root as a non-root user. The host must permit a local PostgreSQL process and Unix-domain sockets; this environment required the approved unsandboxed process permission for those operations. All files below live under `/tmp`.

```sh
set -eu
work=/tmp/sea-forge-pg0-v0.15.2
mkdir -p "$work/home" "$work/data" "$work/wrappers"
chmod 700 "$work/home" "$work/data"
curl --fail --silent --show-error --location \
  https://api.github.com/repos/vectorize-io/pg0/releases/tags/v0.15.2 \
  -o "$work/release.json"
curl --fail --silent --show-error --location \
  https://github.com/vectorize-io/pg0/releases/download/v0.15.2/pg0-linux-x86_64-gnu \
  -o "$work/pg0"
printf '%s  %s\n' \
  06c4678ff7b461dba8ed1f95c5e202ead3b57b70f549a22c17a94985121c0bf5 \
  "$work/pg0" | sha256sum --check
test "$(sha256sum "$work/release.json" | cut -d' ' -f1)" = \
  e9b7727e6f1d01fafcb8a8afed5fd3a5fb639c8729e330f37ed688e2133d970e
chmod 700 "$work/pg0"
"$work/pg0" --version

# pg0 extracts its bundled server on first start. It prints its generated
# bootstrap password and URI, so capture output without displaying it.
pw=$(python3 -c 'import secrets; print(secrets.token_urlsafe(32))')
HOME="$work/home" "$work/pg0" start --name bootstrap --port 55439 \
  --data-dir "$work/data" --username postgres --password "$pw" \
  --config 'listen_addresses=' >"$work/bootstrap.log" 2>&1
unset pw
HOME="$work/home" "$work/pg0" stop --name bootstrap --timeout 10 \
  >/dev/null 2>&1
rm -f "$work/bootstrap.log"
rm -rf "$work/data" "$work/home/.pg0/instances/bootstrap"

bindir="$work/home/.pg0/installation/18.1.0/bin"
libdir="$work/home/.pg0/installation/18.1.0/lib"
for name in psql initdb pg_ctl; do
  cat >"$work/wrappers/$name" <<EOF
#!/bin/sh
LD_LIBRARY_PATH='$libdir' exec '$bindir/$name' "\$@"
EOF
  chmod 700 "$work/wrappers/$name"
done

# The frozen fixture owns its generated cluster and keeps owner/agent database
# credentials separate. Capture machine-readable evidence in /tmp.
SEA_FORGE_SECURITY_PG_BINDIR="$work/wrappers" \
  python3 tests/security-v1/postgres_probe.py >"$work/postgres_probe.json"
cat "$work/postgres_probe.json"
```

The initial unprivileged bootstrap/probe attempts were blocked by this execution sandbox's socket restrictions. The successful bootstrap and probe ran with elevated local-process/socket permission; TCP listening was disabled for the bootstrap and the fixture's own cluster uses a private temporary Unix socket with TCP disabled. The temporary bootstrap output was removed after stopping the server. No bootstrap credential is part of this evidence. The extracted pg0 binary and PostgreSQL bundle remain reusable under `/tmp/sea-forge-pg0-v0.15.2`.

## Observed outcome

The exact frozen probe JSON is in [postgres_probe.json](postgres_probe.json). All eight destructive privilege-escalation attempts were actually submitted to PostgreSQL and rejected with SQLSTATE `42501`; each attempt also established a successful same-session metadata control. The reader role had no superuser, role-creation, database-creation, replication, or RLS-bypass attributes. The owner-side write positive control succeeded. Protected target rows remained 2, and the before/after row serialization SHA-256 was `3d74f767a718a7e53cfcc44cb95c5d5a889d8ac8c9874c6e0848ade47caca20d`. The fixture reported generated schema/role cleanup and local-cluster shutdown.

This evidence proves PostgreSQL-native role enforcement for this fixture only. It does not establish SEA Forge identity issuance, authorization mediation, or outbound network isolation.
