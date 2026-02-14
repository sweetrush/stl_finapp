# Usage Examples

**Developed by: [sweetrush](https://github.com/sweetrush)**

This document provides comprehensive examples for using the Secure Finance Messaging Block Application in various scenarios.

## Table of Contents

- [Quick Start](#quick-start)
- [Non-Interactive Mode](#non-interactive-mode)
  - [Server Setup](#server-setup-non-interactive)
  - [Client Usage](#client-usage-non-interactive)
  - [Key Management](#key-management)
- [Interactive Mode](#interactive-mode)
  - [Starting Interactive Mode](#starting-interactive-mode)
  - [Interactive Commands](#interactive-commands)
  - [Full Interactive Session Example](#full-interactive-session-example)
- [Two-Server Setup](#two-server-setup)
- [Advanced Scenarios](#advanced-scenarios)

---

## Quick Start

```bash
# Build the application
cargo build --release

# Generate keys (first time setup)
./target/release/stl_finapp keygen

# Add your first connect key
./target/release/stl_finapp whitelist --ck "my-secret-key-123"

# Start server in one terminal
./target/release/stl_finapp listen --port 8080

# In another terminal, send a message
./target/release/stl_finapp send -i 127.0.0.1 -p 8080 -f message.txt --ck "my-secret-key-123"
```

---

## Non-Interactive Mode

Non-interactive mode is ideal for scripting, automation, and headless server operations.

### Server Setup (Non-Interactive)

#### Basic Server

```bash
# Start server on default port 8080
./stl_finapp listen

# Output:
# [-] Listening on 0.0.0.0:8080
# [SUCCESS] Server started on port 8080
# [?] Press Ctrl+C to stop the server
```

#### Custom Port

```bash
# Start server on port 9000
./stl_finapp listen --port 9000

# Output:
# [-] Listening on 0.0.0.0:9000
# [SUCCESS] Server started on port 9000
# [?] Press Ctrl+C to stop the server
```

#### Custom Paths

```bash
# Use custom whitelist and keys directories
./stl_finapp listen \
    --port 8080 \
    --whitelist /secure/whitelist.txt \
    --keys /secure/keys

# Output:
# [-] Listening on 0.0.0.0:8080
# [SUCCESS] Server started on port 8080
# [?] Press Ctrl+C to stop the server
```

#### Background Server (Linux/macOS)

```bash
# Run server in background with nohup
nohup ./stl_finapp listen --port 8080 > server.log 2>&1 &

# Check if running
ps aux | grep stl_finapp

# View logs
tail -f server.log

# Stop the server
pkill -f stl_finapp
```

### Client Usage (Non-Interactive)

#### Basic Message Sending

```bash
# Send a message to a server
./stl_finapp send \
    --ip 192.168.1.100 \
    --port 8080 \
    --file documents/report.txt \
    --ck "my-secret-key-123"

# Output:
# [*] Connecting to 192.168.1.100:8080...
# [*] Authenticating...
# [+] Authentication successful
# [INFO] Sending file: report.txt (2048 bytes)
# [*] Encrypting message...
# [*] Sending 2200 bytes...
# [SUCCESS] Message delivered, saved as: report.txt_20240214_153045.ftt
```

#### With Custom Filename

```bash
# Send with a custom remote filename
./stl_finapp send \
    -i 192.168.1.100 \
    -p 8080 \
    -f documents/q1_financial_report.txt \
    --ck "my-secret-key-123" \
    -s "q1_report"

# Output:
# [*] Connecting to 192.168.1.100:8080...
# [*] Authenticating...
# [+] Authentication successful
# [INFO] Sending file: q1_report (5120 bytes)
# [*] Encrypting message...
# [*] Sending 5300 bytes...
# [SUCCESS] Message delivered, saved as: q1_report_20240214_153100.ftt
```

#### Using Custom Keys Directory

```bash
# Use keys from a specific directory
./stl_finapp send \
    -i 192.168.1.100 \
    -p 8080 \
    -f message.txt \
    --ck "my-secret-key-123" \
    -k /secure/keys
```

#### Shorthand Mode (Legacy)

```bash
# Alternative shorthand syntax
./stl_finapp -i 192.168.1.100 -f message.txt --ck "my-secret-key-123"

# With save-as name
./stl_finapp -i 192.168.1.100 -f message.txt --ck "my-secret-key-123" -s "important"
```

### Key Management

#### Generate Keys

```bash
# Generate keys to default directory (keys/)
./stl_finapp keygen

# Output:
# [SUCCESS] Keys generated in keys
# [?] Share your public_key.pem with other servers
# [?] Keep private_key.pem secure and never share it!
```

```bash
# Generate keys to custom directory
./stl_finapp keygen --output /secure/server-keys

# Output:
# [SUCCESS] Keys generated in /secure/server-keys
# [?] Share your public_key.pem with other servers
# [?] Keep private_key.pem secure and never share it!
```

#### Manage Whitelist

```bash
# Add a connect key to the default whitelist
./stl_finapp whitelist --ck "partner-server-key-abc"

# Output:
# [SUCCESS] Connect key added to whitelist: partner-server-key-abc
```

```bash
# Add key to custom whitelist file
./stl_finapp whitelist \
    --ck "partner-server-key-xyz" \
    --file /secure/whitelist.txt

# Output:
# [SUCCESS] Connect key added to whitelist: partner-server-key-xyz
```

#### View Whitelist

```bash
# View the whitelist file
cat keys/whitelist.txt

# Example output:
# # Whitelist for connect keys
# partner-server-key-abc
# partner-server-key-xyz
# branch-office-key-001
```

---

## Interactive Mode

Interactive mode provides a REPL interface for convenient operation and testing.

### Starting Interactive Mode

```bash
# Start interactive mode
./stl_finapp -m

# Or using long form
./stl_finapp --interactive

# Output:
# Secure Finance Messaging Application
# ──────────────────────────────────────────────────────────
# [?] Type 'help' for available commands
# [INFO] Loaded keys from keys
#
# finapp>
```

### Interactive Commands

| Command | Short | Description |
|---------|-------|-------------|
| `listen [port]` | `l` | Start server (default: 8080) |
| `stop` | | Stop the listening server |
| `send <ip> <file> [name]` | `s` | Send message to server |
| `status` | | Show current status |
| `keygen [dir]` | `k` | Generate new key pair |
| `whitelist <key>` | `w` | Add key to whitelist |
| `help` | `h`, `?` | Show help message |
| `exit`, `quit` | `q` | Exit interactive mode |

### Full Interactive Session Example

```
finapp> help

Available Commands
──────────────────────────────────────────────────────────
  listen [port]        Start listening server (default: 8080)
  stop                 Stop the listening server
  send <ip> <file> [name] Send message to server
  status               Show current status
  keygen [dir]         Generate new key pair
  whitelist <key>      Add key to whitelist
  help                 Show this help message
  exit / quit          Exit interactive mode

finapp> status

Current Status
─────────────────────────────────────────────────────────
  Server: Not running
  Keys: Loaded from keys

finapp> listen 8080
[-] Listening on 0.0.0.0:8080
[SUCCESS] Server started on port 8080
[?] Press Ctrl+C to stop the server

finapp> status

Current Status
─────────────────────────────────────────────────────────
  Server: Listening on port 8080
  Keys: Loaded from keys

finapp> whitelist branch-key-2024
[SUCCESS] Connect key added to whitelist: branch-key-2024

finapp> send 192.168.1.200 data/transaction.txt daily_tx
Enter connect key: ********
[*] Connecting to 192.168.1.200:8080...
[*] Authenticating...
[+] Authentication successful
[INFO] Sending file: daily_tx (1024 bytes)
[*] Encrypting message...
[*] Sending 1150 bytes...
[SUCCESS] Message delivered, saved as: daily_tx_20240214_140530.ftt

finapp> stop
[INFO] Server stopped

finapp> status

Current Status
─────────────────────────────────────────────────────────
  Server: Not running
  Keys: Loaded from keys

finapp> exit
[INFO] Goodbye!
```

---

## Two-Server Setup

This example demonstrates a complete setup between two servers exchanging messages.

### Server A (Head Office - 192.168.1.100)

```bash
# Step 1: Generate keys
./stl_finapp keygen --output /secure/ho-keys

# Step 2: Add Branch Office's connect key to whitelist
./stl_finapp whitelist \
    --ck "branch-office-secure-key-2024" \
    --file /secure/ho-whitelist.txt

# Step 3: Start server
./stl_finapp listen \
    --port 8080 \
    --whitelist /secure/ho-whitelist.txt \
    --keys /secure/ho-keys

# Output:
# [-] Listening on 0.0.0.0:8080
# [SUCCESS] Server started on port 8080
# [?] Press Ctrl+C to stop the server

# Server is now waiting for connections...
# When a message arrives:
# [INFO] Connection from 192.168.1.101:54321
# [INFO] Challenge sent to client
# [+] Authentication successful
# [INFO] Public keys exchanged
# [INFO] Receiving file: daily_report (2048 bytes)
# [*] Receiving 2200 bytes...
# [*] Decrypting message...
# [SUCCESS] File saved: daily_report_20240214_150000.ftt
# [SUCCESS] Message transfer complete
```

### Server B (Branch Office - 192.168.1.101)

```bash
# Step 1: Generate keys
./stl_finapp keygen --output /secure/bo-keys

# Step 2: Send message to Head Office
./stl_finapp send \
    --ip 192.168.1.100 \
    --port 8080 \
    --file reports/daily_report.txt \
    --ck "branch-office-secure-key-2024" \
    --save-as "daily_report" \
    --keys /secure/bo-keys

# Output:
# [*] Connecting to 192.168.1.100:8080...
# [*] Authenticating...
# [+] Authentication successful
# [INFO] Sending file: daily_report (2048 bytes)
# [*] Encrypting message...
# [*] Sending 2200 bytes...
# [SUCCESS] Message delivered, saved as: daily_report_20240214_150000.ftt
```

### Exchanging Public Keys (Optional for Enhanced Security)

For additional verification, you can exchange public keys out-of-band:

```bash
# On Server A, export public key
cat /secure/ho-keys/public_key.pem

# On Server B, save Server A's public key for verification
# (This can be used for additional trust verification in future versions)
```

---

## Advanced Scenarios

### Scenario 1: Automated Daily Report Transfer

Create a script for automated daily transfers:

```bash
#!/bin/bash
# daily_transfer.sh

SERVER_IP="192.168.1.100"
SERVER_PORT="8080"
CONNECT_KEY="automated-daily-key-2024"
KEYS_DIR="/secure/keys"
REPORT_FILE="/data/daily_$(date +%Y%m%d).txt"

./stl_finapp send \
    --ip $SERVER_IP \
    --port $SERVER_PORT \
    --file $REPORT_FILE \
    --ck $CONNECT_KEY \
    --save-as "daily_$(date +%Y%m%d)" \
    --keys $KEYS_DIR

# Add to crontab for daily execution at 6 PM
# 0 18 * * * /path/to/daily_transfer.sh >> /var/log/finapp_transfer.log 2>&1
```

### Scenario 2: Multi-Branch Setup

Server at headquarters receiving from multiple branches:

```bash
# Generate keys once
./stl_finapp keygen

# Add all branch keys to whitelist
./stl_finapp whitelist --ck "branch-east-key-001"
./stl_finapp whitelist --ck "branch-west-key-002"
./stl_finapp whitelist --ck "branch-north-key-003"
./stl_finapp whitelist --ck "branch-south-key-004"

# Start server
./stl_finapp listen --port 8080

# The server will now accept connections from any of the whitelisted keys
```

### Scenario 3: High-Security Setup

Using custom paths and restricted permissions:

```bash
# Create secure directories
sudo mkdir -p /opt/finapp/{keys,messages}
sudo chmod 700 /opt/finapp/keys

# Generate keys
./stl_finapp keygen --output /opt/finapp/keys

# Create whitelist with proper permissions
echo "# Secure whitelist" | sudo tee /opt/finapp/keys/whitelist.txt
sudo chmod 600 /opt/finapp/keys/whitelist.txt

# Add authorized keys
./stl_finapp whitelist \
    --ck "high-security-key-$(openssl rand -hex 16)" \
    --file /opt/finapp/keys/whitelist.txt

# Start server
./stl_finapp listen \
    --port 8443 \
    --whitelist /opt/finapp/keys/whitelist.txt \
    --keys /opt/finapp/keys
```

### Scenario 4: Testing Locally

Test the application on a single machine:

```bash
# Terminal 1: Start server
./stl_finapp listen --port 8080

# Terminal 2: Add a test key
./stl_finapp whitelist --ck "test-key-123"

# Terminal 3: Send test message
echo "Hello, this is a test message" > test_message.txt
./stl_finapp send -i 127.0.0.1 -p 8080 -f test_message.txt --ck "test-key-123"

# Check received message
ls -la messages/
cat messages/*.ftt
```

### Scenario 5: Systemd Service Setup (Linux)

Create a systemd service for the server:

```bash
# Create service file
sudo tee /etc/systemd/system/finapp.service << 'EOF'
[Unit]
Description=Secure Finance Messaging Server
After=network.target

[Service]
Type=simple
User=finapp
Group=finapp
WorkingDirectory=/opt/finapp
ExecStart=/opt/finapp/stl_finapp listen --port 8080
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable finapp
sudo systemctl start finapp

# Check status
sudo systemctl status finapp

# View logs
sudo journalctl -u finapp -f
```

### Scenario 6: Docker Container

Create a Dockerfile for containerized deployment:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/stl_finapp /usr/local/bin/
VOLUME ["/app/keys", "/app/messages"]
EXPOSE 8080
CMD ["stl_finapp", "listen", "--port", "8080"]
```

```bash
# Build image
docker build -t finapp:latest .

# Run server container
docker run -d \
    --name finapp-server \
    -p 8080:8080 \
    -v $(pwd)/keys:/app/keys \
    -v $(pwd)/messages:/app/messages \
    finapp:latest

# Send message using local client
./stl_finapp send -i localhost -p 8080 -f message.txt --ck "your-key"
```

---

## Troubleshooting Examples

### Connection Refused

```bash
# Error:
# [ERROR] Failed to connect to 192.168.1.100:8080: Connection refused

# Check if server is running
./stl_finapp send -i 192.168.1.100 -p 8080 -f test.txt --ck "key"
# [ERROR] Failed to connect to 192.168.1.100:8080: Connection refused

# Solution: Ensure server is listening
# On server machine:
./stl_finapp listen --port 8080
```

### Authentication Failed

```bash
# Error:
# [!] Authentication failed: Invalid connect key

# Solution: Add the correct key to server's whitelist
./stl_finapp whitelist --ck "correct-key-123"
```

### Key Not Found

```bash
# Error:
# [WARNING] Failed to load keys: Failed to read private key

# Solution: Generate keys
./stl_finapp keygen
```

### Checksum Verification Failed

```bash
# Error (on server side):
# [ERROR] Checksum verification failed

# This indicates data corruption or tampering
# Solution: Resend the message
```

---

## See Also

- [README.md](README.md) - Full documentation
- [Cargo.toml](Cargo.toml) - Project dependencies
