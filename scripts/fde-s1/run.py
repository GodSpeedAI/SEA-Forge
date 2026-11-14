#!/usr/bin/env python3
"""S1 gate episode run (WP-19, SEA-Forge half). Use scripts/fde-s1-run.sh.

Runs the three `program` episodes of .agents/preregistrations/fde-s1/episodes.yaml through the stack in a
fresh cell and keeps every raw record. Real producers only: the SEA-Forge server (via the trusted-issuer
driver, case_record_append / authority_* verbs), Cognate `program.run` (bun), RealityTrace CLI
(`observed ingest|derive|compare|propose|regression|capability-delta`), DomainForge CLI. This file only
sequences them and writes the inputs they take (reference, profile, mapping, frames). Every step is logged
to <cell>/steps.jsonl with its command, exit code and duration. A step that cannot be done is recorded as a
gap in <cell>/gaps.jsonl and the dependent test results say so; nothing is edited to make a step pass.

Per-test cell checks are written to <cell>/results/<ID>.json ({"status": "ok"|"fail", ...}).
"""
import hashlib, json, os, shutil, subprocess, sys, time, uuid, datetime, pathlib, glob, re

ROOT = pathlib.Path(__file__).resolve().parents[2]
HOME = pathlib.Path.home()
WT = HOME / "projects/_wt"
RT_DIR = pathlib.Path(os.environ.get("FDE_REALITYTRACE_DIR", WT / "fde-sxr-integration"))
COG_DIR = pathlib.Path(os.environ.get("FDE_COGNATE_DIR", WT / "fde-cognate-integration"))
DF_DEV = os.environ.get("FDE_DOMAINFORGE_DEV_BIN", str(WT / "tmp/df-3960326/domainforge"))
DF_REL = os.environ.get("FDE_DOMAINFORGE_BIN", str(WT / "tmp/df022-cli/domainforge-0.22.0-x86_64-unknown-linux-gnu/domainforge"))
RT = os.environ.get("FDE_REALITYTRACE_BIN", str(RT_DIR / "target/debug/realitytrace"))
RT_SEED = os.environ.get("FDE_RT_SEED_BIN", str(WT / "tmp/rt-seed-target/debug/rt-seed"))
DF_REPO = HOME / "projects/domainforge"
DF_TARGET = WT / "fde-df-main-3960326/target"
SF_TARGET = ROOT / "target"
DRIVER = SF_TARGET / "debug/examples/fde_s1_driver"
WORLD_NAME = "godspeed-fde-sop"
CASE = "case-0001"
LOCK = os.environ.get("FDE_CARGO_LOCK", str(WT / "cargo.lock"))
NS = uuid.NAMESPACE_DNS
CARGO_ENV = {"RUSTC_WRAPPER": "", "CARGO_BUILD_JOBS": "1", "CARGO_INCREMENTAL": "0",
             "CARGO_PROFILE_DEV_DEBUG": "0", "CARGO_PROFILE_TEST_DEBUG": "0"}


def now(): return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
def canon(v): return json.dumps(v, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
def sha(v): return hashlib.sha256(canon(v).encode()).hexdigest()
def sha_bytes(b): return hashlib.sha256(b).hexdigest()
def sha_file(p): return sha_bytes(pathlib.Path(p).read_bytes())
def key(ns, name): return "concept:" + str(uuid.uuid5(NS, f"{ns}::{name}"))


class Cell:
    def __init__(self, path):
        self.p = pathlib.Path(path)
        for d in ("raw", "results", "work", "ws", "in"):
            (self.p / d).mkdir(parents=True, exist_ok=True)
        self.sf = self.p / "sf-root"
        self.rthome = self.p / "rt-home"
        self.state_f = self.p / "state.json"
        self.state = json.loads(self.state_f.read_text()) if self.state_f.exists() else {}

    def save(self): self.state_f.write_text(json.dumps(self.state, indent=1, sort_keys=True))

    def step(self, name, argv, env=None, cwd=None, out=None, check=True, stdin=None, timeout=None):
        t0 = time.time()
        e = dict(os.environ); e.update(env or {})
        p = subprocess.run(argv, env=e, cwd=cwd, capture_output=True, text=True, input=stdin, timeout=timeout)
        rec = {"step": name, "argv": [str(a) for a in argv], "rc": p.returncode, "start": datetime.datetime.fromtimestamp(t0, datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
               "duration_s": round(time.time() - t0, 2), "stdout_sha256": sha_bytes(p.stdout.encode()), "stderr_tail": p.stderr[-400:]}
        with open(self.p / "steps.jsonl", "a") as f: f.write(json.dumps(rec) + "\n")
        if out: pathlib.Path(out).write_text(p.stdout)
        if check and p.returncode != 0:
            raise RuntimeError(f"step {name} failed rc={p.returncode}: {p.stderr[-800:]}")
        return p

    def gap(self, test, text):
        with open(self.p / "gaps.jsonl", "a") as f: f.write(json.dumps({"tests": test, "gap": text}) + "\n")

    def result(self, tid, status, **kw):
        (self.p / "results" / f"{tid}.json").write_text(json.dumps({"id": tid, "status": status, **kw}, indent=1, sort_keys=True) + "\n")


def cargo(c, name, args, cwd, env=None, **kw):
    e = dict(CARGO_ENV); e.update(env or {})
    return c.step(name, ["flock", "-o", LOCK, "cargo"] + args, env=e, cwd=cwd, **kw)


# ------------------------------------------------------------------ models and worlds
def declared_keys(rep_json):
    d = json.loads(rep_json)
    return sorted("concept:" + x["id"]["data"]["id"] for x in d["envelope"]["semantic_declarations"] if x["id"]["kind"] == "concept")


def setup_models(c):
    src = ROOT / ".sea/interaction/fde-sop"
    for v in ("v1", "v2"):
        d = c.sf / "worlds" / v
        d.mkdir(parents=True, exist_ok=True)
        for f in ("fde-sop.sea", "evidence.sea", "outcome.sea"): shutil.copy(src / f, d / f)
    c.step("model-v2-patch", ["patch", "-p1", "-s", "-i", str(ROOT / "scripts/fde-s1/model/sop-v2-fresh-context-verification.patch")], cwd=c.sf / "worlds/v2")
    worlds = {}
    for v in ("v1", "v2"):
        d = c.sf / "worlds" / v
        c.step(f"model-{v}-validate", [DF_DEV, "validate", "--application", "fde-sop.sea"], cwd=d, out=c.p / "raw" / f"model-{v}-validate.txt")
        c.step(f"model-{v}-validate-0.22.0", [DF_REL, "validate", "--application", "fde-sop.sea"], cwd=d, out=c.p / "raw" / f"model-{v}-validate-0.22.0.txt")
        env = c.step(f"model-{v}-envelope", [DF_DEV, "envelope", "--emit", "cep", "--world-name", WORLD_NAME, "fde-sop.sea"], cwd=d, out=c.p / "raw" / f"model-{v}-envelope.json")
        env2 = c.step(f"model-{v}-envelope-again", [DF_DEV, "envelope", "--emit", "cep", "--world-name", WORLD_NAME, "fde-sop.sea"], cwd=d)
        ref = json.loads(env.stdout)["scope"]["world_ref"]
        assert ref == json.loads(env2.stdout)["scope"]["world_ref"], "world_ref is not reproducible"
        env22 = c.step(f"model-{v}-envelope-0.22.0", [DF_REL, "envelope", "--emit", "cep", "--world-name", WORLD_NAME, "fde-sop.sea"], cwd=d)
        rep = c.step(f"model-{v}-representation", [DF_DEV, "envelope", "--emit", "representation", "--world-name", WORLD_NAME, "fde-sop.sea"], cwd=d, out=c.p / "raw" / f"model-{v}-representation.json")
        keys = declared_keys(rep.stdout)
        (c.p / "in" / f"declared-{v}.json").write_text(json.dumps(keys))
        worlds[v] = {"world_ref": ref, "world_ref_df_0_22_0": json.loads(env22.stdout)["scope"]["world_ref"], "declared_keys": len(keys),
                     "source_sha256": {f: sha_file(d / f) for f in ("fde-sop.sea", "evidence.sea", "outcome.sea")}}
    # semantic diff (no DomainForge diff command exists): the diff of the two canonical representations
    r1 = json.loads((c.p / "raw/model-v1-representation.json").read_text()); r2 = json.loads((c.p / "raw/model-v2-representation.json").read_text())
    k1 = {canon(x["id"]): x for x in r1["envelope"]["semantic_declarations"]}
    k2 = {canon(x["id"]): x for x in r2["envelope"]["semantic_declarations"]}
    diff = {"added": sorted(k for k in k2 if k not in k1), "removed": sorted(k for k in k1 if k not in k2), "changed": sorted(k for k in k1 if k in k2 and k1[k] != k2[k])}
    (c.p / "raw/semantic-diff.json").write_text(json.dumps(diff, indent=1, sort_keys=True))
    worlds["semantic_diff_sha256"] = "sha256:" + sha(diff)
    c.state["worlds"] = worlds
    c.state["keys"] = {"Episode": key(WORLD_NAME, "Episode"), "Assignment": key(WORLD_NAME, "Assignment"),
                       "FreshContextVerification": key(WORLD_NAME, "FreshContextVerification"), "Settlement": key(WORLD_NAME + ".evidence", "Settlement")}
    c.save()


# ------------------------------------------------------------------ SEA-Forge cell, anchor repo, RealityTrace repo
ACTORS = [("case-keeper", "operator"), ("program-executor", "agent"), ("fresh-verifier", "operator"), ("independent-settler", "operator"),
          ("cognate-service", "service"), ("op-alice", "operator")]
POLICY = ("version: \"0.1\"\nrules:\n  - name: moves\n    verdict: allow\n    actor_role: service\n    operation_kind: world_transition\n"
          "  - name: approvers\n    verdict: allow\n    actor_role: operator\n    operation_kind: approval_resolution\n")


def setup_sf(c):
    sf = c.sf
    (sf / "cases" / CASE).mkdir(parents=True, exist_ok=True)
    (sf / "cases" / CASE / "case.json").write_text(json.dumps({"case_id": CASE, "template_ref": "fde.field_case@1", "title": "FC-001 Close recorded SEA-Forge and DomainForge debt items"}))
    (sf / "cases" / CASE / "case-events.jsonl").write_text("")
    (sf / "sea-forge-policy.yaml").write_text(POLICY)
    (sf / "realitytrace").mkdir(exist_ok=True)
    world = lambda base: {"name": WORLD_NAME, "base": base, "entry": "fde-sop.sea", "files": ["fde-sop.sea", "evidence.sea", "outcome.sea"], "prior_identities": []}
    uid = os.getuid()
    cfg = {"socket_path": str(c.p / "sf.sock"), "root": str(sf),
           "identity": {"bindings": [{"uid": uid if i == 0 else None, "sid": None, "actor_id": a, "roles": [r]} for i, (a, r) in enumerate(ACTORS)]},
           "cep_authority": {"enabled": True, "pinned_baseline": WORLD_NAME, "policy": "sea-forge-policy.yaml", "worlds": [world("worlds/v1"), world("worlds/v2")], "approval_ttl_hours": 24}}
    (c.p / "in/server-config.json").write_text(json.dumps(cfg, indent=1))
    # anchor repo: a scratch git repository with a local bare remote; the script is the repo's own scripts/fde-anchor.sh
    ar = c.p / "anchor-repo"; remote = c.p / "anchor-remote.git"
    (ar / "scripts").mkdir(parents=True, exist_ok=True)
    shutil.copy(ROOT / "scripts/fde-anchor.sh", ar / "scripts/fde-anchor.sh")
    gitenv = {"GIT_AUTHOR_NAME": "Sam Prime", "GIT_COMMITTER_NAME": "Sam Prime", "GIT_AUTHOR_EMAIL": "fde-s1@invalid", "GIT_COMMITTER_EMAIL": "fde-s1@invalid",
              "GIT_CONFIG_COUNT": "2", "GIT_CONFIG_KEY_0": "user.name", "GIT_CONFIG_VALUE_0": "Sam Prime", "GIT_CONFIG_KEY_1": "user.email", "GIT_CONFIG_VALUE_1": "fde-s1@invalid"}
    c.state["gitenv"] = gitenv
    c.step("anchor-remote-init", ["git", "init", "-q", "--bare", "-b", "main", str(remote)])
    for argv in (["git", "init", "-q", "-b", "main"], ["git", "add", "-A"], ["git", "commit", "-q", "-m", "anchor repo"],
                 ["git", "remote", "add", "origin", str(remote)], ["git", "push", "-q", "origin", "main"], ["git", "fetch", "-q", "origin"],
                 ["git", "branch", "-q", "--set-upstream-to=origin/main", "main"]):
        c.step("anchor-repo-" + argv[1], argv, cwd=ar, env=gitenv)
    c.state["anchor_script"] = str(ar / "scripts/fde-anchor.sh")
    # RealityTrace repo + enrolment
    rtrepo = c.p / "rt-workrepo"; rtrepo.mkdir(exist_ok=True)
    (rtrepo / "README").write_text("S1 cell RealityTrace repository\n")
    for argv in (["git", "init", "-q"], ["git", "add", "-A"], ["git", "commit", "-q", "-m", "init"]):
        c.step("rt-repo-" + argv[1], argv, cwd=rtrepo, env=gitenv)
    renv = rt_env(c)
    p = c.step("rt-enroll", [RT, "repo", "enroll", str(rtrepo), "--json"], env=renv)
    c.state["rt_repo_id"] = json.loads(p.stdout)["repo_id"]
    c.state["rt_ledger"] = str(c.rthome / "data/sxr/repos" / c.state["rt_repo_id"] / "ledger.db")
    c.save()


def rt_env(c):
    e = {"XDG_DATA_HOME": str(c.rthome / "data"), "XDG_STATE_HOME": str(c.rthome / "state")}
    return e


def driver(c, name, frames):
    fr = c.p / "in" / f"frames-{name}.ndjson"
    fr.write_text("".join(json.dumps(f) + "\n" for f in frames))
    resp = c.p / "raw/sf-responses" / f"{name}.ndjson"
    resp.parent.mkdir(exist_ok=True)
    env = {"SEA_FORGE_FDE_ANCHOR_SCRIPT": c.state["anchor_script"], "SEA_FORGE_FDE_ANCHOR_TIMEOUT_SECS": "120"}
    env.update(c.state["gitenv"])
    c.step(f"sf-{name}", [str(DRIVER), str(c.p / "in/server-config.json"), str(fr), str(resp)], env=env, timeout=900)
    return [json.loads(l)["response"] for l in resp.read_text().splitlines()][-len(frames):]


def actor(a): return {"actor_id": a, "role": dict(ACTORS)[a]}


# ------------------------------------------------------------------ comparison inputs
def profile():
    def entry(kind, comparison, extra):
        m = {"true": "confirmed", "false": "contradicted", "unknown": "insufficient"}; m.update(extra)
        return {"kind": kind, "comparison": comparison, "quantifier": "universal", "min_support": 1, "allow_vacuous_truth": False, "deadline": None, "class_mapping": m}
    return {"contract": "comparison-profile.v1", "profile_id": "profile:s1", "kinds": [
        entry("policy_evaluable", "episode", {}), entry("structural", "structural", {"undeclared": "observed_undeclared"})]}


def mapping(c, world):
    k = c.state["keys"]
    # The facts for `nav_move` are drawn from the SEA-Forge freeze record (an emitter independent of the executor):
    # a freeze is evidenced by its own record. Evidence from the executor's emitter alone cannot confirm (executor_only_evidence).
    nav_move = {"node": k["Episode"], "input": {
        "assignment_id": {"kind": "string", "pointer": "/source/record_id"},
        "case_id": {"kind": "string", "pointer": "/context/case_id"},
        "episode_ref": {"kind": "string", "pointer": "/context/episode_ref"},
        "episode_kind": {"typed": {"kind": "enum", "data": {"symbol": "godspeed-fde-sop.enum.EpisodeKind", "wire": "evaluated"}}},
        "freeze_recorded": {"kind": "bool", "nonempty": "/context/episode_ref"},
        "consequential": {"kind": "bool", "const": True}}}
    exps = [{"id": "freeze_before_move", "kind": "policy_evaluable", "policy": "freeze_before_move", "operation": "nav_move", "node": k["Episode"], "question": "question:q1"},
            {"id": "authorize_consequential_action", "kind": "policy_evaluable", "policy": "authorize_consequential_action", "operation": "nav_move", "node": k["Episode"], "question": "question:q1"}]
    ops = {"nav_move": nav_move}
    if world == "v2":
        ops["nav_verify"] = {"node": k["FreshContextVerification"], "input": {"verifier_role": {"kind": "string", "pointer": "/actor/role"}}}
        exps.append({"id": "verification_by_independent_role", "kind": "policy_evaluable", "policy": "verification_by_independent_role", "operation": "nav_verify",
                     "node": k["FreshContextVerification"], "question": "question:q1"})
    return {"contract": "compare-mapping.v1", "undeclared_question": "question:q1", "operations": ops, "expectations": exps}


# ------------------------------------------------------------------ episodes
import yaml
PREREG = ROOT / ".agents/preregistrations/fde-s1"
EPS = {e["id"]: e for e in yaml.safe_load((PREREG / "episodes.yaml").read_text())["episodes"]}
EP_REPO = {"ep-1-control": "sea-rs", "ep-2-inquiry": "domainforge", "ep-3-subsequent": "sea-rs"}
EP_PATCH = {"ep-1-control": "ep-1-events-tail.patch", "ep-2-inquiry": "ep-2-kg-roundtrip.patch", "ep-3-subsequent": "ep-3-readiness-stale.patch"}
CARGO_PFX = ["flock", "-o", LOCK, "env", "RUSTC_WRAPPER=", "CARGO_BUILD_JOBS=1", "CARGO_INCREMENTAL=0", "CARGO_PROFILE_DEV_DEBUG=0", "CARGO_PROFILE_TEST_DEBUG=0"]
EP_ARGV = {  # the declared commands of episodes.yaml, as argv, run under the low-memory cargo environment
    "ep-1-control": CARGO_PFX + ["cargo", "test", "--locked", "-p", "sea-forge-server", "--test", "conformance_lifecycle"],
    "ep-2-inquiry": CARGO_PFX + ["cargo", "test", "-p", "domainforge-core", "--features", "cli", "kg_round_trip_policy_and_instance"],
    "ep-3-subsequent": CARGO_PFX + ["cargo", "test", "--locked", "-p", "sea-forge-server", "readiness"],
}
EP_WORLD = {"ep-1-control": "v1", "ep-2-inquiry": "v1", "ep-3-subsequent": "v2"}


def ulid_like(prefix, seed): return f"{prefix}:01J{sha_bytes(seed.encode()).upper()[:23]}"


def extract_repo(c, which, dest):
    shutil.rmtree(dest, ignore_errors=True); dest.mkdir(parents=True)
    if which == "sea-rs":
        argv = ["git", "-C", str(ROOT), "archive", "HEAD"]; commit = c.step("rev-sea-rs", ["git", "-C", str(ROOT), "rev-parse", "HEAD"]).stdout.strip()
    else:
        argv = ["git", "-C", str(DF_REPO), "archive", "origin/main"]; commit = c.step("rev-df", ["git", "-C", str(DF_REPO), "rev-parse", "origin/main"]).stdout.strip()
    a = subprocess.run(argv, capture_output=True, check=True)
    subprocess.run(["tar", "-x", "-C", str(dest)], input=a.stdout, check=True)
    return commit


def freeze_episode(c, ep_id):
    ep = EPS[ep_id]; item = ep["plan_item_id"]; world = EP_WORLD[ep_id]
    wref = c.state["worlds"][world]["world_ref"]
    patch = ROOT / "scripts/fde-s1/patches" / EP_PATCH[ep_id]
    psha = sha_file(patch)
    prof = profile(); (c.p / "in/profile.json").write_text(json.dumps(prof, indent=1))
    body = {"case_id": CASE, "plan_item_id": item, "n": ep["n"], "world_ref": wref, "kind": ep["kind"], "executor_provenance": ep["executor_provenance"],
            "criteria": {"intention": ep["intention"], "expected_change": ep["expected_change"],
                         "invariants": list(ep["invariants"]) + [f"Executor program applies the hash-pinned patch sha256:{psha} ({EP_PATCH[ep_id]}) and runs the declared command: {ep['inputs']['declared_command']}"],
                         "predicted_irrelevancies": ep["predicted_irrelevancies"], "success_region": ep["success_region"], "failure_region": ep["failure_region"],
                         "falsifiers": ep["falsifiers"]},
            "comparison_profile": {"id": "profile:s1", "sha256": sha(prof)},
            "decision_maker": {"principal": "independent-settler", "role": "verifier"},
            "evidence_sources": [{"source_id": "src-case-ledger", "system": "sea-forge.case-ledger", "purpose": "freeze and settlement records"},
                                 {"source_id": "src-cognate", "system": "cognate", "purpose": "run and invocation records of the executor and the verifier"}],
            "coverage_manifest": [{"emitter": "sea-forge.case-ledger", "event_kinds": ["nav_aim", "nav_claim"]},
                                  {"emitter": "cognate", "event_kinds": ["program.run"]}]}
    digest = sha(body)
    eref = f"episode:{CASE}:{item}:{ep['n']}@sha256:{digest}"
    refs = {"plan_id": ulid_like("plan", ep_id), "question_ids": [ulid_like("question", ep_id + "q")], "threshold_ids": [ulid_like("threshold", ep_id + "t")]}
    with open(c.sf / "realitytrace/records.jsonl", "a") as f:
        for i in [refs["plan_id"]] + refs["question_ids"] + refs["threshold_ids"]: f.write(json.dumps({"record_id": i}) + "\n")
    reference = {"episode_ref": eref, "body": body}
    (c.p / "in" / f"reference-{ep_id}.json").write_text(json.dumps(reference, indent=1))
    frame = {"verb": "case_record_append", "case_id": CASE, "kind": "episode.frozen", "idempotency_key": f"case-keeper/{ep_id}", "request_id": f"freeze-{ep_id}",
             "payload": {"episode_ref": eref, "reference": reference, "digest": digest, "realitytrace_refs": refs}, "actor": actor("case-keeper")}
    r = driver(c, f"freeze-{ep_id}", [{"actor": actor("case-keeper"), "frame": frame}])[0]
    c.state.setdefault("episodes", {})[ep_id] = {"episode_ref": eref, "item": item, "world": world, "patch_sha256": psha, "question": refs["question_ids"][0],
                                                  "freeze_response": r}
    c.save()
    if r.get("outcome") != "recorded": raise RuntimeError(f"freeze refused: {r}")
    return r


def cognate_run(c, ep_id, role):
    ep = EPS[ep_id]; st = c.state["episodes"][ep_id]
    which = EP_REPO[ep_id]; ws = c.p / "ws" / which
    commit = extract_repo(c, which, ws)
    st[f"{role}_base_commit"] = commit
    env = {}
    if which == "sea-rs": env["CARGO_TARGET_DIR"] = str(SF_TARGET)
    else: env["CARGO_TARGET_DIR"] = str(DF_TARGET)
    sem = "Assignment" if role == "executor" else "FreshContextVerification"
    who = "program-executor" if role == "executor" else "fresh-verifier"
    out = c.p / "raw/cognate" / f"{ep_id}-{role}"
    argv = json.loads(os.environ["FDE_S1_DEBUG_ARGV"]) if os.environ.get("FDE_S1_DEBUG_ARGV") else EP_ARGV[ep_id]
    if os.environ.get("FDE_S1_DEBUG_ARGV"): c.state["debug_argv"] = True
    spec = {"cognateDir": str(COG_DIR), "outDir": str(out), "actor": who, "role": role, "semanticRef": f"{WORLD_NAME}::{sem}", "worldRef": c.state["worlds"][EP_WORLD[ep_id]]["world_ref"],
            "caseId": CASE, "planItemId": st["item"], "kind": "evaluated", "episodeRef": st["episode_ref"], "assignmentId": f"assign:{CASE}:{st['item']}:{ep['n']}:{1 if role == 'executor' else 2}",
            "expectationRefs": [st["question"]], "workspace": str(ws), "patchFile": str(ROOT / "scripts/fde-s1/patches" / EP_PATCH[ep_id]), "argv": argv}
    sp = c.p / "in" / f"cognate-{ep_id}-{role}.json"; sp.write_text(json.dumps(spec, indent=1))
    p = c.step(f"cognate-{ep_id}-{role}", ["bun", str(ROOT / "scripts/fde-s1/cognate_run.ts"), str(sp)], env=env, cwd=COG_DIR, check=False, out=c.p / "raw" / f"cognate-{ep_id}-{role}.stdout")
    view = json.loads((out / "run-view.json").read_text())["view"]
    st[f"{role}_run"] = {"status": view.get("status"), "rc": p.returncode, "runId": view.get("runId")}
    # the case engine's own event line naming the plan item and its executor (the I-2 source); SEA-Forge's engine does not write it for program runs
    with open(c.sf / "cases" / CASE / "case-events.jsonl", "a") as f:
        f.write(json.dumps({"event_id": f"cog:{ep_id}:{role}", "plan_item_id": st["item"], "actor_id": who, "kind": f"program.run.{role}", "recorded_at": now()}) + "\n")
    c.save()
    return view


# ------------------------------------------------------------------ RealityTrace: ingest, derive, compare
def rt(c, name, args, out=None, check=True):
    return c.step(name, [RT] + args, env=rt_env(c), out=out, check=check)


def ingest_new(c, label):
    """Append the new lines of every producer spool to the RealityTrace ledger via `observed ingest` (evidence rows seeded first)."""
    srcs = [c.sf / "interaction-events/case-0001.jsonl"] + sorted((c.p / "raw/cognate").glob("*/interaction-events.ndjson"))
    done = c.state.setdefault("ingested", {})
    batch = []
    for f in srcs:
        lines = [l for l in f.read_text().splitlines() if l.strip()] if f.exists() else []
        for l in lines[done.get(str(f), 0):]: batch.append(l)
        done[str(f)] = len(lines)
    if not batch: return {"ingested": 0}
    ids = sorted({json.loads(l)["extensions"]["godspeed.interaction_event"]["source"]["record_id"] for l in batch})
    c.step(f"rt-seed-{label}", [RT_SEED, c.state["rt_ledger"], c.state["rt_repo_id"]] + ids, env=rt_env(c))
    bf = c.p / "raw/rt" / f"ingest-{label}.ndjson"; bf.parent.mkdir(exist_ok=True)
    bf.write_text("\n".join(batch) + "\n")
    p = rt(c, f"rt-ingest-{label}", ["observed", "ingest", "--repo", c.state["rt_repo_id"], "--file", str(bf)], out=c.p / "raw/rt" / f"ingest-{label}.out.json", check=False)
    try:
        res = json.loads((c.p / "raw/rt" / f"ingest-{label}.out.json").read_text())["ingest"]["results"]
        c.state.setdefault("last_ordinal", {})[label] = max(r["position"]["append_ordinal"] for r in res if "position" in r)
        c.state.setdefault("ingest_status", {})[label] = {s: sum(1 for r in res if r.get("status") == s) for s in {r.get("status") for r in res}}
    except Exception as e:  # recorded, not hidden
        c.state.setdefault("ingest_status", {})[label] = {"unparsed": str(e)}
    c.save()
    return {"ingested": len(batch), "rc": p.returncode}


def coverage_rows(c, ep_id):
    ref = json.loads((c.p / "in" / f"reference-{ep_id}.json").read_text())
    rows = [{"emitter": r["emitter"], "event_kind": k} for r in ref["body"]["coverage_manifest"] for k in r["event_kinds"]]
    f = c.p / "in" / f"coverage-{ep_id}.json"; f.write_text(json.dumps(rows)); return f


def derive(c, ep_id, tag, world, events_world=None, until=None, ledger=None):
    st = c.state["episodes"][ep_id]
    out = c.p / "raw/rt" / f"{ep_id}-{tag}-observed.json"
    args = ["observed", "derive", "--repo", c.state["rt_repo_id"], "--world", c.state["worlds"][world]["world_ref"], "--scope", f"case:{CASE}", "--episode", st["episode_ref"],
            "--kind", "evaluated", "--profile", "profile:s1", "--independent-system", "sea-forge.case-ledger", "--declared", str(c.p / "in" / f"declared-{world}.json"),
            "--coverage-manifest", str(coverage_rows(c, ep_id)), "--out", str(out)]
    if events_world: args += ["--events-world", c.state["worlds"][events_world]["world_ref"]]
    if not until:  # pin the window to the current head ordinal: the default `head` moves as the ledger grows and could not be replayed
        until = subprocess.run(["sqlite3", c.state["rt_ledger"], "select max(append_ordinal) from ledger"], capture_output=True, text=True, check=True).stdout.strip()
    args += ["--until", str(until)]
    p = rt(c, f"rt-derive-{ep_id}-{tag}", args)
    return out, json.loads(out.read_text())


def compare(c, ep_id, tag, world, observed, reference_world=None):
    mp = c.p / "in" / f"mapping-{world}.json"; mp.write_text(json.dumps(mapping(c, world), indent=1))
    out = c.p / "raw/rt" / f"{ep_id}-{tag}-compare.json"
    args = ["observed", "compare", "--repo", c.state["rt_repo_id"], "--observed", str(observed), "--reference", str(c.p / "in" / f"reference-{ep_id}.json"),
            "--profile", str(c.p / "in/profile.json"), "--mapping", str(mp), "--world", c.state["worlds"][world]["world_ref"], "--declared", f"snapshot:{WORLD_NAME}-{world}",
            "--model", str(c.sf / "worlds" / world / "fde-sop.sea"), "--domainforge-bin", DF_DEV, "--facts-out", str(c.p / "raw/rt" / f"{ep_id}-{tag}-facts.json")]
    if reference_world: args += ["--reference-world", c.state["worlds"][reference_world]["world_ref"]]
    rt(c, f"rt-compare-{ep_id}-{tag}", args, out=out)
    return out, json.loads(out.read_text())


def classes(doc):
    rows = []
    for lst in ("episode", "structural"):
        for r in doc["residuals"][lst]:
            rows.append({"list": lst, "class": r["class"], "basis": r.get("basis"), "rule": r.get("rule"), "residual_id": r["residual_id"],
                         "expectation": doc["annotations"].get(r["residual_id"], {}).get("expectation"), "occurrences": len(r.get("occurrence_ids", []))})
    return rows


def analyze(c, ep_id):
    world = EP_WORLD[ep_id]
    o1, d1 = derive(c, ep_id, "a", world)
    o2, d2 = derive(c, ep_id, "b", world)
    same = o1.read_bytes() == o2.read_bytes()
    oc, cmp_ = compare(c, ep_id, "v1" if world == "v1" else "v2", world, o1)
    st = c.state["episodes"][ep_id]
    st["analysis"] = {"derive_byte_identical": same, "representation_id": d1["representation_id"], "representation_hash": d1["representation_hash"], "window": d1["window"],
                      "classes": classes(cmp_), "proposed_result": cmp_["proposed_result"], "not_exercised": len(cmp_.get("not_exercised", [])), "flags": cmp_.get("flags"),
                      "comparison_id": cmp_["comparison_id"], "settleable": cmp_.get("settleable")}
    c.save()
    return cmp_


def settle(c, ep_id):
    st = c.state["episodes"][ep_id]
    ver = st["verifier_run"]; exe = st["executor_run"]
    proposed = st["analysis"]["proposed_result"]["result"]
    result = "F" if (ver["status"] != "completed" or exe["status"] != "completed") else proposed
    st["settlement_rule"] = "F when the declared command failed in the executor's or the independent verifier's own run; otherwise the compare proposed_result"
    frame = {"verb": "case_record_append", "case_id": CASE, "kind": "episode.settled", "idempotency_key": f"independent-settler/{ep_id}", "request_id": f"settle-{ep_id}",
             "payload": {"episode_ref": st["episode_ref"], "result": result, "reliability": {"K": "declared command re-run by a fresh-context verifier under another principal; one source"},
                         "independence": {"independent_source_count": 1, "verifier_context": "fresh"}, "settlement_ref": ulid_like("settlement", ep_id)}, "actor": actor("independent-settler")}
    r = driver(c, f"settle-{ep_id}", [{"actor": actor("independent-settler"), "frame": frame}])[0]
    st["settlement"] = {"result": result, "response": r, "actor": "independent-settler", "executors": ["program-executor", "fresh-verifier"]}
    c.save()
    return r


# ------------------------------------------------------------------ governed revision (AT-12) and promotion through SEA-Forge
def residual_ids(doc, cls): return [r["residual_id"] for lst in ("episode", "structural") for r in doc["residuals"][lst] if r["class"] == cls]


def transition_request(source, target, op, claims, approval=None):
    ext = {"operation_id": op, "correlation_id": "corr-1", "operation_kind": "world_transition", "operation_name": "transition", "resource_id": target, "claims": claims}
    if approval: ext["approval_id"] = approval
    return {"envelope_id": f"env-authority_request-{op}", "cep_version": "1.0.0", "envelope_version": "1.0", "envelope_kind": "authority_request",
            "created_at": "2026-10-04T12:00:00Z", "created_by": "cognate", "scope": {"evaluation_context": "cognate-authority_request", "world_ref": source},
            "boundary_record": {"scope": "s", "included_sections": ["authority"], "excluded_sections": [], "known_omissions": ["payload not carried"], "unknowns": [], "redactions": [],
                                "compression_notes": [], "out_of_scope_entities": [], "classification": "internal", "limitations": []},
            "completeness_status": "partial", "omission_status": "marked", "provenance_refs": [f"prov-{op}"], "validation_status": "valid",
            "lineage_refs": [f"env-authority_request-{op.removesuffix('-b')}"] if approval else [],
            "provenance": [{"provenance_id": f"prov-{op}", "source_system_refs": ["cognate"], "producer_refs": ["cognate"], "production_method": "governed_operation", "created_at": "2026-10-04T12:00:00Z"}],
            "authority": [{"authority_id": "auth-1", "subject_actor_ref": "actor:alice"}], "transformations": [{"transformation_id": "tr-1", "transformation_type": "capability_invocation"}],
            "references": [{"ref_id": "r1", "ref_type": "artifact_id", "target_uri_or_id": target, "availability_status": "available", "integrity_status": "verifiable"}],
            "states": [{"state_id": "s1", "state_type": "declared_state", "subject_ref": "run:1"}],
            "extensions": {"cep.profile": {"profile_id": "godspeed.authority_request", "profile_version": "1.0.0"}, "godspeed.authority_request": ext,
                           "godspeed.world_transition": {"target_world_ref": target, "transition_kind": "semantic_change", "reason": "adopt the reviewed model", "compatibility": "compatible"}}}


def frame(verb, a, **kw): return {"actor": actor(a), "frame": {"verb": verb, "request_id": kw.pop("request_id"), "actor": actor(a), **kw}}


def revise(c):
    """Diagnose on ep-1 (held-out ep-2), propose, classify the prior episodes under the candidate world, regression matrix, approved promotion."""
    W = c.state["worlds"]; v1, v2 = W["v1"]["world_ref"], W["v2"]["world_ref"]
    docs = {e: json.loads((c.p / "raw/rt" / f"{e}-v1-compare.json").read_text()) for e in ("ep-1-control", "ep-2-inquiry")}
    rule = {"contract": "heldout-rule.v1", "rule_id": "rule:s1-heldout-by-episode", "kind": "exclude_episodes",
            "held_out_episodes": [f"episode:{CASE}:{EPS['ep-2-inquiry']['plan_item_id']}:1"], "statement": EPS and "Occurrences of ep-2-inquiry are excluded from diagnosis by episode id (episodes.yaml revision_candidate.held_out_rule)"}
    (c.p / "in/heldout-rule.json").write_text(json.dumps(rule, indent=1)); rsha = "sha256:" + sha(rule)
    und = [r for r in classes(docs["ep-1-control"]) if r["class"] == "observed_undeclared"]
    intended = [{"element": "verification_by_independent_role", "change": "add_declaration", "supersedes": [r["expectation"] for r in und]}]
    args = ["observed", "propose", "--proposer", "cognate-service", "--scope", f"case:{CASE}", "--old-world", v1, "--candidate-world", v2, "--rule", str(c.p / "in/heldout-rule.json"),
            "--rule-sha256", rsha, "--intended", json.dumps(intended), "--semantic-diff-sha256", W["semantic_diff_sha256"], "--out", str(c.p / "raw/rt/proposal.json")]
    for e in ("ep-1-control", "ep-2-inquiry"): args += ["--compared", str(c.p / "raw/rt" / f"{e}-v1-compare.json")]
    for r in und: args += ["--residual", r["residual_id"]]
    rt(c, "rt-propose", args, out=c.p / "raw/rt/proposal.stdout.json")
    proposal = json.loads((c.p / "raw/rt/proposal.json").read_text())
    # the same prior episodes under the candidate world
    new_docs = []
    for e in ("ep-1-control", "ep-2-inquiry"):
        o, _ = derive(c, e, "cand", "v2", events_world="v1")
        oc, cd = compare(c, e, "v2cand", "v2", o, reference_world="v1")
        new_docs.append(str(oc))
    reg = ["observed", "regression", "--proposal", str(c.p / "raw/rt/proposal.json")]
    for e in ("ep-1-control", "ep-2-inquiry"): reg += ["--old", str(c.p / "raw/rt" / f"{e}-v1-compare.json")]
    for n in new_docs: reg += ["--new", n]
    reg += ["--out", str(c.p / "raw/rt/regression-matrix.json")]
    p = rt(c, "rt-regression", reg, out=c.p / "raw/rt/regression.stdout.json", check=False)
    matrix = json.loads((c.p / "raw/rt/regression-matrix.json").read_text())
    c.state["revision"] = {"proposal_id": proposal["proposal_id"], "matrix_id": matrix["matrix_id"], "matrix_verdict": matrix["verdict"], "matrix_summary": matrix["summary"],
                           "regression_rc": p.returncode, "rule_sha256": rsha, "rows": len(matrix["rows"])}
    c.save()
    if matrix["verdict"] != "pass": raise RuntimeError(f"regression matrix {matrix['verdict']}: {matrix['summary']}")
    digest = "sha256:" + proposal["proposal_id"].removeprefix("proposal:")
    claims = {"proposal_digest": digest, "matrix_id": matrix["matrix_id"], "held_out_rule_sha256": proposal["held_out_rule"]["sha256"]}
    esc = driver(c, "promote-escalate", [frame("authority_request", "cognate-service", request_id="op-p1", envelope=transition_request(v1, v2, "op-p1", claims))])[0]
    approval = esc["envelope"]["extensions"]["godspeed.authority_decision"]["approval_id"]
    second = transition_request(v1, v2, "op-p1-b", claims, approval)
    promote = lambda rid: frame("authority_promote", "cognate-service", request_id=rid, proposal=proposal, matrix=matrix, envelope=second)
    out = driver(c, "promote-attempts", [promote("pr-early"),  # no approval yet: refused, retained
                                         frame("authority_approval", "cognate-service", request_id="ar-self", approval_id=approval, resolution="approved"),  # the proposer cannot approve
                                         frame("authority_approval", "op-alice", request_id="ar-ok", approval_id=approval, resolution="approved"),
                                         promote("pr-final"), frame("authority_pin", "cognate-service", request_id="pin-after"), frame("authority_transitions", "cognate-service", request_id="tr-after")])
    c.state["revision"].update({"early_promote": {k: out[0].get(k) for k in ("code", "error_class")}, "self_approval_refused": bool(out[1].get("error_class")),
                                "approval": {"state": out[2].get("state"), "approver": "op-alice", "proposer": "cognate-service", "assurance": "session"},
                                "promote": out[3], "pin_after": out[4], "transitions": out[5]})
    c.save()
    if out[3].get("ok") is not True: raise RuntimeError(f"promotion refused: {out[3]}")
    return out


# ------------------------------------------------------------------ subsequent episode, control variant, capability delta (AT-13)
def define_ep3(c):
    """Freeze the AT-13 definition (prediction and falsifier) BEFORE the subsequent episode and its control run."""
    e3 = EPS["ep-3-subsequent"]
    frozen = {"contract": "capability-frozen.v1", "frozen_id": "frozen:s1-at13", "episode_key": f"episode:{CASE}:{e3['plan_item_id']}:1", "comparison": "episode",
              "old_expectation": "verification_by_independent_role", "new_expectation": "verification_by_independent_role", "prediction": e3["predicted_at_13_result"],
              "falsifier": ["old_world_confirmed", "new_world_not_confirmed", "control_shows_gain"]}
    (c.p / "in/capability-frozen.json").write_text(json.dumps(frozen, indent=1))
    c.state["capability_frozen"] = {"sha256": "sha256:" + sha(frozen), "frozen_at": now(), "prediction": frozen["prediction"]}
    ctl = json.loads(json.dumps(e3)); ctl["id"] = "ep-3-control"; ctl["plan_item_id"] = e3["plan_item_id"] + "-control"
    ctl["intention"] = "CONTROL VARIANT of ep-3-subsequent: " + e3["intention"] + " The newly declared verification step is NOT performed."
    EPS["ep-3-control"] = ctl; EP_REPO["ep-3-control"] = "sea-rs"; EP_PATCH["ep-3-control"] = EP_PATCH["ep-3-subsequent"]
    EP_ARGV["ep-3-control"] = EP_ARGV["ep-3-subsequent"]; EP_WORLD["ep-3-control"] = "v2"
    c.save()


def ensure_ctl():
    if "ep-3-control" not in EPS:
        e3 = EPS["ep-3-subsequent"]; ctl = json.loads(json.dumps(e3)); ctl["id"] = "ep-3-control"; ctl["plan_item_id"] = e3["plan_item_id"] + "-control"
        ctl["intention"] = "CONTROL VARIANT of ep-3-subsequent: " + e3["intention"] + " The newly declared verification step is NOT performed."
        EPS["ep-3-control"] = ctl; EP_REPO["ep-3-control"] = "sea-rs"; EP_PATCH["ep-3-control"] = EP_PATCH["ep-3-subsequent"]
        EP_ARGV["ep-3-control"] = EP_ARGV["ep-3-subsequent"]; EP_WORLD["ep-3-control"] = "v2"


def analyze_cross(c, ep_id):
    """The same records of ep-3 (or its control) under the old world v1 and the new world v2."""
    o2, _ = derive(c, ep_id, "v2", "v2"); oc2, cd2 = compare(c, ep_id, "v2", "v2", o2)
    o1, _ = derive(c, ep_id, "v1", "v1", events_world="v2"); oc1, cd1 = compare(c, ep_id, "v1", "v1", o1, reference_world="v2")
    c.state["episodes"][ep_id]["analysis"] = {"classes_v2": classes(cd2), "classes_v1": classes(cd1), "proposed_result": cd2["proposed_result"],
                                               "comparison_id": cd2["comparison_id"], "representation_hash": json.loads(o2.read_text())["representation_hash"]}
    c.save()
    return str(oc1), str(oc2)


def capability_delta(c):
    ensure_ctl()
    old, new = analyze_cross(c, "ep-3-subsequent")
    cold, cnew = analyze_cross(c, "ep-3-control")
    args = ["observed", "capability-delta", "--frozen", str(c.p / "in/capability-frozen.json"), "--frozen-sha256", c.state["capability_frozen"]["sha256"], "--old", old, "--new", new,
            "--control-old", cold, "--control-new", cnew, "--out", str(c.p / "raw/rt/capability-delta.json")]
    rt(c, "rt-capability-delta", args, out=c.p / "raw/rt/capability-delta.stdout.json")
    d = json.loads((c.p / "raw/rt/capability-delta.json").read_text())
    c.state["capability_delta"] = {"result": d["result"], "classes": d["classes"], "falsifier_triggered": d["falsifier_triggered"], "prediction_met": d["prediction_met"], "r58_supported": d["r58_supported"], "code": d.get("code")}
    c.save()


# ------------------------------------------------------------------ the whole run and the per-test cell checks
def pipeline(c):
    c.state["started"] = now(); c.save()
    setup_models(c); setup_sf(c)
    define_ep3(c)
    for e in ("ep-1-control", "ep-2-inquiry"):
        freeze_episode(c, e); ingest_new(c, f"{e}-freeze")
        cognate_run(c, e, "executor"); ingest_new(c, f"{e}-exec")
        cognate_run(c, e, "verifier"); ingest_new(c, f"{e}-verify")
        analyze(c, e)
        settle(c, e); ingest_new(c, f"{e}-settled")
    revise(c)
    for e in ("ep-3-subsequent", "ep-3-control"):
        freeze_episode(c, e); ingest_new(c, f"{e}-freeze")
        cognate_run(c, e, "executor"); ingest_new(c, f"{e}-exec")
        if e == "ep-3-subsequent":
            cognate_run(c, e, "verifier"); ingest_new(c, f"{e}-verify")
    capability_delta(c)
    settle(c, "ep-3-subsequent"); ingest_new(c, "ep-3-subsequent-settled")
    checks(c)
    shutil.rmtree(c.p / "ws", ignore_errors=True)  # workspaces are build trees, not records; the patches, argv and outputs are kept
    c.state["finished"] = now(); c.save()
    manifest(c)


def manifest(c):
    """sha256 of every kept raw record (cell-relative). Files are kept in the cell; the evidence bundle in git references big ones by digest."""
    files = {}
    for base in ("raw", "in", "results", "sf-root", "rt-home", "steps.jsonl", "gaps.jsonl", "state.json"):
        b = c.p / base
        for f in ([b] if b.is_file() else sorted(b.rglob("*"))):
            if f.is_file() and not f.is_symlink() and not f.name.endswith(".sock") and f.stat().st_size < 4 * 2**30:
                files[str(f.relative_to(c.p))] = sha_file(f)
    (c.p / "manifest.json").write_text(json.dumps({"generated_at": now(), "files": files}, indent=1, sort_keys=True))


def checks(c):
    S = c.state; eps = S["episodes"]
    # AT-06: byte-identical derive on every analysed episode; occurrences listed; coverage and profile present
    ok6 = all(eps[e]["analysis"]["derive_byte_identical"] for e in ("ep-1-control", "ep-2-inquiry"))
    rep = json.loads((c.p / "raw/rt/ep-1-control-a-observed.json").read_text())["representation"]
    ok6 = ok6 and all(k in rep for k in ("coverage", "profile_id")) and all(n.get("occurrence_ids") for n in rep["nodes"])
    c.result("AT-06", "ok" if ok6 else "fail", episodes=["ep-1-control", "ep-2-inquiry"], representation_hash=eps["ep-1-control"]["analysis"]["representation_hash"])
    # AT-07: classes that arose on the episode set
    arisen = sorted({x["class"] for e in ("ep-1-control", "ep-2-inquiry") for x in eps[e]["analysis"]["classes"]})
    need = "confirmed" in arisen and ("observed_undeclared" in arisen or "contradicted" in arisen)
    c.result("AT-07", "ok" if need else "fail", classes_arisen=arisen, not_tested=sorted(set(["confirmed", "contradicted", "observed_undeclared", "declared_unobserved", "insufficient"]) - set(arisen)))
    # AT-08: the same real ledger cut before the verifier's records (--until): the affected residuals disappear, the hash changes
    e = "ep-1-control"; until = S["last_ordinal"][f"{e}-exec"]
    o, d = derive(c, e, "until", "v1", until=until); oc, cd = compare(c, e, "until-v1", "v1", o)
    full = eps[e]["analysis"]
    full_und = [x for x in full["classes"] if x["class"] == "observed_undeclared"]
    cut_und = [x for x in classes(cd) if x["class"] == "observed_undeclared"]
    ok8 = len(full_und) > 0 and len(cut_und) == 0 and d["representation_hash"] != full["representation_hash"]
    c.result("AT-08", "ok" if ok8 else "fail", until_ordinal=until, undeclared_full=len(full_und), undeclared_cut=len(cut_und), hash_full=full["representation_hash"], hash_cut=d["representation_hash"],
             note="the variant is a real earlier cut of the same append-only ledger; no file was edited")
    # NT-03b: delete one occurrence from a COPY of the ledger: derive must fail CHAIN_VERIFY_FAILED
    cp = c.p / "work/rt-home-nt03b"; shutil.rmtree(cp, ignore_errors=True); shutil.copytree(c.rthome, cp)
    db = cp / "data/sxr/repos" / S["rt_repo_id"] / "ledger.db"
    mid = S["last_ordinal"][f"{e}-exec"] - 1
    subprocess.run(["sqlite3", str(db), f"delete from ledger where append_ordinal={mid}"], check=True)
    st = eps[e]
    p = c.step("nt03b-derive-on-copy", [RT, "observed", "derive", "--repo", S["rt_repo_id"], "--world", S["worlds"]["v1"]["world_ref"], "--scope", f"case:{CASE}", "--episode", st["episode_ref"], "--kind", "evaluated", "--profile", "profile:s1"],
               env={"XDG_DATA_HOME": str(cp / "data"), "XDG_STATE_HOME": str(cp / "state")}, check=False)
    c.result("NT-03b", "ok" if (p.returncode == 5 and "CHAIN_VERIFY_FAILED" in p.stderr) else "fail", rc=p.returncode, deleted_ordinal=mid, stderr=p.stderr[-200:])
    # NT-02: events without identity / without an evidence record are refused on a copy; the live ledger is untouched
    cp2 = c.p / "work/rt-home-nt02"; shutil.rmtree(cp2, ignore_errors=True); shutil.copytree(c.rthome, cp2)
    env2 = {"XDG_DATA_HOME": str(cp2 / "data"), "XDG_STATE_HOME": str(cp2 / "state")}
    line = [l for l in (c.p / "raw/cognate/ep-1-control-executor/interaction-events.ndjson").read_text().splitlines()][0]
    a = json.loads(line); x = a["extensions"]["godspeed.interaction_event"]; x.pop("world_ref"); x.pop("declaration_key")
    b = json.loads(line); b["extensions"]["godspeed.interaction_event"]["source"]["record_id"] = str(uuid.uuid4())
    res = {}
    for name, ev, code in (("a", a, "INGEST_REJECTED_IDENTITY"), ("b", b, "INGEST_REJECTED_EVIDENCE")):
        f = c.p / "work" / f"nt02-{name}.ndjson"; f.write_text(canon(ev) + "\n")
        pr = c.step(f"nt02-{name}-ingest-on-copy", [RT, "observed", "ingest", "--repo", S["rt_repo_id"], "--file", str(f)], env=env2, check=False)
        res[name] = {"rc": pr.returncode, "code_seen": code in pr.stdout, "expected": code}
    c.result("NT-02", "ok" if all(v["rc"] == 4 and v["code_seen"] for v in res.values()) else "fail", cases=res)
    # the live ledger head, for the manifest
    S["rt_ledger_sha256"] = sha_file(S["rt_ledger"]); c.save()
    c.gap("AT-12,NT-07,AT-13", "SEA-Forge freeze does not check the pinned world (WORLD_NOT_PINNED missing in case-record-append.v1): ep-3 froze under v2 by the caller's own declaration; the pin is not consumed on the write path")
    c.gap("AT-04,NT-16", "execution-side I-1 gate is not wired: item_execute/case_advance/agent_run.delegate carry no n or episode_ref; Cognate program.run ran under its own local policy with no SEA-Forge authority decision (authority.decision_id is unknown/not_required)")
    c.gap("AT-05,AT-06", "no in-process RealityTrace writer/delivery from SEA-Forge: interaction events travel as spool files (interaction-events/<case>.jsonl, Cognate NDJSON) into `observed ingest`; evidence.recorded rows are seeded by rt-seed because no producer writes them")
    c.gap("AT-05", "Cognate's ungoverned run events ride Observation.interactionEvent, not a CEP envelope: wrapped in the minimal execution_trace envelope by cognate_run.ts; only run.invocation is emitted (host outboxFor filter)")
    c.gap("AT-04,AT-18", "anchor: scratch git repository with a local bare remote (not the programme origin); time is local-only (OD-6); freeze anchors are not verifiable with fde-anchor.sh verify (refused there by design)")
    c.gap("AT-12", "the non-promotion variant (decline with retained divergence and rationale) was not run; approval is at session assurance, not A3")
    c.gap("AT-13", "the harness's verification step is emitted only by Cognate (the executor's own emitter): RealityTrace refuses to confirm executor-only evidence, so the declared step stays insufficient under v2")
    c.gap("AT-07,NT-03,NT-06", "no model element or mapping form expresses 'declared command passes' / the ep-2 hypothesis or a postcondition: no contradicted residual arises from ep-2; NT-03 real re-run with a changed value and NT-06 silent source were verified by producer tests only")
    c.gap("AT-01,AT-12", "SEA-Forge embeds DomainForge 0.21.2; the stack ran on the 3960326 build (0.21.2 line, world_ref 78ed... equals the frozen episodes.yaml world); DomainForge 0.22.0 gives world_ref e3c3... for the same sources")


if __name__ == "__main__":
    cell = Cell(sys.argv[1])
    for st in sys.argv[2:]:
        name, *args = st.split(":")
        globals()[name](cell, *args)
