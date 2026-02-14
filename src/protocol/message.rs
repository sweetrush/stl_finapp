use serde::{Serialize, Deserialize};
use crate::error::{AppError, Result};

/// Message types for protocol communication
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    /// Authentication challenge from server
    AuthChallenge,
    /// Authentication response from client
    AuthResponse,
    /// Authentication success
    AuthSuccess,
    /// Authentication failure
    AuthFailure,
    /// Public key exchange
    PublicKeyExchange,
    /// Message header with metadata
    MessageHeader,
    /// Message data
    MessageData,
    /// Acknowledgment of receipt
    Acknowledgment,
    /// Error message
    Error,
}

/// Main message structure
#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    /// Type of message
    pub msg_type: MessageType,
    /// Message payload
    pub payload: Vec<u8>,
}

impl Message {
    /// Create a new message
    pub fn new(msg_type: MessageType, payload: Vec<u8>) -> Self {
        Self { msg_type, payload }
    }

    /// Serialize message to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| AppError::Protocol(format!("Failed to serialize message: {}", e)))
    }

    /// Deserialize message from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data)
            .map_err(|e| AppError::Protocol(format!("Failed to deserialize message: {}", e)))
    }
}

/// Message header with metadata
#[derive(Serialize, Deserialize, Debug)]
pub struct MessageHeader {
    /// Original filename
    pub filename: String,
    /// Size of the encrypted data
    pub size: u64,
    /// Timestamp when message was sent
    pub timestamp: String,
    /// SHA-256 checksum of original data
    pub checksum: String,
}

impl MessageHeader {
    /// Create a new message header
    pub fn new(filename: &str, size: u64, checksum: &str) -> Self {
        Self {
            filename: filename.to_string(),
            size,
            timestamp: chrono::Utc::now().to_rfc3339(),
            checksum: checksum.to_string(),
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| AppError::Protocol(format!("Failed to serialize header: {}", e)))
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data)
            .map_err(|e| AppError::Protocol(format!("Failed to deserialize header: {}", e)))
    }
}

/// Authentication challenge
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthChallenge {
    /// Random challenge bytes
    pub challenge: Vec<u8>,
    /// Timestamp
    pub timestamp: String,
}

impl AuthChallenge {
    /// Create a new challenge
    pub fn new() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let challenge: Vec<u8> = (0..32).map(|_| rng.gen::<u8>()).collect();

        Self {
            challenge,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| AppError::Protocol(format!("Failed to serialize challenge: {}", e)))
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data)
            .map_err(|e| AppError::Protocol(format!("Failed to deserialize challenge: {}", e)))
    }
}

/// Authentication response
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthResponse {
    /// Hashed connect key
    pub connect_key_hash: String,
    /// Signed challenge
    pub challenge_response: Vec<u8>,
    /// Timestamp
    pub timestamp: String,
}

impl AuthResponse {
    /// Create a new auth response
    pub fn new(connect_key_hash: String, challenge_response: Vec<u8>) -> Self {
        Self {
            connect_key_hash,
            challenge_response,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| AppError::Protocol(format!("Failed to serialize response: {}", e)))
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data)
            .map_err(|e| AppError::Protocol(format!("Failed to deserialize response: {}", e)))
    }
}

/// Calculate SHA-256 checksum
pub fn calculate_checksum(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Verify checksum
pub fn verify_checksum(data: &[u8], expected: &str) -> Result<bool> {
    let actual = calculate_checksum(data);
    Ok(actual == expected)
}
