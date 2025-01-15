# @jito-foundation/vault-sdk

## Package Purpose
The @jito-foundation/vault-sdk is a TypeScript SDK for interacting with Jito's Vault program on Solana.

## Technical Requirements
- Node.js
- Peer Dependencies:
  - @solana/web3.js
  - @solana/spl-token
- TypeScript type definitions included

## Features
The SDK provides functionality for:

### Account Management
- Config accounts
- Vault accounts
- NCN (Node Consensus Network) tickets
- Slasher tickets
- Operator delegations
- Withdrawal tickets
- State tracking

### Instructions
- Minting operations
- Delegation management
- Withdrawal operations
- Token account management
- Administrative functions
- Vault initialization and updates
- NCN ticket management

## Example Usage
Here's a basic example of creating a mint instruction:

```typescript
const instruction = getMintToInstruction({
    config: configKey,
    vault: vaultKey,
    vrtMint: vrtMintKey,
    depositor: {
        publicKey: depositorKey,
        signTransaction: async () => { /* ... */ },
        signAllTransactions: async () => { /* ... */ },
    },
    depositorTokenAccount: depositorTokenKey,
    vaultTokenAccount: vaultTokenKey,
    depositorVrtTokenAccount: depositorVrtTokenKey,
    vaultFeeTokenAccount: vaultFeeTokenKey,
    amountIn: 1000n,
    minAmountOut: 900n,
});
```

## Installation

Install the SDK:
```bash
yarn add @jito-foundation/vault-sdk
```

Install peer dependencies:
```bash
yarn add @solana/web3.js @solana/spl-token
```

## Repository Information
- License: MIT
- Author: Jito Foundation

## Package Structure
```
packages/vault-sdk/
├── src/           # Source files
│   ├── accounts/  # Account definitions
│   ├── instructions/# Program instructions
│   ├── programs/  # Program definitions
│   ├── shared/    # Shared utilities
│   └── types/     # TypeScript types
├── dist/          # Compiled JavaScript
└── README.md      # Documentation
```

## Development

Build the package:
```bash
yarn build
```

Clean build artifacts:
```bash
yarn clean
```