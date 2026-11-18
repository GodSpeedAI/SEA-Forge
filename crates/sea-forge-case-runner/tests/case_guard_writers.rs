//! SECONDARY lint of Field Case writers (WP-21 part B, review round 5). The
//! primary check is the runtime one in `model_invariants.rs`: every file the
//! case directory sees modified is a file `commit` journaled. This test only
//! reads the source for raw filesystem primitives (any import spelling) and
//! for the verbs' locks.

use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// `(name, body)` of every function of the file's non-test code.
fn functions(text: &str) -> Vec<(String, String)> {
    let end = text.find("#[cfg(test)]").unwrap_or(text.len());
    let text = &text[..end];
    let mut starts: Vec<(usize, String)> = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let mut t = line.trim_start();
        if let Some(rest) = t.strip_prefix("pub") {
            t = match rest.strip_prefix('(') {
                Some(r) => r.split_once(") ").map_or(rest, |(_, after)| after),
                None => rest.trim_start(),
            };
        }
        let t = t.strip_prefix("async ").unwrap_or(t);
        if let Some(rest) = t.strip_prefix("fn ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            starts.push((offset, name));
        }
        offset += line.len();
    }
    let mut out = Vec::new();
    for (i, (start, name)) in starts.iter().enumerate() {
        let stop = starts.get(i + 1).map_or(text.len(), |(s, _)| *s);
        out.push((name.clone(), text[*start..stop].to_string()));
    }
    out
}

/// SECONDARY LINT. The primary check of writer coverage is the runtime one in
/// `model_invariants.rs` (an inotify watch on the case directory against the
/// journal of `commit`). This scan only catches the obvious: a raw filesystem
/// primitive, however it is imported, in code that works with a case directory
/// or takes a path parameter, outside the guard and the allowlist below.
const RAW: [&str; 16] = [
    "fs::write(",
    "File::create(",
    "File::options(",
    "OpenOptions::new()",
    "fs::rename(",
    ".set_len(",
    "fs::remove_file(",
    "fs::remove_dir_all(",
    "fs::hard_link(",
    "fs::copy(",
    ".create_new(",
    "use std::fs::write",
    "use std::fs as",
    "use std::fs::{",
    "fs::File::create",
    "std::fs::OpenOptions",
];

fn case_dir_body(body: &str) -> bool {
    [
        "case_dir",
        "run_dir",
        "\"cases\"",
        "case.json",
        "case-events",
        "plan.json",
        "field_case.json",
        "document_path",
        "events_path",
        "case_events",
        "RECORDS",
        "path: &Path",
        "path: &std::path::Path",
    ]
    .iter()
    .any(|n| body.contains(n))
}

/// Functions that make a raw write and are not the guard, each with why it is
/// outside the case store: a root-level store, an execution workspace the
/// sandboxed process owns, or the plain write of a path that is not a case.
const RAW_ALLOWED: [(&str, &str); 11] = [
    (
        "execute",
        "writes the legacy-migration marker at the store root, not in a case",
    ),
    (
        "execute_with_permission_broker",
        "the transcript artifact under <root>/runs/<run>",
    ),
    (
        "exchange",
        "writes a request to the server socket, not a case file",
    ),
    (
        "write_json",
        "writes a path that is NOT in a case directory (commit_file said so)",
    ),
    (
        "append_line",
        "plain append of a spool outside any case directory",
    ),
    ("save_workspace", "execution workspace of a run"),
    (
        "deliver",
        "the interaction spool under <root>/interaction-events",
    ),
    (
        "initialize_case",
        "creates the runs directory, a directory not a file",
    ),
    (
        "move_file_atomicish",
        "legacy v0.1 -> v0.2 migration of run files",
    ),
    (
        "remove_empty_run_dirs",
        "legacy migration removes empty directories",
    ),
    (
        "write_json_new",
        "writes under a run directory outside any case (pipeline)",
    ),
];

fn scanned_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for dir in [
        "sea-forge-case-runner/src",
        "sea-forge-server/src/sfwp",
        "sea-forge-cli/src",
    ] {
        rust_files(&root().join(dir), &mut files);
    }
    files.push(root().join("sea-forge-server/src/lib.rs"));
    files.push(root().join("sea-forge-server/src/case_dispatch.rs"));
    files.push(root().join("sea-forge-server/src/delegation.rs"));
    // The guard defines the primitives the others use.
    files.retain(|f| !f.ends_with("case_guard.rs"));
    // The correlation index is the server's own store, not a case's.
    files.retain(|f| !f.ends_with("correlation.rs"));
    files
}

#[test]
fn no_raw_write_outside_the_guard_and_its_allowlist() {
    let mut found = 0;
    let mut unguarded: Vec<String> = Vec::new();
    for file in scanned_files() {
        let text = fs::read_to_string(&file).unwrap();
        for (name, body) in functions(&text) {
            if !RAW.iter().any(|r| body.contains(r)) || !case_dir_body(&body) {
                continue;
            }
            found += 1;
            // The private stage creates fresh metadata and syncs its files
            // before publication. Only these two routines own that access.
            let private_stage = file
                .ends_with("sea-forge-case-runner/src/case_ops/versioned_store/stage.rs")
                && matches!(name.as_str(), "write_new" | "flush_tree");
            if !private_stage && !RAW_ALLOWED.iter().any(|(n, _)| *n == name) {
                unguarded.push(format!("{}: fn {name}", file.display()));
            }
        }
    }
    assert!(
        found >= 3,
        "the scan found only {found} raw writers: it is blind"
    );
    assert!(
        unguarded.is_empty(),
        "raw filesystem writes near a case directory; build a Batch and commit it (case_guard::commit):\n{}",
        unguarded.join("\n")
    );
}

/// Every CLI verb of `case_ops` that mutates a case holds the case lock or
/// goes through `commit`/`check`.
#[test]
fn every_cli_verb_takes_the_case_lock() {
    let text =
        fs::read_to_string(root().join("sea-forge-case-runner/src/case_ops/mod.rs")).unwrap();
    let exempt = [
        "append_case_event",
        "append_case_event_with",
        "commit_case_event",
        "reopen",
        "reopen_with",
        "terminate",
        "propose_item",
        "propose_item_with",
        "complete_human_task",
        "write_json",
        "save_case",
    ];
    let mut checked = 0;
    for (name, body) in functions(&text) {
        let mutates = body.contains("append_case_event_with(")
            || body.contains("append_notify(")
            || body.contains("write_json(");
        if !mutates || exempt.contains(&name.as_str()) {
            continue;
        }
        checked += 1;
        assert!(
            body.contains("CaseLock::acquire(")
                || body.contains("case_guard::check(")
                || body.contains("commit("),
            "case_ops::{name} mutates a case with no lock or commit"
        );
    }
    assert!(
        checked >= 3,
        "only {checked} verbs checked: the scan is blind"
    );
}

/// Every server verb that drives a case mutation holds the server's keyed case
/// lock around it (the file lock inside the guard is the cross-process half).
#[test]
fn every_server_case_verb_holds_the_keyed_lock() {
    let text =
        fs::read_to_string(root().join("sea-forge-server/src/sfwp/case_mutations.rs")).unwrap();
    let mutators = [
        "reopen_with_reason(",
        "terminate_with(",
        "complete_human_task_with(",
        "commit_staged_plan(",
        "propose_item_with(",
        "advance_case(",
    ];
    let mut checked = 0;
    for (name, body) in functions(&text) {
        // `*_locked` helpers run inside their caller's lock (checked via the caller).
        if mutators.iter().any(|m| body.contains(m)) && !name.ends_with("_locked") {
            checked += 1;
            assert!(
                body.contains("acquire_case_lock("),
                "server verb {name} mutates a case without the keyed case lock"
            );
        }
    }
    assert!(
        checked >= 3,
        "only {checked} verbs checked: the scan is blind"
    );
}
