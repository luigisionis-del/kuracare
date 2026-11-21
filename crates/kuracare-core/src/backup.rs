//! Encrypted backup: a copy of the vault exactly as stored (header with the
//! wrapped key, sealed records). Nothing is decrypted to make it; opening it
//! needs the passphrase that was in use when it was made.
//!
//! In clear inside the file, as in the database: record ids (random uuids,
//! `cfg:*`, `img:*`), the key-derivation parameters and the wrapped key.

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::crypto::Sealed;
use crate::error::{Error, Result};
use crate::store::{Store, VaultHeader};

pub const FORMAT: &str = "kuracare-backup";
pub const VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    id: String,
    v: u32,
    iv: String,
    ct: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct File {
    format: String,
    version: u32,
    created_at: String,
    vault: VaultHeader,
    records: Vec<Record>,
}

fn b64() -> base64::engine::GeneralPurpose {
    base64::engine::general_purpose::STANDARD
}

/// The backup file (JSON). Works while locked: it only copies sealed bytes.
pub fn create<S: Store>(store: &S) -> Result<Vec<u8>> {
    let vault = store.header()?.ok_or(Error::NoVault)?;
    let records = store
        .list("")?
        .into_iter()
        .map(|(id, s)| Record {
            id,
            v: s.v,
            iv: b64().encode(&s.iv),
            ct: b64().encode(&s.ct),
        })
        .collect();
    let f = File {
        format: FORMAT.into(),
        version: VERSION,
        created_at: crate::time::now_iso(),
        vault,
        records,
    };
    Ok(serde_json::to_vec(&f)?)
}

/// What a backup holds, read without the passphrase.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Info {
    pub created_at: String,
    pub records: usize,
}

fn parse(bytes: &[u8]) -> Result<File> {
    let f: File = serde_json::from_slice(bytes)
        .map_err(|_| Error::Unsupported("questo file non è un backup di Kuracare".into()))?;
    if f.format != FORMAT {
        return Err(Error::Unsupported(
            "questo file non è un backup di Kuracare".into(),
        ));
    }
    if f.version > VERSION {
        return Err(Error::Unsupported(
            "il backup viene da una versione più recente di Kuracare".into(),
        ));
    }
    Ok(f)
}

pub fn info(bytes: &[u8]) -> Result<Info> {
    let f = parse(bytes)?;
    Ok(Info {
        created_at: f.created_at,
        records: f.records.len(),
    })
}

/// Writes the backup into an EMPTY store (no vault yet). The caller then
/// unlocks with the backup's passphrase; if that fails it should `destroy`.
pub fn restore<S: Store>(store: &S, bytes: &[u8]) -> Result<usize> {
    if store.header()?.is_some() {
        return Err(Error::AlreadyExists);
    }
    let f = parse(bytes)?;
    let mut sealed = Vec::with_capacity(f.records.len());
    for r in f.records {
        let iv = b64()
            .decode(&r.iv)
            .map_err(|_| Error::Corrupt(r.id.clone()))?;
        let ct = b64()
            .decode(&r.ct)
            .map_err(|_| Error::Corrupt(r.id.clone()))?;
        sealed.push((r.id, Sealed { v: r.v, iv, ct }));
    }
    for (id, s) in &sealed {
        store.put(id, s)?;
    }
    store.put_header(&f.vault)?;
    Ok(sealed.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::fast_kdf;
    use crate::event::{Event, Origin};
    use crate::kinds::{Payload, Reflection};
    use crate::store::MemStore;
    use crate::vault::{Vault, test_vault};
    use crate::{repo, settings};

    fn filled() -> (Vault<MemStore>, Event) {
        let v = test_vault();
        v.create("una frase lunga e mia").unwrap();
        let e = Event::new(
            Payload::Reflection(Reflection {
                text: "il profumo del mirto".into(),
            }),
            Origin::Spontaneous,
        );
        repo::save(&v, &e).unwrap();
        settings::save(&v, &settings::Settings::default()).unwrap();
        (v, e)
    }

    #[test]
    fn round_trip_needs_the_passphrase_and_keeps_everything() {
        let (v, e) = filled();
        v.lock(); // a backup can be made while locked
        let bytes = create(v.store()).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(!text.contains("mirto"), "the backup is encrypted");
        assert_eq!(info(&bytes).unwrap().records, 2);

        let fresh = Vault::with_kdf(MemStore::default(), fast_kdf());
        assert_eq!(restore(fresh.store(), &bytes).unwrap(), 2);
        assert!(matches!(
            fresh.unlock("sbagliata"),
            Err(Error::WrongPassphrase)
        ));
        fresh.unlock("una frase lunga e mia").unwrap();
        assert_eq!(repo::load(&fresh).unwrap(), vec![e]);
    }

    #[test]
    fn never_over_an_existing_vault_and_rejects_other_files() {
        let (v, _) = filled();
        let bytes = create(v.store()).unwrap();
        assert!(matches!(
            restore(v.store(), &bytes),
            Err(Error::AlreadyExists)
        ));
        let fresh = MemStore::default();
        assert!(restore(&fresh, b"{\"hello\":1}").is_err());
        assert!(restore(&fresh, b"not json").is_err());
        assert!(fresh.header().unwrap().is_none());
    }

    #[test]
    fn a_tampered_record_is_detected() {
        let (v, _) = filled();
        let mut f: serde_json::Value = serde_json::from_slice(&create(v.store()).unwrap()).unwrap();
        // Swap the ciphertexts of two records: the id-bound AAD must reject it.
        let recs = f["records"].as_array_mut().unwrap();
        let ct0 = recs[0]["ct"].clone();
        recs[0]["ct"] = recs[1]["ct"].clone();
        recs[1]["ct"] = ct0;
        let iv0 = recs[0]["iv"].clone();
        recs[0]["iv"] = recs[1]["iv"].clone();
        recs[1]["iv"] = iv0;
        let fresh = Vault::with_kdf(MemStore::default(), fast_kdf());
        restore(fresh.store(), &serde_json::to_vec(&f).unwrap()).unwrap();
        fresh.unlock("una frase lunga e mia").unwrap();
        assert!(repo::load(&fresh).is_err() || settings::load(&fresh).is_err());
    }
}
