# Command-Line Help for `jito-restaking-cli-args`

This document contains the help content for the `jito-restaking-cli-args` command-line program.

**Command Overview:**

* [`jito-restaking-cli-args`↴](#jito-restaking-cli-args)
* [`jito-restaking-cli-args restaking`↴](#jito-restaking-cli-args-restaking)
* [`jito-restaking-cli-args restaking config`↴](#jito-restaking-cli-args-restaking-config)
* [`jito-restaking-cli-args restaking config initialize`↴](#jito-restaking-cli-args-restaking-config-initialize)
* [`jito-restaking-cli-args restaking config get`↴](#jito-restaking-cli-args-restaking-config-get)
* [`jito-restaking-cli-args restaking ncn`↴](#jito-restaking-cli-args-restaking-ncn)
* [`jito-restaking-cli-args restaking ncn initialize`↴](#jito-restaking-cli-args-restaking-ncn-initialize)
* [`jito-restaking-cli-args restaking ncn get`↴](#jito-restaking-cli-args-restaking-ncn-get)
* [`jito-restaking-cli-args restaking ncn list`↴](#jito-restaking-cli-args-restaking-ncn-list)
* [`jito-restaking-cli-args restaking operator`↴](#jito-restaking-cli-args-restaking-operator)
* [`jito-restaking-cli-args restaking operator initialize`↴](#jito-restaking-cli-args-restaking-operator-initialize)
* [`jito-restaking-cli-args restaking operator get`↴](#jito-restaking-cli-args-restaking-operator-get)
* [`jito-restaking-cli-args restaking operator list`↴](#jito-restaking-cli-args-restaking-operator-list)
* [`jito-restaking-cli-args vault`↴](#jito-restaking-cli-args-vault)
* [`jito-restaking-cli-args vault config`↴](#jito-restaking-cli-args-vault-config)
* [`jito-restaking-cli-args vault config initialize`↴](#jito-restaking-cli-args-vault-config-initialize)
* [`jito-restaking-cli-args vault config get`↴](#jito-restaking-cli-args-vault-config-get)
* [`jito-restaking-cli-args vault vault`↴](#jito-restaking-cli-args-vault-vault)
* [`jito-restaking-cli-args vault vault initialize`↴](#jito-restaking-cli-args-vault-vault-initialize)
* [`jito-restaking-cli-args vault vault get`↴](#jito-restaking-cli-args-vault-vault-get)
* [`jito-restaking-cli-args vault vault list`↴](#jito-restaking-cli-args-vault-vault-list)

## `jito-restaking-cli-args`

A CLI for managing restaking and vault operations

**Usage:** `jito-restaking-cli-args [OPTIONS] [COMMAND]`

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



## `jito-restaking-cli-args restaking`

Restaking program commands

**Usage:** `jito-restaking-cli-args restaking <COMMAND>`

###### **Subcommands:**

* `config` — Initialize, get, and set the config struct
* `ncn` — 
* `operator` — 



## `jito-restaking-cli-args restaking config`

Initialize, get, and set the config struct

**Usage:** `jito-restaking-cli-args restaking config <COMMAND>`

###### **Subcommands:**

* `initialize` — Initialize the config
* `get` — Get the config



## `jito-restaking-cli-args restaking config initialize`

Initialize the config

**Usage:** `jito-restaking-cli-args restaking config initialize`



## `jito-restaking-cli-args restaking config get`

Get the config

**Usage:** `jito-restaking-cli-args restaking config get`



## `jito-restaking-cli-args restaking ncn`

**Usage:** `jito-restaking-cli-args restaking ncn <COMMAND>`

###### **Subcommands:**

* `initialize` — Initialize NCN
* `get` — Get NCN
* `list` — List all NCNs



## `jito-restaking-cli-args restaking ncn initialize`

Initialize NCN

**Usage:** `jito-restaking-cli-args restaking ncn initialize`



## `jito-restaking-cli-args restaking ncn get`

Get NCN

**Usage:** `jito-restaking-cli-args restaking ncn get <PUBKEY>`

###### **Arguments:**

* `<PUBKEY>`



## `jito-restaking-cli-args restaking ncn list`

List all NCNs

**Usage:** `jito-restaking-cli-args restaking ncn list`



## `jito-restaking-cli-args restaking operator`

**Usage:** `jito-restaking-cli-args restaking operator <COMMAND>`

###### **Subcommands:**

* `initialize` — Initialize Operator
* `get` — Get operator
* `list` — List all operators



## `jito-restaking-cli-args restaking operator initialize`

Initialize Operator

**Usage:** `jito-restaking-cli-args restaking operator initialize`



## `jito-restaking-cli-args restaking operator get`

Get operator

**Usage:** `jito-restaking-cli-args restaking operator get <PUBKEY>`

###### **Arguments:**

* `<PUBKEY>`



## `jito-restaking-cli-args restaking operator list`

List all operators

**Usage:** `jito-restaking-cli-args restaking operator list`



## `jito-restaking-cli-args vault`

Vault program commands

**Usage:** `jito-restaking-cli-args vault <COMMAND>`

###### **Subcommands:**

* `config` — 
* `vault` — Vault commands



## `jito-restaking-cli-args vault config`

**Usage:** `jito-restaking-cli-args vault config <COMMAND>`

###### **Subcommands:**

* `initialize` — 
* `get` — 



## `jito-restaking-cli-args vault config initialize`

**Usage:** `jito-restaking-cli-args vault config initialize`



## `jito-restaking-cli-args vault config get`

**Usage:** `jito-restaking-cli-args vault config get`



## `jito-restaking-cli-args vault vault`

Vault commands

**Usage:** `jito-restaking-cli-args vault vault <COMMAND>`

###### **Subcommands:**

* `initialize` — Initializes the vault
* `get` — Gets a vault
* `list` — List all vaults



## `jito-restaking-cli-args vault vault initialize`

Initializes the vault

**Usage:** `jito-restaking-cli-args vault vault initialize <TOKEN_MINT> <DEPOSIT_FEE_BPS> <WITHDRAWAL_FEE_BPS> <REWARD_FEE_BPS> <DECIMALS>`

###### **Arguments:**

* `<TOKEN_MINT>` — The token which is allowed to be deposited into the vault
* `<DEPOSIT_FEE_BPS>` — The deposit fee in bips
* `<WITHDRAWAL_FEE_BPS>` — The withdrawal fee in bips
* `<REWARD_FEE_BPS>` — The reward fee in bips
* `<DECIMALS>` — The decimals of the token



## `jito-restaking-cli-args vault vault get`

Gets a vault

**Usage:** `jito-restaking-cli-args vault vault get <PUBKEY>`

###### **Arguments:**

* `<PUBKEY>` — The vault pubkey



## `jito-restaking-cli-args vault vault list`

List all vaults

**Usage:** `jito-restaking-cli-args vault vault list`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

