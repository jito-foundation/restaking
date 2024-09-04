---
title: About
category: Jekyll
layout: post
---

Jito (Re)staking is a multi-asset staking protocol for node consensus networks. The system is made of two programs: the
restaking program and the vault program.

The restaking program acts as a node consensus network and operator registry. The program leverages a flexible system of
admins so NCNs can customize the operators and vaults supported and operators can customize the NCNs they stake to and vaults
they can receive delegations from.

The vault program manages the minting and burning of vault receipt tokens (VRTs). VRTs are SPL tokens that represent
a pro-rata stake of assets in the vault. VRTs provide enhanced liquidity, composability, and interoperability with other
Solana programs. The program also leverages a flexible system of admins so vaults can customize the capacity, operators
that can receive delegations from the vault, the NCNs supported by the vault, and the fee structure for staking and unstaking.

### Key Features

- Universal framework for (re)staking SPL tokens to node consensus networks on Solana and SVM chains.
- Staked assets are tokenized into Vault Receipt Tokens (VRT)
- Flexible opt-in from node consensus networks, operators, and vaults for staking and slashing.

### Entity Opt-in

The restaking and vault programs are designed to be flexible and allow for easy opt-in from node consensus networks,
operators, and vaults. The following diagram shows the opt-in process for the Jito Restaking Ecosystem:

![alt text](/assets/images/opt_in.png)
*Figure: Overview of the Jito Restaking Ecosystem*

When a NCN, operator, and vault have all opted-in to each other and the vault has staked assets to the operator, those
assets are considered staked to the NCN. The operator will then be able to participate in the NCN's consensus protocol.
Assuming the vault has opted-in to the slasher, the staked assets can be slashed.

### Vault Interactions

The following diagram shows the interactions between users, admins, and the vault:

![Vault interactions](/assets/images/vault_interactions.png)
*Figure: Overview of the Vault Interactions*

### Restaking Interactions
The following diagram shows the interactions between users, admins, and the restaking program:

![Restaking interactions](/assets/images/restaking_interactions.png)
*Figure: Overview of the Restaking Interactions*
