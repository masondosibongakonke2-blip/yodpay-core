//! Edge persistence – survives device reboot

use rusqlite::{params, Connection, Result as SqlResult};
use serde_json;
use crate::Transaction;

pub struct EdgeStore {
    conn: Connection,
}

impl EdgeStore {
    pub fn open(path: &str) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS transactions (
                id TEXT PRIMARY KEY,
                payload TEXT NOT NULL,
                state TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS spent_nonces (
                principal TEXT NOT NULL,
                nonce INTEGER NOT NULL,
                PRIMARY KEY (principal, nonce)
            );
            ",
        )?;
        Ok(Self { conn })
    }

    pub fn save_tx(&self, tx: &Transaction) -> SqlResult<()> {
        let payload = serde_json::to_string(tx).unwrap();
        self.conn.execute(
            "INSERT OR REPLACE INTO transactions (id, payload, state, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                tx.id,
                payload,
                format!("{:?}", tx.status),
                chrono::Utc::now().to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn mark_nonce_spent(&self, principal: &str, nonce: u64) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO spent_nonces (principal, nonce) VALUES (?1, ?2)",
            params![principal, nonce as i64],
        )?;
        Ok(())
    }
}
