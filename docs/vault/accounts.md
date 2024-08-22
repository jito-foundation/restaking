---
title: Vault Program Accounts
---

## 1. About the program

The vault program manages the vault receipt tokens (VRTs) and associated deposits. The program stores deposited funds and handles the minting and burning of tokenized stake.

## 2. Relationships

The vault program interacts with other entities in the Jito Restaking protocol:

- Operators: The vault delegates to operators and manages these relationships
- NCN: The vault interacts with NCN for slashing and other protocol-specific operations
- Users: Deposit assets and receive VRTs, or burn VRTs to withdraw assets

Below is a diagram of the relationships between the entities:

### 2.0.1. Vault NCN Ticket

```mermaid
graph TD
    classDef main fill: #f9f, stroke: #333, stroke-width: 2px;
    classDef ticket fill: #fff, stroke: #333, stroke-width: 1px;
    Vault[Vault]:::main
    Ncn[Ncn]:::main
    VaultNcnTicket[VaultNcnTicket]:::ticket
    Vault -->|Creates| VaultNcnTicket
    Vault -.->|Opts in| Ncn
```

### 2.0.2. Vault Operator Delegation

```mermaid
graph TD
    classDef main fill: #f9f, stroke: #333, stroke-width: 2px;
    classDef ticket fill: #fff, stroke: #333, stroke-width: 1px;
    Vault[Vault]:::main
    Operator[Operator]:::main
    VaultOperatorDelegation[VaultOperatorDelegation]:::ticket
    Vault -->|Creates| VaultOperatorDelegation
    Vault -.->|Opts in| Operator
```

### 2.0.3. Vault NCN Slasher Ticket

```mermaid
graph TD
    classDef main fill: #f9f, stroke: #333, stroke-width: 2px;
    classDef ticket fill: #fff, stroke: #333, stroke-width: 1px;
    Vault[Vault]:::main
    NcnVaultSlasherTicket[NcnVaultSlasherTicket]:::ticket
    Vault -->|Creates| VaultNcnSlasherTicket
    Vault -.->|Recognizes and copies from| NcnVaultSlasherTicket
```

### 2.0.4. Vault NCN Slasher Operator Ticket

```mermaid
graph TD
    classDef main fill: #f9f, stroke: #333, stroke-width: 2px;
    classDef ticket fill: #fff, stroke: #333, stroke-width: 1px;
    Vault[Vault]:::main
    Ncn[Ncn]:::main
    Slasher[Slasher]:::main
    Operator[Operator]:::main
    VaultNcnSlasherOperatorTicket[VaultNcnSlasherOperatorTicket]:::ticket
    Vault -->|Creates| VaultNcnSlasherOperatorTicket
    Vault -.->|Tracks slashing for| Ncn
    Vault -.->|Tracks slashing by| Slasher
    Vault -.->|Tracks slashing of| Operator
```

## 3. Tracking State

State in these programs is spread out across many accounts.
To reason about the state of stake at any given time, one can reference the chart below.

Assets are considered staked iff:

- The NCN has opted-in to the operator
- The operator has opted-in to the NCN
- The operator has opted-in to the vault
- The vault has opted-in to the operator
- The vault has opted-in to the NCN
- The NCN has opted-in to the vault
- The Vault is delegated to that operator

When assets are staked and the following conditions are met, the vault can be slashed by a given slasher:

- The NCN has opted in to a slasher for the given vault.
- The vault has agreed to the conditions set by the NCN for slashing the vault.

![img.png](../assets/staked_venn_diagram.png)