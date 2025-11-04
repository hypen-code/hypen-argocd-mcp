#!/bin/bash
# Installation script for ArgoCD MCP Server
# This script copies the Python wrapper and Rust binary to a specified location
# with the correct directory structure for portability.

set -e  # Exit on error

# Default installation directory
DEFAULT_INSTALL_DIR="$HOME/.local/bin/argocd-mcp-server"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored messages
print_error() {
    echo -e "${RED}ERROR: $1${NC}" >&2
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Installation directory (use argument or default)
INSTALL_DIR="${1:-$DEFAULT_INSTALL_DIR}"

# Determine binary name based on platform
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    BINARY_NAME="argocd-mcp-server.exe"
else
    BINARY_NAME="argocd-mcp-server"
fi

# Paths
PYTHON_WRAPPER="$SCRIPT_DIR/argocd_mcp_server.py"
BINARY_PATH="$SCRIPT_DIR/target/release/$BINARY_NAME"

echo "========================================"
echo "ArgoCD MCP Server Installation"
echo "========================================"
echo ""

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    print_error "Binary not found at: $BINARY_PATH"
    print_info "Please build the binary first:"
    echo "  cargo build --release"
    exit 1
fi

# Check if Python wrapper exists
if [ ! -f "$PYTHON_WRAPPER" ]; then
    print_error "Python wrapper not found at: $PYTHON_WRAPPER"
    exit 1
fi

# Create installation directory
print_info "Installing to: $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"
mkdir -p "$INSTALL_DIR/bin"

# Copy files
print_info "Copying Python wrapper..."
cp "$PYTHON_WRAPPER" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/argocd_mcp_server.py"
print_success "Python wrapper copied"

print_info "Copying binary ($BINARY_NAME)..."
cp "$BINARY_PATH" "$INSTALL_DIR/bin/"
chmod +x "$INSTALL_DIR/bin/$BINARY_NAME"
print_success "Binary copied"

# Verify installation
if [ -f "$INSTALL_DIR/argocd_mcp_server.py" ] && [ -f "$INSTALL_DIR/bin/$BINARY_NAME" ]; then
    print_success "Installation completed successfully!"
    echo ""
    echo "Installation directory: $INSTALL_DIR"
    echo ""
    echo "To use the server, you need to set these environment variables:"
    echo "  export ARGOCD_BASE_URL=https://your-argocd-server.com"
    echo "  export ARGOCD_ACCESS_TOKEN=your-access-token"
    echo ""
    echo "Run the server:"
    echo "  python3 $INSTALL_DIR/argocd_mcp_server.py"
    echo ""
    echo "Or add to your MCP configuration:"
    echo '{'
    echo '  "mcpServers": {'
    echo '    "argocd": {'
    echo '      "command": "python3",'
    echo "      \"args\": [\"$INSTALL_DIR/argocd_mcp_server.py\"],"
    echo '      "env": {'
    echo '        "ARGOCD_BASE_URL": "https://your-argocd-server.com",'
    echo '        "ARGOCD_ACCESS_TOKEN": "your-access-token"'
    echo '      }'
    echo '    }'
    echo '  }'
    echo '}'
else
    print_error "Installation verification failed"
    exit 1
fi

echo ""
echo "========================================"
