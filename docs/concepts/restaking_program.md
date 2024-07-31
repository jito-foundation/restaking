# Restaking Program

### Relationships

The restaking protocol leverages the concept of entity tickets to track relationships between AVS, operators, and vaults
on-chain. This allows for an extremely flexible opt-in and opt-out system.

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

This ticket represents the slashing relationship between an AVS and a Vault. It is created by the AVS, allowing the AVS
to potentially slash the Vault under certain conditions.

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
