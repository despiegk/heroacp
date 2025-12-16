#!/bin/bash
# HeroACP Run Script
# Runs the ACP client with the specified agent

set -e

# Change to the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Default to the built-in bogus server
AGENT="${1:-./target/release/acp-server}"

# Check if the binaries exist
if [ ! -f "./target/release/acp-client" ]; then
    echo "Error: acp-client not found. Please run ./install.sh first."
    exit 1
fi

# If using the default server, check if it exists
if [ "$AGENT" = "./target/release/acp-server" ] && [ ! -f "$AGENT" ]; then
    echo "Error: acp-server not found. Please run ./install.sh first."
    exit 1
fi

echo "╔════════════════════════════════════════════╗"
echo "║            HeroACP Demo                    ║"
echo "╚════════════════════════════════════════════╝"
echo
echo "Starting client with agent: $AGENT"
echo

# Run the client
exec ./target/release/acp-client "$AGENT"
