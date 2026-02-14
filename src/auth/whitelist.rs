use std::path::Path;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use crate::error::{AppError, Result};

/// Whitelist manager for connect keys
#[derive(Clone)]
pub struct Whitelist {
    keys: Vec<String>,
    path: String,
}

impl Whitelist {
    /// Load whitelist from file
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Self::create(path);
        }

        let file = fs::File::open(path)
            .map_err(|e| AppError::Auth(format!("Failed to open whitelist: {}", e)))?;
        let reader = BufReader::new(file);

        let keys: Vec<String> = reader
            .lines()
            .filter_map(|line| line.ok())
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();

        Ok(Self {
            keys,
            path: path.to_string_lossy().to_string(),
        })
    }

    /// Check if a connect key is whitelisted
    pub fn contains(&self, connect_key: &str) -> bool {
        self.keys.iter().any(|k| k == connect_key)
    }

    /// Add a new connect key to the whitelist
    pub fn add(&mut self, connect_key: &str) -> Result<()> {
        if self.contains(connect_key) {
            return Ok(());
        }

        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.path)
            .map_err(|e| AppError::Auth(format!("Failed to open whitelist for writing: {}", e)))?;

        writeln!(file, "{}", connect_key)
            .map_err(|e| AppError::Auth(format!("Failed to write to whitelist: {}", e)))?;

        self.keys.push(connect_key.to_string());
        Ok(())
    }

    /// Create a new empty whitelist file
    pub fn create(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Auth(format!("Failed to create directory: {}", e)))?;
        }

        fs::write(path, "# Whitelist for connect keys\n")
            .map_err(|e| AppError::Auth(format!("Failed to create whitelist: {}", e)))?;

        Self::load(path)
    }

    /// Get all keys
    pub fn keys(&self) -> &[String] {
        &self.keys
    }
}
