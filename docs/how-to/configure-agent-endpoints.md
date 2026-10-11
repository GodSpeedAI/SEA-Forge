# How-To: Configure Governed Agent Endpoints

> **A step-by-step guide for setting up Anthropic, OpenAI-compatible, and ACP coding agent endpoints in SEA Forge.**

---

## Goal

Configure and verify a governed agent endpoint in `server.yaml` so that case plans can delegate tasks to AI models (e.g. Anthropic Claude, OpenAI) or local CLI-resident coding agents via the Agent Client Protocol (ACP) under strict credential and network governance.

---

## Prerequisites

* Access to your cell's `server.yaml`.
* An API key for your target model provider, or an installed local ACP-compliant agent binary.
* SOPS and age installed if using encrypted repository secrets.

---

## 1. Configure Endpoints in `server.yaml`

Open your cell's configuration file (e.g. `$SEA_FORGE_ROOT/server.yaml`). Under the `agents:` block, define your endpoints:

### Option A: Anthropic Claude Endpoint
```yaml
agents:
  endpoints:
    - endpoint_ref: "anthropic-claude-3-5-sonnet"
      provider_kind: "anthropic"
      base_url: "https://api.anthropic.com/v1"
      model: "claude-3-5-sonnet-20241022"
      credential_env_var: "ANTHROPIC_API_KEY"
      max_request_bytes: 524288      # 512 KiB request limit
      max_response_bytes: 1048576    # 1 MiB response limit
      timeout_secs: 60
      transcript_retention: "sealed" # Encrypt full conversation
```

### Option B: Local CLI Coding Agent (ACP Driver)
```yaml
agents:
  endpoints:
    - endpoint_ref: "local-codex-acp"
      provider_kind: "acp"
      command: ["/usr/local/bin/agent-cli", "--acp"]
      max_turns: 25
      token_budget: 100000
      transcript_retention: "sealed"
```

---

## 2. Managing Credentials with Least Exposure (Invariant AUTH-05)

To prevent credential leakage:
1. **Never commit plaintext keys:** Credentials are never written into `server.yaml`, plan items, or trace logs.
2. **Environment Allowlisting:** SEA Forge strips all ambient parent environment variables. It forwards *only* the specific variable named by `credential_env_var` (e.g. `ANTHROPIC_API_KEY`) to the outbound request.
3. **SOPS / age Integration:**
   Store your development secrets in an encrypted `.enc.env` file:
   ```sh
   sops -e .env > .enc.env
   ```
   Before running `sea-forge`, decrypt into your active shell session:
   ```sh
   export ANTHROPIC_API_KEY=$(sops -d .enc.env | grep ANTHROPIC_API_KEY | cut -d= -f2)
   ```

---

## 3. Verify the Endpoint with `agent probe`

Run the CLI `agent probe` command to test connectivity without executing any workload side effects:

```sh
./target/debug/sea-forge agent probe \
  --endpoint anthropic-claude-3-5-sonnet \
  --root $SEA_FORGE_ROOT
```

### Expected Output:
```text
Agent probe successful:
  Endpoint: anthropic-claude-3-5-sonnet
  Provider: anthropic
  Model: claude-3-5-sonnet-20241022
  Latency: 380ms
  Status: operational
  Evidence: evi_0001 (recorded in .sea-forge/evidence.jsonl)
```

The probe confirms that the network endpoint is reachable, the credentials are valid, and the model is responding. A trace and evidence record of the probe is committed to the ledger.

---

## 4. Common Failure Modes & Troubleshooting

* **Symptom:** Probe fails with `missing_credential_error`.
  * **Cause:** The environment variable specified in `credential_env_var` is not set in the server's process environment.
  * **Fix:** Export the variable in the shell running `sea-forge-server` and reload the server.
* **Symptom:** Probe fails with `network_posture_denied`.
  * **Cause:** The server policy or host jail blocks outbound HTTPS connections to `api.anthropic.com`.
  * **Fix:** Ensure the host allows outbound TCP port 443 to the specified host.
* **Symptom:** ACP agent fails with `acp_spawn_failed`.
  * **Cause:** The binary path specified in `command` does not exist or lacks execute permissions (`chmod +x`).
  * **Fix:** Provide an absolute, validated path to the agent binary.
