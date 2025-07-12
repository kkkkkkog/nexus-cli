#!/bin/bash
set -e  # Exit on any error

# Version
VERSION="0.9.6-b"
BINARY_NAME="nexus-network"
TAG="v$VERSION"

# First run the build script
echo "Running build script..."
./build.sh

# Git operations
echo "Creating git tag..."
# Add all new files and changes
git add clients/cli/Cargo.toml
git add clients/cli/Cargo.lock
git add clients/cli/.cargo/
git add build.sh
git add release.sh
git add releases/$BINARY_NAME-$VERSION-linux-x86_64-static*
git add releases/checksums-static.txt

# Remove old files
git rm -f releases/checksums.txt || true
git rm -f releases/$BINARY_NAME-0.9.6-a-linux-x86_64* || true

# Commit changes
git commit -m "Release $TAG"
git tag -a "$TAG" -m "Release $TAG"
git push origin main
git push origin "$TAG"

# Create GitHub release
echo "Creating GitHub release..."
gh release create "$TAG" \
  --title "Release $TAG" \
  --notes "Release $TAG" \
  releases/$BINARY_NAME-$VERSION-linux-x86_64-static \
  releases/$BINARY_NAME-$VERSION-linux-x86_64-static.tar.gz \
  releases/$BINARY_NAME-$VERSION-linux-x86_64-static.zip \
  releases/checksums-static.txt

echo "Release $TAG created successfully!" 