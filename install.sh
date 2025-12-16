#!/bin/bash
# HeroACP Installation Script
# Builds the ACP server and client binaries

set -e

# Change to the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "╔════════════════════════════════════════════╗"
echo "║       HeroACP Installation Script          ║"
echo "╚════════════════════════════════════════════╝"
echo

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed."
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo

# Build the project
echo "Building HeroACP..."
cargo build --release

echo
echo "Build complete!"
echo
echo "Binaries created:"
echo "  - target/release/acp-server  (Bogus AI Agent)"
echo "  - target/release/acp-client  (ACP Client)"
echo
echo "To run the demo:"
echo "  ./run.sh"
echo
echo "Or manually:"
echo "  ./target/release/acp-client ./target/release/acp-server"
echo
echo "To connect to Goose:"
echo "  ./target/release/acp-client goose"
