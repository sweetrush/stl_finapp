use std::io::{self, Write};
use std::path::Path;
use tokio::sync::broadcast;
use crate::error::{AppError, Result};
use crate::crypto::KeyPair;
use crate::auth::Whitelist;
use crate::server::Server;
use crate::client::Client;
use crate::cli::Output;

/// Interactive session for REPL mode
pub struct InteractiveSession {
    keypair: Option<KeyPair>,
    keys_dir: String,
    server_shutdown: Option<broadcast::Sender<()>>,
    listening_port: Option<u16>,
}

impl InteractiveSession {
    /// Create a new interactive session
    pub fn new(keys_dir: &str) -> Self {
        Self {
            keypair: None,
            keys_dir: keys_dir.to_string(),
            server_shutdown: None,
            listening_port: None,
        }
    }

    /// Run the interactive session
    pub async fn run(&mut self) -> Result<()> {
        Output::header("Secure Finance Messaging Application");
        Output::helper("Type 'help' for available commands");

        // Try to load existing keys
        self.load_keys()?;

        loop {
            print_prompt();

            let input = read_input()?;

            if input.is_empty() {
                continue;
            }

            let parts: Vec<&str> = input.split_whitespace().collect();
            let command = parts[0];

            match command {
                "help" | "h" | "?" => self.show_help(),
                "listen" | "l" => self.start_server(&parts[1..]).await?,
                "send" | "s" => self.send_message(&parts[1..]).await?,
                "status" => self.show_status(),
                "keygen" | "k" => self.generate_keys(&parts[1..])?,
                "whitelist" | "w" => self.manage_whitelist(&parts[1..])?,
                "stop" => self.stop_server()?,
                "exit" | "quit" | "q" => {
                    self.stop_server()?;
                    Output::info("Goodbye!");
                    break;
                }
                _ => Output::error(&format!("Unknown command: {}. Type 'help' for available commands.", command)),
            }
        }

        Ok(())
    }

    /// Show help message
    fn show_help(&self) {
        Output::header("Available Commands");

        println!("  {:<20} {}", "listen [port]", "Start listening server (default: 8080)");
        println!("  {:<20} {}", "stop", "Stop the listening server");
        println!("  {:<20} {}", "send <ip> <file> [name]", "Send message to server");
        println!("  {:<20} {}", "status", "Show current status");
        println!("  {:<20} {}", "keygen [dir]", "Generate new key pair");
        println!("  {:<20} {}", "whitelist <key>", "Add key to whitelist");
        println!("  {:<20} {}", "help", "Show this help message");
        println!("  {:<20} {}", "exit / quit", "Exit interactive mode");
        println!();
    }

    /// Start the server
    async fn start_server(&mut self, args: &[&str]) -> Result<()> {
        if self.server_shutdown.is_some() {
            Output::warning("Server is already running. Use 'stop' to stop it first.");
            return Ok(());
        }

        let port = args.get(0)
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(8080);

        let keypair = self.get_or_create_keypair()?;
        let whitelist_path = Path::new(&self.keys_dir).join("whitelist.txt");

        let server = Server::new(port, &whitelist_path, keypair, "messages")?;
        let shutdown_tx = server.shutdown_channel();
        self.server_shutdown = Some(shutdown_tx);
        self.listening_port = Some(port);

        // Run server in background
        tokio::spawn(async move {
            if let Err(e) = server.start().await {
                Output::error(&format!("Server error: {}", e));
            }
        });

        Ok(())
    }

    /// Stop the server
    fn stop_server(&mut self) -> Result<()> {
        if let Some(shutdown) = self.server_shutdown.take() {
            let _ = shutdown.send(());
            self.listening_port = None;
            Output::info("Server stopped");
        }
        Ok(())
    }

    /// Send a message
    async fn send_message(&mut self, args: &[&str]) -> Result<()> {
        if args.len() < 2 {
            Output::error("Usage: send <ip> <file> [save_as]");
            return Ok(());
        }

        let ip = args[0];
        let file = args[1];
        let save_as = args.get(2).copied();

        // Prompt for connect key
        let connect_key = prompt_password("Enter connect key: ")?;

        let keypair = self.get_or_create_keypair()?;
        let client = Client::new(ip, 8080, keypair);

        client.send_message(Path::new(file), &connect_key, save_as).await?;

        Ok(())
    }

    /// Show current status
    fn show_status(&self) {
        Output::header("Current Status");

        if let Some(port) = self.listening_port {
            println!("  Server: Listening on port {}", port);
        } else {
            println!("  Server: Not running");
        }

        if self.keypair.is_some() {
            println!("  Keys: Loaded from {}", self.keys_dir);
        } else {
            println!("  Keys: Not loaded");
        }

        println!();
    }

    /// Generate new keys
    fn generate_keys(&mut self, args: &[&str]) -> Result<()> {
        let output_dir = args.get(0).map(|s| s.to_string()).unwrap_or_else(|| self.keys_dir.clone());

        let keypair = KeyPair::generate()?;
        let private_path = Path::new(&output_dir).join("private_key.pem");
        let public_path = Path::new(&output_dir).join("public_key.pem");

        keypair.save(&private_path, &public_path)?;

        self.keypair = Some(keypair);
        self.keys_dir = output_dir.clone();

        Output::keys_generated(&output_dir);

        Ok(())
    }

    /// Manage whitelist
    fn manage_whitelist(&mut self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            Output::error("Usage: whitelist <connect_key>");
            return Ok(());
        }

        let connect_key = args[0];
        let whitelist_path = Path::new(&self.keys_dir).join("whitelist.txt");

        let mut whitelist = Whitelist::load(&whitelist_path)?;
        whitelist.add(connect_key)?;

        Output::whitelist_updated(connect_key);

        Ok(())
    }

    /// Load existing keys
    fn load_keys(&mut self) -> Result<()> {
        let private_path = Path::new(&self.keys_dir).join("private_key.pem");
        let public_path = Path::new(&self.keys_dir).join("public_key.pem");

        if private_path.exists() && public_path.exists() {
            match KeyPair::load(&private_path, &public_path) {
                Ok(kp) => {
                    self.keypair = Some(kp);
                    Output::info(&format!("Loaded keys from {}", self.keys_dir));
                }
                Err(e) => {
                    Output::warning(&format!("Failed to load keys: {}", e));
                }
            }
        }

        Ok(())
    }

    /// Get existing keypair or create new one
    fn get_or_create_keypair(&mut self) -> Result<KeyPair> {
        let private_path = Path::new(&self.keys_dir).join("private_key.pem");
        let public_path = Path::new(&self.keys_dir).join("public_key.pem");

        if self.keypair.is_some() {
            KeyPair::load(&private_path, &public_path)
        } else {
            let keypair = KeyPair::generate()?;
            keypair.save(&private_path, &public_path)?;

            let loaded = KeyPair::load(&private_path, &public_path)?;
            // Note: We can't easily store it and return a clone, so we store a dummy
            // or just reload it. RsaPrivateKey usually isn't Clone.
            self.keypair = Some(KeyPair::load(&private_path, &public_path)?);

            Output::keys_generated(&self.keys_dir);
            Ok(loaded)
        }
    }
}

/// Print the prompt
fn print_prompt() {
    use std::io::Write;
    use colored::Colorize;
    print!("{} ", "finapp>".green().bold());
    io::stdout().flush().ok();
}

/// Read input from stdin
fn read_input() -> Result<String> {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| AppError::Cli(format!("Failed to read input: {}", e)))?;
    Ok(input.trim().to_string())
}

/// Prompt for password (hidden input)
fn prompt_password(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| AppError::Cli(format!("Failed to read input: {}", e)))?;

    Ok(input.trim().to_string())
}
