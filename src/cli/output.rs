use colored::Colorize;

/// Colored CLI output utilities
pub struct Output;

impl Output {
    /// Print an info message in cyan
    pub fn info(msg: &str) {
        println!("{} {}", "[INFO]".cyan().bold(), msg);
    }

    /// Print a success message in green
    pub fn success(msg: &str) {
        println!("{} {}", "[SUCCESS]".green().bold(), msg);
    }

    /// Print a warning message in yellow
    pub fn warning(msg: &str) {
        println!("{} {}", "[WARNING]".yellow().bold(), msg);
    }

    /// Print an error message in red
    pub fn error(msg: &str) {
        eprintln!("{} {}", "[ERROR]".red().bold(), msg);
    }

    /// Print listening status
    pub fn listening(ip: &str, port: u16) {
        println!(
            "{} Listening on {}:{}",
            "[-]".blue().bold(),
            ip.green(),
            port.to_string().green()
        );
    }

    /// Print connecting status
    pub fn connecting(addr: &str) {
        println!("{} Connecting to {}...", "[*]".yellow().bold(), addr.cyan());
    }

    /// Print connected status
    pub fn connected(ip: &str) {
        println!("{} Connected to {}", "[+]".green().bold(), ip.cyan());
    }

    /// Print authenticating status
    pub fn authenticating() {
        println!("{} Authenticating...", "[*]".yellow().bold());
    }

    /// Print authenticated status
    pub fn authenticated() {
        println!("{} Authentication successful", "[+]".green().bold());
    }

    /// Print authentication failed
    pub fn auth_failed(reason: &str) {
        println!("{} Authentication failed: {}", "[!]".red().bold(), reason);
    }

    /// Print encrypting status
    pub fn encrypting() {
        println!("{} Encrypting message...", "[*]".yellow().bold());
    }

    /// Print decrypting status
    pub fn decrypting() {
        println!("{} Decrypting message...", "[*]".yellow().bold());
    }

    /// Print sending status
    pub fn sending(size: usize) {
        println!("{} Sending {} bytes...", "[*]".yellow().bold(), size);
    }

    /// Print receiving status
    pub fn receiving(size: usize) {
        println!("{} Receiving {} bytes...", "[*]".yellow().bold(), size);
    }

    /// Print message received
    pub fn message_received(from: &str, filename: &str) {
        println!(
            "{} Message received from {} - saved as {}",
            "[+]".green().bold(),
            from.cyan(),
            filename.magenta()
        );
    }

    /// Print helper/tip message
    pub fn helper(msg: &str) {
        println!("{} {}", "[?]".magenta().bold(), msg.white());
    }

    /// Print a section header
    pub fn header(msg: &str) {
        println!("\n{}", msg.cyan().bold().underline());
        println!("{}", "─".repeat(50).dimmed());
    }

    /// Print key generation success
    pub fn keys_generated(dir: &str) {
        Self::success(&format!("Keys generated in {}", dir));
        Self::helper("Share your public_key.pem with other servers");
        Self::helper("Keep private_key.pem secure and never share it!");
    }

    /// Print whitelist updated
    pub fn whitelist_updated(key: &str) {
        Self::success(&format!("Connect key added to whitelist: {}", key));
    }

    /// Print server started
    pub fn server_started(port: u16) {
        Self::success(&format!("Server started on port {}", port));
        Self::helper("Press Ctrl+C to stop the server");
    }

    /// Print connection from
    pub fn connection_from(addr: &str) {
        Self::info(&format!("Connection from {}", addr));
    }

    /// Print file saved
    pub fn file_saved(filename: &str) {
        Self::success(&format!("File saved: {}", filename));
    }
}
