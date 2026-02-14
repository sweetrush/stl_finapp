use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use rand::Rng;
use crate::error::{AppError, Result};

/// Authentication token for secure communication
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthToken {
    /// The connect key (hashed)
    pub connect_key_hash: String,
    /// Timestamp when token was created
    pub timestamp: DateTime<Utc>,
    /// Random nonce for uniqueness
    pub nonce: String,
}

impl AuthToken {
    /// Create a new auth token
    pub fn new(connect_key: &str) -> Self {
        Self {
            connect_key_hash: hash_connect_key(connect_key),
            timestamp: Utc::now(),
            nonce: generate_nonce(),
        }
    }

    /// Check if token is expired (5 minute window)
    pub fn is_valid_time(&self) -> bool {
        let now = Utc::now();
        let diff = now.signed_duration_since(self.timestamp);
        diff.num_minutes() < 5
    }

    /// Verify the connect key matches
    pub fn verify_key(&self, connect_key: &str) -> bool {
        self.connect_key_hash == hash_connect_key(connect_key)
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| AppError::Serialization(format!("Failed to serialize token: {}", e)))
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data)
            .map_err(|e| AppError::Serialization(format!("Failed to deserialize token: {}", e)))
    }
}

/// Hash a connect key using SHA-256
pub fn hash_connect_key(connect_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(connect_key.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate a random nonce
fn generate_nonce() -> String {
    let mut rng = rand::thread_rng();
    format!("{:016x}", rng.gen::<u64>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = AuthToken::new("secret123");
        assert!(!token.connect_key_hash.is_empty());
        assert!(token.is_valid_time());
    }

    #[test]
    fn test_key_verification() {
        let token = AuthToken::new("secret123");
        assert!(token.verify_key("secret123"));
        assert!(!token.verify_key("wrong_key"));
    }

    #[test]
    fn test_hash_consistency() {
        let hash1 = hash_connect_key("test");
        let hash2 = hash_connect_key("test");
        assert_eq!(hash1, hash2);

        let hash3 = hash_connect_key("different");
        assert_ne!(hash1, hash3);
    }
}
