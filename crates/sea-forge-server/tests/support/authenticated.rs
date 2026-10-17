//! Trusted integration-test driver for the production private-channel API.
//!
//! Only this driver holds the issuer. A returned client holds one endpoint and
//! cannot mint grants. Its actor claim is still checked by production code.
//! This proves capability authentication, not isolated host workload identity.
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole, Verdict};
use sea_forge_ledger::LedgerStream;
use sea_forge_server::workload_connection::{authentication_action, AuthenticatedConnection};
use sea_forge_server::ServerConfig;
use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{unix::OwnedReadHalf, unix::OwnedWriteHalf, UnixStream};

pub fn principal(actor_id: &str, role: ActorRole) -> Actor {
    Actor {
        actor_id: actor_id.into(),
        role,
    }
}

pub struct Cell {
    pub socket: PathBuf,
    root: PathBuf,
    sender: tokio::sync::mpsc::Sender<AuthenticatedConnection>,
    engine: PolicyAuthorityEngine,
    bundle: AuthorityPolicyBundle,
    ledger: LedgerStream,
    sequence: usize,
    server: tokio::task::JoinHandle<()>,
}

impl Cell {
    pub async fn start(config: ServerConfig) -> Self {
        let root = config.root.clone();
        let socket = config.socket_path.clone();
        let (sender, receiver) = tokio::sync::mpsc::channel(8);
        let server = tokio::spawn(async move {
            sea_forge_server::run_authenticated(config, receiver)
                .await
                .unwrap();
        });
        for _ in 0..500 {
            if socket.exists() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(
            socket.exists(),
            "authenticated server socket never appeared"
        );
        let bundle: AuthorityPolicyBundle = sea_forge_core::yaml::from_str(
            "version: \"0.1\"\nrules:\n  - name: trusted-private-connection-issuer\n    verdict: allow\n    actor_role: R-SO\n    operation_kind: identity_minting\n"
        ).unwrap();
        let engine = PolicyAuthorityEngine::new(bundle.clone()).unwrap();
        let ledger =
            LedgerStream::open(&root, "identity-issuance", "trusted-identity-issuer").unwrap();
        Self {
            socket,
            root,
            sender,
            engine,
            bundle,
            ledger,
            sequence: 0,
            server,
        }
    }

    pub async fn connect_for(&mut self, actor: &Actor, request: &Value) -> Client {
        self.sequence += 1;
        let issuer = principal("trusted-identity-issuer", ActorRole::SecurityOfficer);
        let workload = format!("workload-{}", actor.actor_id);
        let action = authentication_action(&workload, actor, request);
        let decision = self
            .engine
            .evaluate(AuthorityEvaluation {
                actor: &issuer,
                binding: self
                    .bundle
                    .resolve_identity(&issuer.actor_id, issuer.role.clone()),
                run_id: "identity-issuance",
                case_id: "identity-issuance",
                plan_item_id: "private-connection",
                sequence: self.sequence,
                action: &action,
                workspace_root: &self.root,
                evidence_refs: vec![],
                artifacts_root: None,
                timeout_secs: Some(60),
                env_keys: Default::default(),
                domainforge_candidate: None,
                opa_candidate: None,
                environment: None,
            })
            .unwrap();
        assert_eq!(decision.verdict, Verdict::Allow);
        let committed = self
            .ledger
            .commit_typed("authority_decision", vec![], &decision, vec![])
            .unwrap();
        let grant = self
            .engine
            .grant(&decision, &committed, &action, None)
            .unwrap();
        let (server, client) = UnixStream::pair().unwrap();
        let connection = AuthenticatedConnection::new(
            server,
            grant,
            &workload,
            actor.clone(),
            request,
            "identity-issuance",
            "private-connection",
            &self.root,
        )
        .unwrap();
        self.sender
            .send(connection)
            .await
            .unwrap_or_else(|_| panic!("trusted issuer disconnected"));
        let (reader, writer) = client.into_split();
        Client {
            reader: BufReader::new(reader),
            writer,
        }
    }

    pub async fn call_for(&mut self, actor: &Actor, request: Value) -> Value {
        self.connect_for(actor, &request).await.call(request).await
    }

    pub fn verify_authentication(&self) {
        self.ledger.verify().unwrap();
        let decisions = self.ledger.read_entries().unwrap();
        let evidence =
            LedgerStream::open(&self.root, "workload-authentication", "test-observer").unwrap();
        evidence.verify().unwrap();
        let records = evidence.read_entries().unwrap();
        assert!(!records.is_empty());
        for entry in records {
            assert_eq!(entry.record_kind, "workload_request_authenticated");
            let proof = &entry.payload;
            let decision = decisions
                .iter()
                .find(|decision| {
                    Some(decision.entry_ulid.as_str())
                        == proof["committed_decision"]["entry_ulid"].as_str()
                })
                .expect("authentication must reference its committed issuer decision");
            assert_eq!(entry.authority_refs, vec![decision.entry_ulid.clone()]);
            assert_eq!(
                proof["committed_decision"]["payload_hash"],
                decision.payload_hash
            );
            assert_eq!(
                proof["committed_decision"]["ledger_id"],
                "identity-issuance"
            );
            assert_eq!(
                proof["actor"],
                decision.payload["operation"]["parameters"]["actor"]
            );
            assert_eq!(
                proof["principal"],
                decision.payload["operation"]["parameters"]["principal"]
            );
            assert_eq!(
                proof["request_sha256"],
                decision.payload["operation"]["parameters"]["request_sha256"]
            );
        }
    }

    pub async fn shutdown(&mut self) {
        self.server.abort();
        assert!((&mut self.server).await.unwrap_err().is_cancelled());
    }
}

impl Drop for Cell {
    fn drop(&mut self) {
        self.server.abort();
    }
}

pub struct Client {
    writer: OwnedWriteHalf,
    reader: BufReader<OwnedReadHalf>,
}

impl Client {
    pub async fn call(&mut self, request: Value) -> Value {
        self.writer
            .write_all(format!("{request}\n").as_bytes())
            .await
            .unwrap();
        self.writer.flush().await.unwrap();
        let mut line = String::new();
        let n = tokio::time::timeout(Duration::from_secs(30), self.reader.read_line(&mut line))
            .await
            .expect("private response timed out")
            .unwrap();
        assert!(n > 0, "private connection closed unexpectedly");
        serde_json::from_str(line.trim()).unwrap()
    }
}
