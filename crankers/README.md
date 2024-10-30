# Jito Vault Cranker

## Overview and Purpose

The Jito Vault Cranker is a permissionless utility that must be run at the beginning of every epoch to call a series of instructions to update the vault state, for every vault. This ensures that vault states are updated, and operator delegations are processed. By regularly running this cranker, you ensure that the vaults, operators, and delegation states stay up to date, especially in cases where the allocation of additional assets and restaking logic require periodic updates.

## Getting Started

### Options

- RPC URL: The RPC endpoint URL
- Keypair: The path to your keypair file used to pay for transactions
- Vault Program Id: The program ID of Jito Vault Program
- Restaking Program Id: The program ID of Jito Restaking Program
- Crank Interval: Time in seconds between cranking attempts (default: 300)
- Metrics Interval: Time in seconds between metrics emission (default: 300)
- Priority Fees: Priority fees in microlamports per compute unit (default: 10000)

### Run locally

To run the cranker with a specific vault and restaking program:

```bash
cargo run -p jito-vault-cranker -- \
  --keypair <KEYPAIR_PATH> \
  --rpc-url <RPC_URL>
  --vault-program-id "Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8" \
  --restaking-program-id "RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q"
  --crank-interval 300
  --priority-fees 10000
```

### Running with Docker

0. Start in the top level `restaking/` directory.

1. Create a `credentials` directory and add your keypair file `keypair.json` to it.
```bash
mkdir credentials
# Add your keypair file here
```

2. Create a .env file with your configuration:
```bash
KEYPAIR_PATH=./credentials/keypair.json
RPC_URL=https://your-rpc-url
VAULT_PROGRAM_ID=Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8
RESTAKING_PROGRAM_ID=RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q
CRANK_INTERVAL=300
METRICS_INTERVAL=300
PRIORITY_FEES=10000
SOLANA_METRICS_CONFIG=<METRICS_DB_URL> # Optional
```

3. Run with docker-compose:
```bash
docker compose --env-file .env up -d
```

4. View logs:
```bash
docker logs jito-vault-cranker -f
```

### Stop and Rebuild

To stop, remove, and rebuild the container:
```bash
docker stop jito-vault-cranker
docker rm jito-vault-cranker; docker rmi restaking-jito-vault-cranker; docker compose --env-file .env up -d --build
```
