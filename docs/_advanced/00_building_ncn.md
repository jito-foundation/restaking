---
title: Building a NCN
category: Jekyll
layout: post
weight: 1
---

## Overview

### Restaking Program
The Restaking program focuses on:
- Initializing NCNs and Operators.
- Managing NCN authorities.
- Registering relationships between NCNs, Operators, and Vaults.

### Vault Program
The Vault program is responsible for:
- Creating and managing VRTs (Vault Resource Tokens).
- Allowing users to deposit and withdraw assets.
- Managing relationships with NCNs, delegations with Operators, and fee settings.

---

## Typical NCN Design

### Onchain NCN Program
The on-chain NCN program consists of several components:

#### **Pricing**
- Determines the relative weight of assets (supported_mints) deposited in all Vaults linked to the NCN.
- Key considerations:
  - Permissioned or permissionless design.
  - Future integration into a generic [Weight Table Program](https://github.com/jito-foundation/jito-tip-router).

#### **Snapshot**
- Captures the current status of all active NCNs and Vaults.
- Aggregates stake weight per operator for the current epoch.

#### **Core Logic**
- The core of the NCN where Node Operators:
  - Post on-chain data proving they fulfilled their roles.
  - Enable the NCN to execute actions based on these results.

#### **Rewards Payment**
- Calculates and distributes payments pro-rata based on the stake weight of successful operators and their Vaults.
- Future integration into the [Rewards NCN](https://github.com/jito-foundation/jito-rewards-ncn).

---

### Offchain Components

#### **Node Operator Client**
- The core off-chain logic of the NCN.
- Runs arbitrary computation and posts data on-chain via custom instructions.

#### **Permissionless Cranker**
- Automates permissionless operations on a regular cycle.
- Examples:
  - Jito Tip Router Program: Executes price updates, snapshots, initializes Merkle roots post-consensus, and handles rewards payments.

---

## Steps to Build an NCN

### 1. Initialize the NCN
- Deploy the on-chain NCN program.
- Register initial relationships with Operators and Vaults.

### 2. Establish Pricing and Weight Rules
- Define how asset weights are calculated across Vaults.
- Choose between permissioned or permissionless mechanisms.

### 3. Implement Core Logic
- Define the logic for your NCN’s specific purpose.
- Ensure data validation and action execution align with your use case.

### 4. Develop Snapshot Functionality
- Enable the program to periodically aggregate and record the state of NCNs, Vaults, and their respective stake weights.

### 5. Manage Rewards Payment
- Configure reward calculations and distributions based on operator performance.
- Prepare for future integration with the Rewards NCN.

### 6. Create and Deploy Offchain Clients
- Provide an open-source Node Operator Client for operators.
- Ensure the client efficiently handles custom computations and data posting.

### 7. Set Up a Permissionless Cranker
- Build a client to automate routine instructions, ensuring the NCN operates without manual intervention.

---

## Further Reading

- [NCN Cookbook](https://ncn-cookbook.vercel.app/building-ncn/ncn-design.html)
