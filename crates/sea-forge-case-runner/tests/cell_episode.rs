#![cfg(unix)]

//! Governed cell episodes against stub OpenShell executables.
//!
//! The OpenShell CLI and prover are stub executables that run the command in a plain temp directory. These tests establish the governed control flow, evidence and promotion rules; they do not establish isolation.
//!
//! Each stub logs its invocations to `calls.log` and reads injected behaviour from marker files in `ctl/`.
//!
//! An episode only requests promotion. Promotion needs a separate approval by a
//! security officer, and `promote_cell_changeset` is the only path that moves
//! the authoritative ref.
//!
//! Tests marked `requires opa 1.21.1 on PATH` need the pinned binary and run
//! with `--include-ignored`.

use sea_forge_authority::OpaBinding;
use sea_forge_case_runner::cell_episode::{
    promote_cell_changeset, run_cell_episode, staging_dir, CellEpisode, CellEpisodeRequest,
    CellOpaInput, PromotionApproval, PromotionRequest,
};
use sea_forge_changeset::{ChangeStatus, Promotion, Validation};
use sea_forge_core::{
    errors::ForgeError,
    types::{
        ApprovalRequest, ApprovalStatus, AuthorityAction, CriteriaDerivation, DerivationMethod,
        ExecutionStatus, OriginRef, OriginRefKind, OriginRole, SettlementCriteria,
        SettlementCriteriaRecord, SettlementStatus, Verdict,
    },
};
use sea_forge_ledger::{types::hash_canonical, CommittedRecordRef, LedgerStream};
use sea_forge_sandbox::OpenShellBinding;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const WHEN: &str = "2026-01-02T00:00:00Z";
const ENTITY: &str = "operator_local";
const CASE: &str = "case_cell_1";
const ITEM: &str = "item_repair";
const REFERENCE: &str = "refs/heads/main";
const IMAGE: &str = "registry.example/cell:1";
const OTHER_IMAGE: &str = "registry.example/other:9";

const OPENSHELL_STUB: &str = r##"#!/bin/sh
LOG="__LOG__"
CTL="__CTL__"
FAKE="__FAKE__"
PROJECTION="__PROJECTION__"
echo "openshell $*" >> "$LOG"
if [ "$1" = "--gateway" ]; then
  shift 2
fi
case "$1" in
  --version)
    echo "openshell 0.1.3"
    exit 0
    ;;
  status)
    if [ -f "$CTL/gateway_down" ]; then
      echo "connection refused" >&2
      exit 1
    fi
    exit 0
    ;;
  gateway)
    echo '{"compute_drivers":[{"name":"vm","capabilities":{"driver_name":"vm","driver_version":"0.1.3"}}]}'
    exit 0
    ;;
  logs)
    if [ -f "$CTL/logs" ]; then cat "$CTL/logs"; fi
    exit 0
    ;;
  policy)
    echo '{"hash":"h1","version":1,"config_revision":"r1","policy_source":"sandbox"}'
    exit 0
    ;;
  sandbox)
    case "$2" in
      create)
        mkdir -p "$FAKE"
        exit 0
        ;;
      get)
        case "$*" in
          *--policy-only*)
            if [ -f "$CTL/effective_policy" ]; then
              cat "$CTL/effective_policy"
            else
              cat "$PROJECTION/policy.yaml"
            fi
            exit 0
            ;;
        esac
        if [ -f "$CTL/deleted" ]; then exit 1; fi
        exit 0
        ;;
      delete)
        : > "$CTL/deleted"
        exit 0
        ;;
      upload)
        mkdir -p "$FAKE" && cp -a "$4/." "$FAKE/"
        exit $?
        ;;
      download)
        mkdir -p "$5" && cp -a "$FAKE/." "$5/"
        exit $?
        ;;
      exec)
        shift 2
        while [ "$#" -gt 0 ] && [ "$1" != "--" ]; do
          shift
        done
        if [ "$#" -gt 0 ]; then shift; fi
        if [ -f "$CTL/on_exec" ]; then eval "$(cat "$CTL/on_exec")"; fi
        cd "$FAKE" || exit 1
        "$@"
        exit $?
        ;;
    esac
    ;;
esac
echo "stub openshell: unhandled: $*" >&2
exit 2
"##;

const PROVER_STUB: &str = r##"#!/bin/sh
LOG="__LOG__"
CTL="__CTL__"
echo "openshell-prover $*" >> "$LOG"
if [ -f "$CTL/prover_effective_json" ]; then
  case "$2" in
    *effective-policy.yaml)
      cat "$CTL/prover_effective_json"
      exit 0
      ;;
  esac
fi
echo '{"schema_version":1,"result":"within_boundary","exit_code":0,"coverage":{"domains":["filesystem","network_l4","network_rest","process","landlock"]},"counterexample":null,"reason":null}'
exit 0
"##;

/// Which policy the test writes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Policy {
    /// Sandbox command allowed under the fixture's identity; promotion escalates
    /// for a security officer's approval.
    Standard,
    /// Sandbox command allowed; no rule for `promote_changeset`.
    WithoutPromoteRule,
    /// Sandbox command allowed under a policy digest that is not the projection's.
    WrongDigest,
    /// Sandbox command allowed for an image other than the binding's.
    WrongImage,
    /// Sandbox command denied.
    ExecuteDenied,
    /// Only a host `execute_command` rule, with the `openshell` class.
    HostRouteOpenshell,
    /// Only a host `execute_command` rule, with the `local` class.
    HostLocalOnly,
    /// As `Standard`, with a required `opa` engine over the DomainForge Rego bundle.
    WithOpa,
    /// As `WithOpa`, over the gate fixture bundle, whose decision can be deny or escalate.
    WithGateOpa,
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cells/repair-agent/openshell")
}

fn rego_fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cells/repair-agent/rego")
}

fn gate_fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/policies/gate/rego")
}

/// The gate fixture's input for `name` (`allow`, `deny` or `escalate`).
fn gate_input(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/policies/gate/inputs")
        .join(format!("{name}.json"));
    serde_json::from_slice(&fs::read(path).expect("gate input")).expect("gate input json")
}

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

/// A bare authoritative repository at `root/auth.git` with one base commit.
fn authoritative(root: &Path) {
    git(root, &["init", "-q", "-b", "main", "seed"]);
    let seed = root.join("seed");
    write(&seed, "README.md", "hello\n");
    write(&seed, "src/app.py", "print('v1')\n");
    write(&seed, "pyproject.toml", "[project]\nname='x'\n");
    write(&seed, "uv.lock", "lock-v1\n");
    git(&seed, &["add", "-A"]);
    git(&seed, &["commit", "-q", "-m", "base"]);
    git(root, &["clone", "-q", "--bare", "seed", "auth.git"]);
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

fn main_ref(repo: &Path) -> String {
    git(repo, &["rev-parse", "refs/heads/main"])
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create directory");
    for entry in fs::read_dir(from).expect("read directory") {
        let entry = entry.expect("directory entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("file type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy file");
        }
    }
}

fn write_stub(path: &Path, body: &str, root: &Path) {
    let text = body
        .replace("__LOG__", &root.join("calls.log").display().to_string())
        .replace("__CTL__", &root.join("ctl").display().to_string())
        .replace("__FAKE__", &root.join("fake-sandbox").display().to_string())
        .replace(
            "__PROJECTION__",
            &root.join("projection").display().to_string(),
        );
    fs::write(path, text).expect("write stub");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("chmod stub");
}

struct Harness {
    root: tempfile::TempDir,
}

impl Harness {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("temp root");
        let path = root.path();
        authoritative(path);

        // The projection is copied so tests can tamper with the copy only.
        let projection = path.join("projection");
        fs::create_dir_all(&projection).expect("projection dir");
        for entry in fs::read_dir(fixture_dir()).expect("fixture dir") {
            let entry = entry.expect("fixture entry");
            if entry.file_type().expect("file type").is_file() {
                fs::copy(entry.path(), projection.join(entry.file_name())).expect("copy fixture");
            }
        }

        let bin = path.join("bin");
        fs::create_dir_all(&bin).expect("bin dir");
        write_stub(&bin.join("openshell"), OPENSHELL_STUB, path);
        write_stub(&bin.join("openshell-prover"), PROVER_STUB, path);
        fs::create_dir_all(path.join("ctl")).expect("ctl dir");
        Self { root }
    }

    fn path(&self) -> &Path {
        self.root.path()
    }

    fn repo(&self) -> PathBuf {
        self.path().join("auth.git")
    }

    fn ctl(&self, name: &str) -> PathBuf {
        self.path().join("ctl").join(name)
    }

    fn calls(&self) -> String {
        fs::read_to_string(self.path().join("calls.log")).unwrap_or_default()
    }

    fn lock(&self) -> Value {
        let bytes = fs::read(self.path().join("projection/openshell.lock")).expect("lock");
        serde_json::from_slice(&bytes).expect("lock json")
    }

    fn lock_str(&self, pointer: &str) -> String {
        self.lock()
            .pointer(pointer)
            .and_then(Value::as_str)
            .expect("lock field")
            .to_string()
    }

    /// Copies the Rego bundle beside the policy and returns its provenance hash.
    fn copy_rego(&self, from: &Path) -> String {
        let target = self.path().join("rego");
        copy_dir(from, &target);
        let out = Command::new("sha256sum")
            .arg(target.join("provenance.json"))
            .output()
            .expect("sha256sum must be installed");
        String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .next()
            .expect("sha256 digest")
            .to_string()
    }

    fn policy(&self, variant: Policy) -> PathBuf {
        let cell = self.lock_str("/cell_id");
        let model = self.lock_str("/model/hash");
        let digest = if variant == Policy::WrongDigest {
            format!("sha256:{}", "0".repeat(64))
        } else {
            self.lock_str("/policy_digest")
        };
        let image = if variant == Policy::WrongImage {
            OTHER_IMAGE
        } else {
            IMAGE
        };
        let repo = fs::canonicalize(self.repo())
            .expect("canonical repository")
            .display()
            .to_string();
        let bound = format!(
            "      cell_id: [\"{cell}\"]\n      model_hash: [\"{model}\"]\n      openshell_policy_digest: [\"{digest}\"]\n      sandbox_image: [\"{image}\"]\n"
        );
        let verdict = if variant == Policy::ExecuteDenied {
            "deny"
        } else {
            "allow"
        };

        let mut text = String::from("version: \"0.1\"\n");
        text.push_str(
            "identity_bindings:\n  - principal: operator_local\n    actor_type: human\n    role: operator\n  - principal: security_officer\n    actor_type: human\n    role: R-SO\n  - principal: junior_reviewer\n    actor_type: human\n    role: operator\n",
        );
        if matches!(variant, Policy::WithOpa | Policy::WithGateOpa) {
            // A v0.1 bundle keeps `source_base` as written; a v0.2 bundle would
            // demand the full governance schema.
            let bundle = if variant == Policy::WithGateOpa {
                gate_fixture_dir()
            } else {
                rego_fixture_dir()
            };
            let provenance = self.copy_rego(&bundle);
            text.push_str(&format!(
                "source_base: \"{}\"\nsources:\n  - surface: rego_provenance\n    path: rego/provenance.json\n    sha256: {provenance}\npolicy_engines:\n  - engine: opa\n    fail_mode: closed\n    required: true\n",
                self.path().display()
            ));
        }
        text.push_str("rules:\n");
        match variant {
            Policy::HostRouteOpenshell => text.push_str(&format!(
                "  - name: host-shell-in-openshell\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sh\n    sandbox_class: openshell\n    boundary_constraints:\n{bound}"
            )),
            Policy::HostLocalOnly => text.push_str(
                "  - name: host-shell-local\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sh\n    sandbox_class: local\n",
            ),
            _ => text.push_str(&format!(
                "  - name: execute-in-openshell\n    verdict: {verdict}\n    actor_role: operator\n    operation_kind: execute_sandbox_command\n    argv0: sh\n    sandbox_class: openshell\n    boundary_constraints:\n{bound}"
            )),
        }
        if variant != Policy::WithoutPromoteRule {
            text.push_str(&format!(
                "  - name: promote-changeset\n    verdict: allow\n    actor_role: operator\n    operation_kind: promote_changeset\n    requires_approval: true\n    boundary_constraints:\n      repository: [\"{repo}\"]\n      reference: [\"{REFERENCE}\"]\n"
            ));
        }
        text.push_str(
            "sod_rules:\n  - name: promote_requires_security_officer\n    requester_role: operator\n    approver_role: R-SO\n    operation_kind: promote_changeset\n    allow_same_principal: false\n",
        );

        let path = self
            .path()
            .join(format!("policy-{}.yaml", variant_name(variant)));
        fs::write(&path, text).expect("write policy");
        path
    }

    fn binding(&self, image: Option<&str>) -> OpenShellBinding {
        OpenShellBinding {
            projection_dir: self.path().join("projection"),
            gateway: None,
            image: image.map(str::to_string),
            cli: self.path().join("bin/openshell"),
            prover: self.path().join("bin/openshell-prover"),
            local_microvm_gateway: false,
            native_executable: None,
        }
    }

    fn ledger(&self) -> LedgerStream {
        LedgerStream::open(&self.path().join("ledger"), format!("case-{CASE}"), ENTITY)
            .expect("open ledger")
    }

    fn run_dir(&self, run: &str) -> PathBuf {
        self.path().join("runs").join(run)
    }

    fn episode(
        &self,
        stream: &LedgerStream,
        policy: &Path,
        run: &str,
        argv: &[&str],
    ) -> Result<CellEpisode, ForgeError> {
        self.run_episode(stream, policy, run, argv, None, self.binding(Some(IMAGE)))
    }

    fn run_episode(
        &self,
        stream: &LedgerStream,
        policy: &Path,
        run: &str,
        argv: &[&str],
        opa: Option<CellOpaInput>,
        binding: OpenShellBinding,
    ) -> Result<CellEpisode, ForgeError> {
        let run_dir = self.run_dir(run);
        let repo = self.repo();
        run_cell_episode(
            CellEpisodeRequest {
                policy_path: policy,
                entity: ENTITY,
                case_id: CASE,
                run_id: run,
                plan_item_id: ITEM,
                run_dir: &run_dir,
                repository: &repo,
                reference: REFERENCE,
                binding,
                argv: argv.iter().map(|arg| arg.to_string()).collect(),
                env: BTreeMap::new(),
                criteria: SettlementCriteria {
                    require_exit_zero: true,
                    ..Default::default()
                },
                opa,
                committed_at: WHEN,
            },
            stream,
        )
    }
}

fn variant_name(variant: Policy) -> &'static str {
    match variant {
        Policy::Standard => "standard",
        Policy::WithoutPromoteRule => "without-promote",
        Policy::WrongDigest => "wrong-digest",
        Policy::WrongImage => "wrong-image",
        Policy::ExecuteDenied => "execute-denied",
        Policy::HostRouteOpenshell => "host-route-openshell",
        Policy::HostLocalOnly => "host-local-only",
        Policy::WithOpa => "with-opa",
        Policy::WithGateOpa => "with-gate-opa",
    }
}

/// `(record_kind, payload)` for every ledger entry, in commit order.
fn records(stream: &LedgerStream) -> Vec<(String, Value)> {
    stream
        .read_entries()
        .expect("read ledger")
        .into_iter()
        .map(|entry| (entry.record_kind, entry.payload))
        .collect()
}

fn kinds(stream: &LedgerStream) -> Vec<String> {
    records(stream).into_iter().map(|(kind, _)| kind).collect()
}

fn payloads(stream: &LedgerStream, kind: &str) -> Vec<Value> {
    records(stream)
        .into_iter()
        .filter(|(record_kind, _)| record_kind == kind)
        .map(|(_, payload)| payload)
        .collect()
}

fn has_kind(stream: &LedgerStream, kind: &str) -> bool {
    kinds(stream).iter().any(|k| k == kind)
}

// ── Promotion approval helpers ──

/// The records an approved promotion needs, as returned by `approve`.
type Approval = (
    ApprovalRequest,
    CommittedRecordRef,
    SettlementCriteriaRecord,
    CommittedRecordRef,
);

fn criteria() -> SettlementCriteriaRecord {
    let mut record = SettlementCriteriaRecord {
        version: sea_forge_core::RECORD_VERSION.into(),
        criteria_id: "crit_cell_promote".into(),
        criteria: SettlementCriteria {
            require_exit_zero: true,
            required_artifacts: vec![],
            stdout_must_contain: None,
            agent_output_must_contain: None,
            require_approval: true,
            evaluator: None,
            records: None,
            per_record_evaluator: None,
            min_pass_ratio: None,
        },
        origin_refs: vec![OriginRef {
            kind: OriginRefKind::Intent,
            reference: "int_cell_promote".into(),
            sha256: "sha256:intent".into(),
            role: OriginRole::AcceptanceSource,
            evidence_refs: vec![],
            domain_model_ref: None,
        }],
        derivation: CriteriaDerivation {
            method: DerivationMethod::DeterministicPlanner,
            actor_ref: ENTITY.into(),
            producer_ref: None,
            rationale: "cell episode promotion fixture".into(),
        },
        declared_at: WHEN.into(),
        criteria_sha256: String::new(),
        criteria_record_hash: String::new(),
    };
    record.criteria_sha256 = hash_canonical(&record.criteria).unwrap();
    let mut unhashed = record.clone();
    unhashed.criteria_record_hash.clear();
    record.criteria_record_hash = hash_canonical(&unhashed).unwrap();
    record
}

/// Commits the criteria and an approval resolution for `request`, resolved by
/// `resolver` with `status`, exactly as a human resolution would be recorded.
fn approve_with(
    stream: &LedgerStream,
    request: &PromotionRequest,
    resolver: &str,
    status: ApprovalStatus,
) -> Approval {
    let criteria = criteria();
    let committed_criteria = stream
        .commit_typed("settlement_criteria", vec![], &criteria, vec![])
        .expect("commit criteria");
    let resolved = matches!(status, ApprovalStatus::Approved | ApprovalStatus::Rejected);
    let approval = ApprovalRequest {
        version: sea_forge_core::RECORD_VERSION.into(),
        approval_id: "apr_promote".into(),
        run_id: request.decision.run_id.clone(),
        case_id: CASE.into(),
        decision_id: request.decision.decision_id.clone(),
        plan_item_id: request.decision.plan_item_id.clone(),
        criteria_ref: Some(criteria.criteria_id.clone()),
        criteria_sha256: Some(criteria.criteria_sha256.clone()),
        criteria_record_hash: Some(criteria.criteria_record_hash.clone()),
        job_contract_ref: None,
        requested_at: WHEN.into(),
        expires_at: "2099-01-01T00:00:00Z".into(),
        status,
        resolved_by: resolved.then(|| resolver.to_string()),
        resolved_at: resolved.then(|| WHEN.to_string()),
        note: None,
    };
    let committed_approval = stream
        .commit_typed("approval_resolution", vec![], &approval, vec![])
        .expect("commit approval");
    (approval, committed_approval, criteria, committed_criteria)
}

fn approve(stream: &LedgerStream, request: &PromotionRequest, resolver: &str) -> Approval {
    approve_with(stream, request, resolver, ApprovalStatus::Approved)
}

/// Attempts the promotion with the given approval, as the operator would.
fn promote_with(
    h: &Harness,
    stream: &LedgerStream,
    policy: &Path,
    run: &str,
    request: &PromotionRequest,
    approval: &Approval,
) -> Result<Promotion, ForgeError> {
    let (approval, committed_approval, criteria, committed_criteria) = approval;
    promote_cell_changeset(
        policy,
        &h.repo(),
        &staging_dir(&h.run_dir(run)),
        request,
        PromotionApproval {
            approval,
            committed_approval,
            criteria,
            committed_criteria,
        },
        run,
        ITEM,
        stream,
    )
}

/// Another clone pushes a commit to main after the base was captured, so the
/// authoritative ref no longer names the base. Returns the new head.
fn advance_main(h: &Harness) -> String {
    let other = h.path().join("other");
    let sha = h.path().join("concurrent.sha");
    let snippet = format!(
        "export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@example.com GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@example.com; git clone -q '{repo}' '{other}' && cd '{other}' && echo changed > README.md && git add README.md && git commit -q -m concurrent && git push -q origin HEAD:refs/heads/main && git rev-parse HEAD > '{sha}'",
        repo = h.repo().display(),
        other = other.display(),
        sha = sha.display(),
    );
    // Empty environment for the same reason as `git` above: an inherited
    // GIT_DIR would redirect this clone-and-push at another repository.
    let status = Command::new("sh")
        .env_clear()
        .envs(std::env::var_os("PATH").map(|path| ("PATH", path)))
        .arg("-c")
        .arg(snippet)
        .status()
        .expect("sh must run");
    assert!(status.success(), "the concurrent push must succeed");
    fs::read_to_string(&sha)
        .expect("concurrent commit")
        .trim()
        .to_string()
}

/// The `opa` binary on PATH. Tests that need it are ignored by default.
fn real_opa() -> PathBuf {
    std::env::var_os("PATH")
        .and_then(|paths| {
            std::env::split_paths(&paths)
                .map(|dir| dir.join("opa"))
                .find(|candidate| candidate.is_file())
        })
        .expect("requires opa 1.21.1 on PATH")
}

fn opa_input(cli: PathBuf, input: Value) -> CellOpaInput {
    CellOpaInput {
        binding: OpaBinding {
            cli,
            timeout: Duration::from_secs(60),
        },
        input,
    }
}

/// An `opa` stand-in that answers `version` correctly and prints garbage for
/// any evaluation.
fn garbage_opa(dir: &Path) -> PathBuf {
    let path = dir.join("fake-opa");
    fs::write(
        &path,
        "#!/bin/sh\ncase \"$1\" in\n  version) echo 'Version: 1.21.1' ;;\n  *) echo 'not a decision' ;;\nesac\n",
    )
    .expect("write fake opa");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod fake opa");
    path
}

const REPAIR: &str = "printf 'print(\"v2\")\\n' > src/app.py";

// ── 1. An authorized repair is executed, settled and requested for promotion ──

#[test]
fn authorized_repair_is_executed_settled_and_promoted() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);
    let episode = h
        .episode(&stream, &policy, "run_repair", &["sh", "-c", REPAIR])
        .expect("episode");

    assert_eq!(episode.decision.verdict, Verdict::Allow);
    let execution = episode.execution.as_ref().expect("execution");
    assert_eq!(execution.status, ExecutionStatus::Completed);
    assert_eq!(execution.exit_code, Some(0));

    let changeset = episode.changeset.as_ref().expect("changeset");
    assert_eq!(changeset.entries.len(), 1);
    assert_eq!(changeset.entries[0].path, "src/app.py");
    assert!(matches!(
        changeset.entries[0].status,
        ChangeStatus::Modified
    ));
    assert_eq!(episode.validation, Some(Validation::Accept));
    assert_eq!(episode.settlement.status, SettlementStatus::Accepted);

    // The episode only requests promotion: the ref is still at the base.
    let repo = h.repo();
    let request = episode
        .promotion_request
        .as_ref()
        .expect("promotion request");
    assert_eq!(request.decision.verdict, Verdict::Escalate);
    assert_eq!(request.changeset.proposed_commit, changeset.proposed_commit);
    assert_eq!(main_ref(&repo), changeset.base.commit);

    // A human security officer approves, then the trusted path promotes.
    let approval = approve(&stream, request, "security_officer");
    let promotion = promote_with(&h, &stream, &policy, "run_repair", request, &approval)
        .expect("approved promotion");
    assert_eq!(promotion.current, changeset.proposed_commit);
    assert_eq!(main_ref(&repo), changeset.proposed_commit);
    assert!(git(&repo, &["show", "refs/heads/main:src/app.py"]).contains("v2"));

    assert_eq!(
        kinds(&stream),
        vec![
            "authority_decision",
            "openshell_execution",
            "workspace_changeset",
            "settlement_event",
            "authority_decision",
            "settlement_criteria",
            "approval_resolution",
            "approval_grant_consumption",
            "workspace_promotion",
        ]
    );
    stream.verify().expect("ledger verifies");

    let execution_record = &payloads(&stream, "openshell_execution")[0];
    assert_eq!(
        execution_record["sandbox"]["cell_id"],
        h.lock_str("/cell_id")
    );
    assert_eq!(
        execution_record["sandbox"]["projected_policy_digest"],
        h.lock_str("/policy_digest")
    );

    assert!(h.run_dir("run_repair").join("changeset.json").exists());
    let calls = h.calls();
    let download = calls.find("sandbox download").expect("download logged");
    let delete = calls.find("sandbox delete").expect("delete logged");
    assert!(delete > download, "the sandbox is deleted after download");
}

// ── 2. A denied execution creates no sandbox and touches nothing ──

#[test]
fn denied_execution_creates_no_sandbox_and_touches_nothing() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::ExecuteDenied);
    let before = snapshot(&h.repo());

    let episode = h
        .episode(
            &stream,
            &policy,
            "run_denied",
            &["sh", "-c", "printf 'x' > src/app.py"],
        )
        .expect("episode");

    assert_eq!(episode.decision.verdict, Verdict::Deny);
    assert!(episode.execution.is_none());
    assert!(episode.promotion_request.is_none());
    assert_eq!(episode.settlement.status, SettlementStatus::Rejected);
    assert!(h.calls().is_empty(), "no openshell process ran");
    assert!(!h.run_dir("run_denied").join("workspace").exists());
    assert_eq!(snapshot(&h.repo()), before);
    assert!(kinds(&stream).contains(&"authority_decision".to_string()));
}

// ── 3. A grant for a different policy digest is refused ──

#[test]
fn grant_for_a_different_policy_digest_is_refused() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::WrongDigest);
    let before = snapshot(&h.repo());

    let episode = h
        .episode(
            &stream,
            &policy,
            "run_digest",
            &["sh", "-c", "printf 'x' > src/app.py"],
        )
        .expect("the episode is decided");

    assert_ne!(episode.decision.verdict, Verdict::Allow);
    assert!(episode.execution.is_none());
    assert!(episode.promotion_request.is_none());
    assert!(!h.calls().contains("sandbox create"));
    assert_eq!(snapshot(&h.repo()), before);
}

// ── 4. A tampered projection is refused ──

#[test]
fn tampered_projection_is_refused() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);
    let before = snapshot(&h.repo());

    let projected = h.path().join("projection/policy.yaml");
    let mut text = fs::read_to_string(&projected).expect("projection policy");
    text.push_str("# tampered after the grant was written\n");
    fs::write(&projected, text).expect("tamper");

    let error = h
        .episode(
            &stream,
            &policy,
            "run_tamper",
            &["sh", "-c", "printf 'x' > src/app.py"],
        )
        .expect_err("the episode must be refused");

    assert!(
        error.to_string().contains("openshell_projection_tampered"),
        "unexpected error: {error}"
    );
    assert!(!h.calls().contains("sandbox create"));
    assert_eq!(snapshot(&h.repo()), before);
}

// ── 5. A protected dependency file escalates and is not promoted ──

#[test]
fn protected_dependency_file_change_escalates_and_is_not_promoted() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);
    let before = snapshot(&h.repo());

    let episode = h
        .episode(
            &stream,
            &policy,
            "run_protected",
            &["sh", "-c", "printf 'lock-v2\\n' > uv.lock"],
        )
        .expect("episode");

    match &episode.validation {
        Some(Validation::Escalate(findings)) => {
            assert!(findings.iter().any(|f| f.class == "protected_path"));
        }
        other => panic!("expected an escalation, got {other:?}"),
    }
    assert_eq!(episode.settlement.status, SettlementStatus::Escalated);
    assert!(episode.promotion_request.is_none());
    assert_eq!(snapshot(&h.repo()), before);

    let kinds = kinds(&stream);
    assert!(kinds.contains(&"workspace_changeset".to_string()));
    assert!(!kinds.contains(&"workspace_promotion".to_string()));

    let proposed = &episode
        .changeset
        .as_ref()
        .expect("changeset")
        .proposed_commit;
    let run_dir = h.run_dir("run_protected");
    let staging = run_dir.join("staging.git");
    let staging = staging.display().to_string();
    assert_eq!(
        git(
            &run_dir,
            &["--git-dir", &staging, "cat-file", "-t", proposed]
        ),
        "commit"
    );
}

// ── 6. A symlink in the result is rejected and not promoted ──

#[test]
fn symlink_in_result_is_rejected_and_not_promoted() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);
    let before = snapshot(&h.repo());

    let episode = h
        .episode(
            &stream,
            &policy,
            "run_symlink",
            &["sh", "-c", "ln -s /etc/passwd leak"],
        )
        .expect("episode");

    match &episode.validation {
        Some(Validation::Reject(findings)) => {
            assert!(findings.iter().any(|f| f.class == "symlink"));
        }
        other => panic!("expected a rejection, got {other:?}"),
    }
    assert_eq!(episode.settlement.status, SettlementStatus::Rejected);
    assert!(episode.promotion_request.is_none());
    assert_eq!(snapshot(&h.repo()), before);
}

// ── 7. A failed command is rejected and not promoted ──

#[test]
fn failed_command_is_rejected_and_not_promoted() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);
    let before = snapshot(&h.repo());

    let episode = h
        .episode(
            &stream,
            &policy,
            "run_failed",
            &["sh", "-c", "echo partial > src/app.py; exit 7"],
        )
        .expect("episode");

    let execution = episode.execution.as_ref().expect("execution");
    assert_eq!(execution.exit_code, Some(7));
    assert_eq!(episode.settlement.status, SettlementStatus::Rejected);
    assert!(episode.promotion_request.is_none());
    assert_eq!(snapshot(&h.repo()), before);
    assert!(h.run_dir("run_failed").join("changeset.json").exists());
}

// ── 8. A denied network access is recorded and rejected ──

#[test]
fn denied_network_access_is_recorded_and_rejected() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);
    let before = snapshot(&h.repo());
    fs::write(
        h.ctl("logs"),
        "[1775014132.690] [sandbox] [OCSF ] [ocsf] NET:OPEN [MED] DENIED /usr/bin/curl(64) -> evil.example.com:443 [policy:- engine:opa] [reason:endpoint not allowed]\n",
    )
    .expect("write logs");

    let episode = h
        .episode(&stream, &policy, "run_network", &["sh", "-c", "exit 6"])
        .expect("episode");

    let execution = episode.execution.as_ref().expect("execution");
    assert_eq!(execution.status, ExecutionStatus::SandboxViolation);
    assert_eq!(episode.settlement.status, SettlementStatus::Rejected);
    assert!(episode.promotion_request.is_none());

    let execution_record = &payloads(&stream, "openshell_execution")[0];
    let denials = execution_record["denials"].as_array().expect("denials");
    assert_eq!(denials.len(), 1);
    assert!(denials[0]
        .as_str()
        .expect("denial line")
        .contains("evil.example.com"));
    assert_eq!(snapshot(&h.repo()), before);
}

// ── 9. A stale base at commit time is a recorded conflict ──

#[test]
fn stale_base_at_commit_is_a_recorded_conflict() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);

    let episode = h
        .episode(&stream, &policy, "run_stale", &["sh", "-c", REPAIR])
        .expect("episode");
    assert_eq!(episode.settlement.status, SettlementStatus::Accepted);
    let request = episode
        .promotion_request
        .as_ref()
        .expect("promotion request");
    let approval = approve(&stream, request, "security_officer");

    // The base was captured before the approval. Another clone now moves main.
    let concurrent = advance_main(&h);

    let error = promote_with(&h, &stream, &policy, "run_stale", request, &approval)
        .expect_err("a stale base must not be overwritten");
    assert_eq!(error.class(), "stale_base");
    assert_eq!(main_ref(&h.repo()), concurrent);

    let promotions = payloads(&stream, "workspace_promotion");
    assert_eq!(promotions.len(), 1);
    assert_eq!(promotions[0]["error"]["class"], "stale_base");
}

// ── 10. Promotion without commit authority or approval is not performed ──

#[test]
fn promotion_without_commit_authority_is_not_performed() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::WithoutPromoteRule);
    let before = snapshot(&h.repo());

    let episode = h
        .episode(&stream, &policy, "run_no_commit", &["sh", "-c", REPAIR])
        .expect("episode");

    assert_eq!(episode.settlement.status, SettlementStatus::Accepted);
    assert!(episode.promotion_request.is_none());
    assert_eq!(snapshot(&h.repo()), before);

    let decisions = payloads(&stream, "authority_decision");
    assert_eq!(decisions.len(), 2);
    assert_ne!(decisions[1]["verdict"], "allow");

    // With a request but no approval, there is no way to promote.
    let policy = h.policy(Policy::Standard);
    let episode = h
        .episode(&stream, &policy, "run_pending", &["sh", "-c", REPAIR])
        .expect("episode");
    let request = episode
        .promotion_request
        .as_ref()
        .expect("promotion request");
    let pending = approve_with(
        &stream,
        request,
        "security_officer",
        ApprovalStatus::Pending,
    );
    let before = main_ref(&h.repo());
    promote_with(&h, &stream, &policy, "run_pending", request, &pending)
        .expect_err("a pending approval must not promote");
    assert_eq!(main_ref(&h.repo()), before);
    assert!(!has_kind(&stream, "workspace_promotion"));
}

// ── 11. An unavailable gateway fails before any execution ──

#[test]
fn unavailable_gateway_fails_before_any_execution() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);
    let before = snapshot(&h.repo());
    fs::write(h.ctl("gateway_down"), "").expect("gateway marker");

    let error = h
        .episode(
            &stream,
            &policy,
            "run_gateway",
            &["sh", "-c", "printf 'x' > src/app.py"],
        )
        .expect_err("the episode must be refused");

    assert!(
        error.to_string().contains("openshell_gateway_unavailable"),
        "unexpected error: {error}"
    );
    let calls = h.calls();
    assert!(!calls.contains("sandbox create"));
    assert!(!calls.contains("sandbox exec"));
    assert_eq!(snapshot(&h.repo()), before);
}

// ── 12. A provider-expanded effective policy is refused ──

#[test]
fn provider_expanded_effective_policy_is_refused() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);
    let before = snapshot(&h.repo());
    fs::write(
        h.ctl("prover_effective_json"),
        r#"{"schema_version":1,"result":"exceeds_boundary","exit_code":1,"coverage":{"domains":["filesystem","network_l4","network_rest","process","landlock"]},"counterexample":{"domain":"network","host":"extra.example.com","port":443},"reason":null}"#,
    )
    .expect("prover marker");

    let error = h
        .episode(
            &stream,
            &policy,
            "run_expanded",
            &["sh", "-c", "printf 'x' > src/app.py"],
        )
        .expect_err("the episode must be refused");

    assert!(
        error
            .to_string()
            .contains("openshell_effective_policy_exceeds_boundary"),
        "unexpected error: {error}"
    );
    let calls = h.calls();
    assert!(calls.contains("sandbox delete"));
    assert!(!calls.contains("sandbox exec"));
    assert_eq!(snapshot(&h.repo()), before);
}

// ── 13. No changes is accepted with nothing to promote ──

#[test]
fn no_changes_is_accepted_with_nothing_to_promote() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);
    let before = snapshot(&h.repo());

    let episode = h
        .episode(&stream, &policy, "run_noop", &["sh", "-c", "true"])
        .expect("episode");

    assert_eq!(episode.settlement.status, SettlementStatus::Accepted);
    let changeset = episode.changeset.as_ref().expect("changeset");
    assert!(changeset.entries.is_empty());
    assert!(episode.promotion_request.is_none());
    assert_eq!(snapshot(&h.repo()), before);
}

// ── N1. A host execute_command rule cannot authorize an OpenShell episode ──

#[test]
fn host_execute_command_rule_cannot_authorize_the_sandbox_command() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::HostRouteOpenshell);
    let before = snapshot(&h.repo());

    match h.episode(&stream, &policy, "run_host_route", &["sh", "-c", REPAIR]) {
        Err(_) => {}
        Ok(episode) => {
            assert_ne!(episode.decision.verdict, Verdict::Allow);
            assert!(episode.execution.is_none());
        }
    }
    assert!(h.calls().is_empty(), "no openshell process ran");
    assert_eq!(snapshot(&h.repo()), before);
}

// ── N2. A host execute_command grant (local class) does not authorize the sandbox ──

#[test]
fn host_local_grant_does_not_authorize_the_sandbox_command() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::HostLocalOnly);
    let before = snapshot(&h.repo());

    // The engine may refuse the policy at load (a host grant for a non-trusted
    // argv0), or decide the sandbox command without authorization.
    match h.episode(&stream, &policy, "run_host_local", &["sh", "-c", REPAIR]) {
        Err(_) => {}
        Ok(episode) => {
            assert_ne!(episode.decision.verdict, Verdict::Allow);
            assert_ne!(
                episode.decision.sandbox_class_granted.as_deref(),
                Some("openshell")
            );
            assert!(episode.execution.is_none());
        }
    }
    assert!(h.calls().is_empty(), "no openshell process ran");
    assert_eq!(snapshot(&h.repo()), before);
}

// ── N3. A rule pinning another sandbox image does not authorize the binding ──

#[test]
fn image_mismatch_is_not_authorized() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::WrongImage);
    let before = snapshot(&h.repo());

    let episode = h
        .episode(&stream, &policy, "run_image", &["sh", "-c", REPAIR])
        .expect("the episode is decided");

    assert_ne!(episode.decision.verdict, Verdict::Allow);
    assert!(episode.execution.is_none());
    assert!(episode.promotion_request.is_none());
    assert!(!h.calls().contains("sandbox create"));
    assert_eq!(snapshot(&h.repo()), before);
}

// ── N4. The requester cannot approve their own promotion ──

#[test]
fn self_approval_does_not_promote() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);
    let episode = h
        .episode(&stream, &policy, "run_self", &["sh", "-c", REPAIR])
        .expect("episode");
    let request = episode
        .promotion_request
        .as_ref()
        .expect("promotion request");
    let approval = approve(&stream, request, ENTITY);
    let before = main_ref(&h.repo());

    promote_with(&h, &stream, &policy, "run_self", request, &approval)
        .expect_err("self-approval must not promote");
    assert_eq!(main_ref(&h.repo()), before);
    assert!(!has_kind(&stream, "approval_grant_consumption"));
    assert!(!has_kind(&stream, "workspace_promotion"));
}

// ── N5. An approver without the security-officer role does not promote ──

#[test]
fn approver_without_the_sod_role_does_not_promote() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);
    let episode = h
        .episode(&stream, &policy, "run_sod", &["sh", "-c", REPAIR])
        .expect("episode");
    let request = episode
        .promotion_request
        .as_ref()
        .expect("promotion request");
    let approval = approve(&stream, request, "junior_reviewer");
    let before = main_ref(&h.repo());

    promote_with(&h, &stream, &policy, "run_sod", request, &approval)
        .expect_err("an approver without the approver role must not promote");
    assert_eq!(main_ref(&h.repo()), before);
    assert!(!has_kind(&stream, "approval_grant_consumption"));
    assert!(!has_kind(&stream, "workspace_promotion"));
}

// ── N6. A promotion request altered after the decision is refused, and the approval is kept ──

#[test]
fn altered_proposal_is_refused_without_consuming_the_approval() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);
    let episode = h
        .episode(&stream, &policy, "run_altered", &["sh", "-c", REPAIR])
        .expect("episode");
    let request = episode
        .promotion_request
        .as_ref()
        .expect("promotion request");
    let approval = approve(&stream, request, "security_officer");
    let base = request.changeset.base.commit.clone();
    let forged_tree = "f".repeat(40);

    // (a) The changeset was altered after the decision.
    let mut altered = request.clone();
    altered.changeset.proposed_tree = forged_tree.clone();
    let error = promote_with(&h, &stream, &policy, "run_altered", &altered, &approval)
        .expect_err("an altered changeset must not promote");
    assert_eq!(error.class(), "promotion_binding_mismatch");
    assert!(!has_kind(&stream, "approval_grant_consumption"));
    assert_eq!(main_ref(&h.repo()), base);

    // (b) The approved action was altered after the decision.
    let mut altered = request.clone();
    let AuthorityAction::PromoteChangeset {
        repository,
        reference,
        expected_base,
        proposed_commit,
        changeset_digest,
        paths,
        ..
    } = request.action.clone()
    else {
        panic!("the request must be a promote_changeset action");
    };
    altered.action = AuthorityAction::PromoteChangeset {
        repository,
        reference,
        expected_base,
        proposed_commit,
        proposed_tree: forged_tree,
        changeset_digest,
        paths,
    };
    let error = promote_with(&h, &stream, &policy, "run_altered", &altered, &approval)
        .expect_err("an altered action must not promote");
    assert_eq!(error.class(), "promotion_binding_mismatch");
    assert!(!has_kind(&stream, "approval_grant_consumption"));
    assert_eq!(main_ref(&h.repo()), base);

    // The untouched request still promotes with the same, unconsumed approval.
    promote_with(&h, &stream, &policy, "run_altered", request, &approval)
        .expect("the genuine request promotes with the same approval");
    assert_eq!(main_ref(&h.repo()), request.changeset.proposed_commit);
}

// ── N7. An approval grants exactly once ──

#[test]
fn replayed_approval_does_not_promote_again() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);
    let episode = h
        .episode(&stream, &policy, "run_replay", &["sh", "-c", REPAIR])
        .expect("episode");
    let request = episode
        .promotion_request
        .as_ref()
        .expect("promotion request");
    let approval = approve(&stream, request, "security_officer");

    promote_with(&h, &stream, &policy, "run_replay", request, &approval).expect("first promotion");
    let moved = main_ref(&h.repo());
    assert_eq!(moved, request.changeset.proposed_commit);

    let error = promote_with(&h, &stream, &policy, "run_replay", request, &approval)
        .expect_err("the same approval must not promote twice");
    assert!(
        error.to_string().contains("already granted"),
        "unexpected error: {error}"
    );
    assert_eq!(main_ref(&h.repo()), moved);
    assert_eq!(payloads(&stream, "workspace_promotion").len(), 1);
}

// ── N8. A promotion for a different repository is refused ──

#[test]
fn promotion_for_another_repository_is_refused() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);
    let episode = h
        .episode(&stream, &policy, "run_other_repo", &["sh", "-c", REPAIR])
        .expect("episode");
    let request = episode
        .promotion_request
        .as_ref()
        .expect("promotion request");
    let approval = approve(&stream, request, "security_officer");
    let base = main_ref(&h.repo());

    git(
        h.path(),
        &["clone", "-q", "--bare", "auth.git", "other.git"],
    );
    let (approval, committed_approval, criteria, committed_criteria) = &approval;
    let error = promote_cell_changeset(
        &policy,
        &h.path().join("other.git"),
        &staging_dir(&h.run_dir("run_other_repo")),
        request,
        PromotionApproval {
            approval,
            committed_approval,
            criteria,
            committed_criteria,
        },
        "run_other_repo",
        ITEM,
        &stream,
    )
    .expect_err("a different repository must not be promoted");
    assert_eq!(error.class(), "promotion_binding_mismatch");
    assert_eq!(main_ref(&h.repo()), base);
    assert!(!has_kind(&stream, "approval_grant_consumption"));
}

// ── N9. A rejected approval does not promote ──

#[test]
fn rejected_approval_does_not_promote() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);
    let episode = h
        .episode(&stream, &policy, "run_rejected", &["sh", "-c", REPAIR])
        .expect("episode");
    let request = episode
        .promotion_request
        .as_ref()
        .expect("promotion request");
    let approval = approve_with(
        &stream,
        request,
        "security_officer",
        ApprovalStatus::Rejected,
    );
    let before = main_ref(&h.repo());

    promote_with(&h, &stream, &policy, "run_rejected", request, &approval)
        .expect_err("a rejected approval must not promote");
    assert_eq!(main_ref(&h.repo()), before);
    assert!(!has_kind(&stream, "workspace_promotion"));
}

// ── N10. A rejected settlement or an escalated validation never requests promotion ──

#[test]
fn rejected_settlement_and_escalated_validation_request_nothing() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::Standard);

    let failed = h
        .episode(
            &stream,
            &policy,
            "run_n10_failed",
            &["sh", "-c", "echo partial > src/app.py; exit 7"],
        )
        .expect("episode");
    assert_eq!(failed.settlement.status, SettlementStatus::Rejected);
    assert!(failed.promotion_request.is_none());

    let protected = h
        .episode(
            &stream,
            &policy,
            "run_n10_protected",
            &["sh", "-c", "printf 'lock-v2\\n' > uv.lock"],
        )
        .expect("episode");
    assert_eq!(protected.settlement.status, SettlementStatus::Escalated);
    assert!(protected.promotion_request.is_none());
    let base = failed
        .changeset
        .as_ref()
        .expect("changeset")
        .base
        .commit
        .clone();
    assert_eq!(main_ref(&h.repo()), base);
}

// ── N11. OPA evaluation over the real Rego bundle ──

#[test]
#[ignore = "requires opa 1.21.1 on PATH"]
fn opa_allow_is_committed_before_the_sandbox_decision() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::WithOpa);
    let opa = opa_input(real_opa(), json!({"facts": {}}));

    let episode = h
        .run_episode(
            &stream,
            &policy,
            "run_opa_allow",
            &["sh", "-c", REPAIR],
            Some(opa),
            h.binding(Some(IMAGE)),
        )
        .expect("episode");

    assert_eq!(episode.decision.verdict, Verdict::Allow);
    assert_eq!(
        episode.execution.as_ref().expect("execution").status,
        ExecutionStatus::Completed
    );
    let order = kinds(&stream);
    let opa_at = order
        .iter()
        .position(|kind| kind == "opa_evaluation")
        .expect("opa evaluation recorded");
    let decision_at = order
        .iter()
        .position(|kind| kind == "authority_decision")
        .expect("authority decision recorded");
    assert!(
        opa_at < decision_at,
        "opa must precede the decision: {order:?}"
    );
    assert!(episode
        .decision
        .candidate_verdicts
        .iter()
        .any(|verdict| verdict.engine == "opa"));
    let trace = &payloads(&stream, "opa_evaluation")[0];
    assert_eq!(trace["outcome"], "allow");
    assert!(trace["failure"].is_null());
    stream.verify().expect("ledger verifies");
}

#[test]
#[ignore = "requires opa 1.21.1 on PATH"]
fn opa_input_without_facts_is_not_allowed() {
    // `dependency_files_immutable` is `"true"` in the generated policy for every
    // input, so real OPA cannot report a violation of it. The only deny reachable
    // from here is the malformed-input contract: `facts` must be an object. The
    // candidate layer rejects that before `opa` runs, so no sandbox is created.
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::WithOpa);
    let before = snapshot(&h.repo());
    let opa = opa_input(real_opa(), json!({"facts": []}));

    let episode = h
        .run_episode(
            &stream,
            &policy,
            "run_opa_deny",
            &["sh", "-c", REPAIR],
            Some(opa),
            h.binding(Some(IMAGE)),
        )
        .expect("the episode is decided");

    assert_ne!(episode.decision.verdict, Verdict::Allow);
    assert!(episode.execution.is_none());
    assert!(h.calls().is_empty(), "no openshell process ran");
    assert_eq!(snapshot(&h.repo()), before);
    let trace = &payloads(&stream, "opa_evaluation")[0];
    assert_eq!(trace["failure"], "opa_input_invalid");
}

#[test]
#[ignore = "requires opa 1.21.1 on PATH"]
fn required_opa_without_an_input_is_not_allowed() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::WithOpa);
    let before = snapshot(&h.repo());

    let episode = h
        .run_episode(
            &stream,
            &policy,
            "run_opa_none",
            &["sh", "-c", REPAIR],
            None,
            h.binding(Some(IMAGE)),
        )
        .expect("the episode is decided");

    assert_ne!(episode.decision.verdict, Verdict::Allow);
    assert!(episode.execution.is_none());
    assert!(!has_kind(&stream, "opa_evaluation"));
    assert!(h.calls().is_empty(), "no openshell process ran");
    assert_eq!(snapshot(&h.repo()), before);
}

// ── N12. A required OPA engine never yields Allow without a usable decision ──

#[test]
fn required_opa_without_a_usable_decision_is_not_allowed() {
    let h = Harness::new();
    let policy = h.policy(Policy::WithOpa);

    // (a) No candidate input at all.
    let stream = h.ledger();
    let before = snapshot(&h.repo());
    let episode = h
        .run_episode(
            &stream,
            &policy,
            "run_opa_missing",
            &["sh", "-c", REPAIR],
            None,
            h.binding(Some(IMAGE)),
        )
        .expect("the episode is decided");
    assert_ne!(episode.decision.verdict, Verdict::Allow);
    assert!(episode.execution.is_none());
    assert!(!h.calls().contains("sandbox create"));
    assert_eq!(snapshot(&h.repo()), before);

    // (b) An `opa` binary that answers with garbage.
    let fake = garbage_opa(h.path());
    let opa = opa_input(fake, json!({"facts": {}}));
    let episode = h
        .run_episode(
            &stream,
            &policy,
            "run_opa_garbage",
            &["sh", "-c", REPAIR],
            Some(opa),
            h.binding(Some(IMAGE)),
        )
        .expect("the episode is decided");
    assert_ne!(episode.decision.verdict, Verdict::Allow);
    assert!(episode.execution.is_none());
    assert!(!h.calls().contains("sandbox create"));
    assert_eq!(snapshot(&h.repo()), before);
    let trace = &payloads(&stream, "opa_evaluation")[0];
    assert!(trace["failure"].is_string(), "the failure is recorded");
}

// ── N13. Real OPA over the gate fixture denies or escalates before any sandbox ──

/// The ledger opens with the OPA trace and then the authority decision, and
/// records no sandbox execution.
fn assert_opa_decides_without_a_sandbox(stream: &LedgerStream) {
    let order = kinds(stream);
    assert!(
        order.len() >= 2 && order[..2] == ["opa_evaluation", "authority_decision"],
        "opa precedes the decision: {order:?}"
    );
    assert!(!has_kind(stream, "openshell_execution"), "{order:?}");
}

#[test]
#[ignore = "requires opa 1.21.1 on PATH"]
fn real_opa_deny_blocks_the_episode_before_any_sandbox() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::WithGateOpa);
    let before = snapshot(&h.repo());
    let opa = opa_input(real_opa(), gate_input("deny"));

    let episode = h
        .run_episode(
            &stream,
            &policy,
            "run_gate_deny",
            &["sh", "-c", REPAIR],
            Some(opa),
            h.binding(Some(IMAGE)),
        )
        .expect("the episode is decided");

    assert_eq!(episode.decision.verdict, Verdict::Deny);
    assert!(episode.execution.is_none());
    assert!(h.calls().is_empty(), "no openshell process ran");
    assert_eq!(snapshot(&h.repo()), before);
    assert_opa_decides_without_a_sandbox(&stream);
    let trace = &payloads(&stream, "opa_evaluation")[0];
    assert_eq!(trace["outcome"], "deny");
    assert!(trace["failure"].is_null());
    stream.verify().expect("ledger verifies");
}

#[test]
#[ignore = "requires opa 1.21.1 on PATH"]
fn real_opa_escalation_blocks_the_episode_before_any_sandbox() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::WithGateOpa);
    let before = snapshot(&h.repo());
    let opa = opa_input(real_opa(), gate_input("escalate"));

    let episode = h
        .run_episode(
            &stream,
            &policy,
            "run_gate_escalate",
            &["sh", "-c", REPAIR],
            Some(opa),
            h.binding(Some(IMAGE)),
        )
        .expect("the episode is decided");

    assert_eq!(episode.decision.verdict, Verdict::Escalate);
    assert!(episode.execution.is_none());
    assert!(h.calls().is_empty(), "no openshell process ran");
    assert_eq!(snapshot(&h.repo()), before);
    assert_opa_decides_without_a_sandbox(&stream);
    let trace = &payloads(&stream, "opa_evaluation")[0];
    assert_eq!(trace["outcome"], "escalate");
    assert!(trace["failure"].is_null());
    stream.verify().expect("ledger verifies");
}

#[test]
#[ignore = "requires opa 1.21.1 on PATH"]
fn opa_configured_episode_can_still_request_and_complete_an_approved_promotion() {
    let h = Harness::new();
    let stream = h.ledger();
    let policy = h.policy(Policy::WithGateOpa);
    let opa = opa_input(real_opa(), gate_input("allow"));

    let episode = h
        .run_episode(
            &stream,
            &policy,
            "run_gate_promote",
            &["sh", "-c", REPAIR],
            Some(opa),
            h.binding(Some(IMAGE)),
        )
        .expect("episode");

    assert_eq!(episode.decision.verdict, Verdict::Allow);
    assert_eq!(
        episode.execution.as_ref().expect("execution").status,
        ExecutionStatus::Completed
    );
    let changeset = episode.changeset.as_ref().expect("changeset");
    let repo = h.repo();
    let request = episode
        .promotion_request
        .as_ref()
        .expect("promotion request");
    assert_eq!(request.decision.verdict, Verdict::Escalate);
    assert!(request
        .decision
        .candidate_verdicts
        .iter()
        .any(|verdict| verdict.engine == "opa"));
    assert_eq!(main_ref(&repo), changeset.base.commit);

    let approval = approve(&stream, request, "security_officer");
    let promotion = promote_with(&h, &stream, &policy, "run_gate_promote", request, &approval)
        .expect("approved promotion");
    assert_eq!(promotion.current, changeset.proposed_commit);
    assert_eq!(main_ref(&repo), changeset.proposed_commit);
    stream.verify().expect("ledger verifies");
}
