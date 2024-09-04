---
title: Restaking Theory of Operation
category: Jekyll
layout: post
weight: 1
---

# 1. Introduction

The Restaking Program serves as a registry and management system for Node Consensus Networks (NCNs) and operators within Jito's Liquid Restaking ecosystem. It facilitates the registration, warmup, and cooldown processes for these entities.

## 1.2. Key Components

- NCN (Node Consensus Network): Entities that manage consensus and validate operations.
- Operator: Entities that manage stake and perform delegations.

## 1.3. NCN Management

### 1.3.1. NCN Registration
- NCNs can register with the program using the `InitializeNcn` instruction.
- Each NCN is associated with a unique public key and metadata.

## 1.4. Operator Management

### 1.4.1. Operator Registration
- Operators register using the `InitializeOperator` instruction.
- Each operator is associated with a unique public key and metadata.

## 1.5. NCN-Operator Relationships

- The program manages the relationships between NCNs and operators.
- `InitializeNcnOperatorState` establishes a connection between an NCN and an operator.
- These relationships can be warmed up or cooled down using respective instructions.

## 1.6. NCN-Vault Relationships

- The program manages the relationships between NCNs and vaults.
- `InitializeNcnVaultTicket` establishes a connection between an NCN and a vault.
- These relationships can be warmed up or cooled down using respective instructions.

## 1.7 Operator-Vault Relationships

- The program manages the relationships between operators and vaults.
- `InitializeOperatorVaultTicket` establishes a connection between an operator and a vault.
- These relationships can be warmed up or cooled down using respective instructions.


## 1.8. Interaction with Vault Program

- While the Restaking Program doesn't directly manage tokens, it provides the necessary infrastructure for the Vault Program to make informed decisions about delegations and token management.
