use std::path::Path;
use std::fs;
use tokio::net::TcpStream;
use crate::error::{AppError, Result};
use crate::crypto::{KeyPair, encrypt_large};
use crate::protocol::{Handshake, Message, MessageType, MessageHeader, calculate_checksum};
use crate::protocol::handshake::{send_message, receive_message, send_raw_data};
use crate::cli::Output;

/// Client for sending messages to a server
pub struct Client {
    server_addr: String,
    keypair: KeyPair,
}

impl Client {
    /// Create a new client instance
    pub fn new(server_ip: &str, port: u16, keypair: KeyPair) -> Self {
        Self {
            server_addr: format!("{}:{}", server_ip, port),
            keypair,
        }
    }

    /// Send a message to the server
    pub async fn send_message(
        &self,
        message_file: &Path,
        connect_key: &str,
        save_as: Option<&str>,
    ) -> Result<String> {
        Output::connecting(&self.server_addr);

        // Connect to server
        let mut stream = TcpStream::connect(&self.server_addr)
            .await
            .map_err(|e| AppError::Client(format!("Failed to connect to {}: {}", self.server_addr, e)))?;

        // Perform handshake
        Output::authenticating();
        let _server_public = Handshake::client_side(&mut stream, connect_key, &self.keypair).await?;

        // Read message file
        let message_data = fs::read(message_file)
            .map_err(|e| AppError::Client(format!("Failed to read message file: {}", e)))?;

        let filename = save_as.unwrap_or_else(|| {
            message_file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("message")
        });

        Output::info(&format!("Sending file: {} ({} bytes)", filename, message_data.len()));

        // Calculate checksum
        let checksum = calculate_checksum(&message_data);

        // Encrypt message
        Output::encrypting();
        let encrypted = encrypt_large(&self.keypair.public_key, &message_data)?;
        let encrypted_bytes = encrypted.to_bytes()?;

        // Create header
        let header = MessageHeader::new(filename, encrypted_bytes.len() as u64, &checksum);

        // Send header
        let header_bytes = header.to_bytes()?;
        let header_msg = Message::new(MessageType::MessageHeader, header_bytes);
        send_message(&mut stream, &header_msg).await?;

        // Send encrypted data
        Output::sending(encrypted_bytes.len());
        send_raw_data(&mut stream, &encrypted_bytes).await?;

        // Wait for acknowledgment
        let ack_msg = receive_message(&mut stream).await?;

        match ack_msg.msg_type {
            MessageType::Acknowledgment => {
                let saved_filename = String::from_utf8(ack_msg.payload)
                    .unwrap_or_else(|_| filename.to_string());
                Output::success(&format!("Message delivered, saved as: {}", saved_filename));
                Ok(saved_filename)
            }
            MessageType::Error => {
                let error_msg = String::from_utf8_lossy(&ack_msg.payload);
                Err(AppError::Client(format!("Server error: {}", error_msg)))
            }
            _ => Err(AppError::Protocol("Unexpected response from server".to_string())),
        }
    }
}
