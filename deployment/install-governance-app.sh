#!/bin/bash
# Install Governance App (bllvm-commons) - Direct Installation
# Works on ArchLinux and Ubuntu

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}=== BLLVM Commons (Governance App) Installation ===${NC}"
echo ""

# Detect OS
if [ -f /etc/arch-release ]; then
    OS="arch"
    echo "Detected: ArchLinux"
elif [ -f /etc/debian_version ] || [ -f /etc/lsb-release ]; then
    OS="ubuntu"
    echo "Detected: Ubuntu/Debian"
else
    echo -e "${RED}❌ Unsupported OS. This script works on ArchLinux and Ubuntu.${NC}"
    exit 1
fi

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}❌ Please run as root (sudo)${NC}"
    exit 1
fi

# Configuration
INSTALL_DIR="/opt/bllvm-commons"
DATA_DIR="/var/lib/bllvm-commons"
CONFIG_DIR="/etc/bllvm-commons"
KEYS_DIR="/etc/bllvm-commons/keys"
SERVICE_USER="bllvm-commons"
BINARY_URL="https://github.com/BTCDecoded/bllvm-commons/releases/latest/download/bllvm-commons-linux-x86_64.tar.gz"
VERSION="latest"

# Parse arguments
GITHUB_APP_ID=""
GITHUB_WEBHOOK_SECRET=""
PUBLIC_IP=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --github-app-id)
            GITHUB_APP_ID="$2"
            shift 2
            ;;
        --github-webhook-secret)
            GITHUB_WEBHOOK_SECRET="$2"
            shift 2
            ;;
        --public-ip)
            PUBLIC_IP="$2"
            shift 2
            ;;
        --version)
            VERSION="$2"
            BINARY_URL="https://github.com/BTCDecoded/bllvm-commons/releases/download/${VERSION}/bllvm-commons-linux-x86_64.tar.gz"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Get public IP if not provided
if [ -z "$PUBLIC_IP" ]; then
    echo "Detecting public IP..."
    PUBLIC_IP=$(curl -s ifconfig.me || curl -s icanhazip.com || echo "0.0.0.0")
    echo -e "${YELLOW}Detected public IP: ${PUBLIC_IP}${NC}"
    read -p "Is this correct? (y/n): " confirm
    if [ "$confirm" != "y" ]; then
        read -p "Enter public IP: " PUBLIC_IP
    fi
fi

# Create service user
echo ""
echo "Creating service user..."
if ! id "$SERVICE_USER" &>/dev/null; then
    useradd -r -s /bin/false -d "$DATA_DIR" "$SERVICE_USER"
    echo -e "${GREEN}✅ Created user: ${SERVICE_USER}${NC}"
else
    echo -e "${YELLOW}⚠️  User ${SERVICE_USER} already exists${NC}"
fi

# Create directories
echo ""
echo "Creating directories..."
mkdir -p "$INSTALL_DIR"
mkdir -p "$DATA_DIR"
mkdir -p "$CONFIG_DIR"
mkdir -p "$KEYS_DIR/nostr"
mkdir -p "$DATA_DIR/logs"
chown -R "$SERVICE_USER:$SERVICE_USER" "$DATA_DIR"
chown -R "$SERVICE_USER:$SERVICE_USER" "$CONFIG_DIR"
chown -R "$SERVICE_USER:$SERVICE_USER" "$KEYS_DIR"

# Download and install binary
echo ""
echo "Downloading bllvm-commons binary..."
cd /tmp
wget -q "$BINARY_URL" -O bllvm-commons.tar.gz || {
    echo -e "${RED}❌ Failed to download binary${NC}"
    exit 1
}

echo "Extracting binary..."
tar -xzf bllvm-commons.tar.gz
# Try different possible binary names
if [ -f "bllvm-commons" ]; then
    cp bllvm-commons "$INSTALL_DIR/"
elif [ -f "governance-app" ]; then
    cp governance-app "$INSTALL_DIR/bllvm-commons"
else
    find . -name "*commons*" -o -name "*governance*" | grep -v ".tar.gz" | head -1 | xargs -I {} cp {} "$INSTALL_DIR/bllvm-commons"
fi
chmod +x "$INSTALL_DIR/bllvm-commons"
chown root:root "$INSTALL_DIR/bllvm-commons"

# Generate Nostr keys if they don't exist
echo ""
echo "Setting up Nostr keys..."
if [ ! -f "$KEYS_DIR/nostr/gov.nsec" ]; then
    echo -e "${YELLOW}⚠️  Nostr keys not found. Generating placeholder keys.${NC}"
    echo -e "${YELLOW}   You should replace these with real keys from:${NC}"
    echo -e "${YELLOW}   nostr-tool generate > keys/nostr/gov.nsec${NC}"
    for bot in gov dev research network; do
        openssl rand -hex 32 > "$KEYS_DIR/nostr/${bot}.nsec"
        chmod 600 "$KEYS_DIR/nostr/${bot}.nsec"
        chown "$SERVICE_USER:$SERVICE_USER" "$KEYS_DIR/nostr/${bot}.nsec"
    done
else
    echo -e "${GREEN}✅ Nostr keys found${NC}"
fi

# Create config file
echo ""
echo "Creating configuration..."
cat > "$CONFIG_DIR/app.toml" << EOF
[server]
host = "0.0.0.0"
port = 8080

[database]
max_connections = 10
min_connections = 2

[nostr]
enabled = true
governance_config = "commons_mainnet"
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band"
]
publish_interval_secs = 3600

[nostr.bots.gov]
nsec_path = "${KEYS_DIR}/nostr/gov.nsec"
npub = "REPLACE_WITH_NPUB"
lightning_address = "donations@btcdecoded.org"
profile_name = "🏛️ @BTCCommons_Gov"
profile_about = "Official governance announcements from Bitcoin Commons..."
profile_picture = "https://btcdecoded.org/assets/bitcoin-commons-logo.png"

[nostr.bots.dev]
nsec_path = "${KEYS_DIR}/nostr/dev.nsec"
npub = "REPLACE_WITH_NPUB"
lightning_address = "dev@btcdecoded.org"
profile_name = "⚙️ @BTCCommons_Dev"
profile_about = "Development updates from Bitcoin Commons..."
profile_picture = "https://btcdecoded.org/assets/bitcoin-commons-logo.png"

[nostr.bots.research]
nsec_path = "${KEYS_DIR}/nostr/research.nsec"
npub = "REPLACE_WITH_NPUB"
lightning_address = "research@btcdecoded.org"
profile_name = "📚 @BTCCommons_Research"
profile_about = "Educational content from Bitcoin Commons..."
profile_picture = "https://btcdecoded.org/assets/bitcoin-commons-logo.png"

[nostr.bots.network]
nsec_path = "${KEYS_DIR}/nostr/network.nsec"
npub = "REPLACE_WITH_NPUB"
lightning_address = "network@btcdecoded.org"
profile_name = "📊 @BTCCommons_Network"
profile_about = "Network metrics from Bitcoin Commons..."
profile_picture = "https://btcdecoded.org/assets/bitcoin-commons-logo.png"
EOF

chmod 640 "$CONFIG_DIR/app.toml"
chown root:"$SERVICE_USER" "$CONFIG_DIR/app.toml"

# Create environment file
echo ""
echo "Creating environment file..."
cat > "$CONFIG_DIR/environment" << EOF
DATABASE_URL=sqlite://${DATA_DIR}/governance.db
GITHUB_APP_ID=${GITHUB_APP_ID:-123456}
GITHUB_PRIVATE_KEY_PATH=${KEYS_DIR}/github-app.pem
GITHUB_WEBHOOK_SECRET=${GITHUB_WEBHOOK_SECRET:-CHANGE_THIS_SECRET}
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
SERVER_ID=governance-01
NOSTR_ENABLED=true
NOSTR_SERVER_NSEC_PATH=${KEYS_DIR}/nostr/gov.nsec
NOSTR_RELAYS=wss://relay.damus.io,wss://nos.lol,wss://relay.nostr.band
NOSTR_PUBLISH_INTERVAL_SECS=3600
GOVERNANCE_CONFIG=commons_mainnet
NOSTR_ZAP_ADDRESS=donations@btcdecoded.org
NOSTR_LOGO_URL=https://btcdecoded.org/assets/bitcoin-commons-logo.png
OTS_ENABLED=true
OTS_AGGREGATOR_URL=https://alice.btc.calendar.opentimestamps.org
AUDIT_ENABLED=true
RUST_LOG=info
EOF

chmod 640 "$CONFIG_DIR/environment"
chown root:"$SERVICE_USER" "$CONFIG_DIR/environment"

# Create systemd service
echo ""
echo "Creating systemd service..."
cat > /etc/systemd/system/bllvm-commons.service << EOF
[Unit]
Description=BLLVM Commons (Governance App)
After=network.target

[Service]
Type=simple
User=${SERVICE_USER}
Group=${SERVICE_USER}
WorkingDirectory=${DATA_DIR}
EnvironmentFile=${CONFIG_DIR}/environment
ExecStart=${INSTALL_DIR}/bllvm-commons
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=${DATA_DIR}

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd
systemctl daemon-reload

# Instructions
echo ""
echo -e "${GREEN}=== Installation Complete ===${NC}"
echo ""
echo -e "${YELLOW}⚠️  IMPORTANT: Manual steps required:${NC}"
echo ""
echo "1. Add GitHub App private key:"
echo "   sudo cp /path/to/github-app.pem ${KEYS_DIR}/github-app.pem"
echo "   sudo chmod 600 ${KEYS_DIR}/github-app.pem"
echo "   sudo chown ${SERVICE_USER}:${SERVICE_USER} ${KEYS_DIR}/github-app.pem"
echo ""
echo "2. Update environment file with real secrets:"
echo "   sudo nano ${CONFIG_DIR}/environment"
echo ""
echo "3. Get npubs and update config:"
echo "   nostr-tool convert ${KEYS_DIR}/nostr/gov.nsec"
echo "   sudo nano ${CONFIG_DIR}/app.toml"
echo ""
echo "4. Start the service:"
echo "   sudo systemctl enable bllvm-commons"
echo "   sudo systemctl start bllvm-commons"
echo ""
echo "5. Check status:"
echo "   sudo systemctl status bllvm-commons"
echo "   sudo journalctl -u bllvm-commons -f"
echo ""

