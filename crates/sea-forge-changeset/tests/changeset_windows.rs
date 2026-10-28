//! Windows reparse-point and hard-link refusal. The Unix equivalents live in
//! `changeset.rs`. A check that cannot run (for example, symlink creation
//! without Developer Mode or elevation) is labeled `BLOCKED: <reason>` and
//! tallied when `SEA_FORGE_REQUIRE_LINK_PRIVILEGE` is unset. When that
//! variable is `1` (the Portability workflow sets it), the same condition
//! fails the test, so a blocked prerequisite can never count as a pass.
#![cfg(windows)]

use sea_forge_changeset::{
    capture_base, collect, export_base, validate, BaseIdentity, ChangeRules, Changeset, Finding,
    Validation,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const WHEN: &str = "2026-01-02T00:00:00Z";

/// Process-wide tally of scenarios that could not run because a link-creation
/// prerequisite is missing. A required lane (the REQUIRE variable set) must
/// not skip; elsewhere the `BLOCKED:` label keeps the skip honest (F15).
static BLOCKED_SCENARIOS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Reports a link-creation prerequisite that is missing. Fails under
/// `SEA_FORGE_REQUIRE_LINK_PRIVILEGE=1`; otherwise tallies the scenario and
/// labels it `BLOCKED:` so a blocked prerequisite can never read as a pass
/// (`no_blocked_scenarios_in_required_mode` guards the required lane).
fn blocked(reason: &str) {
    BLOCKED_SCENARIOS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if std::env::var("SEA_FORGE_REQUIRE_LINK_PRIVILEGE").as_deref() == Ok("1") {
        panic!("BLOCKED prerequisite, not a pass: {reason}");
    }
    eprintln!("BLOCKED: {reason}");
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .env_clear()
        .envs(std::env::var_os("PATH").map(|path| ("PATH", path)))
        .envs(std::env::var_os("SystemRoot").map(|root| ("SystemRoot", root)))
        .env("GIT_CONFIG_GLOBAL", "NUL")
        .env("GIT_CONFIG_SYSTEM", "NUL")
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z")
        .args(args)
        .output()
        .expect("git must be installed");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn write(dir: &Path, rel: &str, content: impl AsRef<[u8]>) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().expect("parent directory")).expect("create directories");
    fs::write(&path, content).expect("write file");
}

fn authoritative(root: &Path) -> PathBuf {
    git(root, &["init", "-q", "-b", "main", "seed"]);
    let seed = root.join("seed");
    write(&seed, "README.md", "hello\n");
    write(&seed, "src/app.py", "print('v1')\n");
    git(&seed, &["add", "-A"]);
    git(&seed, &["commit", "-q", "-m", "base"]);
    git(root, &["clone", "-q", "--bare", "seed", "auth.git"]);
    root.join("auth.git")
}

fn start(root: &Path) -> (PathBuf, BaseIdentity, PathBuf, PathBuf) {
    let repo = authoritative(root);
    let base = capture_base(&repo, "refs/heads/main").expect("capture base");
    let staging = root.join("staging.git");
    let work = root.join("work");
    export_base(&repo, &base, &staging, &work).expect("export base");
    (repo, base, staging, work)
}

fn has_class(findings: &[Finding], class: &str) -> bool {
    findings.iter().any(|f| f.class == class)
}

fn assert_refused(cs: &Changeset, findings: &[Finding]) {
    assert!(
        matches!(
            validate(cs, &ChangeRules::default(), findings),
            Validation::Reject(_)
        ),
        "{findings:?}"
    );
}

#[test]
fn symlink_in_result_is_refused() {
    let t = tempfile::tempdir().expect("tempdir");
    let (repo, base, staging, work) = start(t.path());
    if let Err(error) =
        std::os::windows::fs::symlink_file(work.join("README.md"), work.join("link.md"))
    {
        blocked(&format!(
            "symlink creation needs Developer Mode or elevation: {error}"
        ));
        return;
    }
    let (cs, findings) = collect(&repo, &base, &work, &staging, WHEN).expect("collect");
    assert!(has_class(&findings, "reparse_point"), "{findings:?}");
    assert_refused(&cs, &findings);
}

#[test]
fn junction_in_result_is_refused() {
    let t = tempfile::tempdir().expect("tempdir");
    let (repo, base, staging, work) = start(t.path());
    write(&work, "real/inner.txt", "inner\n");
    let status = Command::new("cmd")
        .args(["/c", "mklink", "/J"])
        .arg(work.join("junction"))
        .arg(work.join("real"))
        .output();
    match status {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            blocked(&format!(
                "mklink /J failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
            return;
        }
        Err(error) => {
            blocked(&format!("cmd could not run mklink: {error}"));
            return;
        }
    }
    let (cs, findings) = collect(&repo, &base, &work, &staging, WHEN).expect("collect");
    assert!(has_class(&findings, "reparse_point"), "{findings:?}");
    assert_refused(&cs, &findings);
}

#[test]
fn hard_link_in_result_is_refused() {
    let t = tempfile::tempdir().expect("tempdir");
    let (repo, base, staging, work) = start(t.path());
    fs::hard_link(work.join("README.md"), work.join("copy.md")).expect("hard link");
    let (cs, findings) = collect(&repo, &base, &work, &staging, WHEN).expect("collect");
    assert!(has_class(&findings, "hard_link"), "{findings:?}");
    assert_refused(&cs, &findings);
}

/// Required-lane guard (F15): blocked scenarios must not survive a lane that
/// requires the link-creation prerequisite.
#[test]
fn no_blocked_scenarios_in_required_mode() {
    let blocked_count = BLOCKED_SCENARIOS.load(std::sync::atomic::Ordering::SeqCst);
    assert!(
        !(blocked_count > 0
            && std::env::var("SEA_FORGE_REQUIRE_LINK_PRIVILEGE").as_deref() == Ok("1")),
        "required lane (SEA_FORGE_REQUIRE_LINK_PRIVILEGE=1) finished with \
         {blocked_count} BLOCKED scenario(s); required lanes cannot skip"
    );
}
