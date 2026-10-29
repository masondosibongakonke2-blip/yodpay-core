#[derive(Debug, Clone, PartialEq)]
pub enum SystemEvent {
    SecurityModeTransition { from: String, to: String },
    TransactionRejected { nonce: u64, reason: String },
    LocalSyncCompleted { peers_synced: u8, balance_updated: f64 },
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp_utc: u32,
    pub event: SystemEvent,
}

pub struct LocalAuditLedger {
    pub entries: Vec<AuditEntry>,
    max_capacity: usize,
}

impl LocalAuditLedger {
    /// Initializes a strict memory-capped audit log to prevent memory leaks on the device
    pub fn new(max_capacity: usize) -> Self {
        LocalAuditLedger {
            entries: Vec::with_capacity(max_capacity),
            max_capacity,
        }
    }

    /// Appends a new system event. Automatically drops the oldest log if capacity is reached.
    pub fn log_event(&mut self, timestamp: u32, event: SystemEvent) {
        if self.entries.len() >= self.max_capacity {
            self.entries.remove(0); // Maintain a rolling memory window
        }
        self.entries.push(AuditEntry {
            timestamp_utc: timestamp,
            event,
        });
    }

    /// Counts how many transaction rejections have occurred in the current window
    pub fn total_rejections(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| matches!(e.event, SystemEvent::TransactionRejected { .. }))
            .count()
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_audit_capacity() {
        // Enforce a tight memory cap of 2 entries for testing
        let mut ledger = LocalAuditLedger::new(2);

        ledger.log_event(132450, SystemEvent::SecurityModeTransition { 
            from: "3f+1".to_string(), 
            to: "2f+1".to_string() 
        });
        ledger.log_event(132455, SystemEvent::TransactionRejected { 
            nonce: 8842, 
            reason: "DuplicateNonceDetected".to_string() 
        });
        
        // This third event should trigger the rolling cap and drop the first event
        ledger.log_event(132460, SystemEvent::LocalSyncCompleted { 
            peers_synced: 4, 
            balance_updated: 14250.00 
        });

        assert_eq!(ledger.entries.len(), 2);
        assert_eq!(ledger.total_rejections(), 1);
        
        // Verify the oldest event was dropped and the remaining match
        if let SystemEvent::SecurityModeTransition { .. } = ledger.entries[0].event {
            panic!("Oldest event should have been removed by rolling capacity limits");
        }
    }
}
