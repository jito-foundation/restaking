#!/bin/bash
set -e

# Get script directory and project root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

# First remove if exists
rm -rf "$PROJECT_ROOT/packages/restaking-sdk/src"

# Then create fresh directory
mkdir -p "$PROJECT_ROOT/packages/restaking-sdk/src"

# Then copy files
cp -r "$PROJECT_ROOT/clients/js/restaking_client/"* "$PROJECT_ROOT/packages/restaking-sdk/src/"

echo "✅ Prepared restaking SDK" 