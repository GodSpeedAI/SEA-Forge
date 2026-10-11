#!/usr/bin/env python3
"""T23 mechanism-provenance validation for probe observations.

A probe_observation record is admitted into consequential route state only
if its claimed consequence is REPRODUCIBLE from the run's own durable state
database: the database must exist at the pinned path, its sha256 must match
the pinned digest, and the event log inside it must carry EXACTLY ONE
run_terminal event whose state maps to the recorded terminal_class plus a
settlement_committed count equal to the recorded count. A settled claim
with zero settlement_committed events behind it is refused (this is also a
cross-field rule of the frozen T21 route_contracts; here it is enforced
against the database itself, not just the record).

This module is the single validator shared by the route driver (absorb-time,
fail-closed), the teeth (T3 must be REFUSED by it), and the gate (which
additionally recomputes terminals independently). It is authored before the
first T23 probe executes and never modified afterwards.

Ground truth hierarchy (frozen): gauntlet's own event log is the only ground
truth for a probe; hand-authored outcome rows are fabrication.
"""
from __future__ import annotations

import sys

sys.dont_write_bytecode = True

import hashlib
import json
import os
import sqlite3

DB_FILENAME = "gauntlet-state.db"
TERMINAL_EVENT_KIND = "run_terminal"
SETTLEMENT_EVENT_KIND = "settlement_committed"
# Digest pinned for a probe that produced no database at all (run_error):
# the sha256 of zero bytes (T22 convention, preserved).
ZERO_SHA = hashlib.sha256(b"").hexdigest()


def classify_terminal(raw_state) -> str:
    """Frozen T22/T23 typed-terminal mapping from the raw run_terminal state."""
    if raw_state == "settled":
        return "settled"
    if raw_state == "budget_exhausted":
        return "budget_exhausted"
    if isinstance(raw_state, str) and raw_state != "":
        return "other-nonsettlement"
    return "run_error"


def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def read_event_log(db_path: str) -> dict:
    """Read-only read of the run's own event log.

    Returns {"run_terminal_states": [...], "settlement_committed_count": N}.
    Raises OSError/sqlite3.Error on an unreadable database.
    """
    conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    try:
        rows = conn.execute(
            "select payload from event_log where kind=?",
            (TERMINAL_EVENT_KIND,)).fetchall()
        settle_count = conn.execute(
            "select count(*) from event_log where kind=?",
            (SETTLEMENT_EVENT_KIND,)).fetchone()[0]
    finally:
        conn.close()
    states = []
    for (payload,) in rows:
        try:
            states.append(json.loads(payload).get("state"))
        except json.JSONDecodeError:
            states.append(None)
    return {
        "run_terminal_states": states,
        "settlement_committed_count": settle_count,
    }


def recompute(observation: dict) -> dict:
    """Independently recompute a probe observation's consequence facts from
    its pinned state database.

    Returns {"ok": bool, "reasons": [...], "terminal_class": str,
    "settlement_committed_count": int, "db_sha256": str} where terminal_class
    is the class recomputed from the database ("run_error" when no database
    exists or no typed terminal is found) — NEVER the recorded value.
    """
    reasons: list[str] = []
    db_path = observation.get("state_db_path", "")
    recorded_digest = observation.get("state_db_sha256", "")
    if not db_path or not isinstance(db_path, str):
        return {"ok": False, "reasons": ["no state database path pinned"],
                "terminal_class": "run_error",
                "settlement_committed_count": 0, "db_sha256": ""}
    if not os.path.isfile(db_path):
        return {"ok": False,
                "reasons": [f"no state database at the pinned path: {db_path}"],
                "terminal_class": "run_error",
                "settlement_committed_count": 0, "db_sha256": ""}
    digest = sha256_file(db_path)
    if recorded_digest != digest:
        reasons.append(
            f"state database digest mismatch: pinned {recorded_digest} != "
            f"actual {digest}")
    try:
        log = read_event_log(db_path)
    except (OSError, sqlite3.Error) as exc:
        reasons.append(f"state database unreadable read-only: {exc}")
        return {"ok": False, "reasons": reasons, "terminal_class": "run_error",
                "settlement_committed_count": 0, "db_sha256": digest}
    states = log["run_terminal_states"]
    if len(states) != 1:
        reasons.append(
            f"expected exactly one {TERMINAL_EVENT_KIND} event, found "
            f"{len(states)}")
    terminal_class = classify_terminal(states[0] if states else None)
    settle_count = log["settlement_committed_count"]
    if observation.get("terminal_class") != terminal_class:
        reasons.append(
            f"terminal_class mismatch: recorded "
            f"{observation.get('terminal_class')!r} != database "
            f"{terminal_class!r}")
    if observation.get("settlement_committed_count") != settle_count:
        reasons.append(
            f"settlement_committed_count mismatch: recorded "
            f"{observation.get('settlement_committed_count')!r} != database "
            f"{settle_count!r}")
    if terminal_class == "settled" and settle_count < 1:
        reasons.append(
            "settled claim with no settlement_committed event behind it in "
            "the state database")
    return {"ok": not reasons, "reasons": reasons,
            "terminal_class": terminal_class,
            "settlement_committed_count": settle_count, "db_sha256": digest}


def validate(observation: dict) -> tuple[bool, str]:
    """Absorb-time gate: (True, "") when the observation's claimed consequence
    is fully backed by its pinned state database, else (False, reason).

    A run_error claim (no typed terminal) is backed by the ABSENCE of a typed
    terminal: no database at the pinned path with the zero-byte digest, or a
    database whose event log carries no run_terminal event. Every other
    claimed terminal_class must be recomputed exactly from the database with
    a matching digest and settlement count; a settled claim additionally
    requires at least one settlement_committed event behind it.
    """
    recorded = observation.get("terminal_class")
    db_path = observation.get("state_db_path", "")
    if recorded == "run_error":
        if not os.path.isfile(db_path):
            if observation.get("state_db_sha256") == ZERO_SHA:
                return True, ""
            return False, ("run_error observation without a database must pin "
                           "the zero-byte digest")
        result = recompute(observation)
        if result["terminal_class"] != "run_error":
            return False, (f"run_error claim contradicted by database terminal "
                           f"{result['terminal_class']!r}")
        if observation.get("settlement_committed_count") != \
                result["settlement_committed_count"]:
            return False, "settlement_committed_count mismatch against the database"
        return True, ""
    result = recompute(observation)
    return result["ok"], "; ".join(result["reasons"])
