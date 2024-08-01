---
title: Restaking Program
---

### About the program

The restaking program acts as a registry for AVS, operators, and relationships between AVS, operators, and vaults.

It allows users to do the following:

- Registers AVS, operators, and their configurations.
- Stores relationships between AVS, operators, and vaults.

The restaking program does not store any funds; it is purely used as a registry and relationship manager between
entities in the system.

### AVS

AVS are services that provide infrastructure to the network, such as validators, oracles, keepers, bridges, L2s, and
other services that require a staking mechanism for security.

Actively Validated Services (AVS) can be registered through the restaking program.

There are several things one can do after registering an AVS:

- Add and remove support for operators participating in the AVS validator set.
- Add and remove support for vaults
- Add and remove support for slashers
- Withdraw funds sent to the AVS from rewards, airdrops, and other sources.

### Operator

Operators are entities responsible for running AVS software.

Operators can register through the restaking program and configure several variables:

- Add and remove support for vaults
- Add and remove support for AVS
- Change voter keys
- Withdraw funds sent to the operator from rewards, airdrops, and other sources.

### Relationships

The Jito Restaking protocol requires mutual opt-in from all parties entering stake agreements: vaults, operators, and
AVS.

It leverages the concept of entity tickets, which are PDAs representing opt-in from one party to another. These tickets
are created on-chain and can be used to track relationships between AVS, operators, and vaults. In addition to entity
information, these tickets can store additional data like slot activated/deactivated, slashing conditions, and more.

The tickets are detailed below:

#### Operator AVS Ticket

This ticket represents the relationship from the Operator's perspective. It is created by the Operator when it opts in
to work with an AVS.

```mermaid
graph TD
    classDef main fill: #f9f, stroke: #333, stroke-width: 2px;
    classDef ticket fill: #fff, stroke: #333, stroke-width: 1px;
    AVS[AVS]:::main
    Operator[Operator]:::main
    OperatorAvsTicket[OperatorAVSTicket]:::ticket
    Operator -->|Creates| OperatorAvsTicket
    Operator -.->|Opts in| AVS
```

#### Avs Operator Ticket

This ticket represents the relationship from the AVS's perspective. It is created by the AVS when it opts in to work
with an Operator.

```mermaid
graph TD
    classDef main fill: #f9f, stroke: #333, stroke-width: 2px;
    classDef ticket fill: #fff, stroke: #333, stroke-width: 1px;
    AVS[AVS]:::main
    Operator[Operator]:::main
    AvsOperatorTicket[AVSOperatorTicket]:::ticket
    AVS -->|Creates| AvsOperatorTicket
    AVS -.->|Opts in| Operator
```

#### AVS Vault Ticket

This ticket represents the relationship between an AVS and a Vault. It is created by both the AVS and the Vault when
they opt in to work with each other.

```mermaid
graph TD
    classDef main fill: #f9f, stroke: #333, stroke-width: 2px;
    classDef ticket fill: #fff, stroke: #333, stroke-width: 1px;
    AVS[AVS]:::main
    Vault[Vault]:::main
    AvsVaultTicket[AVSVaultTicket]:::ticket
    AVS -->|Creates| AvsVaultTicket
    AVS -.->|Opts in| Vault
```

#### Operator Vault Ticket

This ticket represents the relationship between an Operator and a Vault. It is created by both the Operator and the
Vault when they opt in to work with each other.

```mermaid
graph TD
    classDef main fill: #f9f, stroke: #333, stroke-width: 2px;
    classDef ticket fill: #fff, stroke: #333, stroke-width: 1px;
    Operator[Operator]:::main
    Vault[Vault]:::main
    OperatorVaultTicket[OperatorVaultTicket]:::ticket
    Operator -->|Creates| OperatorVaultTicket
    Operator -.->|Opts in| Vault
```

#### AVS Vault Slasher Ticket

This ticket represents the slashing relationship between an AVS and a Vault.

AVS register slashers, which allows the slasher to potentially slash the Vault under appropriate conditions.

```mermaid
graph TD
    classDef main fill: #f9f, stroke: #333, stroke-width: 2px;
    classDef ticket fill: #fff, stroke: #333, stroke-width: 1px;
    AVS[AVS]:::main
    Vault[Vault]:::main
    AvsVaultSlasherTicket[AVSVaultSlasherTicket]:::ticket
    AVS -->|Creates| AvsVaultSlasherTicket
    AVS -.->|Opts in| Vault
```
