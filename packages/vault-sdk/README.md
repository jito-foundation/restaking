# Jito Restaking Client

A client for interacting with the Jito Vault program.

## Installation

```bash
npm install @jito-foundation/vault-sdk
```

or

```bash
yarn add @jito-foundation/vault-sdk
```

## Example

### Fetch vault config

```typescript
import { address, createSolanaRpc } from "@solana/kit";
import { fetchConfig } from "@jito-foundation/vault-sdk";

const rpc = createSolanaRpc("https://api.devnet.solana.com");

const configPubkey = address("UwuSgAq4zByffCGCrWH87DsjfsewYjuqHfJEpzw1Jq3");

const config = await fetchConfig(rpc, configPubkey);

console.log("Config: {}", config);
```