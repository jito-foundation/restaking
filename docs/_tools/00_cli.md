---
title: CLI
category: Jekyll
layout: post
weight: 1
---

# Command-Line Help for `jito-restaking-cli`

This document contains the help content for the `jito-restaking-cli` command-line program.

## `jito-restaking-cli`

A CLI for managing restaking and vault operations

**Usage:** `jito-restaking-cli [OPTIONS] [COMMAND]`

###### **Subcommands:**

* `restaking` — Restaking program commands
* `vault` — Vault program commands

###### **Options:**

* `--config-file <CONFIG_FILE>` — Path to the configuration file
* `--rpc-url <RPC_URL>` — RPC URL to use
* `--commitment <COMMITMENT>` — Commitment level
* `--restaking-program-id <RESTAKING_PROGRAM_ID>` — Restaking program ID
* `--vault-program-id <VAULT_PROGRAM_ID>` — Vault program ID
* `--keypair <KEYPAIR>` — Keypair
* `--verbose` — Verbose mode



## `jito-restaking-cli restaking`

Restaking program commands

**Usage:** `jito-restaking-cli restaking <COMMAND>`

###### **Subcommands:**

* `config` — Initialize, get, and set the config struct
* `ncn` — 
* `operator` — 



## `jito-restaking-cli restaking config`

Initialize, get, and set the config struct

**Usage:** `jito-restaking-cli restaking config <COMMAND>`

###### **Subcommands:**

* `initialize` — Initialize the config
* `get` — Get the config
* `set-admin` — Set the config admin



## `jito-restaking-cli restaking config initialize`

Initialize the config

**Usage:** `jito-restaking-cli restaking config initialize`



## `jito-restaking-cli restaking config get`

Get the config

**Usage:** `jito-restaking-cli restaking config get`



## `jito-restaking-cli restaking config set-admin`

Set the config admin

**Usage:** `jito-restaking-cli restaking config set-admin <NEW_ADMIN>`

###### **Arguments:**

* `<NEW_ADMIN>` — The new admin's pubkey



## `jito-restaking-cli restaking ncn`

**Usage:** `jito-restaking-cli restaking ncn <COMMAND>`

###### **Subcommands:**

* `initialize` — Initialize NCN
* `get` — Get NCN
* `list` — List all NCNs



## `jito-restaking-cli restaking ncn initialize`

Initialize NCN

**Usage:** `jito-restaking-cli restaking ncn initialize`



## `jito-restaking-cli restaking ncn get`

Get NCN

**Usage:** `jito-restaking-cli restaking ncn get <PUBKEY>`

###### **Arguments:**

* `<PUBKEY>`



## `jito-restaking-cli restaking ncn list`

List all NCNs

**Usage:** `jito-restaking-cli restaking ncn list`



## `jito-restaking-cli restaking operator`

**Usage:** `jito-restaking-cli restaking operator <COMMAND>`

###### **Subcommands:**

* `initialize` — Initialize Operator
* `initialize-operator-vault-ticket` — Initialize Operator Vault Ticket
* `warmup-operator-vault-ticket` — Warmup Operator Vault Ticket
* `get` — Get operator
* `list` — List all operators



## `jito-restaking-cli restaking operator initialize`

Initialize Operator

**Usage:** `jito-restaking-cli restaking operator initialize <OPERATOR_FEE_BPS>`

###### **Arguments:**

* `<OPERATOR_FEE_BPS>`



## `jito-restaking-cli restaking operator initialize-operator-vault-ticket`

Initialize Operator Vault Ticket

**Usage:** `jito-restaking-cli restaking operator initialize-operator-vault-ticket <OPERATOR> <VAULT>`

###### **Arguments:**

* `<OPERATOR>`
* `<VAULT>`



## `jito-restaking-cli restaking operator warmup-operator-vault-ticket`

Warmup Operator Vault Ticket

**Usage:** `jito-restaking-cli restaking operator warmup-operator-vault-ticket <OPERATOR> <VAULT>`

###### **Arguments:**

* `<OPERATOR>`
* `<VAULT>`



## `jito-restaking-cli restaking operator get`

Get operator

**Usage:** `jito-restaking-cli restaking operator get <PUBKEY>`

###### **Arguments:**

* `<PUBKEY>`



## `jito-restaking-cli restaking operator list`

List all operators

**Usage:** `jito-restaking-cli restaking operator list`



## `jito-restaking-cli vault`

Vault program commands

**Usage:** `jito-restaking-cli vault <COMMAND>`

###### **Subcommands:**

* `config` — 
* `vault` — Vault commands



## `jito-restaking-cli vault config`

**Usage:** `jito-restaking-cli vault config <COMMAND>`

###### **Subcommands:**

* `initialize` — Creates global config (can only be done once)
* `get` — Fetches global config
* `set-admin` — Set the config admin



## `jito-restaking-cli vault config initialize`

Creates global config (can only be done once)

**Usage:** `jito-restaking-cli vault config initialize <PROGRAM_FEE_BPS> <PROGRAM_FEE_WALLET>`

###### **Arguments:**

* `<PROGRAM_FEE_BPS>` — The program fee in basis points
* `<PROGRAM_FEE_WALLET>` — The program fee wallet pubkey



## `jito-restaking-cli vault config get`

Fetches global config

**Usage:** `jito-restaking-cli vault config get`



## `jito-restaking-cli vault config set-admin`

Set the config admin

**Usage:** `jito-restaking-cli vault config set-admin <NEW_ADMIN>`

###### **Arguments:**

* `<NEW_ADMIN>` — The new admin's pubkey



## `jito-restaking-cli vault vault`

Vault commands

**Usage:** `jito-restaking-cli vault vault <COMMAND>`

###### **Subcommands:**

* `initialize` — Creates a new vault
* `create-token-metadata` — Creates token metadata for the vault's LRT token
* `initialize-vault-update-state-tracker` — Starts the vault update cycle
* `crank-vault-update-state-tracker` — Cranks the vault update state tracker, needs to be run per operator
* `close-vault-update-state-tracker` — Ends the vault update cycle
* `mint-vrt` — Mints VRT tokens
* `initialize-operator-delegation` — Sets up the delegations for an operator
* `delegate-to-operator` — Delegates tokens to an operator
* `cooldown-operator-delegation` — Cooldown delegation for an operator
* `enqueue-withdrawal` — Starts the withdrawal process
* `burn-withdrawal-ticket` — Burns the withdrawal ticket, ending the withdrawal process
* `get-vault-update-state-tracker` — Gets the update state tracker for a vault
* `get-operator-delegation` — Gets the operator delegation for a vault
* `get-withdrawal-ticket` — 
* `get` — Gets a vault
* `list` — List all vaults
* `set-capacity` — Sets the deposit capacity in the vault



## `jito-restaking-cli vault vault initialize`

Creates a new vault

**Usage:** `jito-restaking-cli vault vault initialize <TOKEN_MINT> <DEPOSIT_FEE_BPS> <WITHDRAWAL_FEE_BPS> <REWARD_FEE_BPS> <DECIMALS> <INITIALIZE_TOKEN_AMOUNT> [VRT_MINT_ADDRESS_FILE_PATH]`

###### **Arguments:**

* `<TOKEN_MINT>` — The token which is allowed to be deposited into the vault
* `<DEPOSIT_FEE_BPS>` — The deposit fee in bips
* `<WITHDRAWAL_FEE_BPS>` — The withdrawal fee in bips
* `<REWARD_FEE_BPS>` — The reward fee in bips
* `<DECIMALS>` — The decimals of the token
* `<INITIALIZE_TOKEN_AMOUNT>` — The amount of tokens to initialize the vault with ( in the smallest unit )
* `<VRT_MINT_ADDRESS_FILE_PATH>` — The file path of VRT mint address



## `jito-restaking-cli vault vault create-token-metadata`

Creates token metadata for the vault's LRT token

**Usage:** `jito-restaking-cli vault vault create-token-metadata <VAULT> <NAME> <SYMBOL> <URI>`

###### **Arguments:**

* `<VAULT>` — The vault pubkey
* `<NAME>` — The name of the token
* `<SYMBOL>` — The symbol of the token
* `<URI>` — The URI for the token metadata



## `jito-restaking-cli vault vault initialize-vault-update-state-tracker`

Starts the vault update cycle

**Usage:** `jito-restaking-cli vault vault initialize-vault-update-state-tracker <VAULT>`

###### **Arguments:**

* `<VAULT>` — Vault account



## `jito-restaking-cli vault vault crank-vault-update-state-tracker`

Cranks the vault update state tracker, needs to be run per operator

**Usage:** `jito-restaking-cli vault vault crank-vault-update-state-tracker <VAULT> <OPERATOR>`

###### **Arguments:**

* `<VAULT>` — Vault account
* `<OPERATOR>` — Operator account



## `jito-restaking-cli vault vault close-vault-update-state-tracker`

Ends the vault update cycle

**Usage:** `jito-restaking-cli vault vault close-vault-update-state-tracker <VAULT> [NCN_EPOCH]`

###### **Arguments:**

* `<VAULT>` — Vault account
* `<NCN_EPOCH>` — Optional NCN epoch to close



## `jito-restaking-cli vault vault mint-vrt`

Mints VRT tokens

**Usage:** `jito-restaking-cli vault vault mint-vrt <VAULT> <AMOUNT_IN> <MIN_AMOUNT_OUT>`

###### **Arguments:**

* `<VAULT>` — Vault account
* `<AMOUNT_IN>` — Amount to deposit
* `<MIN_AMOUNT_OUT>` — Minimum amount of VRT to mint



## `jito-restaking-cli vault vault initialize-operator-delegation`

Sets up the delegations for an operator

**Usage:** `jito-restaking-cli vault vault initialize-operator-delegation <VAULT> <OPERATOR>`

###### **Arguments:**

* `<VAULT>` — Vault account
* `<OPERATOR>` — Operator account



## `jito-restaking-cli vault vault delegate-to-operator`

Delegates tokens to an operator

**Usage:** `jito-restaking-cli vault vault delegate-to-operator <VAULT> <OPERATOR> <AMOUNT>`

###### **Arguments:**

* `<VAULT>` — Vault account
* `<OPERATOR>` — Operator account
* `<AMOUNT>` — Amount to delegate



## `jito-restaking-cli vault vault cooldown-operator-delegation`

Cooldown delegation for an operator

**Usage:** `jito-restaking-cli vault vault cooldown-operator-delegation <VAULT> <OPERATOR> <AMOUNT>`

###### **Arguments:**

* `<VAULT>` — Vault account
* `<OPERATOR>` — Operator account
* `<AMOUNT>` — Amount to cooldown



## `jito-restaking-cli vault vault enqueue-withdrawal`

Starts the withdrawal process

**Usage:** `jito-restaking-cli vault vault enqueue-withdrawal <VAULT> <AMOUNT>`

###### **Arguments:**

* `<VAULT>` — Vault account
* `<AMOUNT>` — Amount to withdraw



## `jito-restaking-cli vault vault burn-withdrawal-ticket`

Burns the withdrawal ticket, ending the withdrawal process

**Usage:** `jito-restaking-cli vault vault burn-withdrawal-ticket <VAULT>`

###### **Arguments:**

* `<VAULT>` — Vault account



## `jito-restaking-cli vault vault get-vault-update-state-tracker`

Gets the update state tracker for a vault

**Usage:** `jito-restaking-cli vault vault get-vault-update-state-tracker <VAULT> <NCN_EPOCH>`

###### **Arguments:**

* `<VAULT>` — Vault account
* `<NCN_EPOCH>` — NCN epoch



## `jito-restaking-cli vault vault get-operator-delegation`

Gets the operator delegation for a vault

**Usage:** `jito-restaking-cli vault vault get-operator-delegation <VAULT> <OPERATOR>`

###### **Arguments:**

* `<VAULT>` — Vault account
* `<OPERATOR>` — Operator account



## `jito-restaking-cli vault vault get-withdrawal-ticket`

**Usage:** `jito-restaking-cli vault vault get-withdrawal-ticket <VAULT> [STAKER]`

###### **Arguments:**

* `<VAULT>` — Vault account
* `<STAKER>` — Staker account



## `jito-restaking-cli vault vault get`

Gets a vault

**Usage:** `jito-restaking-cli vault vault get <PUBKEY>`

###### **Arguments:**

* `<PUBKEY>` — The vault pubkey



## `jito-restaking-cli vault vault list`

List all vaults

**Usage:** `jito-restaking-cli vault vault list`



## `jito-restaking-cli vault vault set-capacity`

Sets the deposit capacity in the vault

**Usage:** `jito-restaking-cli vault vault set-capacity <VAULT> <AMOUNT>`

###### **Arguments:**

* `<VAULT>` — The vault pubkey
* `<AMOUNT>` — The new capacity



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

