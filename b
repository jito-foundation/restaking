#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

# Configuration
CONFIG_DIR="${SCRIPT_DIR}/.config"
VALID_NETWORKS=("mainnet-beta" "testnet" "localhost" "devnet")

# Function to print usage
print_usage() {
    echo "Usage: $(basename "$0") --network <network> [additional cargo build-sbf arguments]"
    echo
    echo "Build Solana program with network-specific configurations"
    echo
    echo "Options:"
    echo "  --network <network>     Network to build for (${VALID_NETWORKS[*]})"
    echo "  [additional arguments]  Additional arguments passed to cargo build-sbf"
    echo
    echo "Examples:"
    echo "  $(basename "$0") --network mainnet-beta"
    echo "  $(basename "$0") --network testnet -- --release"
    echo
    echo "Environment:"
    echo "  Config files should be placed in ${CONFIG_DIR}/{network}"
    echo "  Required variables: RESTAKING_PROGRAM_ID, VAULT_PROGRAM_ID"
}

# Function to log messages
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Parse command line arguments
NETWORK=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --network)
            NETWORK="$2"
            shift 2
            ;;
        --help)
            print_usage
            exit 0
            ;;
        --)
            shift
            break
            ;;
        *)
            log "Error: Unknown option $1"
            print_usage
            exit 1
            ;;
    esac
done

# Check if network argument is provided
if [ -z "$NETWORK" ]; then
    log "Error: --network option is required"
    print_usage
    exit 1
fi

# Validate network
if [[ ! " ${VALID_NETWORKS[*]} " =~ " ${NETWORK} " ]]; then
    log "Error: Invalid network specified: ${NETWORK}"
    print_usage
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

# Check if required variables are set
if [ -z "${RESTAKING_PROGRAM_ID:-}" ] || [ -z "${VAULT_PROGRAM_ID:-}" ]; then
    log "Error: RESTAKING_PROGRAM_ID or VAULT_PROGRAM_ID not set in ${ENV_FILE}"
    exit 1
fi

# Log the variables
log "Using RESTAKING_PROGRAM_ID: ${RESTAKING_PROGRAM_ID}"
log "Using VAULT_PROGRAM_ID: ${VAULT_PROGRAM_ID}"

# Run cargo build-sbf with the environment variables set and pass any additional arguments
log "Running cargo build-sbf for ${NETWORK}..."
cargo build-sbf --features "$NETWORK" "$@"

log "Build completed successfully!"