use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{KonachaError, Result};

pub const SALT_LEN:  usize = 16;
pub const KEY_LEN:   usize = 32;
pub const NONCE_LEN: usize = 12;
const MAGIC: &[u8; 4] = b"KNCH";

/// 32-byte master key. Wiped from memory on drop.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct MasterKey([u8; KEY_LEN]);

impl MasterKey {
    /// Argon2id — 64 MiB, t=3, p=4. ~180 ms M1 / ~340 ms i5.
    pub fn derive(passphrase: &str, salt: &[u8]) -> Result<Self> {
        let params = Params::new(65_536, 3, 4, Some(KEY_LEN))
        .map_err(|e| KonachaError::Kdf(e.to_string()))?;
        let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let mut out = [0u8; KEY_LEN];
        argon
        .hash_password_into(passphrase.as_bytes(), salt, &mut out)
        .map_err(|e| KonachaError::Kdf(e.to_string()))?;
        Ok(Self(out))
    }

    /// Envelope: `"KNCH" || nonce[12] || ciphertext || tag[16]`
    pub fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.0));
        let mut nonce = [0u8; NONCE_LEN];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        let ct = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext)
        .map_err(|_| KonachaError::Tampered)?;

        let mut out = Vec::with_capacity(4 + NONCE_LEN + ct.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ct);
        Ok(out)
    }

    pub fn open(&self, blob: &[u8]) -> Result<Vec<u8>> {
        if blob.len() < 4 + NONCE_LEN + 16 || &blob[..4] != MAGIC {
            return Err(KonachaError::Malformed);
        }
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&self.0));
        cipher
        .decrypt(
            Nonce::from_slice(&blob[4..4 + NONCE_LEN]),
                 &blob[4 + NONCE_LEN..],
        )
        .map_err(|_| KonachaError::Tampered)
    }
}

pub fn new_salt() -> [u8; SALT_LEN] {
    let mut s = [0u8; SALT_LEN];
    rand::rngs::OsRng.fill_bytes(&mut s);
    s
}
