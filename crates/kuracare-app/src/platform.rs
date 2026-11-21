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

/// Where wry stores the raw camera photo before handing it to the page
/// (app-specific external storage, reachable over USB): emptied right after
/// a photo is read and at every start, so no unencrypted copy stays around.
pub fn capture_dir() -> Option<PathBuf> {
    #[cfg(target_os = "android")]
    {
        Some(PathBuf::from(
            "/storage/emulated/0/Android/data/org.kuracare.app/files/Pictures",
        ))
    }
    #[cfg(not(target_os = "android"))]
    {
        None
    }
}

pub fn clear_capture_dir() {
    let Some(dir) = capture_dir() else { return };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let _ = std::fs::remove_file(&path);
        }
    }
}

/// App-private cache: export files wait here for the picker, a picked backup
/// is copied here. Emptied at every start.
pub fn cache_dir() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        PathBuf::from("/data/data/org.kuracare.app/cache")
    }
    #[cfg(not(target_os = "android"))]
    {
        data_dir().join("cache")
    }
}

/// Removes leftover export/import files (never anything else).
pub fn clear_exchange_files() {
    let Ok(entries) = std::fs::read_dir(cache_dir()) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".kuracare") || name.starts_with("kuracare-") {
            let _ = std::fs::remove_file(entry.path());
        }
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
