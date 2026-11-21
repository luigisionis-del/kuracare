//! Persistence. Only record ids and the vault header are stored in clear:
//! kinds, timestamps and values are inside the sealed bytes.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::crypto::{KdfParams, Sealed};
use crate::error::Result;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultHeader {
    pub format: u32,
    pub kdf: KdfParams,
    pub wrapped_dek: Vec<u8>,
}

pub trait Store {
    fn header(&self) -> Result<Option<VaultHeader>>;
    fn put_header(&self, h: &VaultHeader) -> Result<()>;
    fn put(&self, id: &str, s: &Sealed) -> Result<()>;
    fn get(&self, id: &str) -> Result<Option<Sealed>>;
    fn delete(&self, id: &str) -> Result<()>;
    fn list(&self, prefix: &str) -> Result<Vec<(String, Sealed)>>;
    /// Raw ids, for tests and diagnostics.
    fn ids(&self) -> Result<Vec<String>>;
    /// Irreversible: removes everything, header included. The store stays
    /// usable (empty), so a new vault can be created right away.
    fn destroy(&self) -> Result<()>;
}

/// SQLite file in the app's private directory.
pub struct SqliteStore {
    path: PathBuf,
    conn: Mutex<Option<Connection>>,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let conn = Self::connect(&path)?;
        Ok(Self {
            path,
            conn: Mutex::new(Some(conn)),
        })
    }

    fn connect(path: &Path) -> Result<Connection> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA secure_delete = ON;
             CREATE TABLE IF NOT EXISTS vault (id INTEGER PRIMARY KEY CHECK (id = 1), json TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS records (id TEXT PRIMARY KEY, v INTEGER NOT NULL, iv BLOB NOT NULL, ct BLOB NOT NULL);",
        )?;
        Ok(conn)
    }

    fn with<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let guard = self.conn.lock().expect("store mutex poisoned");
        let conn = guard.as_ref().ok_or(crate::error::Error::NoVault)?;
        f(conn)
    }
}

fn row_sealed(r: &rusqlite::Row<'_>, offset: usize) -> rusqlite::Result<Sealed> {
    Ok(Sealed {
        v: r.get(offset)?,
        iv: r.get(offset + 1)?,
        ct: r.get(offset + 2)?,
    })
}

impl Store for SqliteStore {
    fn header(&self) -> Result<Option<VaultHeader>> {
        self.with(|c| {
            let json: Option<String> = c
                .query_row("SELECT json FROM vault WHERE id = 1", [], |r| r.get(0))
                .optional()?;
            Ok(json.map(|j| serde_json::from_str(&j)).transpose()?)
        })
    }

    fn put_header(&self, h: &VaultHeader) -> Result<()> {
        let json = serde_json::to_string(h)?;
        self.with(|c| {
            c.execute(
                "INSERT OR REPLACE INTO vault (id, json) VALUES (1, ?1)",
                params![json],
            )?;
            Ok(())
        })
    }

    fn put(&self, id: &str, s: &Sealed) -> Result<()> {
        self.with(|c| {
            c.execute(
                "INSERT OR REPLACE INTO records (id, v, iv, ct) VALUES (?1, ?2, ?3, ?4)",
                params![id, s.v, s.iv, s.ct],
            )?;
            Ok(())
        })
    }

    fn get(&self, id: &str) -> Result<Option<Sealed>> {
        self.with(|c| {
            Ok(c.query_row(
                "SELECT v, iv, ct FROM records WHERE id = ?1",
                params![id],
                |r| row_sealed(r, 0),
            )
            .optional()?)
        })
    }

    fn delete(&self, id: &str) -> Result<()> {
        self.with(|c| {
            c.execute("DELETE FROM records WHERE id = ?1", params![id])?;
            Ok(())
        })
    }

    fn list(&self, prefix: &str) -> Result<Vec<(String, Sealed)>> {
        self.with(|c| {
            let mut st = c.prepare(
                "SELECT id, v, iv, ct FROM records WHERE substr(id, 1, length(?1)) = ?1",
            )?;
            let rows = st.query_map(params![prefix], |r| Ok((r.get(0)?, row_sealed(r, 1)?)))?;
            Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
        })
    }

    fn ids(&self) -> Result<Vec<String>> {
        self.with(|c| {
            let mut st = c.prepare("SELECT id FROM records")?;
            let rows = st.query_map([], |r| r.get(0))?;
            Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
        })
    }

    fn destroy(&self) -> Result<()> {
        let mut guard = self.conn.lock().expect("store mutex poisoned");
        if let Some(conn) = guard.take() {
            conn.close().map_err(|(_, e)| e)?;
        }
        for suffix in ["", "-wal", "-shm"] {
            let p = PathBuf::from(format!("{}{suffix}", self.path.display()));
            match std::fs::remove_file(&p) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        *guard = Some(Self::connect(&self.path)?);
        Ok(())
    }
}

/// In-memory store, for tests and previews.
#[derive(Default)]
pub struct MemStore {
    header: Mutex<Option<VaultHeader>>,
    records: Mutex<BTreeMap<String, Sealed>>,
}

impl Store for MemStore {
    fn header(&self) -> Result<Option<VaultHeader>> {
        Ok(self.header.lock().unwrap().clone())
    }
    fn put_header(&self, h: &VaultHeader) -> Result<()> {
        *self.header.lock().unwrap() = Some(h.clone());
        Ok(())
    }
    fn put(&self, id: &str, s: &Sealed) -> Result<()> {
        self.records.lock().unwrap().insert(id.into(), s.clone());
        Ok(())
    }
    fn get(&self, id: &str) -> Result<Option<Sealed>> {
        Ok(self.records.lock().unwrap().get(id).cloned())
    }
    fn delete(&self, id: &str) -> Result<()> {
        self.records.lock().unwrap().remove(id);
        Ok(())
    }
    fn list(&self, prefix: &str) -> Result<Vec<(String, Sealed)>> {
        Ok(self
            .records
            .lock()
            .unwrap()
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect())
    }
    fn ids(&self) -> Result<Vec<String>> {
        Ok(self.records.lock().unwrap().keys().cloned().collect())
    }
    fn destroy(&self) -> Result<()> {
        *self.header.lock().unwrap() = None;
        self.records.lock().unwrap().clear();
        Ok(())
    }
}
