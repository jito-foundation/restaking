# Jito Restaking Client

A client for interacting with the Jito Restaking program.

## Installation

```bash
npm install @jito-foundation/restaking-sdk
```

or

```bash
yarn add @jito-foundation/restaking-sdk
```

## Example

### Fetch Restaking Config

```typescript
import { address, createSolanaRpc } from "@solana/kit";
import { fetchConfig } from "@jito-foundation/restaking-sdk";

const rpc = createSolanaRpc("https://api.devnet.solana.com");

const configPubkey = address("4vvKh3Ws4vGzgXRVdo8SdL4jePXDvCqKVmi21BCBGwvn");

const config = await fetchConfig(rpc, configPubkey);

console.log("Config: {}", config);
```
