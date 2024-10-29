# Jito Restaking Cranker

## Overview and Purpose

The Jito Restaking Cranker is a utility that must be run at the end of every epoch to call the `process_crank_vault_update_state_tracker` instruction. This ensures that vault states are updated, operator delegations are processed. By regularly running this cranker, you ensure that the vaults, operators, and delegation states stay up to date, especially in cases where the allocation of additional assets and restaking logic require periodic updates.


## Getting Started

### Options

- RPC URL
- Keypair: The path to your keypair
- Vault Program Id: The program ID of Jito Vault Program
- Restaking Program Id: The program ID of Jito Restaking Program

### Run the cranker

To run the cranker with a specific vault and restaking program:

```bash
cargo run -p jito-restaking-cranker -- \
  -k ~/.config/solana/mykeypair.json \
  --vault-program-id "34X2uqBhEGiWHu43RDEMwrMqXF4CpCPEZNaKdAaUS9jx" \
  --restaking-program-id "78J8YzXGGNynLRpn85MH77PVLBZsWyLCHZAXRvKaB6Ng"
```

