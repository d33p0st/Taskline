#!/bin/bash
# install.sh - Local installation script for development

set -e

echo "🚀 Installing Taskline from source..."

# Install main taskline CLI
echo "📦 Installing taskline (main CLI)..."
cargo install --path . --force

# Install workspace members
echo "📦 Installing taskline-init..."
cargo install --path taskline-init --force

echo "📦 Installing taskline-bump..."
cargo install --path taskline-bump --force

echo "✅ Taskline installation complete!"
echo ""
echo "Available commands:"
echo "  taskline init <filename> [version]  - Initialize a new script"
echo "  taskline bump <filename> [type]     - Bump script version"
echo "  taskline install                    - Install components from crates.io"
echo "  taskline doctor                     - Check installation status"