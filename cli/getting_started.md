# Getting Started

This getting started guide will cover creating a vault, minting some VRT, delegating to an operator, updating the vault and withdrawing VRT.

## Setup

Build and install the CLI

In the root of the repo:

```bash
cargo build --release
cargo install --path ./cli --bin jito-restaking-cli
```

Ensure it has been installed

```bash
jito-restaking-cli --help
```

## Create a Vault

This is a Jito Test Vault, which uses JitoSOL as its supported token.

THIS IS FOR TESTING PURPOSES ONLY
Example Vault: `jkHHVMhQefVuEiFKEyEZgcDZoXv8ZZyjUiK11e61oVY`
Example VRT: `5rN9m6TkyPkzMGVpdmbRVYct1RKa7VssV1AwsHVPFaxJ`

### Initialize Vault

Creating a vault requires:

- `<RPC_URL>`: RPC url
- `<TOKEN_MINT>`: The pubkey of the "supported token" mint
- `<DEPOSIT_FEE_BPS>`: Fee for minting VRT
- `<WITHDRAWAL_FEE_BPS>`: Fee for withdrawing ST
- `<REWARD_FEE_BPS>`: Fee taken when ST rewards are sent to the vault
- `<DECIMALS>`: Decimals of the newly created VRT. ( 9 is Recommended )

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault initialize <TOKEN_MINT> <DEPOSIT_FEE_BPS> <WITHDRAWAL_FEE_BPS> <REWARD_FEE_BPS> <DECIMALS>
```

Note the resulting Vault Pubkey.

### Create VRT Metadata

To create the metadata:

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey
- `<NAME>`: Name for VRT
- `<SYMBOL>`: Symbol for VRT
- `<URI>`: Metadata url

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault create-token-metadata <VAULT> <NAME> <SYMBOL> <URI>
```

## Update a Vault

It is the vault's responsibility to update it once per epoch. If a vault is not updated, no other actions can be taken. This is done by initializing a `vault_update_state_tracker`, cranking it and to finish the update, closing it.

### Initialize Vault Update State Tracker

Starts the update process, this should be the first IX called at the start of an epoch.

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault initialize-vault-update-state-tracker <VAULT>
```

### Crank Vault Update State Tracker

Needs to be called for each operator. If there are no operators, this IX can be skipped. Operators need to be called in order.

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey
- `<OPERATOR>`: The operator that is being updated

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault crank-vault-update-state-tracker <VAULT> <OPERATOR>
```

### Close Vault Update State Tracker

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey
- `[NCN_EPOCH]`: Optional NCN Epoch, for closing older, still-open, vault update state trackers

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault close-vault-update-state-tracker <VAULT> <OPERATOR> [NCN_EPOCH]
```

## Vault Functions

### Mint VRT

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey
- `<AMOUNT_IN>`: In st tokens with no decimals ( lamports instead of SOL )
- `<MIN_AMOUNT_OUT>`: In vrt tokens with no decimals

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault mint-vrt <VAULT> <AMOUNT_IN> <MIN_AMOUNT_OUT>
```

## Create and Delegate to Operator

### Initialize an Operator

Note: This command will output the operator's public key. You will need to save this to use in other commands.

```bash
jito-restaking-cli restaking operator initialize
```

### Initialize Operator Vault Ticket

```bash
jito-restaking-cli restaking operator initialize-operator-vault-ticket OPERATOR VAULT
```

### Warmup Operator Vault Ticket

```bash
jito-restaking-cli restaking operator warmup-operator-vault-ticket OPERATOR VAULT
```

### Initialize Vault Operator Delegation

Note: Since this uses the vault program instead of the restaking program, it requires the vault's key before the operator's key.

```bash
jito-restaking-cli vault vault initialize-operator-delegation VAULT OPERATOR
```

### Delegate to Operator

```bash
jito-restaking-cli vault vault delegate-to-operator VAULT OPERATOR AMOUNT
```

## Withdraw from Vault

### Enqueue withdrawal

```bash
jito-restaking-cli vault vault enqueue-withdrawal VAULT AMOUNT
```

### (Informational Only) Crank Vault Update State Tracker

Note: The update state tracker must exist, and since we closed it to mint the VRT, we don't need to crank it. It's just good to know that it exists.

```bash
jito-restaking-cli vault vault crank-update-state-tracker VAULT OPERATOR
```

### Burn Withdrawal Ticket

Note: you must wait for the cooldown period ( 1 epoch) to pass before you can burn the withdrawal ticket.

```bash
jito-restaking-cli vault vault burn-withdrawal-ticket VAULT MIN_AMOUNT_OUT
```
