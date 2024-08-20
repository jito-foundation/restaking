#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

# Configuration
RESTAKING_PROGRAM_ID=Cnp3S1G3JaaAoFpFR3FhM94DdEF1mMBqy5vjwtDaVZLi
VAULT_PROGRAM_ID=94sqeuUnjJCmwZUp8nGK7Q7o8YnHSW9Y3TcPZJY2PycN
export RESTAKING_PROGRAM_ID
export VAULT_PROGRAM_ID

# Function to print usage
print_usage() {
    echo "Usage: $(basename "$0") [--bpf]"
    echo
    echo "Run tests for the Solana program"
    echo
    echo "Options:"
    echo "  --bpf    Build BPF and run tests with SBF_OUT_DIR set"
    echo
    echo "Without --bpf, this script will run cargo nextest with all features."
    echo "With --bpf, it will first build with cargo-build-sbf, then run tests with SBF_OUT_DIR set."
}

# Function to log messages
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Parse command line arguments
BPF_MODE=false
if [ $# -gt 0 ]; then
    if [ "$1" == "--bpf" ]; then
        BPF_MODE=true
    elif [ "$1" == "--help" ]; then
        print_usage
        exit 0
    else
        log "Error: Unknown option $1"
        print_usage
        exit 1
    fi
fi

if [ "$BPF_MODE" = true ]; then
    log "Building BPF..."
    cargo build-sbf

    log "Running tests with SBF_OUT_DIR set..."
    SBF_OUT_DIR=${SCRIPT_DIR}/target/sbf-solana-solana/release cargo nextest run
else
    log "Running cargo nextest with all features..."
    cargo nextest run --all-features
fi