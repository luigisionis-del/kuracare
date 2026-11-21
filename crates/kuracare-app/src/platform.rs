//! Where the app keeps its data, and the shared vault.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use kuracare_core::{SqliteStore, Vault};

static VAULT: OnceLock<Arc<Vault<SqliteStore>>> = OnceLock::new();

/// App-private directory. On Android: the sandboxed data dir of the package.
pub fn data_dir() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        PathBuf::from("/data/data/org.kuracare.app/files")
    }
    #[cfg(not(target_os = "android"))]
    {
        std::env::var_os("KURACARE_DATA_DIR")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("XDG_DATA_HOME").map(|d| PathBuf::from(d).join("kuracare"))
            })
            .or_else(|| {
                std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share/kuracare"))
            })
            .unwrap_or_else(|| PathBuf::from("kuracare-data"))
    }
}

pub fn db_path() -> PathBuf {
    data_dir().join("kuracare.db")
}

pub fn vault() -> Arc<Vault<SqliteStore>> {
    VAULT
        .get_or_init(|| {
            std::fs::create_dir_all(data_dir()).expect("cannot create the data directory");
            Arc::new(Vault::new(
                SqliteStore::open(db_path()).expect("cannot open the database"),
            ))
        })
        .clone()
}
