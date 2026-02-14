use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rsa::RsaPublicKey;
use rsa::pkcs8::DecodePublicKey;
use crate::error::{AppError, Result};
use crate::crypto::{decrypt, KeyPair};
use crate::auth::{Whitelist, hash_connect_key};
use crate::protocol::message::{Message, MessageType, AuthChallenge, AuthResponse};
use crate::cli::Output;

/// Handshake protocol handler
pub struct Handshake;

impl Handshake {
    /// Server-side handshake
    pub async fn server_side(
        stream: &mut TcpStream,
        whitelist: &Whitelist,
        keypair: &KeyPair,
    ) -> Result<RsaPublicKey> {
        // 1. Send challenge
        let challenge = AuthChallenge::new();
        let challenge_bytes = challenge.to_bytes()
            .map_err(|e| AppError::Protocol(format!("Failed to serialize challenge: {}", e)))?;

        let msg = Message::new(MessageType::AuthChallenge, challenge_bytes);
        send_message(stream, &msg).await?;

        Output::info("Challenge sent to client");

        // 2. Receive and verify response
        let response_msg = receive_message(stream).await?;

        if !matches!(response_msg.msg_type, MessageType::AuthResponse) {
            return Err(AppError::Auth("Expected AuthResponse".to_string()));
        }

        let response: AuthResponse = AuthResponse::from_bytes(&response_msg.payload)?;

        // Check if connect key is whitelisted
        let key_valid = whitelist.keys().iter().any(|k| {
            hash_connect_key(k) == response.connect_key_hash
        });

        if !key_valid {
            let fail_msg = Message::new(MessageType::AuthFailure, b"Invalid connect key".to_vec());
            send_message(stream, &fail_msg).await?;
            return Err(AppError::Auth("Invalid connect key".to_string()));
        }

        // 3. Send success
        let success_msg = Message::new(MessageType::AuthSuccess, vec![]);
        send_message(stream, &success_msg).await?;

        Output::authenticated();

        // 4. Exchange public keys
        let client_public_pem = receive_public_key(stream).await?;
        let client_public = RsaPublicKey::from_public_key_pem(&client_public_pem)
            .map_err(|e| AppError::Crypto(format!("Failed to parse client public key: {}", e)))?;

        send_public_key(stream, &keypair.public_key).await?;

        Output::info("Public keys exchanged");

        Ok(client_public)
    }

    /// Client-side handshake
    pub async fn client_side(
        stream: &mut TcpStream,
        connect_key: &str,
        keypair: &KeyPair,
    ) -> Result<RsaPublicKey> {
        // 1. Receive challenge
        let challenge_msg = receive_message(stream).await?;

        if !matches!(challenge_msg.msg_type, MessageType::AuthChallenge) {
            return Err(AppError::Protocol("Expected AuthChallenge".to_string()));
        }

        let challenge: AuthChallenge = AuthChallenge::from_bytes(&challenge_msg.payload)?;

        Output::info("Received challenge from server");

        // 2. Sign challenge and send response
        // In RSA, "signing" with PKCS1-v15 without a hash is technically decrypting the challenge
        let challenge_response = decrypt(&keypair.private_key, &challenge.challenge)
            .map_err(|e| AppError::Auth(format!("Failed to sign challenge: {}", e)))?;

        let connect_key_hash = hash_connect_key(connect_key);
        let response = AuthResponse::new(connect_key_hash, challenge_response);

        let response_bytes = response.to_bytes()
            .map_err(|e| AppError::Protocol(format!("Failed to serialize response: {}", e)))?;

        let msg = Message::new(MessageType::AuthResponse, response_bytes);
        send_message(stream, &msg).await?;

        Output::info("Sent authentication response");

        // 3. Receive success/failure
        let result_msg = receive_message(stream).await?;

        match result_msg.msg_type {
            MessageType::AuthSuccess => {
                Output::authenticated();
            }
            MessageType::AuthFailure => {
                let reason = String::from_utf8_lossy(&result_msg.payload);
                return Err(AppError::Auth(format!("Authentication failed: {}", reason)));
            }
            _ => {
                return Err(AppError::Protocol("Unexpected message type".to_string()));
            }
        }

        // 4. Exchange public keys
        send_public_key(stream, &keypair.public_key).await?;
        let server_public_pem = receive_public_key(stream).await?;
        let server_public = RsaPublicKey::from_public_key_pem(&server_public_pem)
            .map_err(|e| AppError::Crypto(format!("Failed to parse server public key: {}", e)))?;

        Output::info("Public keys exchanged");

        Ok(server_public)
    }
}

/// Send a message over the stream
pub async fn send_message(stream: &mut TcpStream, msg: &Message) -> Result<()> {
    let data = msg.to_bytes()?;
    let len = data.len() as u32;

    // Send length prefix
    stream.write_all(&len.to_be_bytes())
        .await
        .map_err(|e| AppError::Protocol(format!("Failed to send length: {}", e)))?;

    // Send data
    stream.write_all(&data)
        .await
        .map_err(|e| AppError::Protocol(format!("Failed to send message: {}", e)))?;

    Ok(())
}

/// Receive a message from the stream
pub async fn receive_message(stream: &mut TcpStream) -> Result<Message> {
    // Read length prefix
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)
        .await
        .map_err(|e| AppError::Protocol(format!("Failed to read length: {}", e)))?;

    let len = u32::from_be_bytes(len_buf) as usize;

    // Read data
    let mut data = vec![0u8; len];
    stream.read_exact(&mut data)
        .await
        .map_err(|e| AppError::Protocol(format!("Failed to read message: {}", e)))?;

    Message::from_bytes(&data)
}

/// Send public key
async fn send_public_key(stream: &mut TcpStream, public_key: &RsaPublicKey) -> Result<()> {
    use rsa::pkcs8::EncodePublicKey;
    use rsa::pkcs8::LineEnding;

    let pem = public_key.to_public_key_pem(LineEnding::LF)
        .map_err(|e| AppError::Crypto(format!("Failed to encode public key: {}", e)))?;

    let msg = Message::new(MessageType::PublicKeyExchange, pem.into_bytes());
    send_message(stream, &msg).await
}

/// Receive public key
async fn receive_public_key(stream: &mut TcpStream) -> Result<String> {
    let msg = receive_message(stream).await?;

    if !matches!(msg.msg_type, MessageType::PublicKeyExchange) {
        return Err(AppError::Protocol("Expected PublicKeyExchange".to_string()));
    }

    String::from_utf8(msg.payload)
        .map_err(|e| AppError::Protocol(format!("Invalid public key encoding: {}", e)))
}

/// Send raw data
pub async fn send_raw_data(stream: &mut TcpStream, data: &[u8]) -> Result<()> {
    let len = data.len() as u64;

    // Send length prefix (8 bytes for large data)
    stream.write_all(&len.to_be_bytes())
        .await
        .map_err(|e| AppError::Protocol(format!("Failed to send data length: {}", e)))?;

    // Send data
    stream.write_all(data)
        .await
        .map_err(|e| AppError::Protocol(format!("Failed to send data: {}", e)))?;

    Ok(())
}

/// Receive raw data
pub async fn receive_raw_data(stream: &mut TcpStream, size: usize) -> Result<Vec<u8>> {
    let mut data = vec![0u8; size];
    stream.read_exact(&mut data)
        .await
        .map_err(|e| AppError::Protocol(format!("Failed to read data: {}", e)))?;
    Ok(data)
}
