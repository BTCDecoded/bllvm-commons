#!/bin/bash
# Update installed components
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

COMPONENT="${1:-}"
if [ -z "$COMPONENT" ]; then
    echo "Usage: ./update.sh [component] [options]"
    echo "Components: bllvm, experimental, commons"
    exit 1
fi
shift

VERSION="latest"
while [[ $# -gt 0 ]]; do
    case $1 in
        --version) VERSION="$2"; shift 2 ;;
        *) echo "Unknown: $1"; exit 1 ;;
    esac
done

if [ "$EUID" -ne 0 ]; then echo "Run as root"; exit 1; fi

case "$COMPONENT" in
    bllvm)
        SERVICE_NAME="bllvm"
        INSTALL_DIR="/opt/bllvm"
        BINARY_NAME="bllvm"
        BINARY_URL="https://github.com/BTCDecoded/bllvm/releases/latest/download/bllvm-linux-x86_64.tar.gz"
        if [ "$VERSION" != "latest" ]; then
            BINARY_URL="https://github.com/BTCDecoded/bllvm/releases/download/${VERSION}/bllvm-linux-x86_64.tar.gz"
        fi
        ;;
    experimental)
        SERVICE_NAME="bllvm"
        INSTALL_DIR="/opt/bllvm"
        BINARY_NAME="bllvm-experimental"
        BINARY_URL="https://github.com/BTCDecoded/bllvm/releases/latest/download/bllvm-experimental-linux-x86_64.tar.gz"
        if [ "$VERSION" != "latest" ]; then
            BINARY_URL="https://github.com/BTCDecoded/bllvm/releases/download/${VERSION}/bllvm-experimental-linux-x86_64.tar.gz"
        fi
        ;;
    commons)
        SERVICE_NAME="bllvm-commons"
        INSTALL_DIR="/opt/bllvm-commons"
        BINARY_NAME="bllvm-commons"
        BINARY_URL="https://github.com/BTCDecoded/bllvm-commons/releases/latest/download/bllvm-commons-linux-x86_64.tar.gz"
        if [ "$VERSION" != "latest" ]; then
            BINARY_URL="https://github.com/BTCDecoded/bllvm-commons/releases/download/${VERSION}/bllvm-commons-linux-x86_64.tar.gz"
        fi
        ;;
    *)
        echo "❌ Unknown component: $COMPONENT"
        exit 1
        ;;
esac

if ! systemctl is-active --quiet "$SERVICE_NAME" 2>/dev/null; then
    echo "❌ Service $SERVICE_NAME not running. Install first."
    exit 1
fi

echo "Updating $COMPONENT..."
systemctl stop "$SERVICE_NAME"

cd /tmp
wget -q "$BINARY_URL" -O "${BINARY_NAME}.tar.gz"
tar -xzf "${BINARY_NAME}.tar.gz"
cp "$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"
chown root:root "$INSTALL_DIR/$BINARY_NAME"

systemctl start "$SERVICE_NAME"
sleep 2

if systemctl is-active --quiet "$SERVICE_NAME"; then
    echo "✅ Updated: $COMPONENT"
else
    echo "❌ Update failed. Check: journalctl -u $SERVICE_NAME"
    exit 1
fi

