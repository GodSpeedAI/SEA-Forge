//! Git-backed transactional workspaces.
//!
//! Untrusted execution proposes changes; it never writes the authoritative
//! repository. The lifecycle is:
//!
//! 1. [`capture_base`] pins the commit and tree a task starts from.
//! 2. [`export_base`] materialises that tree as a plain directory (no `.git`)
//!    for upload into a sandbox. The repository itself is never exposed.
//! 3. [`collect`] turns the directory that came back into a proposed tree and
//!    an explicit [`Changeset`], inside a separate staging repository.
//! 4. [`validate`] applies path, file-type and protected-path rules.
//! 5. [`promote`] moves the authoritative ref with a compare-and-swap, only
//!    for the exact changeset that was authorized. A moved ref is a conflict.
//!
//! Git's object store, trees and `update-ref <ref> <new> <old>` provide the
//! identity and atomicity; this crate adds no store of its own. Atomic here
//! means the ref update: either the ref names the proposed commit or it is
//! unchanged. It does not cover effects outside the repository.
//!
//! Every Git invocation ignores user and system configuration, so hooks,
//! filters, aliases and global ignore files cannot change what is staged.
//! A `.gitignore` inside the workspace is honoured: an ignored new file is not
//! part of the proposal and so is never promoted, and a base file that became
//! ignored shows up as an explicit deletion.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const CHANGESET_SCHEMA: &str = "sea-forge.changeset/v1";

const MODE_FILE: &str = "100644";
const MODE_EXECUTABLE: &str = "100755";
const MODE_SYMLINK: &str = "120000";
const MODE_GITLINK: &str = "160000";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangesetError {
    /// Stable machine-readable class, e.g. `stale_base`.
    pub class: &'static str,
    pub message: String,
}

impl ChangesetError {
    fn new(class: &'static str, message: impl Into<String>) -> Self {
        Self {
            class,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ChangesetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.class, self.message)
    }
}

impl std::error::Error for ChangesetError {}

type Result<T> = std::result::Result<T, ChangesetError>;

/// The immutable starting point of a task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseIdentity {
    /// Full ref name, e.g. `refs/heads/main`.
    pub reference: String,
    pub commit: String,
    pub tree: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeStatus {
    Added,
    Modified,
    Deleted,
    TypeChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeEntry {
    pub path: String,
    pub status: ChangeStatus,
    /// Git file modes; `000000` on the missing side of an add or delete.
    pub old_mode: String,
    pub new_mode: String,
    pub old_oid: String,
    pub new_oid: String,
    pub binary: bool,
}

/// An explicit proposal: what would change if it were promoted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Changeset {
    pub schema: String,
    pub base: BaseIdentity,
    pub proposed_tree: String,
    /// Commit in the staging repository: `proposed_tree` on top of the base.
    pub proposed_commit: String,
    /// Sorted by path.
    pub entries: Vec<ChangeEntry>,
    /// sha256 over every field above; see [`Changeset::compute_digest`].
    pub digest: String,
}

impl Changeset {
    pub fn compute_digest(&self) -> String {
        let body = serde_json::json!({
            "schema": self.schema,
            "base": self.base,
            "proposed_tree": self.proposed_tree,
            "proposed_commit": self.proposed_commit,
            "entries": self.entries,
        });
        format!("sha256:{:x}", Sha256::digest(body.to_string().as_bytes()))
    }
}

/// What a protected path's owner decided about changes to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedDecision {
    Deny,
    Escalate,
}

#[derive(Debug, Clone, Default)]
pub struct ChangeRules {
    /// Repository-relative paths whose modification is not the task's to make.
    pub protected_paths: Vec<(String, ProtectedDecision)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub path: String,
    /// Stable class, e.g. `symlink`, `protected_path`.
    pub class: String,
    pub message: String,
}

/// The outcome of [`validate`]. `Reject` findings are never promotable;
/// `Escalate` findings need a human or higher authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "disposition", content = "findings", rename_all = "snake_case")]
pub enum Validation {
    Accept,
    Escalate(Vec<Finding>),
    Reject(Vec<Finding>),
}

/// Permission to promote one specific changeset onto one specific ref.
/// Constructed by the authority path after settlement accepts the changeset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionAuthorization {
    pub decision_id: String,
    pub reference: String,
    pub expected_base: String,
    pub changeset_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Promotion {
    pub reference: String,
    pub previous: String,
    pub current: String,
    pub changeset_digest: String,
    pub decision_id: String,
}

struct Git<'a> {
    git_dir: &'a Path,
    work_tree: Option<&'a Path>,
    index: Option<&'a Path>,
}

impl Git<'_> {
    fn command(&self) -> Command {
        // Commands run from the work tree, so every path must be absolute.
        let absolute =
            |path: &Path| std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
        let mut cmd = Command::new("git");
        cmd.env_clear();
        if let Some(path) = std::env::var_os("PATH") {
            cmd.env("PATH", path);
        }
        cmd.env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("LC_ALL", "C")
            .env("GIT_DIR", absolute(self.git_dir))
            .args(["-c", "core.hooksPath=/dev/null"])
            .args(["-c", "core.autocrlf=false"])
            .args(["-c", "core.fsmonitor=false"])
            .args(["-c", "core.excludesFile=/dev/null"])
            .stdin(Stdio::null());
        if let Some(work_tree) = self.work_tree {
            cmd.env("GIT_WORK_TREE", absolute(work_tree))
                .current_dir(work_tree);
        }
        if let Some(index) = self.index {
            cmd.env("GIT_INDEX_FILE", absolute(index));
        }
        cmd
    }

    fn run<I, S>(&self, class: &'static str, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.run_env(class, args, &[])
    }

    fn run_env<I, S>(&self, class: &'static str, args: I, env: &[(&str, &str)]) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut cmd = self.command();
        cmd.args(args).envs(env.iter().copied());
        let out = cmd
            .output()
            .map_err(|e| ChangesetError::new("git_unavailable", format!("cannot run git: {e}")))?;
        if !out.status.success() {
            return Err(ChangesetError::new(
                class,
                String::from_utf8_lossy(&out.stderr).trim().to_string(),
            ));
        }
        Ok(out.stdout)
    }

    fn line<I, S>(&self, class: &'static str, args: I) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Ok(String::from_utf8_lossy(&self.run(class, args)?)
            .trim()
            .to_string())
    }
}

fn git_dir_of(repo: &Path) -> Result<PathBuf> {
    let dot_git = repo.join(".git");
    let git_dir = if dot_git.is_dir() {
        dot_git
    } else {
        repo.to_path_buf()
    };
    if !git_dir.join("objects").is_dir() {
        return Err(ChangesetError::new(
            "not_a_repository",
            format!("{} is not a Git repository", repo.display()),
        ));
    }
    Ok(git_dir)
}

fn repo_git(git_dir: &Path) -> Git<'_> {
    Git {
        git_dir,
        work_tree: None,
        index: None,
    }
}

/// Pin the commit and tree `reference` names right now.
pub fn capture_base(repo: &Path, reference: &str) -> Result<BaseIdentity> {
    if !reference.starts_with("refs/") {
        return Err(ChangesetError::new(
            "invalid_reference",
            format!("'{reference}' must be a full ref name such as refs/heads/main"),
        ));
    }
    let git_dir = git_dir_of(repo)?;
    let git = repo_git(&git_dir);
    let commit = git.line(
        "unknown_reference",
        ["rev-parse", "--verify", &format!("{reference}^{{commit}}")],
    )?;
    let tree = git.line(
        "unknown_reference",
        ["rev-parse", "--verify", &format!("{commit}^{{tree}}")],
    )?;
    Ok(BaseIdentity {
        reference: reference.to_string(),
        commit,
        tree,
    })
}

/// Create the staging repository: empty, bare, borrowing the authoritative
/// repository's objects read-only through `alternates`.
fn init_staging(repo_git_dir: &Path, staging: &Path) -> Result<()> {
    if staging.join("objects").is_dir() {
        return Ok(());
    }
    fs::create_dir_all(staging)
        .map_err(|e| ChangesetError::new("staging_unavailable", e.to_string()))?;
    let target = std::path::absolute(staging)
        .map_err(|e| ChangesetError::new("staging_unavailable", e.to_string()))?;
    repo_git(staging).run(
        "staging_unavailable",
        [
            OsStr::new("init"),
            OsStr::new("--quiet"),
            OsStr::new("--bare"),
            target.as_os_str(),
        ],
    )?;
    let objects = repo_git_dir
        .join("objects")
        .canonicalize()
        .map_err(|e| ChangesetError::new("not_a_repository", e.to_string()))?;
    fs::write(
        staging.join("objects/info/alternates"),
        format!("{}\n", objects.display()),
    )
    .map_err(|e| ChangesetError::new("staging_unavailable", e.to_string()))
}

/// Materialise the base tree as a plain directory at `dest` (which must be
/// empty or absent). Submodules are refused: their content is not in this
/// repository, so the export would not be the tree it claims to be.
pub fn export_base(repo: &Path, base: &BaseIdentity, staging: &Path, dest: &Path) -> Result<()> {
    let git_dir = git_dir_of(repo)?;
    init_staging(&git_dir, staging)?;
    if dest.exists()
        && fs::read_dir(dest)
            .map_err(|e| ChangesetError::new("workspace_unavailable", e.to_string()))?
            .next()
            .is_some()
    {
        return Err(ChangesetError::new(
            "workspace_not_empty",
            format!("{} is not empty", dest.display()),
        ));
    }
    fs::create_dir_all(dest)
        .map_err(|e| ChangesetError::new("workspace_unavailable", e.to_string()))?;

    let listing = repo_git(staging).run("unknown_base", ["ls-tree", "-r", "-z", &base.tree])?;
    if listing
        .split(|b| *b == 0)
        .any(|entry| entry.starts_with(MODE_GITLINK.as_bytes()))
    {
        return Err(ChangesetError::new(
            "unsupported_submodule",
            "the base tree contains a submodule",
        ));
    }

    let index = staging.join("index.export");
    let git = Git {
        git_dir: staging,
        work_tree: Some(dest),
        index: Some(&index),
    };
    git.run("unknown_base", ["read-tree", &base.tree])?;
    git.run("export_failed", ["checkout-index", "--all", "--force"])?;
    fs::remove_file(&index).ok();
    Ok(())
}

/// Refuse anything Git would not record faithfully or that could alias state
/// outside the workspace.
fn scan(root: &Path, dir: &Path, findings: &mut Vec<Finding>) -> Result<()> {
    use std::os::unix::fs::{FileTypeExt, MetadataExt};
    let entries = fs::read_dir(dir)
        .map_err(|e| ChangesetError::new("workspace_unavailable", e.to_string()))?;
    for entry in entries {
        let entry =
            entry.map_err(|e| ChangesetError::new("workspace_unavailable", e.to_string()))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let meta = fs::symlink_metadata(&path)
            .map_err(|e| ChangesetError::new("workspace_unavailable", e.to_string()))?;
        let file_type = meta.file_type();
        if entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case(".git")
        {
            findings.push(Finding {
                path: relative,
                class: "git_metadata".into(),
                message: "the workspace contains Git metadata".into(),
            });
        } else if file_type.is_dir() {
            scan(root, &path, findings)?;
        } else if file_type.is_fifo()
            || file_type.is_socket()
            || file_type.is_block_device()
            || file_type.is_char_device()
        {
            findings.push(Finding {
                path: relative,
                class: "unsafe_file_type".into(),
                message: "devices, sockets and FIFOs cannot be part of a changeset".into(),
            });
        } else if file_type.is_file() && meta.nlink() > 1 {
            findings.push(Finding {
                path: relative,
                class: "hard_link".into(),
                message: "the file has more than one link".into(),
            });
        }
    }
    Ok(())
}

fn unsafe_path(path: &str) -> Option<&'static str> {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') {
        return Some("the path is absolute or contains a backslash");
    }
    if path.chars().any(char::is_control) {
        return Some("the path contains control characters");
    }
    for part in path.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            return Some("the path contains an empty, '.' or '..' component");
        }
        if part.eq_ignore_ascii_case(".git") {
            return Some("the path enters Git metadata");
        }
    }
    None
}

/// Build the explicit changeset for `result_dir` relative to `base`.
///
/// `committed_at` (RFC 3339 or Git date) fixes the proposal commit's
/// timestamps so the same inputs give the same commit id.
pub fn collect(
    repo: &Path,
    base: &BaseIdentity,
    result_dir: &Path,
    staging: &Path,
    committed_at: &str,
) -> Result<(Changeset, Vec<Finding>)> {
    let git_dir = git_dir_of(repo)?;
    init_staging(&git_dir, staging)?;

    let mut findings = Vec::new();
    scan(result_dir, result_dir, &mut findings)?;
    if findings.iter().any(|f| f.class == "git_metadata") {
        // Staging a tree that contains `.git` would make Git treat part of
        // the workspace as a nested repository.
        return Err(ChangesetError::new(
            "malformed_workspace",
            "the returned workspace contains Git metadata",
        ));
    }

    let index = staging.join("index.proposal");
    fs::remove_file(&index).ok();
    let git = Git {
        git_dir: staging,
        work_tree: Some(result_dir),
        index: Some(&index),
    };
    git.run("staging_failed", ["add", "--all", "."])?;
    let proposed_tree = git.line("staging_failed", ["write-tree"])?;
    fs::remove_file(&index).ok();

    let staged = repo_git(staging);
    let proposed_commit = String::from_utf8_lossy(&staged.run_env(
        "staging_failed",
        [
            "commit-tree",
            &proposed_tree,
            "-p",
            &base.commit,
            "-m",
            "sea-forge: proposed changeset",
        ],
        &[
            ("GIT_AUTHOR_NAME", "sea-forge"),
            ("GIT_AUTHOR_EMAIL", "sea-forge@localhost"),
            ("GIT_AUTHOR_DATE", committed_at),
            ("GIT_COMMITTER_NAME", "sea-forge"),
            ("GIT_COMMITTER_EMAIL", "sea-forge@localhost"),
            ("GIT_COMMITTER_DATE", committed_at),
        ],
    )?)
    .trim()
    .to_string();
    staged.run(
        "staging_failed",
        [
            "update-ref",
            &format!("refs/proposals/{proposed_commit}"),
            &proposed_commit,
        ],
    )?;

    let entries = diff(&staged, &base.tree, &proposed_tree)?;
    let mut changeset = Changeset {
        schema: CHANGESET_SCHEMA.to_string(),
        base: base.clone(),
        proposed_tree,
        proposed_commit,
        entries,
        digest: String::new(),
    };
    changeset.digest = changeset.compute_digest();
    Ok((changeset, findings))
}

fn diff(git: &Git<'_>, old_tree: &str, new_tree: &str) -> Result<Vec<ChangeEntry>> {
    let raw = git.run(
        "staging_failed",
        [
            "diff-tree",
            "-r",
            "-z",
            "--raw",
            "--no-renames",
            "--no-abbrev",
            old_tree,
            new_tree,
        ],
    )?;
    let numstat = git.run(
        "staging_failed",
        [
            "diff-tree",
            "-r",
            "-z",
            "--numstat",
            "--no-renames",
            old_tree,
            new_tree,
        ],
    )?;
    let binary: std::collections::BTreeSet<String> = numstat
        .split(|b| *b == 0)
        .filter_map(|record| {
            let text = String::from_utf8_lossy(record);
            let mut fields = text.splitn(3, '\t');
            match (fields.next(), fields.next(), fields.next()) {
                (Some("-"), Some("-"), Some(path)) => Some(path.to_string()),
                _ => None,
            }
        })
        .collect();

    let mut entries = Vec::new();
    let mut fields = raw.split(|b| *b == 0);
    while let Some(header) = fields.next() {
        if header.is_empty() {
            continue;
        }
        let header = String::from_utf8_lossy(header);
        let path = fields
            .next()
            .map(|p| String::from_utf8_lossy(p).to_string());
        let parts: Vec<&str> = header.trim_start_matches(':').split(' ').collect();
        let (Some(path), [old_mode, new_mode, old_oid, new_oid, status]) = (path, parts.as_slice())
        else {
            return Err(ChangesetError::new(
                "staging_failed",
                format!("unexpected diff record '{header}'"),
            ));
        };
        let status = match *status {
            "A" => ChangeStatus::Added,
            "M" => ChangeStatus::Modified,
            "D" => ChangeStatus::Deleted,
            "T" => ChangeStatus::TypeChanged,
            other => {
                return Err(ChangesetError::new(
                    "staging_failed",
                    format!("unexpected diff status '{other}' for {path}"),
                ))
            }
        };
        entries.push(ChangeEntry {
            binary: binary.contains(&path),
            path,
            status,
            old_mode: old_mode.to_string(),
            new_mode: new_mode.to_string(),
            old_oid: old_oid.to_string(),
            new_oid: new_oid.to_string(),
        });
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

/// Apply the structural rules and the caller's protected paths.
/// `scan_findings` are the workspace findings [`collect`] returned.
pub fn validate(
    changeset: &Changeset,
    rules: &ChangeRules,
    scan_findings: &[Finding],
) -> Validation {
    let mut reject: Vec<Finding> = scan_findings.to_vec();
    let mut escalate = Vec::new();
    let finding = |entry: &ChangeEntry, class: &str, message: &str| Finding {
        path: entry.path.clone(),
        class: class.to_string(),
        message: message.to_string(),
    };

    for entry in &changeset.entries {
        if let Some(reason) = unsafe_path(&entry.path) {
            reject.push(finding(entry, "unsafe_path", reason));
            continue;
        }
        match entry.new_mode.as_str() {
            MODE_SYMLINK => reject.push(finding(
                entry,
                "symlink",
                "a changeset may not add or retarget a symbolic link",
            )),
            MODE_GITLINK => reject.push(finding(
                entry,
                "submodule",
                "a changeset may not add a submodule",
            )),
            MODE_FILE | MODE_EXECUTABLE | "000000" => {}
            _ => reject.push(finding(entry, "unsafe_file_mode", "unexpected file mode")),
        }
        if entry.status == ChangeStatus::Modified
            && entry.old_oid == entry.new_oid
            && entry.old_mode != entry.new_mode
            && entry.new_mode == MODE_EXECUTABLE
        {
            escalate.push(finding(
                entry,
                "permission_change",
                "the file became executable without a content change",
            ));
        }
        for (protected, decision) in &rules.protected_paths {
            if &entry.path == protected {
                let item = finding(entry, "protected_path", "the path is protected by the Cell");
                match decision {
                    ProtectedDecision::Deny => reject.push(item),
                    ProtectedDecision::Escalate => escalate.push(item),
                }
            }
        }
    }

    if !reject.is_empty() {
        Validation::Reject(reject)
    } else if !escalate.is_empty() {
        Validation::Escalate(escalate)
    } else {
        Validation::Accept
    }
}

/// Promote an authorized changeset: move `reference` from the base commit to
/// the proposed commit with a compare-and-swap.
///
/// Errors (the authoritative ref is unchanged in every case):
/// - `tampered_changeset`: the changeset no longer matches its digest, the
///   authorization, or the staged commit.
/// - `stale_base`: the ref no longer names the base commit.
/// - `checked_out_reference`: the ref is checked out in a working tree, which
///   a bare ref update would silently desynchronise.
pub fn promote(
    repo: &Path,
    changeset: &Changeset,
    staging: &Path,
    authorization: &PromotionAuthorization,
) -> Result<Promotion> {
    let git_dir = git_dir_of(repo)?;
    let git = repo_git(&git_dir);
    let tampered = |message: &str| ChangesetError::new("tampered_changeset", message);

    if changeset.compute_digest() != changeset.digest {
        return Err(tampered("the changeset does not match its own digest"));
    }
    if authorization.changeset_digest != changeset.digest
        || authorization.reference != changeset.base.reference
        || authorization.expected_base != changeset.base.commit
    {
        return Err(ChangesetError::new(
            "unauthorized_promotion",
            "the authorization does not cover this changeset, ref and base",
        ));
    }

    // Re-derive everything from the staged commit rather than trusting the
    // changeset document.
    let staged = repo_git(staging);
    let commit = &changeset.proposed_commit;
    let tree = staged
        .line(
            "tampered_changeset",
            ["rev-parse", "--verify", &format!("{commit}^{{tree}}")],
        )
        .map_err(|_| tampered("the proposed commit is not in the staging repository"))?;
    let parents = staged.line(
        "tampered_changeset",
        ["rev-list", "--parents", "-n", "1", commit],
    )?;
    if tree != changeset.proposed_tree || parents != format!("{commit} {}", changeset.base.commit) {
        return Err(tampered("the staged commit does not match the changeset"));
    }
    if diff(&staged, &changeset.base.tree, &tree)? != changeset.entries {
        return Err(tampered(
            "the changeset entries do not match the staged tree",
        ));
    }

    let reference = &changeset.base.reference;
    let worktrees =
        String::from_utf8_lossy(&git.run("not_a_repository", ["worktree", "list", "--porcelain"])?)
            .to_string();
    if worktrees
        .lines()
        .any(|line| line == format!("branch {reference}"))
    {
        return Err(ChangesetError::new(
            "checked_out_reference",
            format!("{reference} is checked out in a working tree"),
        ));
    }

    let current = git.line("unknown_reference", ["rev-parse", "--verify", reference])?;
    if current != changeset.base.commit {
        return Err(ChangesetError::new(
            "stale_base",
            format!(
                "{reference} is at {current}, not the base {}",
                changeset.base.commit
            ),
        ));
    }

    let staging_path = staging
        .canonicalize()
        .map_err(|e| ChangesetError::new("staging_unavailable", e.to_string()))?;
    git.run(
        "promotion_failed",
        [
            OsStr::new("fetch"),
            OsStr::new("--quiet"),
            OsStr::new("--no-tags"),
            OsStr::new("--no-write-fetch-head"),
            staging_path.as_os_str(),
            OsStr::new(&format!("refs/proposals/{commit}")),
        ],
    )?;
    git.run(
        "promotion_failed",
        ["cat-file", "-e", &format!("{commit}^{{commit}}")],
    )?;

    // The compare-and-swap. A concurrent promotion makes this fail.
    git.run(
        "stale_base",
        [
            "update-ref",
            "-m",
            &format!("sea-forge: promote {}", changeset.digest),
            reference,
            commit,
            &changeset.base.commit,
        ],
    )?;

    Ok(Promotion {
        reference: reference.clone(),
        previous: changeset.base.commit.clone(),
        current: commit.clone(),
        changeset_digest: changeset.digest.clone(),
        decision_id: authorization.decision_id.clone(),
    })
}
