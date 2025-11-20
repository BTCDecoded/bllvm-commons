#!/bin/bash
# Show status of installed components
set -e

echo "=== Bitcoin Commons Status ==="
echo ""

for component in bllvm experimental commons; do
    case "$component" in
        bllvm|experimental)
            SERVICE_NAME="bllvm"
            INSTALL_DIR="/opt/bllvm"
            if [ "$component" = "bllvm" ]; then
                BINARY_NAME="bllvm"
            else
                BINARY_NAME="bllvm-experimental"
            fi
            BINARY_PATH="$INSTALL_DIR/$BINARY_NAME"
            ;;
        commons)
            SERVICE_NAME="bllvm-commons"
            BINARY_PATH=""
            ;;
    esac
    
    if systemctl list-unit-files | grep -q "${SERVICE_NAME}.service"; then
        if systemctl is-active --quiet "$SERVICE_NAME" 2>/dev/null; then
            STATUS="✅ Running"
            # For node components, try to get detailed status
            if [ "$component" != "commons" ] && [ -x "$BINARY_PATH" ] 2>/dev/null; then
                echo -n "$component: $STATUS"
                # Try to get node status (non-blocking, timeout quickly)
                NODE_STATUS=$("$BINARY_PATH" status 2>/dev/null | head -3 || echo "")
                if [ -n "$NODE_STATUS" ]; then
                    echo ""
                    echo "  $NODE_STATUS" | sed 's/^/  /'
                else
                    echo ""
                fi
            else
                echo "$component: $STATUS"
            fi
        else
            STATUS="❌ Stopped"
            echo "$component: $STATUS"
        fi
    else
        echo "$component: Not installed"
    fi
done

echo ""
echo "Use 'systemctl status bllvm' or 'systemctl status bllvm-commons' for details"

