// ============================================================
// YODPAY Core – Full Integrated Version
// ============================================================
// Adaptive Consensus + Signatures + Human Authority
// Persistence + Telemetry + Mesh + Reconciliation + Keystore
// ============================================================

mod crypto;
mod persist;
mod telemetry;
mod mesh;
mod reconciliation;
mod keystore;

use crypto::{SignedIntent, require_human_approval_if_extreme};
use persist::EdgeStore;
use telemetry::SatelliteTelemetry;
use mesh::{InMemoryMesh, MeshTransport};
use reconciliation::Reconciler;
use keystore::Keystore;

use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TxStatus {
    PendingOffline,
    ConfirmedLocal,
    Settled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
    pub timestamp: u64,
    pub status: TxStatus,
    pub signature: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct Wallet {
    pub id: String,
    pub balance: u64,
    pub offline_limit: u64,
    pub spent_offline: u64,
    pub nonce: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NetworkMode {
    Normal,      // 3f+1
    Partitioned, // 2f+1
    Extreme,     // 1f+1 + human approval
}

pub struct YodPayNode {
    pub node_id: String,
    pub wallets: HashMap<String, Wallet>,
    pub pending: HashMap<String, Transaction>,
    pub confirmed: HashMap<String, Transaction>,
    pub spent_nonces: HashSet<(String, u64)>,
    pub mode: NetworkMode,
    pub store: Option<EdgeStore>,
    pub mesh: InMemoryMesh,
    pub reconciler: Reconciler,
    pub keystore: Keystore,
}

impl YodPayNode {
    pub fn new(node_id: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
            wallets: HashMap::new(),
            pending: HashMap::new(),
            confirmed: HashMap::new(),
            spent_nonces: HashSet::new(),
            mode: NetworkMode::Normal,
            store: None,
            mesh: InMemoryMesh::new(),
            reconciler: Reconciler::new(),
            keystore: Keystore::new(),
        }
    }

    pub fn with_persistence(mut self, path: &str) -> Self {
        if let Ok(store) = EdgeStore::open(path) {
            self.store = Some(store);
            println!("[YODPAY] Edge persistence enabled");
        }
        self
    }

    pub fn set_mode(&mut self, mode: NetworkMode) {
        if self.mode != mode {
            println!("[YODPAY] Mode change: {:?} → {:?}", self.mode, mode);
            self.mode = mode;
        }
    }

    pub fn update_mode_from_telemetry(&mut self, telemetry: &SatelliteTelemetry) {
        let new_mode = match telemetry.network_status {
            0 => NetworkMode::Normal,
            1 => NetworkMode::Partitioned,
            _ => NetworkMode::Extreme,
        };
        self.set_mode(new_mode);
    }

    pub fn create_wallet(&mut self, id: &str, balance: u64, offline_limit: u64) {
        // Generate cryptographic identity for the wallet
        self.keystore.generate_keypair(id);

        self.wallets.insert(
            id.to_string(),
            Wallet {
                id: id.to_string(),
                balance,
                offline_limit,
                spent_offline: 0,
                nonce: 0,
            },
        );
        println!("[YODPAY] Wallet '{}' created", id);
    }

    pub fn send_offline(
        &mut self,
        from: &str,
        to: &str,
        amount: u64,
    ) -> Result<Transaction, String> {
        let wallet = self.wallets.get_mut(from).ok_or("Wallet not found")?;

        if wallet.spent_offline + amount > wallet.offline_limit {
            return Err("Offline limit exceeded".into());
        }
        if amount > wallet.balance {
            return Err("Insufficient balance".into());
        }

        let nonce_key = (from.to_string(), wallet.nonce);
        if self.spent_nonces.contains(&nonce_key) {
            return Err("Possible double-spend detected".into());
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let tx_id = format!("tx_{}_{}", from, wallet.nonce);

        let intent = SignedIntent {
            tx_id: tx_id.clone(),
            from: from.to_string(),
            to: to.to_string(),
            amount,
            nonce: wallet.nonce,
            timestamp,
            signature: vec![],
        };

        // Sign with keystore
        let signature = self
            .keystore
            .sign(from, &intent.message_bytes())
            .map_err(|e| e.to_string())?;

        let tx = Transaction {
            id: tx_id.clone(),
            from: from.to_string(),
            to: to.to_string(),
            amount,
            nonce: wallet.nonce,
            timestamp,
            status: TxStatus::PendingOffline,
            signature: Some(signature.to_bytes().to_vec()),
        };

        // Update local state
        self.spent_nonces.insert(nonce_key);
        wallet.nonce += 1;
        wallet.spent_offline += amount;
        wallet.balance -= amount;

        // Persist
        if let Some(store) = &self.store {
            let _ = store.save_tx(&tx);
            let _ = store.mark_nonce_spent(from, tx.nonce);
        }

        // Broadcast to local mesh
        let _ = self.mesh.broadcast(&tx);

        self.pending.insert(tx_id.clone(), tx.clone());
        println!("[YODPAY] Offline tx created & broadcasted: {} → {} | {}", from, to, amount);

        Ok(tx)
    }

    pub fn confirm_local(
        &mut self,
        tx_id: &str,
        human_verifying_key: Option<&VerifyingKey>,
        human_sig: Option<&Signature>,
    ) -> Result<(), String> {
        let tx = self.pending.get(tx_id).ok_or("Transaction not found")?.clone();

        // Extreme mode requires human cryptographic approval
        if self.mode == NetworkMode::Extreme {
            let human_key = human_verifying_key.ok_or("Human key required in Extreme mode")?;
            let human_sig = human_sig.ok_or("Human signature required in Extreme mode")?;

            let intent = SignedIntent {
                tx_id: tx.id.clone(),
                from: tx.from.clone(),
                to: tx.to.clone(),
                amount: tx.amount,
                nonce: tx.nonce,
                timestamp: tx.timestamp,
                signature: vec![],
            };

            require_human_approval_if_extreme("Extreme", &intent, human_key, human_sig)
                .map_err(|e| e.to_string())?;
        }

        let mut confirmed_tx = tx;
        confirmed_tx.status = TxStatus::ConfirmedLocal;

        self.pending.remove(tx_id);
        self.confirmed.insert(tx_id.to_string(), confirmed_tx);

        println!("[YODPAY] Local finality reached: {} ({:?})", tx_id, self.mode);
        Ok(())
    }

    pub fn reconcile_online(&mut self) -> Result<(), String> {
        let local_confirmed: Vec<Transaction> = self.confirmed.values().cloned().collect();

        let report = self.reconciler.reconcile_batch(local_confirmed);

        println!("[RECONCILIATION] Accepted: {}", report.accepted.len());
        println!("[RECONCILIATION] Conflicts: {}", report.conflicts.len());
        println!("[RECONCILIATION] Rejected: {}", report.rejected.len());

        // Move accepted transactions to settled
        for tx_id in &report.accepted {
            if let Some(tx) = self.confirmed.remove(tx_id) {
                if let Some(receiver) = self.wallets.get_mut(&tx.to) {
                    receiver.balance += tx.amount;
                }
                if let Some(sender) = self.wallets.get_mut
