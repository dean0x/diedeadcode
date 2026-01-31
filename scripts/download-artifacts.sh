#!/bin/bash
set -e

# Download build artifacts from the latest release workflow run
# Usage: ./scripts/download-artifacts.sh [run-id]

RUN_ID=${1:-$(gh run list --workflow=release.yml --limit=1 --json databaseId --jq '.[0].databaseId')}

if [ -z "$RUN_ID" ]; then
  echo "No release workflow runs found"
  exit 1
fi

echo "Downloading artifacts from run $RUN_ID..."

mkdir -p artifacts
cd artifacts

# Download all artifacts
gh run download "$RUN_ID"

# Flatten directory structure (gh downloads each artifact to its own folder)
for dir in */; do
  if [ -d "$dir" ]; then
    mv "$dir"* . 2>/dev/null || true
    rmdir "$dir" 2>/dev/null || true
  fi
done

echo ""
echo "Downloaded artifacts:"
ls -la

echo ""
echo "Next steps:"
echo "  1. npm login"
echo "  2. node scripts/generate-npm-packages.mjs <version> artifacts"
echo "  3. ./scripts/publish-npm.sh"
