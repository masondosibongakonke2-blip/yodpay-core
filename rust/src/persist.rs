//! Lightweight edge persistence so ConfirmedLocal state survives reboot

use rusqlite::{params, Connection, Result as SqlResult};
use serde_json;
use crate::Transaction; // your existing Transaction type

pub struct EdgeStore {
    conn: Connection,
}

impl EdgeStore {
    pub fn open(path: &str) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS transactions (
                id TEXT PRIMARY KEY,
                payload TEXT NOT NULL,
                state TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS spent_nonces (
                principal TEXT NOT NULL,
                nonce INTEGER NOT NULL,
                PRIMARY KEY (principal, nonce)
            );"
        )?;
        Ok(Self { conn })
    }

    pub fn save_tx(&self, tx: &Transaction) -> SqlResult<()> {
        let payload = serde_json::to_string(tx).unwrap();
        self.conn.execute(
            "INSERT OR REPLACE INTO transactions (id, payload, state, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![tx.id, payload, format!("{:?}", tx.state), chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn load_all_pending_and_confirmed(&self) -> SqlResult<Vec<Transaction>> {
        let mut stmt = self.conn.prepare(
            "SELECT payload FROM transactions WHERE state IN ('PendingOffline', 'ConfirmedLocal')"
        )?;
        let rows = stmt.query_map([], |row| {
            let payload: String = row.get(0)?;
            Ok(serde_json::from_str(&payload).unwrap())
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn mark_nonce_spent(&self, principal: &str, nonce: u64) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO spent_nonces (principal, nonce) VALUES (?1, ?2)",
            params![principal, nonce as i64],
        )?;
        Ok(())
    }
          }
