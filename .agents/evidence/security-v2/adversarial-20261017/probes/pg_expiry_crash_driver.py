# Host-side driver. Mints a short-lived session role, launches a hand-reproduced isolated workload that
# holds a PostgreSQL backend, SIGKILLs the launcher (simulating a server crash), waits past VALID UNTIL,
# then checks whether the orphaned backend survives and whether new logins are refused.
import os, sys, time, signal, subprocess, secrets, datetime
sys.path.insert(0, "/home/sprime01/projects/sea-rs/.agents/evidence/security-v2/adversarial-20261017/probes")
import launchlib as L
P = "/home/sprime01/.adv-probe-20261017"; PG = os.path.expanduser("~/.pg0/installation/18.1.0")
env_lib = {"LD_LIBRARY_PATH": PG + "/lib", "PATH": "/usr/bin:/bin"}
A = [PG + "/bin/psql", "-h", P + "/pgsock", "-U", "sea_admin", "-X", "-q", "-At", "-d", "advtest"]
def admin(sql):
    return subprocess.run(A + ["-c", sql], capture_output=True, text=True, env=env_lib)
pw = secrets.token_hex(16); role = "sfw_100001_" + secrets.token_hex(3)
exp = (datetime.datetime.now(datetime.timezone.utc) + datetime.timedelta(seconds=15)).strftime("%Y-%m-%dT%H:%M:%SZ")
r = admin(f"CREATE ROLE {role} LOGIN NOSUPERUSER CONNECTION LIMIT 4 PASSWORD '{pw}' VALID UNTIL '{exp}'; GRANT grp_read TO {role} WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;")
print("mint", r.returncode, r.stderr.strip()[:120], "role", role, "valid_until", exp)
os.makedirs(P + "/ws1", exist_ok=True)
wl_env = dict(env_lib, PG_BIN=PG + "/bin", PGHOST=P + "/pgsock", PGPORT="5432", PGDATABASE="advtest", PGUSER=role, PGPASSWORD=pw)
cmd = L.launch_cmd(100001, 100001, L.HELPER, P + "/ws1", P + "/art1",
                   ["/bin/sh", "-c", PG + "/bin/psql -X -q -At -c 'SELECT pg_sleep(120)' >/dev/null 2>&1; echo done"], True)
p = subprocess.Popen(cmd, cwd=P + "/ws1", env=wl_env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
time.sleep(3)
print("backends_before_crash", admin(f"SELECT count(*) FROM pg_stat_activity WHERE usename = '{role}'").stdout.strip())
os.kill(os.getpid(), signal.SIGKILL)  # launcher dies; unshare parent is orphaned, not killed
