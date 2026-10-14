// ============================================================
// YODPAY – Offline-First Payment Core (Rust)
// ============================================================

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: String,
    pub balance: u64,
    pub offline_limit: u64,
    pub spent_offline: u64,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
    pub timestamp: u64,
    pub status: TxStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TxStatus {
    PendingOffline,
    ConfirmedLocal,
    Settled,
}

#[derive(Debug, Default)]
pub struct YodPayNode {
    pub wallets: HashMap<String, Wallet>,
    pub pending: Vec<Transaction>,
    pub confirmed: Vec<Transaction>,
    pub spent_nonces: HashSet<(String, u64)>,
}

impl YodPayNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_wallet(&mut self, id: &str, initial_balance: u64, offline_limit: u64) {
        self.wallets.insert(
            id.to_string(),
            Wallet {
                id: id.to_string(),
                balance: initial_balance,
                offline_limit,
                spent_offline: 0,
                nonce: 0,
            },
        );
        println!(
            "[NODE] Wallet '{}' created | Balance: {} | Offline limit: {}",
            id, initial_balance, offline_limit
        );
    }

    pub fn send_offline(&mut self, from: &str, to: &str, amount: u64) -> Result<Transaction, String> {
        let wallet = self
            .wallets
            .get_mut(from)
            .ok_or_else(|| "Wallet not found".to_string())?;

        if wallet.spent_offline + amount > wallet.offline_limit {
            return Err(format!(
                "Offline limit exceeded. Spent: {}, Limit: {}, Tried: {}",
                wallet.spent_offline, wallet.offline_limit, amount
            ));
        }

        if amount > wallet.balance {
            return Err("Insufficient balance".to_string());
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Clock error: {}", e))?
            .as_secs();

        let tx = Transaction {
            id: format!("tx_{}_{}", from, wallet.nonce),
            from: from.to_string(),
            to: to.to_string(),
            amount,
            nonce: wallet.nonce,
            timestamp: now,
            status: TxStatus::PendingOffline,
        };

        if !self.spent_nonces.insert((from.to_string(), wallet.nonce)) {
            return Err("Nonce already used – possible double-spend".to_string());
        }

        wallet.nonce += 1;
        wallet.spent_offline += amount;
        wallet.balance -= amount;

        self.pending.push(tx.clone());

        println!(
            "[OFFLINE] Created tx {}: {} → {} | Amount: {}",
            tx.id, from, to, amount
        );

        Ok(tx)
    }

    pub fn confirm_local(&mut self, tx_id: &str) -> Result<(), String> {
        if let Some(pos) = self.pending.iter().position(|t| t.id == tx_id) {
            let mut tx = self.pending.remove(pos);
            tx.status = TxStatus::ConfirmedLocal;
            self.confirmed.push(tx);
            println!("[QUORUM] Transaction {} reached local finality", tx_id);
            Ok(())
        } else {
            Err("Transaction not found in pending".to_string())
        }
    }

    pub fn settle_online(&mut self, tx_id: &str) -> Result<(), String> {
        if let Some(pos) = self.confirmed.iter().position(|t| t.id == tx_id) {
            let mut tx = self.confirmed.remove(pos);
            tx.status = TxStatus::Settled;

            let sender_id = tx.from.clone();
            let receiver_id = tx.to.clone();
            let amount = tx.amount;

            if let Some(receiver) = self.wallets.get_mut(&receiver_id) {
                receiver.balance += amount;
            }

            if let Some(sender) = self.wallets.get_mut(&sender_id) {
                sender.spent_offline = sender.spent_offline.saturating_sub(amount);
            }

            println!("[SETTLED] Transaction {} settled on the network", tx_id);
            Ok(())
        } else {
            Err("Transaction not found in confirmed".to_string())
        }
    }

    pub fn print_status(&self) {
        println!("\n========== YODPAY NODE STATUS ==========");
        for (id, wallet) in &self.wallets {
            println!(
                "Wallet {:<12} | Balance: {:<6} | Offline: {}/{}",
                id, wallet.balance, wallet.spent_offline, wallet.offline_limit
            );
        }
        println!("----------------------------------------");
        println!(
            "Pending: {} | Confirmed: {}",
            self.pending.len(),
            self.confirmed.len()
        );
        println!("========================================\n");
    }
}

fn main() {
    let mut node = YodPayNode::new();

    println!("--- Creating wallets ---");
    node.create_wallet("alice", 10000, 2000);
    node.create_wallet("bob", 5000, 1500);
    node.print_status();

    println!("--- Sending offline payment ---");
    match node.send_offline("alice", "bob", 600) {
        Ok(tx) => {
            node.print_status();

            println!("--- Confirming locally ---");
            let _ = node.confirm_local(&tx.id);
            node.print_status();

            println!("--- Settling online ---");
            let _ = node.settle_online(&tx.id);
            node.print_status();
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("--- Testing offline limit protection ---");
    match node.send_offline("alice", "bob", 2000) {
        Ok(_) => println!("Unexpected: limit was not enforced"),
        Err(e) => println!("Correctly blocked: {}", e),
    }
  }
