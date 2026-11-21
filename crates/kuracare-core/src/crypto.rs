//! Two-key scheme.
//!
//! - DEK (data-encryption key): 32 random bytes, AES-256-GCM, encrypts every record.
//! - KEK (key-encryption key): Argon2id(passphrase), wraps the DEK with AES-KW.
//!
//! Changing the passphrase only re-wraps the DEK. Keys live in `Zeroizing`
//! buffers and are wiped from memory when dropped (on lock).

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use aes_kw::KwAes256;
use argon2::{Algorithm, Argon2, Params, Version};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::error::{Error, Result};

pub const KEY_LEN: usize = 32;
pub const WRAPPED_LEN: usize = KEY_LEN + aes_kw::IV_LEN;
pub const NONCE_LEN: usize = 12;
/// Version of the sealed-record format (not of the payload schema).
pub const SEAL_VERSION: u32 = 1;

pub type Key = Zeroizing<[u8; KEY_LEN]>;

/// Argon2id parameters, stored next to the wrapped key so they can be
/// raised later without breaking existing vaults.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KdfParams {
    pub alg: String,
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
    /// 16 random bytes
    pub salt: Vec<u8>,
}

impl KdfParams {
    /// ~64 MiB, 3 passes: well above the OWASP minimum, about a second on a recent phone.
    pub fn strong() -> Self {
        Self::with_cost(64 * 1024, 3, 1)
    }

    pub fn with_cost(memory_kib: u32, iterations: u32, parallelism: u32) -> Self {
        Self {
            alg: "argon2id".into(),
            memory_kib,
            iterations,
            parallelism,
            salt: random_bytes::<16>().to_vec(),
        }
    }

    /// Same cost, fresh salt.
    pub fn renewed(&self) -> Self {
        Self::with_cost(self.memory_kib, self.iterations, self.parallelism)
    }
}

pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut b = [0u8; N];
    rand::fill(&mut b[..]);
    b
}

/// Normalised (NFC) so the same passphrase typed on different keyboards matches.
pub fn normalise_passphrase(passphrase: &str) -> Zeroizing<String> {
    use unicode_normalization::UnicodeNormalization;
    Zeroizing::new(passphrase.nfc().collect())
}

pub fn derive_kek(passphrase: &str, p: &KdfParams) -> Result<Key> {
    if p.alg != "argon2id" {
        return Err(Error::Unsupported(format!("KDF {}", p.alg)));
    }
    let params = Params::new(p.memory_kib, p.iterations, p.parallelism, Some(KEY_LEN))
        .map_err(|e| Error::Kdf(e.to_string()))?;
    let mut out = Zeroizing::new([0u8; KEY_LEN]);
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(
            normalise_passphrase(passphrase).as_bytes(),
            &p.salt,
            out.as_mut(),
        )
        .map_err(|e| Error::Kdf(e.to_string()))?;
    Ok(out)
}

pub fn new_dek() -> Key {
    Zeroizing::new(random_bytes::<KEY_LEN>())
}

pub fn wrap_dek(dek: &Key, kek: &Key) -> Result<Vec<u8>> {
    let kw = KwAes256::new_from_slice(kek.as_ref()).map_err(|e| Error::Kdf(e.to_string()))?;
    let mut buf = [0u8; WRAPPED_LEN];
    let out = kw
        .wrap_key(dek.as_ref(), &mut buf)
        .map_err(|e| Error::Kdf(e.to_string()))?;
    Ok(out.to_vec())
}

/// AES-KW has a built-in integrity check: a wrong KEK fails here.
pub fn unwrap_dek(wrapped: &[u8], kek: &Key) -> Result<Key> {
    let kw = KwAes256::new_from_slice(kek.as_ref()).map_err(|e| Error::Kdf(e.to_string()))?;
    let mut buf = Zeroizing::new([0u8; KEY_LEN]);
    let out = kw
        .unwrap_key(wrapped, buf.as_mut())
        .map_err(|_| Error::WrongPassphrase)?;
    if out.len() != KEY_LEN {
        return Err(Error::WrongPassphrase);
    }
    Ok(buf)
}

/// A sealed (encrypted) record as stored on disk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sealed {
    pub v: u32,
    pub iv: Vec<u8>,
    pub ct: Vec<u8>,
}

/// Associated data binds a ciphertext to its record id: records cannot be swapped.
fn aad(id: &str, v: u32) -> Vec<u8> {
    format!("kuracare:{v}:{id}").into_bytes()
}

pub fn seal(dek: &Key, id: &str, plain: &[u8]) -> Result<Sealed> {
    let cipher = Aes256Gcm::new_from_slice(dek.as_ref()).map_err(|e| Error::Kdf(e.to_string()))?;
    let iv = random_bytes::<NONCE_LEN>();
    let nonce = Nonce::try_from(&iv[..]).map_err(|_| Error::Corrupt(id.into()))?;
    let ct = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: plain,
                aad: &aad(id, SEAL_VERSION),
            },
        )
        .map_err(|_| Error::Corrupt(id.into()))?;
    Ok(Sealed {
        v: SEAL_VERSION,
        iv: iv.to_vec(),
        ct,
    })
}

pub fn unseal(dek: &Key, id: &str, s: &Sealed) -> Result<Zeroizing<Vec<u8>>> {
    if s.v != SEAL_VERSION {
        return Err(Error::Unsupported(format!("seal version {}", s.v)));
    }
    let cipher = Aes256Gcm::new_from_slice(dek.as_ref()).map_err(|e| Error::Kdf(e.to_string()))?;
    let nonce = Nonce::try_from(&s.iv[..]).map_err(|_| Error::Corrupt(id.into()))?;
    cipher
        .decrypt(
            &nonce,
            Payload {
                msg: &s.ct,
                aad: &aad(id, s.v),
            },
        )
        .map(Zeroizing::new)
        .map_err(|_| Error::Corrupt(id.into()))
}

#[cfg(test)]
pub(crate) fn fast_kdf() -> KdfParams {
    KdfParams::with_cost(256, 1, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_parameters_stay_strong() {
        let p = KdfParams::strong();
        assert_eq!(p.alg, "argon2id");
        assert!(p.memory_kib >= 19 * 1024);
        assert!(p.iterations >= 2);
    }

    #[test]
    fn nfc_and_nfd_forms_give_the_same_key() {
        let p = fast_kdf();
        let nfc = "caff\u{e8} lungo";
        let nfd = "caffe\u{300} lungo";
        assert_ne!(nfc, nfd);
        assert_eq!(*derive_kek(nfc, &p).unwrap(), *derive_kek(nfd, &p).unwrap());
    }

    #[test]
    fn fresh_salt_each_time() {
        assert_ne!(fast_kdf().salt, fast_kdf().salt);
    }

    #[test]
    fn wrong_passphrase_is_rejected() {
        let p = fast_kdf();
        let wrapped = wrap_dek(&new_dek(), &derive_kek("giusta", &p).unwrap()).unwrap();
        let err = unwrap_dek(&wrapped, &derive_kek("sbagliata", &p).unwrap()).unwrap_err();
        assert!(matches!(err, Error::WrongPassphrase));
    }

    #[test]
    fn rewrap_keeps_the_same_data_key() {
        let dek = new_dek();
        let (p1, p2) = (fast_kdf(), fast_kdf());
        let w1 = wrap_dek(&dek, &derive_kek("vecchia", &p1).unwrap()).unwrap();
        let back = unwrap_dek(&w1, &derive_kek("vecchia", &p1).unwrap()).unwrap();
        let w2 = wrap_dek(&back, &derive_kek("nuova", &p2).unwrap()).unwrap();
        let dek2 = unwrap_dek(&w2, &derive_kek("nuova", &p2).unwrap()).unwrap();
        assert_eq!(*dek, *dek2);
    }

    #[test]
    fn seal_round_trip_without_plaintext() {
        let dek = new_dek();
        let s = seal(&dek, "evt:1", b"una passeggiata al mare").unwrap();
        assert!(!String::from_utf8_lossy(&s.ct).contains("passeggiata"));
        assert_eq!(
            &*unseal(&dek, "evt:1", &s).unwrap(),
            b"una passeggiata al mare"
        );
    }

    #[test]
    fn different_iv_every_time() {
        let dek = new_dek();
        assert_ne!(
            seal(&dek, "a", b"1").unwrap().iv,
            seal(&dek, "a", b"1").unwrap().iv
        );
    }

    #[test]
    fn ciphertext_moved_to_another_id_is_refused() {
        let dek = new_dek();
        let s = seal(&dek, "evt:1", b"segreto").unwrap();
        assert!(matches!(unseal(&dek, "evt:2", &s), Err(Error::Corrupt(_))));
    }

    #[test]
    fn tampering_is_detected() {
        let dek = new_dek();
        let mut s = seal(&dek, "evt:1", b"segreto").unwrap();
        s.ct[0] ^= 1;
        assert!(unseal(&dek, "evt:1", &s).is_err());
    }
}
