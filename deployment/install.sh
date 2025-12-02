#!/bin/bash
# Bitcoin Commons Unified Installer
# Single entry point for all installations
# Usage: ./install.sh [component] [options]

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Show usage
show_usage() {
    cat << EOF
${BLUE}Bitcoin Commons Unified Installer${NC}

${GREEN}Usage:${NC}
  ./install.sh [component] [options]

${GREEN}Components:${NC}
  bllvm               Install BLLVM node (base build, full blockchain)
  experimental        Install experimental node (UTXO commitments, custom features)
  commons             Install governance app (bllvm-commons)

${GREEN}Examples:${NC}
  # Install BLLVM node
  sudo ./install.sh bllvm --public-ip 1.2.3.4

  # Install experimental node with default features
  sudo ./install.sh experimental --public-ip 1.2.3.4

  # Install experimental node with custom feature flags
  sudo ./install.sh experimental --public-ip 1.2.3.4 \\
    --features "utxo-commitments,dandelion,ctv,stratum-v2"

  # Install governance app
  sudo ./install.sh commons --github-app-id 123456

${GREEN}Options:${NC}
  --public-ip IP           Public IP address for P2P
  --rpc-password PASSWORD   RPC password (auto-generated if not provided)
  --features FEATURES        Custom feature flags (experimental only)
  --github-app-id ID         GitHub App ID (commons only)
  --github-webhook-secret    GitHub webhook secret (commons only)
  --version VERSION          Specific version to install (default: latest)

${GREEN}Advanced:${NC}
  --build-from-source        Build from source instead of using pre-built binary
  --source-dir DIR           Source directory for building (requires --build-from-source)
  --custom-binary PATH       Use custom binary instead of downloading

EOF
}

# Parse component
COMPONENT="${1:-}"

if [ -z "$COMPONENT" ]; then
    show_usage
    exit 1
fi

# Shift to get remaining arguments
shift

# Route to appropriate installer
case "$COMPONENT" in
    bllvm)
        echo -e "${BLUE}Installing BLLVM Node (Base Build)${NC}"
        exec "$SCRIPT_DIR/install-bllvm-node.sh" "$@"
        ;;
    experimental)
        echo -e "${BLUE}Installing Experimental Node (UTXO Commitments + Custom Features)${NC}"
        exec "$SCRIPT_DIR/install-experimental-node.sh" "$@"
        ;;
    commons)
        echo -e "${BLUE}Installing Governance App (bllvm-commons)${NC}"
        exec "$SCRIPT_DIR/install-governance-app.sh" "$@"
        ;;
    help|--help|-h)
        show_usage
        exit 0
        ;;
    *)
        echo -e "${RED}❌ Unknown component: ${COMPONENT}${NC}"
        echo ""
        show_usage
        exit 1
        ;;
esac

