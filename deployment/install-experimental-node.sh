#!/bin/bash
# Install BLLVM Experimental Node - Custom Features
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

INSTALL_DIR="/opt/bllvm"
DATA_DIR="/var/lib/bllvm"
CONFIG_DIR="/etc/bllvm"
CONFIG_FILE="$CONFIG_DIR/bllvm.toml"
SERVICE_USER="bllvm"
SERVICE_NAME="bllvm"
BINARY_NAME="bllvm-experimental"
BINARY_URL="https://github.com/BTCDecoded/bllvm/releases/latest/download/bllvm-experimental-linux-x86_64.tar.gz"

PUBLIC_IP=""
RPC_PASSWORD=""
RPC_PORT="8332"
P2P_PORT="8333"
FEATURES="utxo-commitments,dandelion,ctv"
BUILD_FROM_SOURCE=false
SOURCE_DIR=""
CUSTOM_BINARY=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --public-ip) PUBLIC_IP="$2"; shift 2 ;;
        --rpc-password) RPC_PASSWORD="$2"; shift 2 ;;
        --features) FEATURES="$2"; shift 2 ;;
        --build-from-source) BUILD_FROM_SOURCE=true; shift ;;
        --source-dir) SOURCE_DIR="$2"; BUILD_FROM_SOURCE=true; shift 2 ;;
        --custom-binary) CUSTOM_BINARY="$2"; shift 2 ;;
        --version) BINARY_URL="https://github.com/BTCDecoded/bllvm/releases/download/$2/bllvm-experimental-linux-x86_64.tar.gz"; shift 2 ;;
        *) echo "Unknown: $1"; exit 1 ;;
    esac
done

if [ "$EUID" -ne 0 ]; then echo "Run as root"; exit 1; fi

if [ -z "$PUBLIC_IP" ]; then
    PUBLIC_IP=$(curl -s ifconfig.me || curl -s icanhazip.com || echo "0.0.0.0")
fi

if [ -z "$RPC_PASSWORD" ]; then
    RPC_PASSWORD=$(openssl rand -hex 32)
fi

if ! id "$SERVICE_USER" &>/dev/null; then
    useradd -r -s /bin/false -d "$DATA_DIR" "$SERVICE_USER"
fi

mkdir -p "$INSTALL_DIR" "$DATA_DIR" "$CONFIG_DIR"
chown -R "$SERVICE_USER:$SERVICE_USER" "$DATA_DIR" "$CONFIG_DIR"

if [ -n "$CUSTOM_BINARY" ]; then
    cp "$CUSTOM_BINARY" "$INSTALL_DIR/$BINARY_NAME"
elif [ "$BUILD_FROM_SOURCE" = true ]; then
    if [ -z "$SOURCE_DIR" ]; then
        SOURCE_DIR="/tmp/bllvm-source"
        git clone https://github.com/BTCDecoded/bllvm.git "$SOURCE_DIR"
    fi
    cd "$SOURCE_DIR"
    if ! command -v cargo &> /dev/null; then
        echo "❌ Install Rust: https://rustup.rs/"
        exit 1
    fi
    cargo build --release --features "$FEATURES" --bin bllvm
    cp "target/release/bllvm" "$INSTALL_DIR/$BINARY_NAME"
else
    cd /tmp
    wget -q "$BINARY_URL" -O bllvm-experimental.tar.gz
    tar -xzf bllvm-experimental.tar.gz
    cp bllvm "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null || \
    cp bllvm-experimental "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null || \
    find . -name "bllvm*" -type f -executable | head -1 | xargs -I {} cp {} "$INSTALL_DIR/$BINARY_NAME"
fi

chmod +x "$INSTALL_DIR/$BINARY_NAME"
chown root:root "$INSTALL_DIR/$BINARY_NAME"

cat > "$CONFIG_FILE" << EOF
[network]
network = "mainnet"
listen_address = "0.0.0.0:${P2P_PORT}"
external_address = "${PUBLIC_IP}:${P2P_PORT}"

[storage]
mode = "pruned"
prune_mode = "normal"
keep_from_height = 0
min_blocks_to_keep = 288
data_dir = "${DATA_DIR}"

[rpc]
enabled = true
listen_address = "0.0.0.0:${RPC_PORT}"
rpc_user = "btc"
rpc_password = "${RPC_PASSWORD}"

[features]
EOF

IFS=',' read -ra FEATURE_ARRAY <<< "$FEATURES"
for feature in "${FEATURE_ARRAY[@]}"; do
    echo "$(echo "$feature" | xargs) = true" >> "$CONFIG_FILE"
done

cat >> "$CONFIG_FILE" << EOF

[logging]
level = "info"
EOF

chmod 640 "$CONFIG_FILE"
chown root:"$SERVICE_USER" "$CONFIG_FILE"

cat > /etc/systemd/system/${SERVICE_NAME}.service << EOF
[Unit]
Description=BLLVM Node (Experimental)
After=network.target

[Service]
Type=simple
User=${SERVICE_USER}
Group=${SERVICE_USER}
WorkingDirectory=${DATA_DIR}
ExecStart=${INSTALL_DIR}/${BINARY_NAME} --config ${CONFIG_FILE}
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=${DATA_DIR}

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable ${SERVICE_NAME}
systemctl start ${SERVICE_NAME}
sleep 2

if systemctl is-active --quiet ${SERVICE_NAME}; then
    echo "✅ Installed: ${SERVICE_NAME} (Features: ${FEATURES})"
    echo "RPC: localhost:${RPC_PORT} (btc:${RPC_PASSWORD})"
else
    echo "❌ Failed. Check: journalctl -u ${SERVICE_NAME}"
    exit 1
fi
