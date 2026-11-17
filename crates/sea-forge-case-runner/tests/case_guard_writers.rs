//! The registry of Field Case writers (WP-21 part B, review round 3): every
//! function that writes a case store file calls THE guard
//! (`case_guard::admit_write`), directly or through the one primitive it
//! names. The test reads the source and FAILS when a writer skips the guard or
//! a new sink appears that this list does not know.

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

fn guarded(body: &str) -> bool {
    body.contains("admit_write(")
        || body.contains("admit_file_write(")
        || body.contains("CaseLock::acquire(")
}

/// The raw filesystem writes. Under a case directory they exist only inside
/// `WriteToken` (the guard module) or in a function that holds a token.
const RAW: [&str; 7] = [
    "fs::write(",
    "File::create(",
    "OpenOptions::new()",
    "fs::rename(",
    ".set_len(",
    "fs::remove_file(",
    "fs::copy(",
];

/// A body that names a case directory or one of its files.
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
    ]
    .iter()
    .any(|n| body.contains(n))
}

/// Functions that make a raw write and are NOT guarded themselves, with why
/// each is sound. Everything else that writes under a case directory holds a
/// token (`admit_write`, `admit_file_write` or `CaseLock`).
const RAW_ALLOWED: [(&str, &str); 4] = [
    (
        "sync_append",
        "the byte primitive; called only by append_line and repair_torn_tail, which hold the token",
    ),
    (
        "commit_event",
        "appends through the token it is handed; the plain branch is for a path that is not a case directory",
    ),
    (
        "append_case_event_with",
        "appends through its token",
    ),
    (
        "write_json",
        "admits first (admit_file_write), then replaces through the token",
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
fn every_raw_write_under_a_case_dir_holds_a_token() {
    let mut found = 0;
    let mut unguarded: Vec<String> = Vec::new();
    for file in scanned_files() {
        let text = fs::read_to_string(&file).unwrap();
        for (name, body) in functions(&text) {
            if !RAW.iter().any(|r| body.contains(r)) || !case_dir_body(&body) {
                continue;
            }
            found += 1;
            let allowed = RAW_ALLOWED.iter().any(|(n, _)| *n == name);
            if !(guarded(&body) || body.contains("token") || allowed) {
                unguarded.push(format!("{}: fn {name}", file.display()));
            }
        }
    }
    assert!(
        found >= 4,
        "the scan found only {found} raw writers: it is blind"
    );
    assert!(
        unguarded.is_empty(),
        "raw writes under a case directory with no token (take one from case_guard::admit_write):\n{}",
        unguarded.join("\n")
    );
}

/// The sanctioned entry points are the only callers of the byte primitive.
#[test]
fn the_byte_primitive_has_only_its_known_callers() {
    let mut callers: Vec<String> = Vec::new();
    for file in scanned_files() {
        let text = fs::read_to_string(&file).unwrap();
        for (name, body) in functions(&text) {
            // A definition mentions itself; count calls from other functions.
            if name != "sync_append" && body.contains("sync_append(") {
                callers.push(name);
            }
        }
    }
    callers.sort();
    assert_eq!(callers, ["append_line", "repair_torn_tail"]);
}

/// Every CLI verb of `case_ops` that mutates a case holds the case lock or
/// admits the write itself.
#[test]
fn every_cli_verb_takes_the_case_lock() {
    let text =
        fs::read_to_string(root().join("sea-forge-case-runner/src/case_ops/mod.rs")).unwrap();
    // Wrappers that only delegate, and the primitives the guard sits inside.
    let exempt = [
        "append_case_event",
        "append_case_event_with",
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
            guarded(&body),
            "case_ops::{name} mutates a case with no lock or admission"
        );
    }
    assert!(
        checked >= 3,
        "only {checked} verbs checked: the scan is blind"
    );
}

/// The one event appender of the case engine admits before it commits, and the
/// store primitive admits before it appends: the guard is at BOTH appenders.
#[test]
fn both_event_appenders_and_the_record_appender_admit() {
    let lib = fs::read_to_string(root().join("sea-forge-case-runner/src/lib.rs")).unwrap();
    let append = functions(&lib)
        .into_iter()
        .find(|(n, _)| n == "append_event")
        .unwrap()
        .1;
    assert!(append.contains("admit_write("), "CaseRunner::append_event");
    let mod_rs =
        fs::read_to_string(root().join("sea-forge-case-runner/src/case_ops/mod.rs")).unwrap();
    let append = functions(&mod_rs)
        .into_iter()
        .find(|(n, _)| n == "append_case_event_with")
        .unwrap()
        .1;
    assert!(append.contains("admit_write("), "append_case_event_with");
    let records =
        fs::read_to_string(root().join("sea-forge-server/src/sfwp/case_records.rs")).unwrap();
    let append = functions(&records)
        .into_iter()
        .find(|(n, _)| n == "append_line")
        .unwrap()
        .1;
    assert!(
        append.contains("admit_file_write("),
        "case_records::append_line"
    );
    let kinds =
        fs::read_to_string(root().join("sea-forge-server/src/sfwp/field_case_kinds.rs")).unwrap();
    let doc = functions(&kinds)
        .into_iter()
        .find(|(n, _)| n == "write_document")
        .unwrap()
        .1;
    assert!(doc.contains("admit_file_write("), "write_document");
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
