# Jito Shank CLI

A command-line interface tool for managing and generating IDL files for Solana programs using the shank-idl library.

## Overview

Jito Shank CLI simplifies the process of generating IDL files.
It allows you to extract IDL definitions from multiple module paths and combine them into a single comprehensive IDL file.

## Installation

```bash
cargo install jito-shank-cli
```

## Getting Started

### Generate IDL

### Command Structure

The CLI supports the following commands and options:

#### Global Options

- `--program-env-path`: Path to the environment file containing program IDs
- `--output-idl-path`: Directory where the generated IDL file will be saved

#### Commands

##### Generate

Generates an IDL file.

```bash
shank-cli --program-env-path ./.env --output-idl-path ./idl generate --program-env-key MY_PROGRAM_ID --idl-name my_program --module-paths core program sdk
```

###### Options

- `--program-env-key`: Key in the environment file that contains the program ID
- `--idl-name`: Name for the generated IDL file
- `--module-paths`: One or more paths to Rust modules containing shank annotations

## Environment File Format

The environment file should contain key-value pairs:

```
RESTAKING_PROGRAM_ID=RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q
VAULT_PROGRAM_ID=Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8
```

## Example

Jito Restaking Program

```bash
cargo r -p jito-shank-cli -- \
--program-env-path ./config/program.env \
--output-idl-path ./idl \
generate \
--program-id-key "RESTAKING_PROGRAM_ID" \
--idl-name jito_restaking \
--module-paths "restaking_sdk" \
--module-paths "restaking_core" \
--module-paths "restaking_program" \
--module-paths "bytemuck" \
--module-paths "core"
```

This will:
1. Read the program ID from the `PROGRAM_ID` key in `./config/program.env`
2. Extract IDL definitions from the modules in `./restaking_sdk`, `./restaking_core`, `./restaking_program`, `./bytemuck` and `core`.
3. Combine them into a single IDL
4. Save the result as `./idl/jito_restaking.json`


Jito Vault Program

```bash
cargo r -p jito-shank-cli -- \
--program-env ./config/program.env \
--output-idl-path ./idl \
generate \
--program-id-key "VAULT_PROGRAM_ID" \
--idl-name jito_vault \
--module-paths "vault_sdk" \
--module-paths "vault_core" \
--module-paths "vault_program" \
--module-paths "bytemuck" \
--module-paths "core"
```
