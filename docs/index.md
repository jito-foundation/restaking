# Jito Multi-Asset Restaking on Solana

Welcome to the documentation for Jito's Multi-Asset (Re)staking project on Solana. This project implements a
next-generation (re)staking platform for Solana and SVM environments.

## Overview

Jito Restaking is a comprehensive framework for staking, restaking, and liquid restaking on the Solana blockchain. It
provides a flexible and secure way to manage assets across multiple operators and actively validated services (AVS).

### Key Features

- Universal framework for staking and restaking **any** SPL token on Solana
- All stake is tokenized into a receipt token representing the staked assets (LRT)
- Customizable slashing conditions and administration across all functionality.
- Flexible AVS and operator management

## Core Concepts

Understanding these core concepts will help you navigate the Jito Restaking ecosystem:

- [(Re)staking Explained](restaking.md)
- [Architecture](architecture.md)
- [Restaking Program](concepts/restaking_program.md)
- [Vault Program](concepts/vault_program.md)
- [Slashing](concepts/slashing.md)

## Developer Resources

- [Testing](development/testing.md)

## Advanced Topics

- [Building an AVS](advanced/building_avs.md)
- [Building an LRT](advanced/building_lrt.md)

## License

This project is licensed under the Business Source License 1.1 - see the [LICENSE.md](../LICENSE.md) file for details.
