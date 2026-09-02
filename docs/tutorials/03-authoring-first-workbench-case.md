# Tutorial: Authoring Your First Workbench Case

> **A guided tour through the SEA Forge Workbench desktop interface: from cell readiness and preflight to live case horizon monitoring.**

---

## Prerequisites

Before starting this tutorial:
* Ensure Bun 1.4.0+ is installed (`bun --version`).
* Ensure you have built the local server binary:
  ```sh
  cargo build -p sea-forge-server
  ```

---

## 1. Launch the Workbench Environment

In SEA Forge, the desktop application connects to the local cell server over an owner-only Unix domain socket (`server.sock`).

### Terminal 1: Start the Local Cell Server
Set the cell root to a temporary tutorial directory and start the server:
```sh
export SEA_FORGE_ROOT=/tmp/workbench-tutorial-cell
./target/debug/sea-forge-server
```
You will see:
```text
[INFO] sea-forge-server listening on /tmp/workbench-tutorial-cell/server.sock
[INFO] cell lock acquired: /tmp/workbench-tutorial-cell/server.sock.lock
```

### Terminal 2: Launch the Workbench Frontend
Start the local development renderer:
```sh
cd workbench
bun install
bun run dev
```
Open your browser to `http://localhost:1420`.

---

## 2. Verify Cell Readiness (`/readiness`)

When the Workbench opens, it lands on the **Readiness Dashboard** (`/readiness`).

The dashboard evaluates route guards against source-backed cell data:
* **G1 (Cell Connection):** Connected to `/tmp/workbench-tutorial-cell/server.sock`.
* **G2 (SFWP Protocol):** Negotiated Protocol Version `1`.
* **G3 (Actor Identity):** Local actor bound as `operator_local`.
* **G4 (Policy Engine):** Active policy bundle loaded and verified.
* **G6 (Sandbox Availability):** Linux Landlock jail verified operational.

*Note:* If any guard fails, the UI renders the `GovernedDenialSurface`, preventing interaction until the environment issue is remediated.

---

## 3. Case Preflight Check (`/cases/create`)

Navigate to **Case Creation** in the sidebar.

1. Select **Intent Mode** or **Template Mode**.
2. Select the built-in template: `sea_model_demo`.
3. Click **Run Preflight**:
   * The Workbench issues the SFWP query `case.preflight`.
   * The server runs a dry-run through `sea-forge-planner` and `sea-forge-authority`.
   * **Crucial Invariant:** Preflight is strictly read-only. No case record is created, no run ID is issued, and no directory is touched on disk.
4. Review the Preflight Results:
   * **Planned Items:** `generate_and_validate_sea_model`.
   * **Sandbox Class:** `jail` (Landlock).
   * **Settlement Criteria:** File `model.sea` required, stdout match `"sea-forge: model valid"`.
   * **Authority Preview:** `Verdict: Allow`.

---

## 4. Commit the Governed Case

1. Click **Commit Case**:
   * The Workbench generates a unique `request_id` (`req_...`) to guarantee idempotency.
   * Sends the SFWP command `case.commit`.
   * The server admits the request through its admission queue, creates `.sea-forge/cases/<case_id>/`, and commits the initial case events.
2. The UI automatically navigates to the **Case Horizon** route (`/cases/<case_id>`).

---

## 5. Monitor the Live Case Horizon

On the Case Horizon page:
* **Visual Sentry Graph:** Displays case stages and their entry sentries.
* **Live Event Stream:** Shows tasks transitioning from `Enabled` → `Activated` → `Completed`.
* **Sub-Episode Progression:** As the sub-episode runs, you will observe:
  * `ItemActivated` with monotonic `dispatch_ordinal`.
  * `SettlementRecorded` with `status: accepted` and `settlement_ordinal`.
* **Case Completion:** When all required tasks settle, the case status indicator updates to **Completed**.

---

## 6. Inspect Evidence & Work Products (`/runs/:runId`)

1. Click on the completed run in the horizon view.
2. The Workbench opens the **Run Record Inspector**:
   * **Authority Tab:** Displays the evaluated policy rules and the exact `ActionGrant` metadata.
   * **Settlement Tab:** Displays the factual basis array (`authority_allow`, `exit_zero`, `required_artifact_present:model.sea`).
   * **Artifacts Tab:** Displays the captured `model.sea`, its content preview, SHA-256 hash, and pre-mint identity `ifl:hash:...`.
   * **Trace Tab:** Displays the raw, chronological `trace.jsonl` event stream.

---

## 7. Clean Up

Stop the development server in Terminal 2 (`Ctrl+C`) and stop the cell server in Terminal 1 (`Ctrl+C`).

To learn more about the desktop architecture and Tauri IPC bridge, see the [Workbench Desktop Subsystem Guide](file:///c:/Users/sprim/projects/sea-rs/docs/subsystems/workbench-desktop.md).
