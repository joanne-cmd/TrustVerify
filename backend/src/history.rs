//! History store for attestation quotes: persistence, TCB regression and migration detection.

use rusqlite::params;
use serde::Serialize;

/// A single stored quote record.
#[derive(Debug, Clone, Serialize)]
pub struct QuoteRecord {
    pub ppid: String,
    pub tcb_svn: String,
    pub mr_td: String,
    pub timestamp: String,
    pub provider: Option<String>,
}

/// TCB version went backwards (security regression).
#[derive(Debug, Clone, Serialize)]
pub struct RegressionEvent {
    pub previous_svn: String,
    pub current_svn: String,
}

/// PPID changed (VM migrated to different hardware).
#[derive(Debug, Clone, Serialize)]
pub struct MigrationEvent {
    pub previous_ppid: String,
    pub current_ppid: String,
}

pub struct HistoryStore {
    conn: rusqlite::Connection,
}

impl HistoryStore {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS quotes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ppid TEXT NOT NULL,
                tcb_svn TEXT NOT NULL,
                mr_td TEXT NOT NULL,
                provider TEXT,
                timestamp TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_quotes_ppid ON quotes(ppid);
            CREATE INDEX IF NOT EXISTS idx_quotes_timestamp ON quotes(timestamp);
            ",
        )?;
        Ok(Self { conn })
    }

    pub fn insert(&self, record: &QuoteRecord) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO quotes (ppid, tcb_svn, mr_td, provider, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                record.ppid,
                record.tcb_svn,
                record.mr_td,
                record.provider,
                record.timestamp,
            ],
        )?;
        Ok(())
    }

    /// Regression = any stored TCB SVN has a byte greater than current at the same position (numeric comparison).
    pub fn detect_regression(
        &self,
        ppid: &str,
        current_svn_hex: &str,
    ) -> rusqlite::Result<Option<RegressionEvent>> {
        let records = self.list_by_ppid(ppid)?;
        let current_bytes = hex::decode(current_svn_hex.trim()).unwrap_or_default();

        for record in &records {
            let stored_bytes = hex::decode(record.tcb_svn.trim()).unwrap_or_default();
            let is_higher = stored_bytes
                .iter()
                .zip(current_bytes.iter())
                .any(|(s, c)| s > c);
            if is_higher {
                return Ok(Some(RegressionEvent {
                    previous_svn: record.tcb_svn.clone(),
                    current_svn: current_svn_hex.to_string(),
                }));
            }
        }
        Ok(None)
    }

    /// Returns MigrationEvent if the most recent record has a different PPID (suggests migration).
    pub fn detect_migration(&self, current_ppid: &str) -> Option<MigrationEvent> {
        let mut stmt = self
            .conn
            .prepare("SELECT ppid FROM quotes ORDER BY timestamp DESC LIMIT 1")
            .ok()?;
        let mut rows = stmt.query([]).ok()?;
        if let Some(row) = rows.next().ok()? {
            let previous_ppid: String = row.get(0).ok()?;
            if previous_ppid.trim().to_lowercase() != current_ppid.trim().to_lowercase() {
                return Some(MigrationEvent {
                    previous_ppid,
                    current_ppid: current_ppid.to_string(),
                });
            }
        }
        None
    }

    /// List all records for a given PPID (for GET /api/history).
    pub fn list_by_ppid(&self, ppid: &str) -> Result<Vec<QuoteRecord>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT ppid, tcb_svn, mr_td, timestamp, provider FROM quotes WHERE ppid = ?1 ORDER BY timestamp DESC",
        )?;
        let rows = stmt.query_map(params![ppid], |row| {
            Ok(QuoteRecord {
                ppid: row.get(0)?,
                tcb_svn: row.get(1)?,
                mr_td: row.get(2)?,
                timestamp: row.get(3)?,
                provider: row.get(4)?,
            })
        })?;
        rows.collect()
    }
}

