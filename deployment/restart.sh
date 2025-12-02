#!/bin/bash
# Restart service
set -e

COMPONENT="${1:-}"
if [ -z "$COMPONENT" ]; then
    echo "Usage: ./restart.sh [component]"
    echo "Components: bllvm, experimental, commons"
    exit 1
fi

if [ "$EUID" -ne 0 ]; then echo "Run as root"; exit 1; fi

case "$COMPONENT" in
    bllvm|experimental)
        SERVICE_NAME="bllvm"
        ;;
    commons)
        SERVICE_NAME="bllvm-commons"
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

systemctl restart "$SERVICE_NAME"
sleep 2

if systemctl is-active --quiet "$SERVICE_NAME"; then
    echo "✅ Restarted: $COMPONENT"
else
    echo "❌ Restart failed. Check: journalctl -u $SERVICE_NAME"
    exit 1
fi

