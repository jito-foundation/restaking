# Jito Restaking API

## Overview

This API allows users to:

- Retrieve TVL for each vault in native and USD vaules.
- List all vaults with their details.

## Endpoints

1. Get TVL for All Vaults

Fetches TVL for each vault in **native unit** and USD.

Endpoint:

```http
GET api/v1/vaults/tvl
```

Response:

```json
[
    {
        "vault_pubkey":"CugziSqZXcUStNPXbtRmq6atsrHqWY2fH2tAhsyApQrV",
        "supported_mint":"J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
        "native_unit_tvl":291871.554825083,
        "native_unit_symbol":"JITOSOL",
        "usd_tvl":82042175.34578258
    },
    {
        "vault_pubkey":"CQpvXgoaaawDCLh8FwMZEwQqnPakRUZ5BnzhjnEBPJv",
        "supported_mint":"J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
        "native_unit_tvl":134310.541925135,
        "native_unit_symbol":"JITOSOL",
        "usd_tvl":37753350.229736194
    },
]
```

2. Get TVL for a Specific Vault

Fetches TVL for a single vault.

Endpoint:

```http
GET api/v1/vaults/{vault_pubkey}/tvl
```

Response:

```json
{
    "vault_pubkey":"CugziSqZXcUStNPXbtRmq6atsrHqWY2fH2tAhsyApQrV",
    "supported_mint":"J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
    "native_unit_tvl":291871.554825083,
    "native_unit_symbol":"JITOSOL",
    "usd_tvl":82042175.34578258
}
```

3. List All Vaults

Retrieves all vaults with details.

Endpoint:

```http
GET api/v1/vaults
```

Response:

```json
[
    {
        "discriminator":2,
        "base":"AbH5RtAgpnxyRuT9LqXR9ye4JuuJoHs6E5ENPvCnSRDk",
        "vrt_mint":"CtJcH6BeUPKEfBNaUoPjmEc88E4aLuEJZU4NkuSdnpZo",
        "supported_mint":"J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
        "vrt_supply":9306868868,
        "tokens_deposited":9306868868,
        "deposit_capacity":680411000000000,
        "delegation_state":
            {
                "staked_amount":0,
                "enqueued_for_cooldown_amount":0,
                "cooling_down_amount":0,
                ...
            }
        ...
    }
...
]
```

4. Get Vault Details

Retrieves details of a specific vault.

Endpoint:

```http
GET api/v1/vaults/{vault_pubkey}
```

Response:

```json
{
    "discriminator":2,
    "base":"AbH5RtAgpnxyRuT9LqXR9ye4JuuJoHs6E5ENPvCnSRDk",
    "vrt_mint":"CtJcH6BeUPKEfBNaUoPjmEc88E4aLuEJZU4NkuSdnpZo",
    "supported_mint":"J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
    "vrt_supply":9306868868,
    "tokens_deposited":9306868868,
    "deposit_capacity":680411000000000,
    "delegation_state":
        {
            "staked_amount":0,
            "enqueued_for_cooldown_amount":0,
            "cooling_down_amount":0,
            ...
        }
    ...
}
```


## Build

### Prerequisites

- [Solana's RPC Client](https://docs.rs/solana-rpc-client/latest/solana_rpc_client/)
- [Axum](https://docs.rs/axum/latest/axum/)

### Build for release

To build the API for release, run the following command:

```bash
cargo b --release --bin jito-restaking-api
```

### Check available options

To view the options available for configuring the API:

```bash
./target/release/jito-restaking-api --help

# Jito Restaking API
# 
# Usage: jito-restaking-api [OPTIONS]
# 
# Options:
#       --bind-addr <BIND_ADDR>  Bind address for the server [env: BIND_ADDR=] [default: 0.0.0.0:7001]
#       --rpc-url <RPC_URL>      RPC url [env: RPC_URL=] [default: https://api.mainnet-beta.solana.com]
#   -h, --help                   Print help
#   -V, --version                Print version
```

### Running the API

Once built, run the API using the following command:

```bash
./target/release/jito-restaking-api --rpc-url "your-rpc-url"
```
