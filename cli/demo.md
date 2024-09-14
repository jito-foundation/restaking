# Jito CLI Demo

This demo provides all of the commands to make a vault, mint some VRT, create and delegate to an operator, and withdraw from the vault.

## Setup

Build and install the CLI

```bash
cargo build --release
cargo install --path ./cli --bin jito-restaking-cli
```

Ensure it has been installed

```bash
jito-restaking-cli --help
```

## Create a Vault

Sample Account Addresses:
MINT = `So11111111111111111111111111111111111111112`
VAULT = `9FfXNgFuhoiR84T2ithM5cYsLeBeomnrdaoXjiwq5Czh`
OPERATOR = `42SRQttacU14PjEoWo2ZmRws7snKoFYwu4vXV6pPR1SQ`

### Initialize Vault

For this demo, it is recommended to use wSOL as the base asset mint: `So11111111111111111111111111111111111111112`.

You can wrap your own SOL using:

```bash
spl-token wrap AMOUNT
```

Note: This command will output the vault's public key. You will need to save this to use in other commands.

```bash
jito-restaking-cli vault vault initialize MINT DEPOSIT_FEE_BPS WITHDRAWAL_FEE_BPS REWARD_FEE_BPS
```

### Initialize Update State Tracker

Note: Initializing and closing the update state tracker effectively "cranks" the state. This needs to be done before any VRT can be minted.

```bash
jito-restaking-cli vault vault initialize-update-state-tracker VAULT
```

### Close Vault Update State Tracker

```bash
jito-restaking-cli vault vault close-update-state-tracker VAULT
```

### Mint VRT

```bash
jito-restaking-cli vault vault mint-vrt VAULT AMOUNT_IN MIN_AMOUNT_OUT
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

## Printing Accounts

### Get Vault

```bash
jito-restaking-cli vault vault get VAULT
```

### Get State Tracker

```bash
jito-restaking-cli vault vault get-state-tracker VAULT NCN_EPOCH
```

### Get Operator Delegation

```bash
jito-restaking-cli vault vault get-operator-delegation VAULT OPERATOR
```

### Get Withdrawal Ticket

```bash
jito-restaking-cli vault vault get-withdrawal-ticket VAULT
```
