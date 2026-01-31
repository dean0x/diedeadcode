#!/bin/bash
set -e

# Publish all npm packages locally
# Run `npm login` first!

echo "Publishing platform packages..."
for pkg in cli-darwin-arm64 cli-darwin-x64 cli-linux-x64 cli-linux-arm64 cli-linux-x64-musl cli-linux-arm64-musl cli-win32-x64; do
  echo "Publishing @diedeadcode/$pkg..."
  npm publish npm/$pkg --access public
done

echo ""
echo "Publishing main package..."
npm publish npm/diedeadcode --access public

echo ""
echo "Done! All packages published."
