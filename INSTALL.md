# Installation Guide

This guide explains how to install and deploy the ArgoCD MCP Server to any location.

## Understanding the Structure

The ArgoCD MCP Server consists of two components:

1. **Python Wrapper** (`argocd_mcp_server.py`) - A lightweight launcher script
2. **Rust Binary** (`target/release/argocd-mcp-server`) - The actual MCP server

The Python wrapper looks for the binary in these locations (in order):
- `target/release/argocd-mcp-server` (development)
- `bin/argocd-mcp-server` (installed/production)
- `argocd-mcp-server` (same directory as wrapper)

## Installation Methods

### Method 1: Automated Installation (Recommended)

Use the provided installation script:

```bash
# Build the binary first
cargo build --release

# Install to default location (~/.local/bin/argocd-mcp-server)
./install.sh

# Or install to custom location
./install.sh /path/to/installation/directory
```

The script will:
- ✅ Create the correct directory structure
- ✅ Copy the Python wrapper and binary
- ✅ Set executable permissions
- ✅ Verify the installation
- ✅ Provide usage instructions

### Method 2: Manual Installation

#### Option A: Same Directory Structure

Copy both files to the same directory:

```bash
# Create installation directory
mkdir -p /path/to/install

# Copy files
cp argocd_mcp_server.py /path/to/install/
cp target/release/argocd-mcp-server /path/to/install/

# Make executable
chmod +x /path/to/install/argocd_mcp_server.py
chmod +x /path/to/install/argocd-mcp-server
```

#### Option B: bin/ Subdirectory Structure

Create a cleaner structure with a `bin/` subdirectory:

```bash
# Create installation directories
mkdir -p /path/to/install/bin

# Copy files
cp argocd_mcp_server.py /path/to/install/
cp target/release/argocd-mcp-server /path/to/install/bin/

# Make executable
chmod +x /path/to/install/argocd_mcp_server.py
chmod +x /path/to/install/bin/argocd-mcp-server
```

### Method 3: System-Wide Installation

Install for all users (requires sudo):

```bash
# Install to /usr/local
sudo ./install.sh /usr/local/lib/argocd-mcp-server

# Create symlink for easy access
sudo ln -s /usr/local/lib/argocd-mcp-server/argocd_mcp_server.py /usr/local/bin/argocd-mcp-server
```

### Method 4: Python Package Installation (Future)

When published to PyPI, you'll be able to install via pip:

```bash
# This will be available in the future
pip install argocd-mcp-server
# or
uvx argocd-mcp-server
```

## Verification

After installation, verify it works:

```bash
# Set required environment variables
export ARGOCD_BASE_URL=https://your-argocd-server.com
export ARGOCD_ACCESS_TOKEN=your-access-token

# Test the installation
python3 /path/to/install/argocd_mcp_server.py
```

You should see MCP protocol messages (JSON-RPC) on stdout if it's working correctly.

## Configuration for MCP Clients

### Claude Desktop / Claude Code

Add to your `.mcp.json` or `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "argocd": {
      "command": "python3",
      "args": ["/absolute/path/to/install/argocd_mcp_server.py"],
      "env": {
        "ARGOCD_BASE_URL": "https://your-argocd-server.com",
        "ARGOCD_ACCESS_TOKEN": "your-access-token",
        "ARGOCD_INSECURE": "true"
      }
    }
  }
}
```

**Important**: Always use absolute paths in MCP configuration!

### MCP Inspector (Testing)

```bash
npx @modelcontextprotocol/inspector python3 /path/to/install/argocd_mcp_server.py
```

## Common Issues

### "ArgoCD MCP Server binary not found"

**Problem**: The Python wrapper can't find the Rust binary.

**Solutions**:
1. Ensure you copied both files (not just the Python wrapper)
2. Check the directory structure matches one of the expected layouts
3. Verify the binary has executable permissions: `chmod +x argocd-mcp-server`
4. Run with `DEBUG_MCP=true` to see where it's searching:
   ```bash
   DEBUG_MCP=true python3 argocd_mcp_server.py
   ```

### Permission Denied

**Problem**: Files aren't executable.

**Solution**:
```bash
chmod +x /path/to/install/argocd_mcp_server.py
chmod +x /path/to/install/bin/argocd-mcp-server  # or wherever the binary is
```

### "ModuleNotFoundError" or Python Import Errors

**Problem**: Missing Python dependencies (unlikely - the wrapper uses only standard library).

**Solution**: Ensure you're using Python 3.8 or later:
```bash
python3 --version
```

### MCP Client Can't Find the Server

**Problem**: Incorrect path in MCP configuration.

**Solutions**:
1. Use absolute paths (not relative: `~/` or `./`)
2. Expand `~` to full path: `/home/username/...`
3. Verify the path exists: `ls -l /path/to/install/argocd_mcp_server.py`

## Platform-Specific Notes

### Linux / macOS

- Recommended installation: `~/.local/bin/argocd-mcp-server/`
- System-wide: `/usr/local/lib/argocd-mcp-server/`

### Windows

- Binary name: `argocd-mcp-server.exe`
- Recommended installation: `%USERPROFILE%\AppData\Local\argocd-mcp-server\`

## Deployment Best Practices

1. **Version Control**: Keep track of which binary version you're deploying
2. **Environment Variables**: Use a secure method to manage `ARGOCD_ACCESS_TOKEN`
3. **Updates**: Re-run installation after rebuilding:
   ```bash
   cargo build --release
   ./install.sh /path/to/install
   ```
4. **Backups**: Keep the old version before updating
5. **Testing**: Test with MCP Inspector before deploying to production

## Troubleshooting Checklist

- [ ] Binary exists and is executable
- [ ] Python wrapper exists and is executable
- [ ] Directory structure matches one of the expected layouts
- [ ] Environment variables are set (`ARGOCD_BASE_URL`, `ARGOCD_ACCESS_TOKEN`)
- [ ] Absolute paths used in MCP configuration
- [ ] Python 3.8+ is installed
- [ ] ArgoCD server is accessible from the installation location

## Advanced: Creating a Portable Archive

To create a portable archive for distribution:

```bash
# Build the release binary
cargo build --release

# Create archive directory
mkdir -p argocd-mcp-server-portable/bin

# Copy files
cp argocd_mcp_server.py argocd-mcp-server-portable/
cp target/release/argocd-mcp-server argocd-mcp-server-portable/bin/
cp README.md argocd-mcp-server-portable/
cp INSTALL.md argocd-mcp-server-portable/

# Create tarball
tar -czf argocd-mcp-server-portable.tar.gz argocd-mcp-server-portable/

# Distribute the tarball
# Users can extract and run:
#   tar -xzf argocd-mcp-server-portable.tar.gz
#   cd argocd-mcp-server-portable
#   python3 argocd_mcp_server.py
```

## Getting Help

If you encounter issues not covered here:

1. Enable debug mode: `DEBUG_MCP=true python3 argocd_mcp_server.py`
2. Check the error messages carefully
3. Verify all prerequisites are met
4. Consult the main README.md for additional troubleshooting
