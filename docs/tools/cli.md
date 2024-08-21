---
title: CLI
---

The Jito CLI is a command-line interface for interacting with the Jito Restaking and Vault programs. It allows you to
register NCNs, operators, and vaults, and manage relationships between them.

### Building

```
cargo build
```

### Usage:

```
./target/debug/jito-restaking-cli --help
```

#### Restaking

##### NCN

```
./target/release/jito-restaking-cli restaking ncn initialize # initialize
./target/release/jito-restaking-cli restaking ncn list # list all NCNs
```

##### Operator

```
./target/debug/jito-restaking-cli restaking operator initialize # initialize
./target/debug/jito-restaking-cli restaking operator list # list all operators
```

#### Vault

```
# initialize vault with wSOL token mint, 1 bps deposit fee, 2 bps withdrawal fee, and 3 bps reward fee
./target/debug/jito-restaking-cli vault vault initialize So11111111111111111111111111111111111111112 1 2 3
./target/debug/jito-restaking-cli vault vault list # list all vaults
```
