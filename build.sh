#!/bin/bash
set -e  # Exit on any error

# Version
VERSION="0.9.7"
BINARY_NAME="nexus-network"

# Install dependencies
rustup target add x86_64-unknown-linux-musl
apt-get update && apt-get install -y musl-tools

# Build static binary
echo "Building static binary..."
cd clients/cli
RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --features build_proto --target x86_64-unknown-linux-musl
cd ../..

# Create releases directory
mkdir -p releases

# Copy binary and create archives
echo "Creating release artifacts..."
cp clients/cli/target/x86_64-unknown-linux-musl/release/$BINARY_NAME releases/$BINARY_NAME-$VERSION-linux-x86_64-static

# Generate archives
cd releases
tar -czf $BINARY_NAME-$VERSION-linux-x86_64-static.tar.gz $BINARY_NAME-$VERSION-linux-x86_64-static
zip $BINARY_NAME-$VERSION-linux-x86_64-static.zip $BINARY_NAME-$VERSION-linux-x86_64-static

# Generate checksums
sha256sum $BINARY_NAME-$VERSION-linux-x86_64-static* > checksums-static.txt

echo "Build complete! Artifacts are in the releases directory."
echo "Verifying static binary..."
ldd $BINARY_NAME-$VERSION-linux-x86_64-static || echo "Binary is static (this is good!)" 