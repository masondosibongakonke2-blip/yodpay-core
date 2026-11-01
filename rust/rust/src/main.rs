// ============================================================
// YODPAY Core – Offline-First Payment Protocol
// ============================================================
// Adaptive consensus • Ed25519 signatures • Human authority
// Edge persistence • Telemetry-driven mode switching
// ============================================================

mod crypto;
mod persist;
mod telemetry;

use crypto::{Keypair, SignedIntent, require_human_approval_if_extreme};
use persist::EdgeStore;
use telemetry::SatelliteTelemetry;
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
    Extreme,     // 1f+1 + human cryptographic approval
}

pub struct YodPayNode {
    pub node_id: String,
    pub wallets: HashMap<String, Wallet>,
    pub pending: HashMap<String, Transaction>,
    pub confirmed: HashMap<String, Transaction>,
    pub spent_nonces: HashSet<(String, u64)>,
    pub mode: NetworkMode,
    pub store: Option<EdgeStore>,
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
    }

    pub fn send_offline(
        &mut self,
        from: &str,
        to: &str,
        amount: u64,
        signing_key: &Keypair,
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

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
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

        let signature = signing_key.sign(&intent.message_bytes());

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

        self.spent_nonces.insert(nonce_key);
        wallet.nonce += 1;
        wallet.spent_offline += amount;
        wallet.balance -= amount;

        if let Some(store) = &self.store {
            let _ = store.save_tx(&tx);
            let _ = store.mark_nonce_spent(from, tx.nonce);
        }

        self.pending.insert(tx_id.clone(), tx.clone());
        println!("[YODPAY] Offline tx created: {} → {} | {}", from, to, amount);
        Ok(tx)
    }

    pub fn confirm_local(
        &mut self,
        tx_id: &str,
        human_key: Option<&VerifyingKey>,
        human_sig: Option<&Signature>,
    ) -> Result<(), String> {
        let tx = self.pending.get(tx_id).ok_or("Transaction not found")?.clone();

        if self.mode == NetworkMode::Extreme {
            let human_key = human_key.ok_or("Human key required")?;
            let human_sig = human_sig.ok_or("Human signature required")?;

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

    pub fn settle(&mut self, tx_id: &str) -> Result<(), String> {
        let tx = self.confirmed.remove(tx_id).ok_or("Not found")?;
        if let Some(r) = self.wallets.get_mut(&tx.to) {
            r.balance += tx.amount;
        }
        if let Some(s) = self.wallets.get_mut(&tx.from) {
            s.spent_offline = s.spent_offline.saturating_sub(tx.amount);
        }
        println!("[YODPAY] Settled: {}", tx_id);
        Ok(())
    }

    pub fn print_status(&self) {
        println!("\n=== YODPAY Node: {} | Mode: {:?} ===", self.node_id, self.mode);
        for (id, w) in &self.wallets {
            println!(
                "  {:<10} Balance: {:<7} Offline: {}/{}",
                id, w.balance, w.spent_offline, w.offline_limit
            );
        }
        println!(
            "  Pending: {} | Confirmed: {}\n",
            self.pending.len(),
            self.confirmed.len()
        );
    }
}

fn main() {
    println!("=== YODPAY Core ===\n");

    let alice_keys = Keypair::generate();
    let human_keys = Keypair::generate();

    let mut node = YodPayNode::new("node-01").with_persistence("yodpay_edge.db");

    node.create_wallet("alice", 50_000, 12_000);
    node.create_wallet("bob", 20_000, 1_500);
    node.create_wallet("clinic", 100_000, 60_000);

    // Telemetry changes network conditions
    let extreme_packet = "$YODTE,28.45,-30.12,132450,2*4D";
    if let Ok(t) = SatelliteTelemetry::parse_from_broadcast(extreme_packet) {
        node.update_mode_from_telemetry(&t);
    }

    // Extreme mode payment with human approval
    match node.send_offline("clinic", "alice", 15_000, &alice_keys) {
        Ok(tx) => {
            let intent = SignedIntent {
                tx_id: tx.id.clone(),
                from: tx.from.clone(),
                to: tx.to.clone(),
                amount: tx.amount,
                nonce: tx.nonce,
                timestamp: tx.timestamp,
                signature: vec![],
            };
            let sig = human_keys.sign(&intent.message_bytes());
            let _ = node.confirm_local(&tx.id, Some(&human_keys.verifying), Some(&sig));
        }
        Err(e) => println!("Error: {}", e),
    }

    node.print_status();
    println!("YODPAY core ready.");
}
