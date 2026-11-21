//! Encrypted key-value store. Nothing can be written before a passphrase
//! exists (`create`) or while locked.

use std::sync::RwLock;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::crypto::{self, KdfParams, Key};
use crate::error::{Error, Result};
use crate::store::{Store, VaultHeader};

pub struct Vault<S: Store> {
    store: S,
    kdf_cost: KdfParams,
    /// Wiped from memory (Zeroizing) when dropped, i.e. on lock.
    dek: RwLock<Option<Key>>,
}

impl<S: Store> Vault<S> {
    pub fn new(store: S) -> Self {
        Self::with_kdf(store, KdfParams::strong())
    }

    /// `kdf_cost` is used for new vaults and passphrase changes (salt is always fresh).
    pub fn with_kdf(store: S, kdf_cost: KdfParams) -> Self {
        Self {
            store,
            kdf_cost,
            dek: RwLock::new(None),
        }
    }

    pub fn store(&self) -> &S {
        &self.store
    }

    pub fn exists(&self) -> Result<bool> {
        Ok(self.store.header()?.is_some())
    }

    pub fn is_unlocked(&self) -> bool {
        self.dek.read().unwrap().is_some()
    }

    pub fn create(&self, passphrase: &str) -> Result<()> {
        if self.exists()? {
            return Err(Error::AlreadyExists);
        }
        let kdf = self.kdf_cost.renewed();
        let kek = crypto::derive_kek(passphrase, &kdf)?;
        let dek = crypto::new_dek();
        let wrapped_dek = crypto::wrap_dek(&dek, &kek)?;
        self.store.put_header(&VaultHeader {
            format: 1,
            kdf,
            wrapped_dek,
        })?;
        *self.dek.write().unwrap() = Some(dek);
        Ok(())
    }

    pub fn unlock(&self, passphrase: &str) -> Result<()> {
        let h = self.store.header()?.ok_or(Error::NoVault)?;
        let kek = crypto::derive_kek(passphrase, &h.kdf)?;
        let dek = crypto::unwrap_dek(&h.wrapped_dek, &kek)?;
        *self.dek.write().unwrap() = Some(dek);
        Ok(())
    }

    pub fn lock(&self) {
        *self.dek.write().unwrap() = None;
    }

    pub fn change_passphrase(&self, old: &str, new: &str) -> Result<()> {
        let h = self.store.header()?.ok_or(Error::NoVault)?;
        let dek = crypto::unwrap_dek(&h.wrapped_dek, &crypto::derive_kek(old, &h.kdf)?)?;
        let kdf = self.kdf_cost.renewed();
        let wrapped_dek = crypto::wrap_dek(&dek, &crypto::derive_kek(new, &kdf)?)?;
        self.store.put_header(&VaultHeader {
            kdf,
            wrapped_dek,
            ..h
        })
    }

    fn with_key<T>(&self, f: impl FnOnce(&Key) -> Result<T>) -> Result<T> {
        let guard = self.dek.read().unwrap();
        f(guard.as_ref().ok_or(Error::Locked)?)
    }

    pub fn put<T: Serialize>(&self, id: &str, value: &T) -> Result<()> {
        self.with_key(|k| {
            let plain = zeroize::Zeroizing::new(serde_json::to_vec(value)?);
            self.store.put(id, &crypto::seal(k, id, &plain)?)
        })
    }

    pub fn get<T: DeserializeOwned>(&self, id: &str) -> Result<Option<T>> {
        self.with_key(|k| match self.store.get(id)? {
            None => Ok(None),
            Some(s) => Ok(Some(serde_json::from_slice(&crypto::unseal(k, id, &s)?)?)),
        })
    }

    pub fn remove(&self, id: &str) -> Result<()> {
        self.with_key(|_| self.store.delete(id))
    }

    /// Decrypts every record whose id starts with `prefix`. Volumes are small (thousands).
    pub fn list<T: DeserializeOwned>(&self, prefix: &str) -> Result<Vec<T>> {
        self.with_key(|k| {
            self.store
                .list(prefix)?
                .into_iter()
                .map(|(id, s)| Ok(serde_json::from_slice(&crypto::unseal(k, &id, &s)?)?))
                .collect()
        })
    }

    /// Irreversible: deletes everything, header included.
    pub fn destroy(&self) -> Result<()> {
        self.lock();
        self.store.destroy()
    }
}

#[cfg(test)]
pub(crate) fn test_vault() -> Vault<crate::store::MemStore> {
    Vault::with_kdf(crate::store::MemStore::default(), crypto::fast_kdf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::SqliteStore;

    #[test]
    fn nothing_can_be_saved_before_a_passphrase() {
        let v = test_vault();
        assert!(!v.exists().unwrap());
        assert!(matches!(v.put("evt:1", &1), Err(Error::Locked)));
        assert!(v.store().ids().unwrap().is_empty());
    }

    #[test]
    fn lock_refuses_access_and_unlock_needs_the_right_passphrase() {
        let v = test_vault();
        v.create("una frase lunga e mia").unwrap();
        v.put("evt:1", &serde_json::json!({"umore": 6})).unwrap();
        v.lock();
        assert!(!v.is_unlocked());
        assert!(matches!(
            v.get::<serde_json::Value>("evt:1"),
            Err(Error::Locked)
        ));
        assert!(matches!(
            v.unlock("altra frase"),
            Err(Error::WrongPassphrase)
        ));
        v.unlock("una frase lunga e mia").unwrap();
        assert_eq!(
            v.get::<serde_json::Value>("evt:1").unwrap().unwrap()["umore"],
            6
        );
    }

    #[test]
    fn change_passphrase_keeps_data() {
        let v = test_vault();
        v.create("vecchia frase segreta").unwrap();
        v.put("evt:1", &"dato").unwrap();
        v.change_passphrase("vecchia frase segreta", "nuova frase segreta")
            .unwrap();
        v.lock();
        assert!(matches!(
            v.unlock("vecchia frase segreta"),
            Err(Error::WrongPassphrase)
        ));
        v.unlock("nuova frase segreta").unwrap();
        assert_eq!(v.get::<String>("evt:1").unwrap().unwrap(), "dato");
    }

    #[test]
    fn create_twice_is_refused() {
        let v = test_vault();
        v.create("una frase lunga e mia").unwrap();
        assert!(matches!(v.create("altra"), Err(Error::AlreadyExists)));
    }

    #[test]
    fn list_by_prefix() {
        let v = test_vault();
        v.create("una frase lunga e mia").unwrap();
        v.put("evt:a", &1).unwrap();
        v.put("evt:b", &2).unwrap();
        v.put("cfg:settings", &3).unwrap();
        let mut got: Vec<i32> = v.list("evt:").unwrap();
        got.sort();
        assert_eq!(got, vec![1, 2]);
    }

    #[test]
    fn sqlite_file_holds_only_ids_and_ciphertext() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kuracare.db");
        let v = Vault::with_kdf(SqliteStore::open(&path).unwrap(), crypto::fast_kdf());
        v.create("una frase lunga e mia").unwrap();
        v.put(
            "evt:1",
            &serde_json::json!({"kind": "checkin", "nota": "passeggiata al mare"}),
        )
        .unwrap();
        drop(v);
        let bytes = std::fs::read(&path).unwrap();
        let wal = std::fs::read(dir.path().join("kuracare.db-wal")).unwrap_or_default();
        for b in [&bytes, &wal] {
            let text = String::from_utf8_lossy(b);
            assert!(!text.contains("checkin"));
            assert!(!text.contains("passeggiata"));
        }
        // ...and the data is still readable after reopening.
        let v = Vault::with_kdf(SqliteStore::open(&path).unwrap(), crypto::fast_kdf());
        v.unlock("una frase lunga e mia").unwrap();
        assert_eq!(
            v.get::<serde_json::Value>("evt:1").unwrap().unwrap()["nota"],
            "passeggiata al mare"
        );
    }

    #[test]
    fn destroy_removes_the_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kuracare.db");
        let v = Vault::with_kdf(SqliteStore::open(&path).unwrap(), crypto::fast_kdf());
        v.create("una frase lunga e mia").unwrap();
        v.put("evt:1", &1).unwrap();
        v.destroy().unwrap();
        assert!(!path.exists());
        assert!(!dir.path().join("kuracare.db-wal").exists());
    }
}
