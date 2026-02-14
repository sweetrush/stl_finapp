use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use crate::error::{AppError, Result};
use crate::crypto::KeyPair;
use crate::auth::Whitelist;
use crate::cli::Output;

/// TCP server for receiving messages
pub struct Server {
    port: u16,
    whitelist: Whitelist,
    keypair: Arc<KeyPair>,
    shutdown_tx: broadcast::Sender<()>,
    messages_dir: String,
}

impl Server {
    /// Create a new server instance
    pub fn new(port: u16, whitelist_path: &Path, keypair: KeyPair, messages_dir: &str) -> Result<Self> {
        let whitelist = Whitelist::load(whitelist_path)?;
        let (shutdown_tx, _) = broadcast::channel(1);

        Ok(Self {
            port,
            whitelist,
            keypair: Arc::new(keypair),
            shutdown_tx,
            messages_dir: messages_dir.to_string(),
        })
    }

    /// Start the server
    pub async fn start(&self) -> Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| AppError::Server(format!("Failed to bind to {}: {}", addr, e)))?;

        Output::listening("0.0.0.0", self.port);
        Output::server_started(self.port);

        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, peer_addr)) => {
                            Output::connection_from(&peer_addr.to_string());

                            let whitelist = self.whitelist.clone();
                            let keypair = Arc::clone(&self.keypair);
                            let messages_dir = self.messages_dir.clone();

                            tokio::spawn(async move {
                                if let Err(e) = super::handler::handle_connection(
                                    stream,
                                    &whitelist,
                                    &keypair,
                                    &messages_dir,
                                ).await {
                                    Output::error(&format!("Connection error: {}", e));
                                }
                            });
                        }
                        Err(e) => {
                            Output::error(&format!("Failed to accept connection: {}", e));
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    Output::info("Server shutting down...");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Get shutdown channel sender
    pub fn shutdown_channel(&self) -> broadcast::Sender<()> {
        self.shutdown_tx.clone()
    }

    /// Trigger shutdown
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}
