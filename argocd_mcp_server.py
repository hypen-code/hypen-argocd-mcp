#!/usr/bin/env python3
"""
Python wrapper for ArgoCD MCP Server (Rust binary)

This wrapper allows the Rust-based MCP server to be launched using Python,
making it compatible with MCP frameworks that require standard executables.

The wrapper:
- Spawns the Rust binary as a child process
- Pipes stdin/stdout/stderr between the parent and child
- Forwards all environment variables
- Handles process termination gracefully
- Adds minimal overhead (~1-2ms startup time)
"""

import os
import sys
import subprocess
import platform
from pathlib import Path


def get_binary_path():
    """Determine the binary path based on platform."""
    script_dir = Path(__file__).parent.absolute()

    # Binary name varies by platform
    if platform.system() == 'Windows':
        binary_name = 'argocd-mcp-server.exe'
    else:
        binary_name = 'argocd-mcp-server'

    # Try different possible locations
    possible_paths = [
        # Development: target/release directory
        script_dir / 'target' / 'release' / binary_name,
        # Installed via pip: bin directory
        script_dir / 'bin' / binary_name,
        # Alternative: same directory as this script
        script_dir / binary_name,
    ]

    # Find the first path that exists
    for bin_path in possible_paths:
        if bin_path.exists():
            return str(bin_path)

    # If not found, provide helpful error message
    print('Error: ArgoCD MCP Server binary not found.', file=sys.stderr)
    print('Searched in:', file=sys.stderr)
    for p in possible_paths:
        print(f'  - {p}', file=sys.stderr)
    print('\nPlease build the binary first:', file=sys.stderr)
    print('  cargo build --release', file=sys.stderr)
    sys.exit(1)


def main():
    """Main execution."""
    binary_path = get_binary_path()

    # Debug: Log if we're missing required environment variables (only to stderr)
    debug_mode = os.environ.get('DEBUG_MCP', 'false').lower() == 'true'
    if debug_mode:
        print('=== MCP Wrapper Debug Info (Python) ===', file=sys.stderr)
        print(f'Binary path: {binary_path}', file=sys.stderr)
        print(f"ARGOCD_BASE_URL: {'SET' if os.environ.get('ARGOCD_BASE_URL') else 'NOT SET'}", file=sys.stderr)
        print(f"ARGOCD_ACCESS_TOKEN: {'SET' if os.environ.get('ARGOCD_ACCESS_TOKEN') else 'NOT SET'}", file=sys.stderr)
        print(f'Python version: {sys.version}', file=sys.stderr)
        print(f'Platform: {platform.system()}', file=sys.stderr)
        print('========================================', file=sys.stderr)

    # Spawn the Rust binary as a child process
    # Pass through all arguments and environment variables
    # Use stdin/stdout/stderr directly for MCP communication
    try:
        # Pass all command-line arguments (excluding script name)
        args = [binary_path] + sys.argv[1:]

        # Execute the binary, passing through stdin/stdout/stderr and environment
        # This is crucial for MCP protocol which uses stdio for communication
        result = subprocess.run(
            args,
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
            env=os.environ.copy(),
        )

        # Exit with the same code as the child process
        sys.exit(result.returncode)

    except KeyboardInterrupt:
        # Handle Ctrl+C gracefully
        sys.exit(130)
    except Exception as e:
        print(f'Failed to start ArgoCD MCP Server: {e}', file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
