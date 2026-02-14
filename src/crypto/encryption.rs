use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce, AeadCore,
};
use rsa::{RsaPublicKey, RsaPrivateKey, Pkcs1v15Encrypt};
use serde::{Serialize, Deserialize};
use crate::error::{AppError, Result};

/// Maximum data size that can be encrypted directly with RSA 2048 (PKCS1v15 padding)
pub const RSA_MAX_ENCRYPT_SIZE: usize = 190;

/// Encrypt data using RSA public key (for small data only)
pub fn encrypt(public_key: &RsaPublicKey, data: &[u8]) -> Result<Vec<u8>> {
    let mut rng = OsRng;
    public_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, data)
        .map_err(|e| AppError::Crypto(format!("RSA encryption failed: {}", e)))
}

/// Decrypt data using RSA private key
pub fn decrypt(private_key: &RsaPrivateKey, data: &[u8]) -> Result<Vec<u8>> {
    private_key
        .decrypt(Pkcs1v15Encrypt, data)
        .map_err(|e| AppError::Crypto(format!("RSA decryption failed: {}", e)))
}

/// Hybrid encrypted message (RSA + AES)
#[derive(Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// AES key encrypted with RSA
    pub encrypted_key: Vec<u8>,
    /// Nonce for AES-GCM
    pub nonce: Vec<u8>,
    /// Data encrypted with AES-GCM
    pub encrypted_data: Vec<u8>,
}

impl EncryptedMessage {
    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| AppError::Serialization(format!("Failed to serialize message: {}", e)))
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data)
            .map_err(|e| AppError::Serialization(format!("Failed to deserialize message: {}", e)))
    }
}

/// Encrypt large data using hybrid encryption (RSA + AES-256-GCM)
pub fn encrypt_large(public_key: &RsaPublicKey, data: &[u8]) -> Result<EncryptedMessage> {
    // Generate random AES-256 key
    let aes_key = Aes256Gcm::generate_key(&mut OsRng);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // Encrypt data with AES-GCM
    let cipher = Aes256Gcm::new(&aes_key);
    let encrypted_data = cipher
        .encrypt(&nonce, data)
        .map_err(|e| AppError::Crypto(format!("AES encryption failed: {}", e)))?;

    // Encrypt AES key with RSA
    let encrypted_key = encrypt(public_key, &aes_key)?;

    Ok(EncryptedMessage {
        encrypted_key,
        nonce: nonce.to_vec(),
        encrypted_data,
    })
}

/// Decrypt hybrid encrypted message
pub fn decrypt_large(private_key: &RsaPrivateKey, message: &EncryptedMessage) -> Result<Vec<u8>> {
    // Decrypt AES key with RSA
    let aes_key = decrypt(private_key, &message.encrypted_key)?;

    // Create AES cipher
    let aes_key = aes_gcm::Key::<Aes256Gcm>::from_slice(&aes_key);
    let cipher = Aes256Gcm::new(aes_key);
    let nonce = Nonce::from_slice(&message.nonce);

    // Decrypt data
    cipher
        .decrypt(nonce, message.encrypted_data.as_slice())
        .map_err(|e| AppError::Crypto(format!("AES decryption failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::keys::KeyPair;

    #[test]
    fn test_encrypt_decrypt_small() {
        let keypair = KeyPair::generate().unwrap();
        let data = b"Hello, World!";

        let encrypted = encrypt(&keypair.public_key, data).unwrap();
        let decrypted = decrypt(&keypair.private_key, &encrypted).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_large() {
        let keypair = crate::crypto::keys::KeyPair::generate().unwrap();
        let data = vec![0u8; 10000]; // 10KB of data

        let encrypted = encrypt_large(&keypair.public_key, &data).unwrap();
        let decrypted = decrypt_large(&keypair.private_key, &encrypted).unwrap();

        assert_eq!(data, decrypted);
    }
}
