#!/bin/bash
# Bitcoin Commons Unified CLI
# Usage: ./bllvm.sh [command] [component] [options]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

show_usage() {
    cat << EOF
Bitcoin Commons Unified CLI

Usage:
  ./bllvm.sh [command] [component] [options]

Commands:
  install     Install a component
  update      Update an installed component
  uninstall   Remove an installed component
  status      Show status of installed components
  logs        View service logs
  restart     Restart a service
  health      Check RPC health/connectivity
  info        Show detailed component info
  config      Show/edit config file

Components:
  bllvm        BLLVM node (base build)
  experimental Experimental node (UTXO commitments + features)
  commons      Governance app (bllvm-commons)

Examples:
  # Install
  sudo ./bllvm.sh install bllvm --public-ip 1.2.3.4
  sudo ./bllvm.sh install experimental --public-ip 1.2.3.4
  sudo ./bllvm.sh install commons --github-app-id 123456

  # Update
  sudo ./bllvm.sh update bllvm
  sudo ./bllvm.sh update experimental --version v1.0.0

  # Uninstall
  sudo ./bllvm.sh uninstall bllvm
  sudo ./bllvm.sh uninstall experimental

  # Status
  ./bllvm.sh status

  # Logs
  ./bllvm.sh logs bllvm
  ./bllvm.sh logs bllvm --follow

  # Restart
  sudo ./bllvm.sh restart bllvm

  # Health check
  ./bllvm.sh health bllvm

  # Info
  ./bllvm.sh info bllvm

  # Config
  ./bllvm.sh config bllvm
  sudo ./bllvm.sh config bllvm --edit

EOF
}

COMMAND="${1:-}"
if [ -z "$COMMAND" ]; then
    show_usage
    exit 1
fi
shift

case "$COMMAND" in
    install)
        exec "$SCRIPT_DIR/install.sh" "$@"
        ;;
    update)
        exec "$SCRIPT_DIR/update.sh" "$@"
        ;;
    uninstall)
        exec "$SCRIPT_DIR/uninstall.sh" "$@"
        ;;
    status)
        exec "$SCRIPT_DIR/status.sh" "$@"
        ;;
    logs)
        exec "$SCRIPT_DIR/logs.sh" "$@"
        ;;
    restart)
        exec "$SCRIPT_DIR/restart.sh" "$@"
        ;;
    health)
        exec "$SCRIPT_DIR/health.sh" "$@"
        ;;
    info)
        exec "$SCRIPT_DIR/info.sh" "$@"
        ;;
    config)
        exec "$SCRIPT_DIR/config.sh" "$@"
        ;;
    help|--help|-h)
        show_usage
        exit 0
        ;;
    *)
        echo "❌ Unknown command: $COMMAND"
        show_usage
        exit 1
        ;;
esac

