# Jito Restaking Client

An client for interacting with the Jito Restaking program.

## Installation

```bash
npm install jito-restaking-client
```

or

```bash
yarn add jito-restaking-client
```

## Example

### Fetch Restaking Config

```typescript
import { address, createSolanaRpc } from "@solana/web3.js";
import { fetchConfig } from "jito-restaking-client";

const rpc = createSolanaRpc("https://api.devnet.solana.com");

const configPubkey = address("4vvKh3Ws4vGzgXRVdo8SdL4jePXDvCqKVmi21BCBGwvn");

const config = await fetchConfig(rpc, configPubkey);

console.log("Config: {}", config);
```