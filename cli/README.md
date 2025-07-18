# Jito Restaking CLI

## Overview

The Jito Restaking CLI is a command-line tool that provides access to Jito's Restaking protocol.
This tool enables users to interact with the Jito Restaking protocol, allowing for vault management, operator delegation, and so on.

With this CLI, you can:
- NCN, Operator, Vault Operation
- Mint and burn VRT
- Monitor account statuses with JSON output support
- Preview transactions before sending them with the `print-tx` flag
- Connect and sign transactions with Ledger hardware wallets

## Features 

### JSON Output

The CLI supports JSON output for easier parsing and integration with other tools.
To enable JSON output, use `--print-json` flag with command like `get`, `list`.
This is especially useful for scripting and automation.

#### JSON Output Options:

- `--print-json`: Outputs full account information in JSON format while automatically filtering out any `reserved` fields
- `--print-json-with-reserves`: Outputs account information in JSON format with `reserved` fields

```bash
jito-restaking-cli --rpc-url <RPC_URL> restaking operator get <OPERATOR_ADDRESS> --print-json
```

Example JSON output:

```json
{
  "address": "MBAN56N5NjttLmDF5QFgZPwRxjTCS7tUA5ToZoLekJp",
  "data": {
    "discriminator": 3,
    "base": "7s2yLogXkNSR837HHGJcHohFuDSaDhDbpDvQxYsYMX1C",
    "admin": "aaaDerwdMyzNkoX1aSoTi3UtFe2W45vh5wCgQNhsjF8",
    "ncn_admin": "aaaDerwdMyzNkoX1aSoTi3UtFe2W45vh5wCgQNhsjF8",
    "vault_admin": "aaaDerwdMyzNkoX1aSoTi3UtFe2W45vh5wCgQNhsjF8",
    "delegate_admin": "aaaDerwdMyzNkoX1aSoTi3UtFe2W45vh5wCgQNhsjF8",
    "metadata_admin": "aaaDerwdMyzNkoX1aSoTi3UtFe2W45vh5wCgQNhsjF8",
    "voter": "aaaDerwdMyzNkoX1aSoTi3UtFe2W45vh5wCgQNhsjF8",
    "index": 13,
    "ncn_count": 1,
    "vault_count": 2,
    "operator_fee_bps": 100,
    "bump": 252
  }
}
```

```bash
jito-restaking-cli --rpc-url <RPC_URL> restaking operator get <OPERATOR_ADDRESS> --print-json-with-reserves
```

Example JSON output:

```json
{
  "discriminator": 3,
  "base": "7s2yLogXkNSR837HHGJcHohFuDSaDhDbpDvQxYsYMX1C",
  "admin": "aaaDerwdMyzNkoX1aSoTi3UtFe2W45vh5wCgQNhsjF8",
  "ncn_admin": "aaaDerwdMyzNkoX1aSoTi3UtFe2W45vh5wCgQNhsjF8",
  "vault_admin": "aaaDerwdMyzNkoX1aSoTi3UtFe2W45vh5wCgQNhsjF8",
  "delegate_admin": "aaaDerwdMyzNkoX1aSoTi3UtFe2W45vh5wCgQNhsjF8",
  "metadata_admin": "aaaDerwdMyzNkoX1aSoTi3UtFe2W45vh5wCgQNhsjF8",
  "voter": "aaaDerwdMyzNkoX1aSoTi3UtFe2W45vh5wCgQNhsjF8",
  "index": 13,
  "ncn_count": 1,
  "vault_count": 2,
  "operator_fee_bps": 100,
  "bump": 252,
  "reserved_space": [
    0,
    ...
    0
  ]
}
```

JSON output can be piped to tools like `jq` for further processing:

```bash
jito-restaking-cli --rpc-url <RPC_URL> restaking operator get <OPERATOR_ADDRESS> --print-json | jq
```

### Transaction Inspection

You can preview transactions before sending them using the --print-tx flag.

```bash
jito-restaking-cli --rpc-url <RPC_URL> restaking ncn initialize --print-tx
```

Example output:

```bash
    ------ IX ------

RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q

4vvKh3Ws4vGzgXRVdo8SdL4jePXDvCqKVmi21BCBGwvn  W
6PPdbsLZUyAxPXQ4PWXtZWfnQFQ4w3iWcu1E9AL1PpnG  W
2V6Abua9BY6Ga8HUeLWSLXh4Gm6oKsn3GpTzP4eYMFqT  W S
G4iZadrtSPkGWXwF6SKGNQaS6JW4Zu4tgURM1AFXeGV     S
11111111111111111111111111111111


2
```

When using this flag, the transaction will not be processed - only printed for inspection.
Note that instruction data shown in the output is **base58** encoded, which provides a compact text representation of binary data.

### Ledger Hardware Wallet Support

The CLI now supports integration with Ledger hardware wallets for enhanced security when signing transactions.
This allows you to keep your private keys secure on your hardware device instead of storing them as local keypair files.

#### Using Ledger for Signing

You can specify a Ledger device as the signer by using the `usb://ledger?key=0` in any command that accepts keypair arguments.

```bash
--signer "usb://ledger?key=0"
```

When you specify a Ledger path, the CLI will automatically connect to your Ledger device and prompt you to confirm the transaction on the device.

##### Set Admin

```bash
jito-restaking-cli vault vault set-admin --old-admin-keypair <OLD_ADMIN_KEYPAIR> --new-admin-keypair "usb://ledger?key=0" <VAULT>
```

##### Using Ledger with Other Commands

You can use your Ledger device with any command that accepts a signer argument. The CLI will automatically handle the connection and signing process with your Ledger device.

```bash
jito-restaking-cli -- vault vault set-secondary-admin <VAULT> <SECONDARY_ADMIN> --set-metadata-admin --signer "usb://ledger?key=0"
```

## Getting Started

This getting started guide will cover creating a vault, minting some VRT, delegating to an operator, updating the vault and withdrawing VRT.

### Setup

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

### Create a Vault

This is a Jito Test Vault, which uses JitoSOL as its supported token.

THIS IS FOR TESTING PURPOSES ONLY
Example Vault: `jkHHVMhQefVuEiFKEyEZgcDZoXv8ZZyjUiK11e61oVY`
Example VRT: `5rN9m6TkyPkzMGVpdmbRVYct1RKa7VssV1AwsHVPFaxJ`
Example Operator: `EN7drMzCkZqpuyMVW1QBu8Ciw4Se76KNxNvFZYhDnyUH`

#### Initialize Vault

Creating a vault requires:

- `<RPC_URL>`: RPC url
- `<TOKEN_MINT>`: The pubkey of the "supported token" mint
- `<DEPOSIT_FEE_BPS>`: Fee for minting VRT
- `<WITHDRAWAL_FEE_BPS>`: Fee for withdrawing ST
- `<REWARD_FEE_BPS>`: Fee taken when ST rewards are sent to the vault
- `<DECIMALS>`: Decimals of the newly created VRT. ( 9 is Recommended )
- `<INITIALIZE_TOKEN_AMOUNT>`: The amount of tokens to initialize the vault with ( in the smallest unit )
- `<VRT_MINT_ADDRESS_FILE_PATH>`: The file path of VRT mint address (**Optional**)

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault initialize <TOKEN_MINT> <DEPOSIT_FEE_BPS> <WITHDRAWAL_FEE_BPS> <REWARD_FEE_BPS> <DECIMALS> <INITIALIZE_TOKEN_AMOUNT>
```

Note the resulting Vault Pubkey.

#### Create VRT Metadata

To create the metadata:

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey
- `<NAME>`: Name for VRT
- `<SYMBOL>`: Symbol for VRT
- `<URI>`: Metadata url

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault create-token-metadata <VAULT> <NAME> <SYMBOL> <URI>
```

#### Update VRT Metadata

To update the metadata:

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey
- `<NAME>`: Name for VRT
- `<SYMBOL>`: Symbol for VRT
- `<URI>`: Metadata url

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault update-token-metadata <VAULT> <NAME> <SYMBOL> <URI>
```

### Update a Vault

It is the vault's responsibility to update it once per epoch. If a vault is not updated, no other actions can be taken. This is done by initializing a `vault_update_state_tracker`, cranking it and to finish the update, closing it.

#### Initialize Vault Update State Tracker

Starts the update process, this should be the first IX called at the start of an epoch.

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault initialize-vault-update-state-tracker <VAULT>
```

#### Crank Vault Update State Tracker

Needs to be called for each operator. If there are no operators, this IX can be skipped. Operators need to be called in order.

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey
- `<OPERATOR>`: The operator that is being updated

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault crank-vault-update-state-tracker <VAULT> <OPERATOR>
```

#### Close Vault Update State Tracker

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey
- `[NCN_EPOCH]`: Optional NCN Epoch, for closing older, still-open, vault update state trackers

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault close-vault-update-state-tracker <VAULT> <OPERATOR> [NCN_EPOCH]
```

### Vault Functions

#### Mint VRT

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey
- `<AMOUNT_IN>`: In st tokens with no decimals ( lamports instead of SOL )
- `<MIN_AMOUNT_OUT>`: In vrt tokens with no decimals

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault mint-vrt <VAULT> <AMOUNT_IN> <MIN_AMOUNT_OUT>
```

### Create and Delegate to Operator

#### Initialize an Operator

Note: This command will output the operator's public key.

- `<RPC_URL>`: RPC url
- `<OPERATOR_FEE_BPS>`: On-chain operator fee used for external reward calculations

```bash
jito-restaking-cli --rpc-url <RPC_URL> restaking operator initialize <OPERATOR_FEE_BPS>
```

#### Initialize Operator Vault Ticket

This ticket is a the operator telling the vault that it's ready to receive delegation.

- `<RPC_URL>`: RPC url
- `<OPERATOR>`: Pubkey of the operator
- `<VAULT>`: Pubkey of the vault

```bash
jito-restaking-cli --rpc-url <RPC_URL> restaking operator initialize-operator-vault-ticket <OPERATOR> <VAULT>
```

#### Warmup Operator Vault Ticket

To allow the operator to receive delegation.

- `<RPC_URL>`: RPC url
- `<OPERATOR>`: Pubkey of the operator
- `<VAULT>`: Pubkey of the vault

```bash
jito-restaking-cli --rpc-url <RPC_URL> restaking operator warmup-operator-vault-ticket <OPERATOR> <VAULT>
```

#### Initialize Vault Operator Delegation

To complete the handshake, the vault has to allow delegation to the operator.

Note: vault comes first since this is a vault program ix

- `<RPC_URL>`: RPC url
- `<VAULT>`: Pubkey of the vault
- `<OPERATOR>`: Pubkey of the operator

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault initialize-operator-delegation <VAULT> <OPERATOR>
```

#### Delegate to Operator

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey
- `<OPERATOR>`: Pubkey of the operator
- `<AMOUNT>`: In st tokens with no decimals

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault delegate-to-operator <VAULT> <OPERATOR> <AMOUNT>
```

#### Cooldown Delegation

Undelegating stake requires a cooldown of one epoch, so this IX starts the undelegation process. The funds will be undelegated during the update vault process.

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey
- `<OPERATOR>`: Pubkey of the operator
- `<AMOUNT>`: In st tokens with no decimals

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault cooldown-operator-delegation <VAULT> <OPERATOR> <AMOUNT>
```

### Withdraw from Vault

#### Enqueue withdrawal

Withdrawing the supported mint from the vault, involves a full epoch cooldown period and then burning the equivalent VRT. To finish the withdrawal, call `burn-withdrawal-ticket`.

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey
- `<AMOUNT>`: To burn in VRT tokens with no decimals

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault enqueue-withdrawal <VAULT> <AMOUNT>
```

#### Burn Withdrawal Ticket

Burn the withdrawal ticket after the cooldown period to complete the withdrawal.

- `<RPC_URL>`: RPC url
- `<VAULT>`: The vault Pubkey

```bash
jito-restaking-cli --rpc-url <RPC_URL> vault vault burn-withdrawal-ticket <VAULT>
```
