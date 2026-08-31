// Validates every response the SFWP IPC mock (sfwp-full-mock.js) actually
// returns against the real generated AJV schemas at
// workbench/packages/contracts/schema/*.schema.json.
//
// This does not hand-duplicate the mock's fixtures — it loads the real mock
// file into a fake `window`, invokes it exactly the way the real Workbench
// frontend does (`window.__TAURI_INTERNALS__.invoke(cmd, args)`), and
// schema-checks whatever comes back. A drift between the mock and the real
// IPC contract fails this script, not just the settlement gauntlet.
//
// Usage: node validate_mock_payloads.mjs   (exit 0 = all responses conform)
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "../../../../");
const SCHEMA_DIR = path.join(REPO_ROOT, "workbench/packages/contracts/schema");
const MOCK_FILE = path.join(__dirname, "sfwp-full-mock.js");

// Resolve ajv the way the contracts package itself would (it declares ajv as
// a direct dependency), rather than assuming a specific hoisted layout.
const contractsRequire = createRequire(path.join(REPO_ROOT, "workbench/packages/contracts/package.json"));
const ajvEntry = contractsRequire.resolve("ajv/dist/2020.js");
const { default: Ajv2020 } = await import(`file://${ajvEntry}`);

const ajv = new Ajv2020({ strict: false, allErrors: true });
function compile(name) {
  const raw = JSON.parse(fs.readFileSync(path.join(SCHEMA_DIR, `${name}.schema.json`), "utf-8"));
  return ajv.compile(raw);
}

// Minimal fake browser global so the mock's IIFE (`window.X = ...`) runs.
const fakeWindow = {};
globalThis.window = fakeWindow;
await import(`file://${MOCK_FILE}?t=${Date.now()}`);
const invoke = fakeWindow.__TAURI_INTERNALS__.invoke;

const QUERY_CHECKS = [
  ["readiness_get", "ReadinessView"],
  ["identity_get", "IdentityView"],
  ["case_entry_options", "EntryOptionsResult"],
  ["case_list", "CaseListResult"],
  ["case_get_overview", "CaseOverview"],
  ["case_get_horizon", "CaseHorizon"],
  ["approval_list", "ApprovalListResult"],
  ["run_list", "RunListResult"],
  ["run_get", "RunRecord"],
  ["asset_list", "AssetListResult"],
  ["delegation_preview", "DelegationPreviewResult"],
  ["delegation_list", "DelegationListResult"],
];

let allOk = true;
for (const [verb, schemaName] of QUERY_CHECKS) {
  const payload = await invoke("sfwp_query", { query: { verb } });
  const validate = compile(schemaName);
  const ok = validate(payload);
  if (!ok) {
    allOk = false;
    console.log(`\n[FAIL] sfwp_query verb=${verb} -> ${schemaName}`);
    console.log(JSON.stringify(validate.errors, null, 1));
  } else {
    console.log(`[OK]   sfwp_query verb=${verb} -> ${schemaName}`);
  }
}

// case_preflight is stateful (increments a call counter); check separately.
{
  const payload = await invoke("sfwp_query", { query: { verb: "case_preflight" } });
  const validate = compile("PreflightResult");
  const ok = validate(payload);
  console.log(ok ? "[OK]   sfwp_query verb=case_preflight -> PreflightResult" : "[FAIL] case_preflight");
  if (!ok) {
    allOk = false;
    console.log(JSON.stringify(validate.errors, null, 1));
  }
}

if (!allOk) {
  console.error("\nMock payload validation FAILED — the frontend would hit a validation-error fallback, not the intended view.");
  process.exit(1);
}
console.log("\nAll mocked IPC responses are schema-valid against the real generated contracts.");
