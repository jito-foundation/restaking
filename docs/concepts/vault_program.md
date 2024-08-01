---
title: Vault Program
---

## About the program

The vault program manages the liquid restaking tokens (LRTs) and associated deposits. The program stores deposited funds
and handles the minting and burning of tokenized stake.

The vault program allows users to do the following:

- Create LRTs (staked assets)
- Deposit assets and receive LRTs in return
- Burn LRTs to withdraw assets
- Manage delegations to operators
- Handle slashing events

The vault program stores user funds and is responsible for the issuance and redemption of LRTs.
Funds do not leave the program under any conditions unless they are withdrawn by the user or a slashing takes place.

## Vault

Vaults are the core entities that manage deposits, withdrawals, and LRT minting/burning.

Several operations can be performed with a vault:

- Initialize a new vault with specific parameters
- Add and remove support for operators
- Add and remove support for AVS
- Manage delegations to operators
- Process deposits and withdrawals
- Handle slashing events

## LRT (Liquid Restaking Token)

LRTs represent a user's share in the vault's assets.
They are minted when users deposit and burned when users withdraw.

## Relationships

The vault program interacts with other entities in the Jito Restaking protocol:

- Operators: The vault delegates to operators and manages these relationships
- AVS: The vault interacts with AVS for slashing and other protocol-specific operations
- Users: Deposit assets and receive LRTs, or burn LRTs to withdraw assets

The vault program uses similar ticket structures as the restaking program to manage these relationships, ensuring mutual
opt-in from all parties involved. Those tickets include:

#### Vault AVS Ticket

```mermaid
graph TD
    classDef main fill: #f9f, stroke: #333, stroke-width: 2px;
    classDef ticket fill: #fff, stroke: #333, stroke-width: 1px;
    Vault[Vault]:::main
    AVS[AVS]:::main
    VaultAvsTicket[VaultAVSTicket]:::ticket
    Vault -->|Creates| VaultAvsTicket
    Vault -.->|Opts in| AVS
```

#### Vault Operator Ticket

```mermaid
graph TD
    classDef main fill: #f9f, stroke: #333, stroke-width: 2px;
    classDef ticket fill: #fff, stroke: #333, stroke-width: 1px;
    Vault[Vault]:::main
    Operator[Operator]:::main
    VaultOperatorTicket[VaultOperatorTicket]:::ticket
    Vault -->|Creates| VaultOperatorTicket
    Vault -.->|Opts in| Operator
```

#### Vault AVS Slasher Ticket

```mermaid
graph TD
    classDef main fill: #f9f, stroke: #333, stroke-width: 2px;
    classDef ticket fill: #fff, stroke: #333, stroke-width: 1px;
    Vault[Vault]:::main
    AvsVaultSlasherTicket[AVSVaultSlasherTicket]:::ticket
    Vault -->|Creates| VaultAvsSlasherTicket
    Vault -.->|Recognizes and copies from| AvsVaultSlasherTicket
```

#### Vault AVS Slasher Operator Ticket

```mermaid
graph TD
    classDef main fill: #f9f, stroke: #333, stroke-width: 2px;
    classDef ticket fill: #fff, stroke: #333, stroke-width: 1px;
    Vault[Vault]:::main
    AVS[AVS]:::main
    Slasher[Slasher]:::main
    Operator[Operator]:::main
    VaultAvsSlasherOperatorTicket[VaultAVSSlasherOperatorTicket]:::ticket
    Vault -->|Creates| VaultAvsSlasherOperatorTicket
    Vault -.->|Tracks slashing for| AVS
    Vault -.->|Tracks slashing by| Slasher
    Vault -.->|Tracks slashing of| Operator
```

## Delegations

The vault program manages delegations to operators. This involves:

- Adding new delegations to operators
- Removing delegations from operators
- Updating delegations at epoch boundaries

Delegations are stored in the VaultDelegationList.

## Slashing

The vault program handles slashing events, which may occur if an operator misbehaves. This includes:

- Processing slash instructions from authorized slashers
- Adjusting the vault's total assets and individual delegations
- Ensuring the integrity of the LRT exchange rate
- Respects the maximum slashing conditions set by the AVS

## Tracking State

State in these programs is spread out across many accounts.
To reason about the state of stake at any given time, one can reference the chart below.

Assets are considered staked iff:

- The AVS has opted-in to the operator
- The operator has opted-in to the AVS
- The operator has opted-in to the vault
- The vault has opted-in to the operator
- The vault has opted-in to the AVS
- The AVS has opted-in to the vault
- The Vault is delegated to that operator

When assets are staked and the following conditions are met, the vault can be slashed by a given slasher:

- The AVS has opted in to a slasher for the given vault.
- The vault has agreed to the conditions set by the AVS for slashing the vault.

![img.png](../assets/staked_venn_diagram.png)