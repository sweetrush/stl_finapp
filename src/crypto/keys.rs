use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs8::{EncodePublicKey, DecodePublicKey, EncodePrivateKey, DecodePrivateKey, LineEnding};
use std::path::Path;
use std::fs;
use crate::error::{AppError, Result};

/// RSA key size in bits
pub const KEY_SIZE: usize = 2048;

/// RSA key pair for encryption/decryption
pub struct KeyPair {
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
}

impl KeyPair {
    /// Generate a new RSA key pair
    pub fn generate() -> Result<Self> {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, KEY_SIZE)
            .map_err(|e| AppError::Crypto(format!("Failed to generate key: {}", e)))?;
        let public_key = RsaPublicKey::from(&private_key);
        Ok(Self { private_key, public_key })
    }

    /// Load key pair from PEM files
    pub fn load(private_path: &Path, public_path: &Path) -> Result<Self> {
        let private_pem = fs::read_to_string(private_path)
            .map_err(|e| AppError::Crypto(format!("Failed to read private key: {}", e)))?;
        let public_pem = fs::read_to_string(public_path)
            .map_err(|e| AppError::Crypto(format!("Failed to read public key: {}", e)))?;

        let private_key = RsaPrivateKey::from_pkcs8_pem(&private_pem)
            .map_err(|e| AppError::Crypto(format!("Failed to parse private key: {}", e)))?;
        let public_key = RsaPublicKey::from_public_key_pem(&public_pem)
            .map_err(|e| AppError::Crypto(format!("Failed to parse public key: {}", e)))?;

        Ok(Self { private_key, public_key })
    }

    /// Save key pair to PEM files
    pub fn save(&self, private_path: &Path, public_path: &Path) -> Result<()> {
        // Ensure parent directories exist
        if let Some(parent) = private_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Crypto(format!("Failed to create directory: {}", e)))?;
        }

        let private_pem = self.private_key.to_pkcs8_pem(LineEnding::LF)
            .map_err(|e| AppError::Crypto(format!("Failed to encode private key: {}", e)))?;
        let public_pem = self.public_key.to_public_key_pem(LineEnding::LF)
            .map_err(|e| AppError::Crypto(format!("Failed to encode public key: {}", e)))?;

        // Set restrictive permissions on private key (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            use std::io::Write;
            fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(private_path)
                .and_then(|mut f| f.write_all(private_pem.as_bytes()))
                .map_err(|e| AppError::Crypto(format!("Failed to write private key: {}", e)))?;
        }

        #[cfg(not(unix))]
        {
            fs::write(private_path, private_pem.as_bytes())
                .map_err(|e| AppError::Crypto(format!("Failed to write private key: {}", e)))?;
        }

        fs::write(public_path, public_pem.as_bytes())
            .map_err(|e| AppError::Crypto(format!("Failed to write public key: {}", e)))?;

        Ok(())
    }

    /// Load only public key from PEM file
    pub fn load_public(path: &Path) -> Result<RsaPublicKey> {
        let pem = fs::read_to_string(path)
            .map_err(|e| AppError::Crypto(format!("Failed to read public key file: {}", e)))?;
        let public_key = RsaPublicKey::from_public_key_pem(&pem)
            .map_err(|e| AppError::Crypto(format!("Failed to parse public key: {}", e)))?;
        Ok(public_key)
    }

    /// Get public key as PEM string
    pub fn public_key_pem(&self) -> Result<String> {
        self.public_key.to_public_key_pem(LineEnding::LF)
            .map_err(|e| AppError::Crypto(format!("Failed to encode public key: {}", e)))
    }
}
