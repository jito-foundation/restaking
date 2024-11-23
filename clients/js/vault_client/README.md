# Jito Restaking Client

An client for interacting with the Jito Vault program.

## Installation

```bash
npm install jito-vault-client
```

or

```bash
yarn add jito-vault-client
```

## Example

### Fetch vault config

```typescript
import { address, createSolanaRpc } from "@solana/web3.js";
import { fetchConfig } from "jito-vault-client";

const rpc = createSolanaRpc("https://api.devnet.solana.com");

const configPubkey = address("UwuSgAq4zByffCGCrWH87DsjfsewYjuqHfJEpzw1Jq3");

const config = await fetchConfig(rpc, configPubkey);

console.log("Config: {}", config);
```