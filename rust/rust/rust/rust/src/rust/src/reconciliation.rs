//! Deterministic Reconciliation Layer
//! Merges offline ConfirmedLocal transactions back into the wider network safely.

use crate::{Transaction, TxStatus};
use std::collections::HashSet;

#[derive(Debug)]
pub struct ReconciliationReport {
    pub accepted: Vec<String>,
    pub rejected: Vec<String>,
    pub conflicts: Vec<String>,
}

pub struct Reconciler {
    seen_nonces: HashSet<(String, u64)>,
}

impl Reconciler {
    pub fn new() -> Self {
        Self {
            seen_nonces: HashSet::new(),
        }
    }

    /// Reconcile a batch of locally confirmed transactions when connectivity returns
    pub fn reconcile_batch(&mut self, local_txs: Vec<Transaction>) -> ReconciliationReport {
        let mut report = ReconciliationReport {
            accepted: Vec::new(),
            rejected: Vec::new(),
            conflicts: Vec::new(),
        };

        for tx in local_txs {
            let nonce_key = (tx.from.clone(), tx.nonce);

            // Basic double-spend / replay protection at reconciliation time
            if self.seen_nonces.contains(&nonce_key) {
                report.conflicts.push(tx.id.clone());
                continue;
            }

            // Only accept transactions that reached local finality
            if tx.status != TxStatus::ConfirmedLocal {
                report.rejected.push(tx.id.clone());
                continue;
            }

            // Mark nonce as consumed
            self.seen_nonces.insert(nonce_key);
            report.accepted.push(tx.id.clone());
        }

        report
    }
}
