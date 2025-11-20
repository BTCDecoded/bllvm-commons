#!/bin/bash
# Check RPC health/connectivity
set -e

COMPONENT="${1:-}"
if [ -z "$COMPONENT" ]; then
    echo "Usage: ./health.sh [component]"
    echo "Components: bllvm, experimental, commons"
    exit 1
fi

case "$COMPONENT" in
    bllvm|experimental)
        SERVICE_NAME="bllvm"
        INSTALL_DIR="/opt/bllvm"
        if [ "$COMPONENT" = "bllvm" ]; then
            BINARY_NAME="bllvm"
        else
            BINARY_NAME="bllvm-experimental"
        fi
        BINARY_PATH="$INSTALL_DIR/$BINARY_NAME"
        ;;
    commons)
        SERVICE_NAME="bllvm-commons"
        echo "⚠️  Commons doesn't have RPC, checking service status only"
        if systemctl is-active --quiet "$SERVICE_NAME"; then
            echo "✅ Service is running"
        else
            echo "❌ Service is not running"
            exit 1
        fi
        exit 0
        ;;
    *)
        echo "❌ Unknown component: $COMPONENT"
        exit 1
        ;;
esac

if ! systemctl list-unit-files | grep -q "${SERVICE_NAME}.service"; then
    echo "❌ Service $SERVICE_NAME not installed"
    exit 1
fi

# Check service status
if ! systemctl is-active --quiet "$SERVICE_NAME"; then
    echo "❌ Service is not running"
    exit 1
fi

# Use bllvm binary health command
if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Binary not found: $BINARY_PATH"
    exit 1
fi

if "$BINARY_PATH" health 2>/dev/null; then
    echo "✅ Node is healthy"
else
    echo "❌ Health check failed"
    exit 1
fi

