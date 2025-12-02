#!/bin/bash
# Show detailed info about installed component
set -e

COMPONENT="${1:-}"
if [ -z "$COMPONENT" ]; then
    echo "Usage: ./info.sh [component]"
    echo "Components: bllvm, experimental, commons"
    exit 1
fi

case "$COMPONENT" in
    bllvm|experimental)
        SERVICE_NAME="bllvm"
        INSTALL_DIR="/opt/bllvm"
        DATA_DIR="/var/lib/bllvm"
        CONFIG_FILE="/etc/bllvm/bllvm.toml"
        if [ "$COMPONENT" = "bllvm" ]; then
            BINARY_NAME="bllvm"
        else
            BINARY_NAME="bllvm-experimental"
        fi
        BINARY_PATH="$INSTALL_DIR/$BINARY_NAME"
        ;;
    commons)
        SERVICE_NAME="bllvm-commons"
        INSTALL_DIR="/opt/bllvm-commons"
        DATA_DIR="/var/lib/bllvm-commons"
        CONFIG_FILE="/etc/bllvm-commons/app.toml"
        BINARY_NAME="bllvm-commons"
        BINARY_PATH="$INSTALL_DIR/$BINARY_NAME"
        ;;
    *)
        echo "❌ Unknown component: $COMPONENT"
        exit 1
        ;;
esac

echo "=== $COMPONENT Info ==="
echo ""

if ! systemctl list-unit-files | grep -q "${SERVICE_NAME}.service"; then
    echo "❌ Not installed"
    exit 1
fi

# Service status
if systemctl is-active --quiet "$SERVICE_NAME"; then
    STATUS="✅ Running"
else
    STATUS="❌ Stopped"
fi
echo "Status: $STATUS"

# Binary info
if [ -f "$BINARY_PATH" ]; then
    echo "Binary: $BINARY_PATH"
    if command -v file &> /dev/null; then
        echo "Type: $(file "$BINARY_PATH" | cut -d':' -f2 | xargs)"
    fi
    if [ -x "$BINARY_PATH" ]; then
        # Use bllvm version command if available
        if [ "$COMPONENT" != "commons" ]; then
            VERSION=$("$BINARY_PATH" version 2>/dev/null || "$BINARY_PATH" --version 2>/dev/null || echo "unknown")
        else
            VERSION=$("$BINARY_PATH" --version 2>/dev/null || echo "unknown")
        fi
        echo "Version: $VERSION"
    fi
else
    echo "Binary: Not found"
fi

# Config
if [ -f "$CONFIG_FILE" ]; then
    echo "Config: $CONFIG_FILE"
    
    # Extract key info from config
    if [ "$COMPONENT" != "commons" ]; then
        RPC_PORT=$(grep -E "^listen_address" "$CONFIG_FILE" | grep -oE '[0-9]+' | tail -1 || echo "unknown")
        P2P_PORT=$(grep -E "^listen_address" "$CONFIG_FILE" | grep -oE '[0-9]+' | head -1 || echo "unknown")
        echo "RPC Port: $RPC_PORT"
        echo "P2P Port: $P2P_PORT"
    fi
else
    echo "Config: Not found"
fi

# Data directory
if [ -d "$DATA_DIR" ]; then
    SIZE=$(du -sh "$DATA_DIR" 2>/dev/null | cut -f1 || echo "unknown")
    echo "Data: $DATA_DIR ($SIZE)"
else
    echo "Data: Not found"
fi

# Systemd service
echo "Service: $SERVICE_NAME"
if systemctl is-enabled --quiet "$SERVICE_NAME" 2>/dev/null; then
    echo "Enabled: Yes"
else
    echo "Enabled: No"
fi

# Node-specific info using bllvm commands (if node is running)
if [ "$COMPONENT" != "commons" ] && [ -x "$BINARY_PATH" ] && systemctl is-active --quiet "$SERVICE_NAME" 2>/dev/null; then
    echo ""
    echo "=== Node Status ==="
    if "$BINARY_PATH" status 2>/dev/null; then
        echo ""
    else
        echo "⚠️  Could not get node status (node may be starting up)"
    fi
fi

