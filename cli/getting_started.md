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
Example Operator: `EN7drMzCkZqpuyMVW1QBu8Ciw4Se76KNxNvFZYhDnyUH`

### Initialize Vault

Creating a vault requires:

- `<RPC_URL>`: RPC url
- `<TOKEN_MINT>`: The pubkey of the "supported token" mint
- `<DEPOSIT_FEE_BPS>`: Fee for minting VRT
- `<WITHDRAWAL_FEE_BPS>`: Fee for withdrawing ST
- `<REWARD_FEE_BPS>`: Fee taken when ST rewards are sent to the vault
- `<DECIMALS>`: Decimals of the newly created VRT. ( 9 is Recommended )
- `<INITIALIZE_TOKEN_AMOUNT>`: The amount of tokens to initialize the vault with ( in the smallest unit )
- `<VRT_MINT_ADDRESS_FILE_PATH>`: The file path of VRT mint address (**optional**)

Normal:

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault initialize <TOKEN_MINT> <DEPOSIT_FEE_BPS> <WITHDRAWAL_FEE_BPS> <REWARD_FEE_BPS> <DECIMALS> <INITIALIZE_TOKEN_AMOUNT>
```

With Vanity VRT mint address:

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault initialize <TOKEN_MINT> <DEPOSIT_FEE_BPS> <WITHDRAWAL_FEE_BPS> <REWARD_FEE_BPS> <DECIMALS> <INITIALIZE_TOKEN_AMOUNT> <VRT_MINT_ADDRESS_FILE_PATH>
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

Note: This command will output the operator's public key.

- `<RPC_URL>`: RPC url
- `<OPERATOR_FEE_BPS>`: On-chain operator fee used for external reward calculations

```bash
jito-restaking-cli --rpc-url <RPC_URL> restaking operator initialize <OPERATOR_FEE_BPS>
```

### Initialize Operator Vault Ticket

This ticket is a the operator telling the vault that it's ready to receive delegation.

- `<RPC_URL>`: RPC url
- `<OPERATOR>`: Pubkey of the operator
- `<VAULT>`: Pubkey of the vault

```bash
jito-restaking-cli --rpc-url <RPC_URL> restaking operator initialize-operator-vault-ticket <OPERATOR> <VAULT>
```

### Warmup Operator Vault Ticket

To allow the operator to receive delegation.

- `<RPC_URL>`: RPC url
- `<OPERATOR>`: Pubkey of the operator
- `<VAULT>`: Pubkey of the vault

```bash
jito-restaking-cli --rpc-url <RPC_URL> restaking operator warmup-operator-vault-ticket <OPERATOR> <VAULT>
```

### Initialize Vault Operator Delegation

To complete the handshake, the vault has to allow delegation to the operator.

Note: vault comes first since this is a vault program ix

- `<RPC_URL>`: RPC url
- `<VAULT>`: Pubkey of the vault
- `<OPERATOR>`: Pubkey of the operator

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault initialize-operator-delegation <VAULT> <OPERATOR>
```

### Delegate to Operator

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey
- `<OPERATOR>`: Pubkey of the operator
- `<AMOUNT>`: In st tokens with no decimals

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault delegate-to-operator <VAULT> <OPERATOR> <AMOUNT>
```

### Cooldown Delegation

Undelegating stake requires a cooldown of one epoch, so this IX starts the undelegation process. The funds will be undelegated during the update vault process.

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey
- `<OPERATOR>`: Pubkey of the operator
- `<AMOUNT>`: In st tokens with no decimals

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault cooldown-operator-delegation <VAULT> <OPERATOR> <AMOUNT>
```

## Withdraw from Vault

### Enqueue withdrawal

Withdrawing the supported mint from the vault, involves a full epoch cooldown period and then burning the equivalent VRT. To finish the withdrawal, call `burn-withdrawal-ticket`.

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey
- `<AMOUNT>`: To burn in VRT tokens with no decimals

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault enqueue-withdrawal <VAULT> <AMOUNT>
```

### Burn Withdrawal Ticket

Burn the withdrawal ticket after the cooldown period to complete the withdrawal.

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault burn-withdrawal-ticket <VAULT>
```
