#![cfg(unix)]

use sea_forge_changeset::{
    capture_base, collect, export_base, promote, validate, BaseIdentity, ChangeEntry, ChangeRules,
    ChangeStatus, Changeset, ChangesetError, Finding, Promotion, PromotionAuthorization,
    ProtectedDecision, Validation,
};
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;

const WHEN: &str = "2026-01-02T00:00:00Z";

fn git(dir: &Path, args: &[&str]) -> String {
    // Start from an empty environment: a Git hook exports GIT_DIR and
    // GIT_INDEX_FILE, and an inherited value would aim these fixture commands
    // at the repository running the tests instead of the temporary one.
    let out = Command::new("git")
        .current_dir(dir)
        .env_clear()
        .envs(std::env::var_os("PATH").map(|path| ("PATH", path)))
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
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

fn chmod(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("chmod");
}

/// The fixture tree shared by every authoritative repository in these tests.
fn populate(dir: &Path) {
    write(dir, "README.md", "hello\n");
    write(dir, "src/app.py", "print('v1')\n");
    write(dir, "pyproject.toml", "[project]\nname='x'\n");
    write(dir, "uv.lock", "lock-v1\n");
    write(dir, "bin/run.sh", "#!/bin/sh\necho hi\n");
    chmod(&dir.join("bin/run.sh"), 0o755);
}

/// A bare authoritative repository at `root/auth.git` with one base commit.
fn authoritative(root: &Path) -> PathBuf {
    git(root, &["init", "-q", "-b", "main", "seed"]);
    let seed = root.join("seed");
    populate(&seed);
    git(&seed, &["add", "-A"]);
    git(&seed, &["commit", "-q", "-m", "base"]);
    git(root, &["clone", "-q", "--bare", "seed", "auth.git"]);
    root.join("auth.git")
}

fn list_files(root: &Path, dir: &Path, out: &mut Vec<(String, u64)>) {
    for entry in fs::read_dir(dir).expect("read directory") {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            list_files(root, &path, out);
        } else {
            let relative = path
                .strip_prefix(root)
                .expect("under root")
                .display()
                .to_string();
            out.push((relative, fs::metadata(&path).expect("metadata").len()));
        }
    }
}

/// Deterministic fingerprint of the authoritative repository's durable state.
fn snapshot(repo: &Path) -> String {
    let mut out = format!(
        "{}\n{}\n",
        git(repo, &["for-each-ref"]),
        git(repo, &["count-objects", "-v"])
    );
    let mut files = Vec::new();
    list_files(repo, repo, &mut files);
    files.sort();
    for (path, len) in files {
        out.push_str(&format!("{path} {len}\n"));
    }
    out
}

fn head(repo: &Path) -> String {
    git(repo, &["rev-parse", "refs/heads/main"])
}

fn start(root: &Path) -> (PathBuf, BaseIdentity, PathBuf, PathBuf) {
    let repo = authoritative(root);
    let base = capture_base(&repo, "refs/heads/main").expect("capture base");
    let staging = root.join("staging.git");
    let work = root.join("work");
    export_base(&repo, &base, &staging, &work).expect("export base");
    (repo, base, staging, work)
}

fn authorize(cs: &Changeset) -> PromotionAuthorization {
    PromotionAuthorization {
        decision_id: "dec-1".into(),
        reference: cs.base.reference.clone(),
        expected_base: cs.base.commit.clone(),
        changeset_digest: cs.digest.clone(),
    }
}

fn err_class<T: std::fmt::Debug>(result: Result<T, ChangesetError>) -> &'static str {
    result.expect_err("expected an error").class
}

fn has_class(findings: &[Finding], class: &str) -> bool {
    findings.iter().any(|f| f.class == class)
}

/// Modify `src/app.py`, delete `README.md`, add `docs/new.md`.
fn edit_basic(work: &Path) {
    write(work, "src/app.py", "print('v2')\n");
    fs::remove_file(work.join("README.md")).expect("remove README.md");
    write(work, "docs/new.md", "new\n");
}

#[test]
fn export_is_a_plain_tree_without_git_metadata() {
    let t = tempfile::tempdir().expect("tempdir");
    let (_repo, _base, _staging, work) = start(t.path());

    assert_eq!(
        fs::read_to_string(work.join("README.md")).expect("read"),
        "hello\n"
    );
    let mode = fs::metadata(work.join("bin/run.sh"))
        .expect("metadata")
        .permissions()
        .mode();
    assert!((mode & 0o111) != 0, "run.sh must stay executable");
    assert!(!work.join(".git").exists());
}

#[test]
fn capture_base_requires_a_full_ref_name() {
    let t = tempfile::tempdir().expect("tempdir");
    let repo = authoritative(t.path());

    assert_eq!(err_class(capture_base(&repo, "main")), "invalid_reference");
    assert_eq!(
        err_class(capture_base(&repo, "refs/heads/nope")),
        "unknown_reference"
    );
}

#[test]
fn export_refuses_a_non_empty_destination() {
    let t = tempfile::tempdir().expect("tempdir");
    let (repo, base, staging, _work) = start(t.path());
    let work2 = t.path().join("work2");
    write(&work2, "stale.txt", "left over\n");

    assert_eq!(
        err_class(export_base(&repo, &base, &staging, &work2)),
        "workspace_not_empty"
    );
}

#[test]
fn collects_additions_modifications_deletions_and_binary_changes() {
    let t = tempfile::tempdir().expect("tempdir");
    let (repo, base, staging, work) = start(t.path());
    edit_basic(&work);
    let blob: Vec<u8> = (0..3).flat_map(|_| 0u8..=255).collect();
    write(&work, "assets/blob.bin", blob);

    let (cs, findings) = collect(&repo, &base, &work, &staging, WHEN).expect("collect");

    let paths: Vec<&str> = cs.entries.iter().map(|e| e.path.as_str()).collect();
    assert_eq!(
        paths,
        ["README.md", "assets/blob.bin", "docs/new.md", "src/app.py"]
    );
    let statuses: Vec<ChangeStatus> = cs.entries.iter().map(|e| e.status).collect();
    assert_eq!(
        statuses,
        [
            ChangeStatus::Deleted,
            ChangeStatus::Added,
            ChangeStatus::Added,
            ChangeStatus::Modified
        ]
    );
    let is_binary = |path: &str| {
        cs.entries
            .iter()
            .find(|e| e.path == path)
            .expect("entry present")
            .binary
    };
    assert!(is_binary("assets/blob.bin"));
    assert!(!is_binary("src/app.py"));
    assert_eq!(cs.digest, cs.compute_digest());
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
    assert_eq!(
        validate(&cs, &ChangeRules::default(), &findings),
        Validation::Accept
    );
}

#[test]
fn collect_is_deterministic() {
    let run = |root: &Path| {
        let (repo, base, staging, work) = start(root);
        edit_basic(&work);
        collect(&repo, &base, &work, &staging, WHEN)
            .expect("collect")
            .0
    };
    let t1 = tempfile::tempdir().expect("tempdir");
    let t2 = tempfile::tempdir().expect("tempdir");
    let first = run(t1.path());
    let second = run(t2.path());

    assert_eq!(first.proposed_tree, second.proposed_tree);
    assert_eq!(first.proposed_commit, second.proposed_commit);
    assert_eq!(first.digest, second.digest);
}

#[test]
fn authorized_promotion_moves_the_ref_atomically() {
    let t = tempfile::tempdir().expect("tempdir");
    let (repo, base, staging, work) = start(t.path());
    write(&work, "src/app.py", "print('v2')\n");
    let (cs, findings) = collect(&repo, &base, &work, &staging, WHEN).expect("collect");
    assert_eq!(
        validate(&cs, &ChangeRules::default(), &findings),
        Validation::Accept
    );

    let promotion: Promotion = promote(&repo, &cs, &staging, &authorize(&cs)).expect("promote");

    assert_eq!(promotion.previous, base.commit);
    assert_eq!(promotion.current, cs.proposed_commit);
    assert_eq!(head(&repo), cs.proposed_commit);
    assert_eq!(
        git(&repo, &["show", "refs/heads/main:src/app.py"]),
        "print('v2')"
    );
    assert_eq!(git(&repo, &["rev-parse", "refs/heads/main^"]), base.commit);
}

#[test]
fn rejected_changeset_leaves_the_authoritative_repository_byte_identical() {
    let t = tempfile::tempdir().expect("tempdir");
    let (repo, base, staging, work) = start(t.path());
    let before = snapshot(&repo);

    write(&work, "src/app.py", "print('v2')\n");
    symlink("/etc/passwd", work.join("link")).expect("symlink");
    let (cs, findings) = collect(&repo, &base, &work, &staging, WHEN).expect("collect");

    match validate(&cs, &ChangeRules::default(), &findings) {
        Validation::Reject(found) => assert!(has_class(&found, "symlink"), "{found:?}"),
        other => panic!("expected rejection, got {other:?}"),
    }
    assert_eq!(snapshot(&repo), before);
    // The rejected proposal is kept in staging as evidence.
    assert_eq!(
        git(&staging, &["cat-file", "-t", cs.proposed_commit.as_str()]),
        "commit"
    );
}

#[test]
fn protected_paths_escalate_or_deny() {
    let t = tempfile::tempdir().expect("tempdir");
    let (repo, base, staging, work) = start(t.path());
    write(&work, "uv.lock", "lock-v2\n");
    let (cs, findings) = collect(&repo, &base, &work, &staging, WHEN).expect("collect");

    let escalate = ChangeRules {
        protected_paths: vec![("uv.lock".into(), ProtectedDecision::Escalate)],
    };
    match validate(&cs, &escalate, &findings) {
        Validation::Escalate(found) => assert!(
            found
                .iter()
                .any(|f| f.class == "protected_path" && f.path == "uv.lock"),
            "{found:?}"
        ),
        other => panic!("expected escalation, got {other:?}"),
    }

    let deny = ChangeRules {
        protected_paths: vec![("uv.lock".into(), ProtectedDecision::Deny)],
    };
    assert!(matches!(
        validate(&cs, &deny, &findings),
        Validation::Reject(_)
    ));
}

#[test]
fn stale_base_is_a_conflict_not_a_merge() {
    let t = tempfile::tempdir().expect("tempdir");
    let root = t.path();
    let (repo, base, staging, work) = start(root);
    write(&work, "src/app.py", "print('v2')\n");
    let (cs, findings) = collect(&repo, &base, &work, &staging, WHEN).expect("collect");
    assert_eq!(
        validate(&cs, &ChangeRules::default(), &findings),
        Validation::Accept
    );

    // Advance the authoritative ref independently.
    git(root, &["clone", "-q", "auth.git", "other"]);
    let other = root.join("other");
    write(&other, "README.md", "changed upstream\n");
    git(&other, &["commit", "-q", "-am", "upstream"]);
    git(&other, &["push", "-q", "origin", "main"]);
    let pushed = head(&repo);
    assert_ne!(pushed, cs.proposed_commit);

    assert_eq!(
        err_class(promote(&repo, &cs, &staging, &authorize(&cs))),
        "stale_base"
    );
    assert_eq!(head(&repo), pushed);
}

#[test]
fn concurrent_changesets_from_one_base_conflict() {
    let t = tempfile::tempdir().expect("tempdir");
    let root = t.path();
    let repo = authoritative(root);
    let base = capture_base(&repo, "refs/heads/main").expect("capture base");
    let staging1 = root.join("staging1.git");
    let staging2 = root.join("staging2.git");
    let work1 = root.join("work1");
    let work2 = root.join("work2");
    export_base(&repo, &base, &staging1, &work1).expect("export 1");
    export_base(&repo, &base, &staging2, &work2).expect("export 2");
    write(&work1, "src/app.py", "print('one')\n");
    write(&work2, "src/app.py", "print('two')\n");

    let (cs1, _) = collect(&repo, &base, &work1, &staging1, WHEN).expect("collect 1");
    let (cs2, _) = collect(&repo, &base, &work2, &staging2, WHEN).expect("collect 2");

    promote(&repo, &cs1, &staging1, &authorize(&cs1)).expect("first promotion");
    assert_eq!(
        err_class(promote(&repo, &cs2, &staging2, &authorize(&cs2))),
        "stale_base"
    );
    assert_eq!(head(&repo), cs1.proposed_commit);
}

#[test]
fn tampered_changeset_is_refused() {
    let t = tempfile::tempdir().expect("tempdir");
    let (repo, base, staging, work) = start(t.path());
    write(&work, "src/app.py", "print('v2')\n");
    let (cs, _) = collect(&repo, &base, &work, &staging, WHEN).expect("collect");

    // (a) An extra entry without a digest update.
    let mut extra = cs.clone();
    extra.entries.push(ChangeEntry {
        path: "fake.txt".into(),
        status: ChangeStatus::Added,
        old_mode: "000000".into(),
        new_mode: "100644".into(),
        old_oid: "0".repeat(40),
        new_oid: "1".repeat(40),
        binary: false,
    });
    assert_eq!(
        err_class(promote(&repo, &extra, &staging, &authorize(&extra))),
        "tampered_changeset"
    );

    // (b) A renamed entry with a recomputed, authorized digest.
    let mut renamed = cs.clone();
    renamed
        .entries
        .iter_mut()
        .find(|e| e.path == "src/app.py")
        .expect("entry present")
        .path = "src/renamed.py".into();
    renamed.digest = renamed.compute_digest();
    assert_eq!(
        err_class(promote(&repo, &renamed, &staging, &authorize(&renamed))),
        "tampered_changeset"
    );

    // (c) Authorization for a different digest.
    let mut wrong_digest = authorize(&cs);
    wrong_digest.changeset_digest = "sha256:00".into();
    assert_eq!(
        err_class(promote(&repo, &cs, &staging, &wrong_digest)),
        "unauthorized_promotion"
    );

    // (d) Authorization for a different ref.
    let mut wrong_ref = authorize(&cs);
    wrong_ref.reference = "refs/heads/other".into();
    assert_eq!(
        err_class(promote(&repo, &cs, &staging, &wrong_ref)),
        "unauthorized_promotion"
    );

    assert_eq!(head(&repo), base.commit);
}

#[test]
fn unsafe_file_types_and_git_metadata_are_refused() {
    // (a) FIFO.
    {
        let t = tempfile::tempdir().expect("tempdir");
        let (repo, base, staging, work) = start(t.path());
        let status = Command::new("mkfifo")
            .arg(work.join("pipe"))
            .status()
            .expect("mkfifo must be installed");
        assert!(status.success());
        let (cs, findings) = collect(&repo, &base, &work, &staging, WHEN).expect("collect");
        assert!(has_class(&findings, "unsafe_file_type"), "{findings:?}");
        assert!(matches!(
            validate(&cs, &ChangeRules::default(), &findings),
            Validation::Reject(_)
        ));
    }

    // (b) Hard link.
    {
        let t = tempfile::tempdir().expect("tempdir");
        let (repo, base, staging, work) = start(t.path());
        fs::hard_link(work.join("README.md"), work.join("copy.md")).expect("hard link");
        let (cs, findings) = collect(&repo, &base, &work, &staging, WHEN).expect("collect");
        assert!(has_class(&findings, "hard_link"), "{findings:?}");
        assert!(matches!(
            validate(&cs, &ChangeRules::default(), &findings),
            Validation::Reject(_)
        ));
    }

    // (c) Nested Git metadata.
    {
        let t = tempfile::tempdir().expect("tempdir");
        let (repo, base, staging, work) = start(t.path());
        write(&work, "sub/.git/config", "[core]\n");
        assert_eq!(
            err_class(collect(&repo, &base, &work, &staging, WHEN)),
            "malformed_workspace"
        );
    }
}

#[test]
fn executable_bit_only_change_escalates() {
    let t = tempfile::tempdir().expect("tempdir");
    let (repo, base, staging, work) = start(t.path());
    chmod(&work.join("src/app.py"), 0o755);
    let (cs, findings) = collect(&repo, &base, &work, &staging, WHEN).expect("collect");

    match validate(&cs, &ChangeRules::default(), &findings) {
        Validation::Escalate(found) => assert!(has_class(&found, "permission_change"), "{found:?}"),
        other => panic!("expected escalation, got {other:?}"),
    }
}

#[test]
fn promotion_refuses_a_checked_out_branch() {
    let t = tempfile::tempdir().expect("tempdir");
    let root = t.path();
    let live = root.join("live");
    git(root, &["init", "-q", "-b", "main", "live"]);
    populate(&live);
    git(&live, &["add", "-A"]);
    git(&live, &["commit", "-q", "-m", "base"]);

    let base = capture_base(&live, "refs/heads/main").expect("capture base");
    let staging = root.join("staging.git");
    let work = root.join("work");
    export_base(&live, &base, &staging, &work).expect("export base");
    write(&work, "src/app.py", "print('v2')\n");
    let (cs, _) = collect(&live, &base, &work, &staging, WHEN).expect("collect");

    let before = head(&live);
    assert_eq!(
        err_class(promote(&live, &cs, &staging, &authorize(&cs))),
        "checked_out_reference"
    );
    assert_eq!(head(&live), before);
}

#[test]
fn base_with_a_submodule_is_refused() {
    let t = tempfile::tempdir().expect("tempdir");
    let root = t.path();
    let seed = root.join("seed");
    git(root, &["init", "-q", "-b", "main", "seed"]);
    write(&seed, "README.md", "hello\n");
    git(&seed, &["add", "-A"]);
    git(&seed, &["commit", "-q", "-m", "base"]);
    let existing = git(&seed, &["rev-parse", "HEAD"]);
    let gitlink = format!("160000,{existing},vendor/lib");
    git(
        &seed,
        &["update-index", "--add", "--cacheinfo", gitlink.as_str()],
    );
    git(&seed, &["commit", "-q", "-m", "gitlink"]);

    let base = capture_base(&seed, "refs/heads/main").expect("capture base");
    let staging = root.join("staging.git");
    let work = root.join("work");
    assert_eq!(
        err_class(export_base(&seed, &base, &staging, &work)),
        "unsupported_submodule"
    );
}

/// An interrupted safe write leaves its `.sf-staging-<pid>` directory and
/// `.sf-tmp-<pid>-<n>` temp files in the workspace (the writer publishes by
/// rename and removes the staging directory only on success). The scan must
/// refuse them by name — an internal incomplete write is never a delivered
/// artifact — and the error must name the leftover and the recovery remedy.
#[test]
fn staging_leftovers_fail_the_scan() {
    let t = tempfile::tempdir().expect("tempdir");
    let (repo, base, staging, work) = start(t.path());
    edit_basic(&work);
    // Control: a workspace with no staging leftover scans clean.
    collect(&repo, &base, &work, &staging, WHEN).expect("clean workspace collects");

    // The leftover, planted exactly as the interrupted writer leaves it.
    write(&work, ".sf-staging-123/.x.sf-tmp-1-2", "partial bytes\n");
    let error = collect(&repo, &base, &work, &staging, WHEN)
        .expect_err("staging leftovers must fail the scan");
    assert_eq!(error.class, "staging_leftovers");
    assert!(
        error.message.contains(".sf-staging-123"),
        "the error must name the leftover path: {error}"
    );
    assert!(
        error.message.contains("remove"),
        "the error must state the recovery remedy: {error}"
    );
}
