//! Windows reparse-point and hard-link refusal. The Unix equivalents live in
//! `changeset.rs`. A check that cannot run (for example, symlink creation
//! without Developer Mode or elevation) prints `SKIPPED: <reason>` and returns;
//! it never passes silently.
#![cfg(windows)]

use sea_forge_changeset::{
    capture_base, collect, export_base, validate, BaseIdentity, ChangeRules, Changeset, Finding,
    Validation,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const WHEN: &str = "2026-01-02T00:00:00Z";

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
        println!("SKIPPED: symlink creation needs Developer Mode or elevation: {error}");
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
            println!(
                "SKIPPED: mklink /J failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
            return;
        }
        Err(error) => {
            println!("SKIPPED: cmd could not run mklink: {error}");
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
