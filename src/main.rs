use std::path::Path;
use clap::Parser;
use stl_finapp::cli::{Args, Commands, Output};
use stl_finapp::error::{AppError, Result};
use stl_finapp::crypto::KeyPair;
use stl_finapp::server::Server;
use stl_finapp::client::Client;
use stl_finapp::interactive::InteractiveSession;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Set default keys directory
    let keys_dir = "keys";
    let messages_dir = "messages";

    match args.command {
        Some(Commands::Listen { port, whitelist, keys_dir }) => {
            run_server(port, &whitelist, &keys_dir, messages_dir).await?;
        }
        Some(Commands::Send { ip, port, file, connect_key, save_as, keys_dir }) => {
            run_client(&ip, port, &file, &connect_key, save_as.as_deref(), &keys_dir).await?;
        }
        Some(Commands::Keygen { output }) => {
            generate_keys(&output)?;
        }
        Some(Commands::Whitelist { connect_key, file }) => {
            add_to_whitelist(&connect_key, &file)?;
        }
        None => {
            if args.interactive {
                let mut session = InteractiveSession::new(keys_dir);
                session.run().await?;
            } else if let (Some(ip), Some(file), Some(ck)) =
                (args.ip, args.file, args.connect_key) {
                run_client(&ip, args.port, &file, &ck, args.save_as.as_deref(), keys_dir).await?;
            } else {
                // Show help if no valid combination and not interactive
                use clap::CommandFactory;
                let mut cmd = Args::command();
                cmd.print_help().unwrap();
                println!();
            }
        }
    }

    Ok(())
}

async fn run_server(port: u16, whitelist_path: &str, keys_dir: &str, messages_dir: &str) -> Result<()> {
    let keypair = load_or_generate_keypair(keys_dir)?;
    let server = Server::new(port, Path::new(whitelist_path), keypair, messages_dir)?;

    // Handle Ctrl+C gracefully
    let shutdown_tx = server.shutdown_channel();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        let _ = shutdown_tx.send(());
    });

    server.start().await?;
    Ok(())
}

async fn run_client(
    ip: &str,
    port: u16,
    file: &str,
    connect_key: &str,
    save_as: Option<&str>,
    keys_dir: &str,
) -> Result<()> {
    let keypair = load_or_generate_keypair(keys_dir)?;
    let client = Client::new(ip, port, keypair);

    client.send_message(Path::new(file), connect_key, save_as).await?;
    Ok(())
}

fn generate_keys(output_dir: &str) -> Result<()> {
    std::fs::create_dir_all(output_dir)
        .map_err(|e| AppError::Io(e))?;

    let keypair = KeyPair::generate()?;
    let private_path = Path::new(output_dir).join("private_key.pem");
    let public_path = Path::new(output_dir).join("public_key.pem");

    keypair.save(&private_path, &public_path)?;

    Output::keys_generated(output_dir);
    Ok(())
}

fn add_to_whitelist(connect_key: &str, whitelist_path: &str) -> Result<()> {
    use stl_finapp::auth::Whitelist;
    let mut whitelist = Whitelist::load(Path::new(whitelist_path))?;
    whitelist.add(connect_key)?;
    Output::whitelist_updated(connect_key);
    Ok(())
}

fn load_or_generate_keypair(keys_dir: &str) -> Result<KeyPair> {
    let private_path = Path::new(keys_dir).join("private_key.pem");
    let public_path = Path::new(keys_dir).join("public_key.pem");

    if private_path.exists() && public_path.exists() {
        KeyPair::load(&private_path, &public_path)
    } else {
        Output::info("Keys not found, generating new key pair...");
        std::fs::create_dir_all(keys_dir)
            .map_err(|e| AppError::Io(e))?;
        let keypair = KeyPair::generate()?;
        keypair.save(&private_path, &public_path)?;
        // Reload to be sure
        KeyPair::load(&private_path, &public_path)
    }
}
