use clap::{Parser, Subcommand};

/// Secure Finance Messaging Block Application
#[derive(Parser, Debug)]
#[command(name = "stl_finapp")]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Server IP to connect and send messages to
    #[arg(short = 'i', long = "ip", value_name = "IP_ADDRESS")]
    pub ip: Option<String>,

    /// Text file with the message block
    #[arg(short = 'f', long = "file", value_name = "FILE_PATH")]
    pub file: Option<String>,

    /// Key file for encryption
    #[arg(short = 'k', long = "key", value_name = "KEY_FILE")]
    pub key: Option<String>,

    /// Filename to be stored on other server (with date/time appended, .ftt extension)
    #[arg(short = 's', long = "save-as", value_name = "FILENAME")]
    pub save_as: Option<String>,

    /// Interactive mode
    #[arg(short = 'm', long = "interactive")]
    pub interactive: bool,

    /// Connect key for authentication
    #[arg(long = "ck", value_name = "KEY")]
    pub connect_key: Option<String>,

    /// Listening port number
    #[arg(long = "lp", value_name = "PORT", default_value = "8080")]
    pub port: u16,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the server in listening mode
    Listen {
        /// Port to listen on
        #[arg(short = 'p', long = "port", default_value = "8080")]
        port: u16,

        /// Path to whitelist file
        #[arg(short = 'w', long = "whitelist", default_value = "keys/whitelist.txt")]
        whitelist: String,

        /// Path to keys directory
        #[arg(short = 'k', long = "keys", default_value = "keys")]
        keys_dir: String,
    },

    /// Send a message to a server
    Send {
        /// Server IP address
        #[arg(short = 'i', long = "ip")]
        ip: String,

        /// Server port
        #[arg(short = 'p', long = "port", default_value = "8080")]
        port: u16,

        /// Message file path
        #[arg(short = 'f', long = "file")]
        file: String,

        /// Connect key for authentication
        #[arg(long = "ck")]
        connect_key: String,

        /// Remote filename (without extension)
        #[arg(short = 's', long = "save-as")]
        save_as: Option<String>,

        /// Path to keys directory
        #[arg(short = 'k', long = "keys", default_value = "keys")]
        keys_dir: String,
    },

    /// Generate new key pair
    Keygen {
        /// Output directory for keys
        #[arg(short = 'o', long = "output", default_value = "keys")]
        output: String,
    },

    /// Add a connect key to whitelist
    Whitelist {
        /// Connect key to add
        #[arg(long = "ck")]
        connect_key: String,

        /// Whitelist file path
        #[arg(short = 'f', long = "file", default_value = "keys/whitelist.txt")]
        file: String,
    },
}
