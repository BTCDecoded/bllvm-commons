# Bitcoin Commons Deployment Guide

Complete guide for deploying and managing Bitcoin Commons infrastructure.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Unified CLI (`bllvm.sh`)](#unified-cli-bllvmsh)
3. [Component Installation](#component-installation)
4. [Management Commands](#management-commands)
5. [Configuration](#configuration)
6. [Multi-Machine Deployment](#multi-machine-deployment)
7. [Using Binary Commands Directly](#using-binary-commands-directly)
8. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Single Machine Setup (5 minutes)

```bash
# Clone/download deployment scripts
cd deployment
chmod +x bllvm.sh

# Install BLLVM node (base build)
sudo ./bllvm.sh install bllvm --public-ip YOUR_IP

# Check status
./bllvm.sh status

# View logs
./bllvm.sh logs bllvm --follow
```

### Multi-Component Setup

```bash
# Install base node
sudo ./bllvm.sh install bllvm --public-ip 1.2.3.4

# Install experimental node (same machine, different binary)
sudo ./bllvm.sh install experimental --public-ip 1.2.3.4

# Install governance app
sudo ./bllvm.sh install commons --github-app-id 123456
```

---

## Unified CLI (`bllvm.sh`)

The `bllvm.sh` script is the single entry point for all deployment operations.

### Commands

| Command | Description | Requires Root |
|---------|-------------|---------------|
| `install` | Install a component | ✅ |
| `update` | Update installed component | ✅ |
| `uninstall` | Remove a component | ✅ |
| `status` | Show status of all components | ❌ |
| `logs` | View service logs | ❌ |
| `restart` | Restart a service | ✅ |
| `health` | Check RPC health/connectivity | ❌ |
| `info` | Show detailed component info | ❌ |
| `config` | Show/edit config file | ❌ (edit: ✅) |

### Components

- **`bllvm`** - Base BLLVM node (production build, full blockchain)
- **`experimental`** - Experimental node (UTXO commitments, dandelion, CTV, etc.)
- **`commons`** - Governance app (bllvm-commons)

### Usage Pattern

```bash
./bllvm.sh [command] [component] [options]
```

---

## Component Installation

### 1. BLLVM Node (Base Build)

**Purpose:** Full Bitcoin node with production optimizations only.

**Installation:**

```bash
sudo ./bllvm.sh install bllvm --public-ip 1.2.3.4
```

**Options:**
- `--public-ip IP` - Public IP for P2P (auto-detected if not provided)
- `--rpc-password PASSWORD` - RPC password (auto-generated if not provided)
- `--version VERSION` - Specific version (default: latest)

**What Gets Installed:**
- Binary: `/opt/bllvm/bllvm`
- Config: `/etc/bllvm/bllvm.toml`
- Data: `/var/lib/bllvm`
- Service: `bllvm.service` (systemd)

**Default Ports:**
- RPC: `8332`
- P2P: `8333`

---

### 2. Experimental Node

**Purpose:** Bitcoin node with experimental features (UTXO commitments, Dandelion++, CTV, Stratum V2, etc.)

**Installation (Pre-built):**

```bash
sudo ./bllvm.sh install experimental --public-ip 1.2.3.4
```

**Installation (Custom Features):**

```bash
sudo ./bllvm.sh install experimental \
  --public-ip 1.2.3.4 \
  --features "utxo-commitments,dandelion,ctv,stratum-v2"
```

**Installation (Build from Source):**

```bash
sudo ./bllvm.sh install experimental \
  --public-ip 1.2.3.4 \
  --build-from-source \
  --source-dir /path/to/bllvm \
  --features "utxo-commitments,dandelion,ctv"
```

**Installation (Custom Binary):**

```bash
sudo ./bllvm.sh install experimental \
  --public-ip 1.2.3.4 \
  --custom-binary /path/to/custom-bllvm
```

**Options:**
- `--public-ip IP` - Public IP for P2P
- `--rpc-password PASSWORD` - RPC password
- `--features FEATURES` - Comma-separated feature flags
- `--build-from-source` - Build from source instead of downloading
- `--source-dir DIR` - Source directory (requires `--build-from-source`)
- `--custom-binary PATH` - Use custom binary file
- `--version VERSION` - Specific version

**What Gets Installed:**
- Binary: `/opt/bllvm/bllvm-experimental`
- Config: `/etc/bllvm/bllvm.toml` (same as base node)
- Data: `/var/lib/bllvm` (can share with base node)
- Service: `bllvm.service` (uses experimental binary)

**Note:** Experimental and base nodes can run on the same machine but use the same service name. Only one can be active at a time unless you configure different ports.

---

### 3. Governance App (bllvm-commons)

**Purpose:** GitHub App for cryptographic governance enforcement.

**Installation:**

```bash
sudo ./bllvm.sh install commons \
  --github-app-id 123456 \
  --github-webhook-secret your-secret
```

**Options:**
- `--github-app-id ID` - GitHub App ID (required)
- `--github-webhook-secret SECRET` - GitHub webhook secret
- `--version VERSION` - Specific version

**What Gets Installed:**
- Binary: `/opt/bllvm-commons/bllvm-commons`
- Config: `/etc/bllvm-commons/app.toml`
- Data: `/var/lib/bllvm-commons`
- Service: `bllvm-commons.service` (systemd)

**Configuration:**

After installation, configure Nostr bots and other settings:

```bash
sudo ./bllvm.sh config commons --edit
```

**Nostr Configuration:**

The governance app supports multi-bot Nostr integration. Configure in `app.toml`:

```toml
[nostr]
enabled = true
governance_config = "commons_mainnet"
zap_address = "donations@your-ln-address.com"
logo_url = "https://btcdecoded.org/assets/bitcoin-commons-logo.png"

[nostr.bots.gov]
nsec_path = "env:NOSTR_NSEC_GOV"  # GitHub Secret
npub = "npub1..."
lightning_address = "gov@your-ln-address.com"

[nostr.bots.dev]
nsec_path = "env:NOSTR_NSEC_DEV"
npub = "npub1..."
lightning_address = "dev@your-ln-address.com"
```

**GitHub Secrets Required:**
- `NOSTR_NSEC_GOV` - Governance bot private key
- `NOSTR_NSEC_DEV` - Development bot private key
- `NOSTR_NSEC_RESEARCH` - Research bot private key (optional)
- `NOSTR_NSEC_NETWORK` - Network bot private key (optional)

---

## Management Commands

### Status

**View all components:**

```bash
./bllvm.sh status
```

**Output:**
```
=== Bitcoin Commons Status ===

bllvm: ✅ Running
  Chain: mainnet
  Blocks: 850000
  Peers: 12

experimental: ❌ Stopped

commons: ✅ Running

Use 'systemctl status bllvm' or 'systemctl status bllvm-commons' for details
```

---

### Health Check

**Check node health:**

```bash
./bllvm.sh health bllvm
```

**Output:**
```
✅ Node is healthy
```

**Note:** Uses `bllvm health` command internally (no manual RPC calls needed).

---

### Info

**Detailed component information:**

```bash
./bllvm.sh info bllvm
```

**Output:**
```
=== bllvm Info ===

Status: ✅ Running
Binary: /opt/bllvm/bllvm
Type: ELF 64-bit LSB executable
Version: BLLVM 0.1.0
Config: /etc/bllvm/bllvm.toml
RPC Port: 8332
P2P Port: 8333
Data: /var/lib/bllvm (250G)
Service: bllvm
Enabled: Yes

=== Node Status ===
Chain: mainnet
Blocks: 850000
Sync: Complete
Peers: 12
```

---

### Logs

**View service logs:**

```bash
# Last 50 lines
./bllvm.sh logs bllvm

# Follow logs
./bllvm.sh logs bllvm --follow

# Last 100 lines
./bllvm.sh logs bllvm -n 100
```

---

### Restart

**Restart a service:**

```bash
sudo ./bllvm.sh restart bllvm
```

**Output:**
```
✅ Restarted: bllvm
```

---

### Config

**View config:**

```bash
./bllvm.sh config bllvm
```

**Edit config:**

```bash
sudo ./bllvm.sh config bllvm --edit
```

**Note:** Uses `bllvm config show` internally for node components.

---

### Update

**Update a component:**

```bash
# Update to latest
sudo ./bllvm.sh update bllvm

# Update to specific version
sudo ./bllvm.sh update experimental --version v1.0.0
```

**What happens:**
1. Downloads new binary
2. Stops service
3. Replaces binary
4. Restarts service
5. Verifies health

---

### Uninstall

**Remove a component:**

```bash
sudo ./bllvm.sh uninstall bllvm
```

**What gets removed:**
- Systemd service
- Binary (`/opt/bllvm/bllvm`)
- Config (`/etc/bllvm/bllvm.toml`)
- Data directory (`/var/lib/bllvm`) - **WARNING: This deletes blockchain data!**

**To keep data:**

```bash
# Uninstall but keep data
sudo ./bllvm.sh uninstall bllvm
# Data remains at /var/lib/bllvm
```

---

## Configuration

### Node Configuration (`bllvm.toml`)

**Location:** `/etc/bllvm/bllvm.toml`

**Key Settings:**

```toml
network = "mainnet"  # mainnet, testnet, regtest

[server]
listen_address = "0.0.0.0:8333"  # P2P
rpc_listen_address = "127.0.0.1:8332"  # RPC

[rpc]
user = "btc"
password = "your-secure-password"

[storage]
data_dir = "/var/lib/bllvm"
```

**View config:**

```bash
./bllvm.sh config bllvm
```

**Edit config:**

```bash
sudo ./bllvm.sh config bllvm --edit
```

---

### Governance App Configuration (`app.toml`)

**Location:** `/etc/bllvm-commons/app.toml`

**Key Settings:**

```toml
[github]
app_id = 123456
webhook_secret = "your-secret"

[nostr]
enabled = true
governance_config = "commons_mainnet"
relays = ["wss://relay.damus.io", "wss://nos.lol"]

[database]
path = "/var/lib/bllvm-commons/db.sqlite"
```

---

## Multi-Machine Deployment

### Scenario: 3 Separate Machines

**Machine 1 (ArchLinux - Innovation Hub):**
- Base BLLVM node (archival)

**Machine 2 (Ubuntu - Linode):**
- Experimental node (UTXO commitments)

**Machine 3 (Ubuntu - Innovation Hub):**
- Experimental node (UTXO commitments)

**Machine 4 (Optional - Any):**
- Governance app (bllvm-commons)

---

### Deployment Steps

**1. On Machine 1 (ArchLinux):**

```bash
cd deployment
chmod +x bllvm.sh
sudo ./bllvm.sh install bllvm --public-ip MACHINE1_IP
./bllvm.sh status
```

**2. On Machine 2 (Ubuntu - Linode):**

```bash
cd deployment
chmod +x bllvm.sh
sudo ./bllvm.sh install experimental --public-ip MACHINE2_IP
./bllvm.sh status
```

**3. On Machine 3 (Ubuntu - Innovation Hub):**

```bash
cd deployment
chmod +x bllvm.sh
sudo ./bllvm.sh install experimental --public-ip MACHINE3_IP
./bllvm.sh status
```

**4. On Machine 4 (Governance App):**

```bash
cd deployment
chmod +x bllvm.sh
sudo ./bllvm.sh install commons --github-app-id 123456
./bllvm.sh status
```

---

### Verification

**Check all nodes:**

```bash
# On each machine
./bllvm.sh health bllvm  # or experimental
./bllvm.sh info bllvm
```

**From a central location:**

```bash
# Test RPC connectivity
curl -u btc:password http://MACHINE1_IP:8332 \
  -d '{"method":"getblockchaininfo","params":[]}'
```

---

## Using Binary Commands Directly

The `bllvm` binary includes native subcommands that can be used directly:

### Available Commands

```bash
# Version
/opt/bllvm/bllvm version

# Status
/opt/bllvm/bllvm status

# Health
/opt/bllvm/bllvm health

# Chain info
/opt/bllvm/bllvm chain

# Peers
/opt/bllvm/bllvm peers

# Network info
/opt/bllvm/bllvm network

# Sync status
/opt/bllvm/bllvm sync

# Config
/opt/bllvm/bllvm config show
/opt/bllvm/bllvm config validate
/opt/bllvm/bllvm config path

# RPC (generic)
/opt/bllvm/bllvm rpc getblockchaininfo
/opt/bllvm/bllvm rpc getpeerinfo '[]'
```

### With Custom RPC Address

```bash
/opt/bllvm/bllvm --rpc-addr 127.0.0.1:8332 status
```

### Integration

The deployment scripts (`health.sh`, `info.sh`, `config.sh`, `status.sh`) use these binary commands internally, so you get the same functionality whether you use the scripts or the binary directly.

---

## Troubleshooting

### Service Won't Start

**Check logs:**

```bash
./bllvm.sh logs bllvm
journalctl -u bllvm -n 100
```

**Check config:**

```bash
./bllvm.sh config bllvm
/opt/bllvm/bllvm config validate
```

**Check permissions:**

```bash
ls -la /opt/bllvm/bllvm
ls -la /var/lib/bllvm
ls -la /etc/bllvm/bllvm.toml
```

---

### RPC Not Responding

**Check if service is running:**

```bash
./bllvm.sh status
systemctl status bllvm
```

**Test health:**

```bash
./bllvm.sh health bllvm
```

**Check firewall:**

```bash
sudo ufw status
sudo firewall-cmd --list-all
```

**Check RPC address in config:**

```bash
./bllvm.sh config bllvm | grep rpc_listen_address
```

---

### Binary Commands Fail

**Check binary exists:**

```bash
ls -la /opt/bllvm/bllvm
file /opt/bllvm/bllvm
```

**Check binary permissions:**

```bash
chmod +x /opt/bllvm/bllvm
```

**Test binary directly:**

```bash
/opt/bllvm/bllvm version
/opt/bllvm/bllvm --help
```

**Note:** Scripts fall back gracefully if binary commands fail.

---

### Experimental Node Issues

**Check feature flags:**

```bash
/opt/bllvm/bllvm-experimental version
```

**Rebuild with different features:**

```bash
sudo ./bllvm.sh uninstall experimental
sudo ./bllvm.sh install experimental \
  --build-from-source \
  --features "utxo-commitments,dandelion"
```

---

### Governance App Issues

**Check Nostr configuration:**

```bash
./bllvm.sh config commons | grep -A 20 nostr
```

**Check GitHub App credentials:**

```bash
./bllvm.sh config commons | grep -A 10 github
```

**View logs:**

```bash
./bllvm.sh logs commons --follow
```

---

## Best Practices

### Security

1. **Use strong RPC passwords:**
   ```bash
   openssl rand -hex 32
   ```

2. **Restrict RPC access:**
   ```toml
   rpc_listen_address = "127.0.0.1:8332"  # Localhost only
   ```

3. **Use firewall:**
   ```bash
   sudo ufw allow 8333/tcp  # P2P only
   sudo ufw deny 8332/tcp   # RPC (if exposed)
   ```

4. **Protect Nostr keys:**
   - Use GitHub Secrets for `NOSTR_NSEC_*`
   - Never commit keys to repository

---

### Monitoring

1. **Regular health checks:**
   ```bash
   ./bllvm.sh health bllvm
   ```

2. **Monitor logs:**
   ```bash
   ./bllvm.sh logs bllvm --follow
   ```

3. **Check disk space:**
   ```bash
   df -h /var/lib/bllvm
   ```

4. **Monitor sync status:**
   ```bash
   ./bllvm.sh info bllvm | grep Sync
   ```

---

### Updates

1. **Test updates on non-production first**

2. **Backup data before updates:**
   ```bash
   sudo systemctl stop bllvm
   sudo cp -r /var/lib/bllvm /var/lib/bllvm.backup
   sudo ./bllvm.sh update bllvm
   ```

3. **Verify after update:**
   ```bash
   ./bllvm.sh health bllvm
   ./bllvm.sh info bllvm
   ```

---

## Summary

- **Single Entry Point:** `bllvm.sh` for all operations
- **Three Components:** `bllvm`, `experimental`, `commons`
- **Native Commands:** Binary subcommands integrated into scripts
- **Multi-Machine:** Deploy across separate machines easily
- **Production Ready:** Full Bitcoin node with governance

**Quick Reference:**

```bash
# Install
sudo ./bllvm.sh install [bllvm|experimental|commons] [options]

# Manage
./bllvm.sh [status|health|info|logs|config|restart] [component]

# Update/Remove
sudo ./bllvm.sh [update|uninstall] [component]
```

---

**Status:** Production Ready  
**Last Updated:** 2024  
**Maintained By:** Bitcoin Commons Team

