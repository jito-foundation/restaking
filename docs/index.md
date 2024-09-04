---
title: Jito (Re)staking
---

Jito (Re)staking is a multi-asset staking protocol for node consensus networks. The protocol tokenizes staked assets as
vault receipt tokens for enhanced liquidity and composability. Node consensus networks can use Jito Restaking to easily
customize staking parameters, slashing conditions, and economic incentives to tailor their security and tokenomics.

### Key Features

- Universal framework for (re)staking SPL tokens to node consensus networks on Solana and SVM chains.
- Staked assets are tokenized into Vault Receipt Tokens (VRT)
- Flexible opt-in from node consensus networks, operators, and vaults for staking and slashing.

### Addresses

| Network | Program   | Address                                      | Version |
|---------|-----------|----------------------------------------------|---------|
| Testnet | Restaking | 78J8YzXGGNynLRpn85MH77PVLBZsWyLCHZAXRvKaB6Ng | 0.0.1   |
| Testnet | Vault     | 34X2uqBhEGiWHu43RDEMwrMqXF4CpCPEZNaKdAaUS9jx | 0.0.1   |
| Devnet  | Restaking | 78J8YzXGGNynLRpn85MH77PVLBZsWyLCHZAXRvKaB6Ng | 0.0.1   |
| Devnet  | Vault     | 34X2uqBhEGiWHu43RDEMwrMqXF4CpCPEZNaKdAaUS9jx | 0.0.1   |

## Core Concepts

Understanding these core concepts will help you navigate the Jito Restaking ecosystem:

- [Terminology](terminology.md)

## Restaking

- [Restaking Program Accounts](restaking/accounts.md)
- [Restaking Program Theory Of Operation](restaking/theory_of_operation.md)
- [Building an Node Consensus Network](advanced/building_ncn.md)

## Vault

- [Vault Program Accounts](vault/accounts.md)
- [Vault Program Theory Of Operation](vault/theory_of_operation.md)

## Developers

- [API](api/jito_jsm_core/index.html)
- [Jito CLI](tools/cli.md)

## License

This project is licensed under the Business Source License 1.1 - see the [LICENSE.md](../LICENSE.md) file for details.
