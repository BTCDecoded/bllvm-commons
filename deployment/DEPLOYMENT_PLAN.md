# Bitcoin Commons Deployment Plan

**Target Infrastructure:**
- **Linode (Remote)**: Governance App + Experimental Node + GitHub Runner
- **Start9 (LAN)**: Archival Node

---

## Overview

### Deployment Targets

| Location | Components | Access Method | IP/Address |
|----------|-----------|---------------|------------|
| **Linode** | Governance App + Experimental Node | SSH | `jswift@mybitcoinfuture` |
| **Start9** | Archival Node | HTTPS (StartOS) | `https://192.168.2.101/` |

---

## Prerequisites

### 1. Linode Server (Remote)

**Requirements:**
- Ubuntu/Debian Linux
- SSH access via VPN
- Root or sudo access
- Minimum 4GB RAM, 100GB disk (for experimental node)
- Public IP address (for P2P networking)

**Verify Access:**
```bash
# Test SSH connection
ssh jswift@mybitcoinfuture

# Check system info
uname -a
df -h
free -h

# Verify GitHub runner is accessible (if already set up)
# Check if runner is running
```

**Required Information:**
- Public IP address of Linode (for P2P)
- GitHub App ID (for governance app)
- GitHub Webhook Secret (for governance app)
- Nostr private keys (nsec) for bots (or generate new ones)

---

### 2. Start9 Server (LAN)

**Requirements:**
- StartOS installed and running
- Accessible at `https://192.168.2.101/`
- SSH access (if available) or StartOS service management
- Minimum 2TB disk space (for full blockchain)
- Public IP or port forwarding (for P2P)

**Verify Access:**
```bash
# Test HTTPS access
curl -k https://192.168.2.101/

# If SSH available:
ssh start9@192.168.2.101
```

**Required Information:**
- Public IP address (if available) or port forwarding setup
- SSH credentials (if using command-line deployment)

---

## Deployment Steps

### Phase 1: Prepare Deployment Materials

#### 1.1 Clone/Download Deployment Scripts

**On your local machine:**

```bash
# Navigate to deployment directory
cd /home/user/src/BTCDecoded/deployment

# Verify scripts are executable
chmod +x bllvm.sh install*.sh *.sh

# Create deployment package (optional, for transfer)
tar -czf deployment-scripts.tar.gz *.sh bllvm.sh
```

#### 1.2 Prepare Configuration Values

**Create a deployment config file (local):**

```bash
cat > deployment-config.env << EOF
# Linode Configuration
LINODE_PUBLIC_IP="<LINODE_PUBLIC_IP>"
LINODE_SSH="jswift@mybitcoinfuture"
LINODE_PUBLIC_HOSTNAME="mybitcoinfuture.com"  # For GitHub webhook URL

# Start9 Configuration
START9_IP="192.168.2.101"
START9_PUBLIC_IP="<START9_PUBLIC_IP_OR_PORT_FORWARD>"

# Governance App Configuration
GITHUB_APP_ID="<YOUR_GITHUB_APP_ID>"
GITHUB_WEBHOOK_SECRET="<YOUR_WEBHOOK_SECRET>"

# RPC Passwords (will be auto-generated if not provided)
# LINODE_RPC_PASSWORD="<OPTIONAL>"
# START9_RPC_PASSWORD="<OPTIONAL>"
EOF

# Keep this file secure, don't commit it
chmod 600 deployment-config.env
```

---

### Phase 2: Deploy to Linode (Governance App + Experimental Node)

#### 2.1 Transfer Deployment Scripts

**From local machine:**

```bash
# Transfer scripts to Linode
scp -r bllvm.sh install*.sh *.sh jswift@mybitcoinfuture:/tmp/bllvm-deployment/

# Or transfer tarball
scp deployment-scripts.tar.gz jswift@mybitcoinfuture:/tmp/
```

**On Linode:**

```bash
# Extract if using tarball
cd /tmp
tar -xzf deployment-scripts.tar.gz
cd bllvm-deployment

# Make scripts executable
chmod +x bllvm.sh install*.sh *.sh
```

---

#### 2.2 Install Experimental Node

**On Linode:**

```bash
# Get public IP (if not already known)
PUBLIC_IP=$(curl -s ifconfig.me || curl -s icanhazip.com)
echo "Public IP: $PUBLIC_IP"

# Install experimental node
sudo ./bllvm.sh install experimental --public-ip "$PUBLIC_IP"

# Verify installation
./bllvm.sh status
./bllvm.sh health experimental
```

**Expected Output:**
- Binary: `/opt/bllvm/bllvm-experimental`
- Config: `/etc/bllvm/bllvm.toml`
- Service: `bllvm.service` (running)
- RPC: `localhost:8332`
- P2P: `$PUBLIC_IP:8333`

**Verify Node Status:**

```bash
# Check service
systemctl status bllvm

# Check logs
./bllvm.sh logs experimental --follow

# Get node info
./bllvm.sh info experimental

# Test RPC
/opt/bllvm/bllvm-experimental health
```

---

#### 2.3 Install Governance App

**On Linode:**

```bash
# Install governance app
sudo ./bllvm.sh install commons \
  --github-app-id "$GITHUB_APP_ID" \
  --github-webhook-secret "$GITHUB_WEBHOOK_SECRET"

# Verify installation
./bllvm.sh status
systemctl status bllvm-commons
```

**Expected Output:**
- Binary: `/opt/bllvm-commons/bllvm-commons`
- Config: `/etc/bllvm-commons/app.toml`
- Service: `bllvm-commons.service` (running)
- Port: `8080` (default)

**Configure Nostr Bots:**

```bash
# Edit config
sudo ./bllvm.sh config commons --edit

# Or manually edit
sudo nano /etc/bllvm-commons/app.toml
```

**Update Nostr configuration in `/etc/bllvm-commons/app.toml`:**

```toml
[nostr]
enabled = true
governance_config = "commons_mainnet"
logo_url = "https://btcdecoded.org/assets/bitcoin-commons-logo.png"

[nostr.bots.gov]
nsec_path = "env:NOSTR_NSEC_GOV"  # Or file path
npub = "npub1..."  # Replace with actual npub
lightning_address = "donations@btcdecoded.org"
profile_name = "🏛️ @BTCCommons_Gov"

[nostr.bots.dev]
nsec_path = "env:NOSTR_NSEC_DEV"
npub = "npub1..."
lightning_address = "dev@btcdecoded.org"
profile_name = "⚙️ @BTCCommons_Dev"

# ... (research and network bots if needed)
```

**Set GitHub Secrets (if using env: paths):**

```bash
# These should be set in GitHub Secrets, not locally
# The app will read them at runtime if configured
```

**Restart governance app:**

```bash
sudo ./bllvm.sh restart commons
./bllvm.sh logs commons --follow
```

---

#### 2.4 Configure Firewall (Linode)

**Allow necessary ports:**

```bash
# Check if ufw is installed
sudo ufw status

# Allow P2P port (if not already open)
sudo ufw allow 8333/tcp

# Allow RPC port (restrict to localhost/VPN only)
sudo ufw allow from 10.0.0.0/8 to any port 8332  # VPN subnet
sudo ufw allow from 192.168.0.0/16 to any port 8332  # LAN subnet

# Allow governance app port (REQUIRED for GitHub webhooks)
sudo ufw allow 8080/tcp

# Enable firewall
sudo ufw enable
```

**Important: GitHub Webhook Port & Security**

✅ **YES, you MUST open port 8080** for GitHub webhooks to work.

GitHub needs to send webhook events to your governance app. The app listens on port 8080.

**⚠️ Security: Minimal Exposure**

**Option 1: Whitelist GitHub IP Ranges (RECOMMENDED)**

Restrict port 8080 to GitHub's IP addresses only:

```bash
# GitHub webhook IP ranges (updated regularly, check GitHub docs)
# Primary ranges:
sudo ufw allow from 140.82.112.0/20 to any port 8080
sudo ufw allow from 143.55.64.0/20 to any port 8080
sudo ufw allow from 185.199.108.0/22 to any port 8080
sudo ufw allow from 192.30.252.0/22 to any port 8080
sudo ufw allow from 2a0a:a440::/29 to any port 8080  # IPv6

# Or use GitHub's meta API to get current IPs (recommended)
# https://api.github.com/meta
```

**Get current GitHub IPs:**

```bash
# Fetch current GitHub IP ranges
curl -s https://api.github.com/meta | jq -r '.hooks[]' | while read ip; do
    sudo ufw allow from "$ip" to any port 8080
done
```

**Option 2: VPN/Tailscale (NOT RECOMMENDED for Webhooks)**

❌ **VPN/Tailscale won't work** because:
- GitHub's servers need to reach your webhook from the internet
- They can't connect through your VPN
- Webhooks must be publicly accessible (but can be IP-restricted)

**Option 3: Reverse Proxy with IP Restrictions**

Set up nginx/caddy with IP whitelisting:

```nginx
# /etc/nginx/sites-available/webhook
server {
    listen 443 ssl;
    server_name mybitcoinfuture.com;
    
    # Whitelist GitHub IPs
    allow 140.82.112.0/20;
    allow 143.55.64.0/20;
    allow 185.199.108.0/22;
    allow 192.30.252.0/22;
    deny all;
    
    location /webhook {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

**Webhook URL Configuration:**

When setting up the GitHub App, configure the webhook URL as:
```
https://mybitcoinfuture.com:8080/webhook
```

Or with reverse proxy:
```
https://mybitcoinfuture.com/webhook
```

**Since GitHub runner is on the same Linode:**
- The runner can use `http://localhost:8080/webhook` for local operations
- GitHub's servers use the public URL (IP-restricted)
- Both work simultaneously

**Best Practice:**
1. ✅ Whitelist GitHub IP ranges (most secure)
2. ✅ Use HTTPS (reverse proxy recommended)
3. ✅ Monitor webhook logs for unauthorized access attempts
4. ✅ Rotate webhook secrets regularly

---

### Phase 3: Deploy to Start9 (Archival Node)

#### 3.1 Access Start9

**Option A: Via StartOS Web Interface**

1. Open browser: `https://192.168.2.101/`
2. Log in to StartOS
3. Navigate to Services/Apps
4. Look for SSH or Terminal access option

**Option B: Via SSH (if available)**

```bash
# From local machine
ssh start9@192.168.2.101
```

---

#### 3.2 Transfer Deployment Scripts to Start9

**From local machine:**

```bash
# Transfer scripts
scp -r bllvm.sh install-bllvm-node.sh start9@192.168.2.101:/tmp/bllvm-deployment/
```

**On Start9:**

```bash
cd /tmp/bllvm-deployment
chmod +x bllvm.sh install-bllvm-node.sh
```

---

#### 3.3 Verify Disk Space and Port Conflicts (CRITICAL - Do This First!)

**On Start9:**

```bash
# Check total disk space
df -h

# Check Bitcoin Core data size (if already synced)
du -sh ~/.bitcoin 2>/dev/null || \
du -sh /embassy-data/package-data/volumes/bitcoin* 2>/dev/null || \
du -sh /var/lib/bitcoin* 2>/dev/null || \
echo "Bitcoin Core not found in standard locations"

# Calculate space requirements:
# - Bitcoin Core (if exists): ~600GB
# - BLLVM archival node: ~600GB (estimated, similar to Bitcoin Core)
# - Total needed if both: ~1.2TB
# - Recommended free space: 1.5TB+ for safety

# Check available space on root or data partition
df -h / | tail -1
df -h /embassy-data 2>/dev/null | tail -1
df -h /var/lib 2>/dev/null | tail -1
```

**⚠️ Do NOT proceed if less than 600GB free space available for BLLVM.**

**Check for Port Conflicts (if Bitcoin Core is running):**

```bash
# Check what ports Bitcoin Core is using
sudo netstat -tlnp | grep -E "8332|8333"

# Check Bitcoin Core config for custom ports
grep -E "rpcport|port" ~/.bitcoin/bitcoin.conf 2>/dev/null || \
grep -E "rpcport|port" /embassy-data/package-data/volumes/bitcoin*/bitcoin.conf 2>/dev/null || \
echo "Bitcoin Core using default ports (8332 RPC, 8333 P2P)"
```

**⚠️ IMPORTANT: Port Conflict Resolution**

**Both Bitcoin Core and BLLVM use the same default ports:**
- RPC: 8332 (both)
- P2P: 8333 (both)

**They CAN run simultaneously, but MUST use different ports.**

**If Bitcoin Core is running:**
- BLLVM will need to use different ports (e.g., 8334/8335)
- You'll need to modify BLLVM config after installation (see below)

**If space is limited, options:**

1. **Prune Bitcoin Core** (reduces to ~10GB, but loses full archival capability):
   ```bash
   # Edit Bitcoin Core config to enable pruning
   # This is a Bitcoin Core setting, not BLLVM
   ```

2. **Remove Bitcoin Core** (if not needed):
   ```bash
   # Uninstall Bitcoin Core via StartOS interface
   # Or stop/remove the service
   ```

3. **Upgrade storage** on Start9

4. **Use BLLVM only** (remove Bitcoin Core if BLLVM is sufficient)

---

#### 3.4 Install Archival Node

**On Start9 (after verifying disk space and port conflicts):**

```bash
# Get public IP (if available) or use port forwarding address
# For LAN-only, use 0.0.0.0 or the Start9's LAN IP
PUBLIC_IP=$(curl -s ifconfig.me 2>/dev/null || echo "192.168.2.101")
echo "Using IP: $PUBLIC_IP"

# Install archival node
sudo ./bllvm.sh install bllvm --public-ip "$PUBLIC_IP"

# Verify installation
./bllvm.sh status
./bllvm.sh health bllvm
```

**⚠️ If Bitcoin Core is running, configure BLLVM to use different ports:**

```bash
# Edit BLLVM config
sudo nano /etc/bllvm/bllvm.toml

# Change ports to avoid conflict with Bitcoin Core:
# [network]
# listen_address = "0.0.0.0:8334"  # Instead of 8333
#
# [rpc]
# listen_address = "0.0.0.0:8335"  # Instead of 8332

# Restart BLLVM
sudo systemctl restart bllvm

# Verify both are running on different ports
sudo netstat -tlnp | grep -E "8332|8333|8334|8335"
```

**Port Configuration:**
- Bitcoin Core: RPC on 8332, P2P on 8333 (default)
- BLLVM: RPC on 8335, P2P on 8334 (if modified to avoid conflict)

**Expected Output:**
- Binary: `/opt/bllvm/bllvm`
- Config: `/etc/bllvm/bllvm.toml`
- Service: `bllvm.service` (running)
- RPC: `localhost:8335` (or 8332 if Bitcoin Core not running)
- P2P: `192.168.2.101:8334` (or 8333 if Bitcoin Core not running)

**Verify Node Status:**

```bash
# Check service
systemctl status bllvm

# Check logs
./bllvm.sh logs bllvm --follow

# Get node info
./bllvm.sh info bllvm

# Test RPC
/opt/bllvm/bllvm health
```

---

#### 3.5 Configure Start9 Firewall/Port Forwarding

**If Start9 has port forwarding:**

1. Log in to StartOS: `https://192.168.2.101/`
2. Navigate to Network/Port Forwarding
3. Forward external port 8333 → 192.168.2.101:8333 (P2P)
4. Forward external port 8332 → 192.168.2.101:8332 (RPC, restrict access)

**Or configure in router:**
- Forward port 8333 to 192.168.2.101:8333

---

### Phase 4: Verification and Testing

#### 4.1 Verify All Services

**Linode:**

```bash
# Check experimental node
./bllvm.sh status
./bllvm.sh health experimental
/opt/bllvm/bllvm-experimental status

# Check governance app
systemctl status bllvm-commons
curl http://localhost:8080/health  # If health endpoint exists
```

**Start9:**

```bash
# Check archival node
./bllvm.sh status
./bllvm.sh health bllvm
/opt/bllvm/bllvm status
```

---

#### 4.2 Test Node Connectivity

**From local machine or Linode:**

```bash
# Test Start9 node RPC (if accessible)
curl -u btc:PASSWORD http://192.168.2.101:8332 \
  -d '{"method":"getblockchaininfo","params":[]}'

# Test Linode node RPC
curl -u btc:PASSWORD http://LINODE_IP:8332 \
  -d '{"method":"getblockchaininfo","params":[]}'
```

**Get RPC passwords:**

```bash
# On each server
grep rpc_password /etc/bllvm/bllvm.toml
```

---

#### 4.3 Verify P2P Connectivity

**Check peer connections:**

```bash
# On Linode
/opt/bllvm/bllvm-experimental peers

# On Start9
/opt/bllvm/bllvm peers
```

**Expected:**
- Nodes should discover each other via P2P
- Both should connect to Bitcoin network
- Both should sync blockchain

---

#### 4.4 Monitor Initial Sync

**On both nodes:**

```bash
# Watch sync progress
watch -n 5 '/opt/bllvm/bllvm sync'

# Or check status
/opt/bllvm/bllvm status | grep -i sync
```

**Expected Timeline:**
- Initial sync: Several hours to days (depending on network speed)
- Experimental node: May sync faster (fewer historical blocks if pruning enabled)
- Archival node: Full sync (all blocks)

---

### Phase 5: Post-Deployment Configuration

#### 5.1 Configure Monitoring

**Set up log monitoring:**

```bash
# On Linode
# Add to crontab for regular health checks
(crontab -l 2>/dev/null; echo "*/5 * * * * /opt/bllvm/bllvm-experimental health > /dev/null 2>&1") | crontab -

# On Start9
(crontab -l 2>/dev/null; echo "*/5 * * * * /opt/bllvm/bllvm health > /dev/null 2>&1") | crontab -
```

---

#### 5.2 Backup Configuration

**Backup configs:**

```bash
# On Linode
sudo tar -czf /root/bllvm-configs-backup-$(date +%Y%m%d).tar.gz \
  /etc/bllvm/bllvm.toml \
  /etc/bllvm-commons/app.toml

# On Start9
sudo tar -czf /root/bllvm-config-backup-$(date +%Y%m%d).tar.gz \
  /etc/bllvm/bllvm.toml
```

---

#### 5.3 Document Deployment

**Create deployment record:**

```bash
cat > deployment-record-$(date +%Y%m%d).txt << EOF
Bitcoin Commons Deployment Record
Date: $(date)

Linode (Remote):
- Experimental Node: $LINODE_PUBLIC_IP:8333
- Governance App: $LINODE_PUBLIC_IP:8080
- RPC: $LINODE_PUBLIC_IP:8332

Start9 (LAN):
- Archival Node: 192.168.2.101:8333
- RPC: 192.168.2.101:8332

RPC Passwords:
- Linode: [stored in /etc/bllvm/bllvm.toml]
- Start9: [stored in /etc/bllvm/bllvm.toml]

GitHub App ID: $GITHUB_APP_ID
EOF
```

---

## Troubleshooting

### Common Issues

#### 1. Cannot Connect to Linode via VPN

**Solution:**
- Verify VPN connection is active
- Check VPN routing table
- Test: `ping mybitcoinfuture`
- Verify SSH key is added to Linode

---

#### 2. Start9 Not Accessible

**Solution:**
- Verify Start9 is powered on
- Check network connection: `ping 192.168.2.101`
- Verify HTTPS access: `curl -k https://192.168.2.101/`
- Check StartOS is running

---

#### 3. Node Not Syncing

**Solution:**
```bash
# Check logs
./bllvm.sh logs bllvm --follow

# Check network connectivity
/opt/bllvm/bllvm peers

# Check disk space
df -h /var/lib/bllvm

# Restart node
sudo ./bllvm.sh restart bllvm
```

---

#### 4. Governance App Not Starting

**Solution:**
```bash
# Check logs
./bllvm.sh logs commons --follow

# Verify config
sudo ./bllvm.sh config commons

# Check GitHub App credentials
grep -A 5 github /etc/bllvm-commons/app.toml

# Restart
sudo ./bllvm.sh restart commons
```

---

#### 5. RPC Not Responding

**Solution:**
```bash
# Check if service is running
systemctl status bllvm

# Check RPC port
netstat -tlnp | grep 8332

# Test locally
/opt/bllvm/bllvm health

# Check firewall
sudo ufw status
```

---

## Security Considerations

### 1. RPC Access

- **Linode**: Restrict RPC to VPN subnet or localhost only
- **Start9**: Restrict RPC to LAN only (192.168.2.0/24)

**Update config:**

```toml
[rpc]
listen_address = "127.0.0.1:8332"  # Localhost only
# Or
listen_address = "192.168.2.101:8332"  # LAN only
```

---

### 2. P2P Port

- Port 8333 should be publicly accessible for P2P
- Use firewall to restrict other ports

---

### 3. Governance App

- Expose only if needed
- Use reverse proxy with HTTPS
- Restrict access to trusted IPs

---

### 4. Credentials

- Store RPC passwords securely
- Use strong passwords (auto-generated)
- Don't commit credentials to git
- Rotate passwords periodically

---

## Maintenance

### Regular Tasks

**Weekly:**
- Check node sync status
- Review logs for errors
- Verify services are running

**Monthly:**
- Update binaries (if new releases)
- Review disk space
- Backup configurations

**Commands:**

```bash
# Check status
./bllvm.sh status

# Update (when new version available)
sudo ./bllvm.sh update bllvm
sudo ./bllvm.sh update experimental
sudo ./bllvm.sh update commons

# Check disk space
df -h /var/lib/bllvm
```

---

## Quick Reference

### Linode Commands

```bash
# SSH
ssh jswift@mybitcoinfuture

# Status
./bllvm.sh status

# Logs
./bllvm.sh logs experimental --follow
./bllvm.sh logs commons --follow

# Restart
sudo ./bllvm.sh restart experimental
sudo ./bllvm.sh restart commons
```

### Start9 Commands

```bash
# Access
# Via HTTPS: https://192.168.2.101/
# Via SSH: ssh start9@192.168.2.101

# Status
./bllvm.sh status

# Logs
./bllvm.sh logs bllvm --follow

# Restart
sudo ./bllvm.sh restart bllvm
```

---

## Summary

**Deployment Complete When:**

✅ Experimental node running on Linode  
✅ Governance app running on Linode  
✅ Archival node running on Start9  
✅ All nodes syncing blockchain  
✅ P2P connectivity established  
✅ RPC accessible (restricted)  
✅ Services auto-start on boot  
✅ Monitoring in place  

**Next Steps:**

1. Monitor initial sync (may take days)
2. Configure Nostr bots (if not done)
3. Set up automated monitoring
4. Document any custom configurations
5. Plan for regular maintenance

---

**Status:** Ready to Deploy  
**Estimated Time:** 2-4 hours (excluding sync time)  
**Difficulty:** Medium

