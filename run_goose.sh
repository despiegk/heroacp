#!/bin/bash
# HeroACP Run Script for Goose
# Connects the ACP client to the Goose AI agent

set -e

# Change to the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Check if the client binary exists
if [ ! -f "./target/release/acp-client" ]; then
    echo "Error: acp-client not found. Please run ./install.sh first."
    exit 1
fi

# Check if goose is available
if ! command -v goose &> /dev/null; then
    echo "Error: Goose is not installed or not in PATH."
    echo
    echo "To install Goose, visit:"
    echo "  https://block.github.io/goose/"
    echo
    echo "Or use the built-in bogus agent:"
    echo "  ./run.sh"
    exit 1
fi

echo "╔════════════════════════════════════════════╗"
echo "║       HeroACP + Goose Integration          ║"
echo "╚════════════════════════════════════════════╝"
echo
echo "Goose version: $(goose --version 2>/dev/null || echo 'unknown')"
echo

# Run the client with goose
exec ./target/release/acp-client goose
