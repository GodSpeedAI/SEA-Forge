//! Disposable PostgreSQL cluster for security tests: peer-authenticated
//! administrator for the launcher's OS user, SCRAM for everything else, no
//! TCP listener, and a protected `audit_log` table with reader/writer roles.
#![allow(dead_code)]
use sea_forge_sandbox::workload::{sibling_helper, WorkloadIdentityPool};
use sea_forge_server::postgres::{PostgresBroker, PostgresResourceConfig};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Cluster {
    _base: tempfile::TempDir,
    pub base: PathBuf,
    pub install: PathBuf,
    pub data: PathBuf,
    pub config: PostgresResourceConfig,
}

impl Drop for Cluster {
    fn drop(&mut self) {
        let _ = self
            .tool("pg_ctl")
            .args(["-D"])
            .arg(&self.data)
            .args(["-m", "immediate", "stop"])
            .output();
    }
}

impl Cluster {
    pub fn tool(&self, name: &str) -> Command {
        let mut command = Command::new(self.install.join("bin").join(name));
        command
            .env_clear()
            .env("LD_LIBRARY_PATH", self.install.join("lib"));
        command
    }
}

pub type Prerequisites = (PathBuf, WorkloadIdentityPool, PathBuf, tempfile::TempDir);

pub fn prerequisites() -> Option<Prerequisites> {
    let skip = |why: String| {
        assert!(
            std::env::var_os("SEA_FORGE_REQUIRE_POSTGRES").is_none(),
            "PostgreSQL qualification required but unavailable: {why}"
        );
        eprintln!("NOT-EXECUTABLE: {why}");
        None
    };
    let Some(install) = std::env::var_os("SEA_FORGE_SECURITY_PG_INSTALL").map(PathBuf::from) else {
        return skip("SEA_FORGE_SECURITY_PG_INSTALL unset".into());
    };
    let state = tempfile::tempdir().unwrap();
    let pool = match WorkloadIdentityPool::from_host() {
        Ok(pool) => pool,
        Err(error) => return skip(error.to_string()),
    };
    let helper = match sibling_helper() {
        Ok(helper) => helper,
        Err(error) => return skip(error.to_string()),
    };
    Some((install, pool, helper, state))
}

pub fn start_cluster(install: &Path) -> Cluster {
    let base = tempfile::tempdir().unwrap();
    fs::set_permissions(base.path(), fs::Permissions::from_mode(0o711)).unwrap();
    let data = base.path().join("data");
    let socket_dir = base.path().join("sock");
    fs::create_dir(&socket_dir).unwrap();
    fs::set_permissions(&socket_dir, fs::Permissions::from_mode(0o755)).unwrap();
    static NEXT_PORT: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(0);
    let port = 50_000
        + (std::process::id() % 5_000) as u16 * 2
        + NEXT_PORT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let mut cluster = Cluster {
        base: base.path().to_path_buf(),
        _base: base,
        install: install.to_path_buf(),
        data: data.clone(),
        config: PostgresResourceConfig {
            name: "app-db".into(),
            psql_bindir: install.join("bin"),
            lib_dir: Some(install.join("lib")),
            socket_dir: socket_dir.clone(),
            port,
            database: "postgres".into(),
            admin_role: "sea_admin".into(),
            scopes: BTreeMap::from([
                ("audit-read".into(), "audit_reader".into()),
                ("audit-write".into(), "audit_writer".into()),
            ]),
            session_ttl_secs: 600,
            offer: vec![],
            exposure: BTreeMap::new(),
        },
    };
    let init = cluster
        .tool("initdb")
        .arg("-D")
        .arg(&data)
        .args([
            "-U",
            "sea_admin",
            "--auth-local=peer",
            "--auth-host=reject",
            "-E",
            "UTF8",
        ])
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    let os_user =
        String::from_utf8(Command::new("id").arg("-un").output().unwrap().stdout).unwrap();
    fs::write(
        data.join("pg_hba.conf"),
        "local all sea_admin peer map=launcher\nlocal all all scram-sha-256\n",
    )
    .unwrap();
    fs::write(
        data.join("pg_ident.conf"),
        format!("launcher {} sea_admin\n", os_user.trim()),
    )
    .unwrap();
    let start = cluster
        .tool("pg_ctl")
        .arg("-D")
        .arg(&data)
        .arg("-l")
        .arg(cluster.base.join("server.log"))
        .arg("-o")
        .arg(format!(
            "-k {} -p {port} -c listen_addresses='' -c password_encryption=scram-sha-256",
            socket_dir.display()
        ))
        .args(["-w", "start"])
        .output()
        .unwrap();
    assert!(
        start.status.success(),
        "{}",
        String::from_utf8_lossy(&start.stderr)
    );
    cluster.config.port = port;
    let broker = PostgresBroker::new(cluster.config.clone()).unwrap();
    broker
        .admin_sql(
            "CREATE TABLE audit_log (id int PRIMARY KEY, event text NOT NULL); \
             INSERT INTO audit_log VALUES (1,'login'),(2,'export'),(3,'delete-request'); \
             CREATE ROLE audit_reader NOLOGIN; CREATE ROLE audit_writer NOLOGIN; \
             GRANT USAGE ON SCHEMA public TO audit_reader, audit_writer; \
             GRANT SELECT ON audit_log TO audit_reader; \
             GRANT SELECT, INSERT, UPDATE, DELETE ON audit_log TO audit_writer;",
        )
        .unwrap();
    cluster
}

pub fn digest(broker: &PostgresBroker) -> String {
    broker
        .admin_sql("SELECT md5(string_agg(id || ':' || event, ',' ORDER BY id)) FROM audit_log")
        .unwrap()
}
