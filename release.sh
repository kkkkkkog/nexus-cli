#!/bin/bash
set -e  # Exit on any error

# Version
VERSION="0.10.0"
BINARY_NAME="nexus-network"
TAG="v$VERSION"

# First run the build script
echo "Running build script..."
./build.sh

# Git operations
echo "Creating git tag..."
# Delete existing tag locally and remotely
git tag -d "$TAG" 2>/dev/null || true
git push origin ":refs/tags/$TAG" 2>/dev/null || true

# Remove old files if they exist
git rm -f releases/checksums.txt 2>/dev/null || true
git rm -f releases/$BINARY_NAME-0.9.6-b-linux-x86_64* 2>/dev/null || true

# Add all new files and changes
git add clients/cli/Cargo.toml
git add clients/cli/Cargo.lock
git add clients/cli/.cargo/ 2>/dev/null || true
git add build.sh
git add release.sh
git add releases/$BINARY_NAME-$VERSION-linux-x86_64-static* 2>/dev/null || true
git add releases/checksums-static.txt 2>/dev/null || true

# Commit changes
git commit -m "Release $TAG"
git tag -a "$TAG" -m "Release $TAG"
git push origin main
git push origin "$TAG"

# Delete existing release if it exists
gh release delete "$TAG" --yes 2>/dev/null || true

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