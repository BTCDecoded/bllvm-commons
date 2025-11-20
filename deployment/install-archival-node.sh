#!/bin/bash
# Install BLLVM Archival Node (Direct Installation)
# Works on ArchLinux and Ubuntu

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}=== BLLVM Archival Node Installation ===${NC}"
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
INSTALL_DIR="/opt/bllvm"
DATA_DIR="/var/lib/bllvm-archival"
CONFIG_DIR="/etc/bllvm"
SERVICE_USER="bllvm"
BINARY_URL="https://github.com/BTCDecoded/bllvm/releases/latest/download/bllvm-linux-x86_64.tar.gz"
VERSION="latest"

# Parse arguments
PUBLIC_IP=""
RPC_PASSWORD=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --public-ip)
            PUBLIC_IP="$2"
            shift 2
            ;;
        --rpc-password)
            RPC_PASSWORD="$2"
            shift 2
            ;;
        --version)
            VERSION="$2"
            BINARY_URL="https://github.com/BTCDecoded/bllvm/releases/download/${VERSION}/bllvm-linux-x86_64.tar.gz"
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

# Generate RPC password if not provided
if [ -z "$RPC_PASSWORD" ]; then
    RPC_PASSWORD=$(openssl rand -hex 32)
    echo -e "${GREEN}Generated RPC password: ${RPC_PASSWORD}${NC}"
    echo -e "${YELLOW}⚠️  Save this password!${NC}"
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
chown -R "$SERVICE_USER:$SERVICE_USER" "$DATA_DIR"
chown -R "$SERVICE_USER:$SERVICE_USER" "$CONFIG_DIR"

# Download and install binary
echo ""
echo "Downloading BLLVM binary..."
cd /tmp
wget -q "$BINARY_URL" -O bllvm.tar.gz || {
    echo -e "${RED}❌ Failed to download binary${NC}"
    exit 1
}

echo "Extracting binary..."
tar -xzf bllvm.tar.gz
cp bllvm "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/bllvm"
chown root:root "$INSTALL_DIR/bllvm"

# Create config file
echo ""
echo "Creating configuration..."
cat > "$CONFIG_DIR/archival.toml" << EOF
[network]
network = "mainnet"
listen_address = "0.0.0.0:8333"
external_address = "${PUBLIC_IP}:8333"

[storage]
# Archival mode - keep all blocks
mode = "archival"
data_dir = "${DATA_DIR}"

[rpc]
enabled = true
listen_address = "0.0.0.0:8332"
rpc_user = "btc"
rpc_password = "${RPC_PASSWORD}"

[logging]
level = "info"
EOF

chmod 640 "$CONFIG_DIR/archival.toml"
chown root:"$SERVICE_USER" "$CONFIG_DIR/archival.toml"

# Create systemd service
echo ""
echo "Creating systemd service..."
cat > /etc/systemd/system/bllvm-archival.service << EOF
[Unit]
Description=BLLVM Archival Node
After=network.target

[Service]
Type=simple
User=${SERVICE_USER}
Group=${SERVICE_USER}
WorkingDirectory=${DATA_DIR}
ExecStart=${INSTALL_DIR}/bllvm --config ${CONFIG_DIR}/archival.toml
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

# Enable and start service
echo ""
echo "Starting service..."
systemctl enable bllvm-archival
systemctl start bllvm-archival

# Wait a moment for service to start
sleep 3

# Check status
if systemctl is-active --quiet bllvm-archival; then
    echo -e "${GREEN}✅ Service started successfully${NC}"
else
    echo -e "${RED}❌ Service failed to start. Check logs: journalctl -u bllvm-archival${NC}"
    exit 1
fi

# Display information
echo ""
echo -e "${GREEN}=== Installation Complete ===${NC}"
echo ""
echo "Service: bllvm-archival"
echo "Status: $(systemctl is-active bllvm-archival)"
echo "Config: ${CONFIG_DIR}/archival.toml"
echo "Data: ${DATA_DIR}"
echo "RPC: http://localhost:8332"
echo "P2P: ${PUBLIC_IP}:8333"
echo ""
echo "RPC Credentials:"
echo "  User: btc"
echo "  Password: ${RPC_PASSWORD}"
echo ""
echo -e "${YELLOW}⚠️  Save the RPC password!${NC}"
echo ""
echo "Useful commands:"
echo "  sudo systemctl status bllvm-archival"
echo "  sudo journalctl -u bllvm-archival -f"
echo "  sudo systemctl restart bllvm-archival"
echo ""

