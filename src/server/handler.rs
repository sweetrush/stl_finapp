use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use crate::error::{AppError, Result};
use crate::crypto::{KeyPair, decrypt_large};
use crate::auth::Whitelist;
use crate::protocol::{Handshake, Message, MessageType, MessageHeader, verify_checksum};
use crate::protocol::handshake::{send_message, receive_message, receive_raw_data};
use crate::cli::Output;
use std::fs;

/// Handle an incoming connection
pub async fn handle_connection(
    mut stream: TcpStream,
    whitelist: &Whitelist,
    keypair: &Arc<KeyPair>,
    messages_dir: &str,
) -> Result<()> {
    // Perform handshake
    match Handshake::server_side(&mut stream, whitelist, keypair).await {
        Ok(_) => {},
        Err(e) => {
            Output::auth_failed(&e.to_string());
            return Err(e);
        }
    };

    // Receive message header
    let header_msg = receive_message(&mut stream).await?;

    if !matches!(header_msg.msg_type, MessageType::MessageHeader) {
        return Err(AppError::Protocol("Expected MessageHeader".to_string()));
    }

    let header: MessageHeader = MessageHeader::from_bytes(&header_msg.payload)?;
    Output::info(&format!("Receiving file: {} ({} bytes)", header.filename, header.size));

    // Receive encrypted data length (8 bytes)
    let mut len_buf = [0u8; 8];
    stream.read_exact(&mut len_buf)
        .await
        .map_err(|e| AppError::Protocol(format!("Failed to read data length: {}", e)))?;
    let data_len = u64::from_be_bytes(len_buf) as usize;

    // Receive encrypted message data
    Output::receiving(data_len);
    let encrypted_data = receive_raw_data(&mut stream, data_len).await?;

    // Decrypt message
    Output::decrypting();
    let encrypted_msg = crate::crypto::EncryptedMessage::from_bytes(&encrypted_data)?;
    let decrypted_data = decrypt_large(&keypair.private_key, &encrypted_msg)?;

    // Verify checksum
    if !verify_checksum(&decrypted_data, &header.checksum)? {
        let err_msg = Message::new(MessageType::Error, b"Checksum verification failed".to_vec());
        send_message(&mut stream, &err_msg).await?;
        return Err(AppError::Protocol("Checksum verification failed".to_string()));
    }

    // Ensure messages directory exists
    fs::create_dir_all(messages_dir)
        .map_err(|e| AppError::Server(format!("Failed to create messages directory: {}", e)))?;

    // Save to file with timestamp
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_{}.ftt", header.filename, timestamp);
    let filepath = Path::new(messages_dir).join(&filename);

    fs::write(&filepath, &decrypted_data)
        .map_err(|e| AppError::Server(format!("Failed to save message: {}", e)))?;

    Output::file_saved(&filename);

    // Send acknowledgment
    let ack_payload = filename.as_bytes().to_vec();
    let ack_msg = Message::new(MessageType::Acknowledgment, ack_payload);
    send_message(&mut stream, &ack_msg).await?;

    Output::success("Message transfer complete");

    Ok(())
}
