#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

CONFIG_DIR="${SCRIPT_DIR}/.config"
VALID_NETWORKS=("mainnet-beta" "testnet" "localhost" "devnet")

# Function to print usage
print_usage() {
    echo "Usage: $(basename "$0") --network <network> [--bpf]"
    echo
    echo "Run tests for the Solana program"
    echo
    echo "Options:"
    echo "  --network <network>  Network to run tests on (${VALID_NETWORKS[*]})"
    echo "  --bpf                Build BPF and run tests with SBF_OUT_DIR set"
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
NETWORK=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --network)
            NETWORK="$2"
            shift 2
            ;;
        --bpf)
            BPF_MODE=true
            shift
            ;;
        --help)
            print_usage
            exit 0
            ;;
        *)
            log "Error: Unknown option $1"
            print_usage
            exit 1
            ;;
    esac
done

# Validate network
if [[ -z "$NETWORK" ]]; then
    log "Error: --network option is required"
    print_usage
    exit 1
fi

if [[ ! " ${VALID_NETWORKS[*]} " =~ " ${NETWORK} " ]]; then
    log "Error: Invalid network '${NETWORK}'. Valid networks are: ${VALID_NETWORKS[*]}"
    exit 1
fi

# Set the path to the config file
ENV_FILE="${CONFIG_DIR}/${NETWORK}"

# Check if the env file exists
if [ ! -f "$ENV_FILE" ]; then
    log "Error: Environment file not found: ${ENV_FILE}"
    exit 1
fi

# Load environment variables from the file
set -a
source "$ENV_FILE"
set +a

if [ "$BPF_MODE" = true ]; then
    log "Building BPF for ${NETWORK}..."
    cargo build-sbf

    log "Running tests with SBF_OUT_DIR set for ${NETWORK}..."
    SBF_OUT_DIR=${SCRIPT_DIR}/target/sbf-solana-solana/release cargo nextest run
else
    log "Running cargo nextest with all features for ${NETWORK}..."
    cargo nextest run --all-features
fi