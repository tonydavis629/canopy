#!/bin/bash

# Canopy Demo Script
# This script demonstrates how to use Canopy with the sample Rust project

echo "🌳 Canopy Demo - Analyzing Sample Rust Project"
echo "=============================================="

# Check if Canopy is built
if [ ! -f "../../target/release/canopy" ]; then
    echo "Building Canopy..."
    cd ../..
    cargo build --release
    cd examples/sample-rust-project
fi

# Set up environment
export RUST_LOG=info
export OPENROUTER_API_KEY=${OPENROUTER_API_KEY:-"your-api-key-here"}

echo ""
echo "📁 Project Structure:"
echo "- src/lib.rs    - User management library"
echo "- src/main.rs   - Application entry point"
echo "- Cargo.toml    - Project configuration"
echo ""

echo "🔍 Code Entities Canopy Will Extract:"
echo "- User struct with fields"
echo "- UserService struct"
echo "- Methods: new(), add_role(), has_role()"
echo "- Functions: create_user(), find_user(), list_users(), update_email()"
echo ""

echo "🤖 AI Will Infer Relationships:"
echo "- create_user() -> User::new() [Calls]"
echo "- find_user() -> users HashMap [Uses]"
echo "- update_email() -> User [Configures]"
echo ""

echo "🚀 Starting Canopy..."
echo "Open http://localhost:7890 in your browser"
echo "Press Ctrl+C to stop"
echo ""

# Run Canopy
../../target/release/canopy serve --path .