---
title: Jito (Re)staking Overview
category: Jekyll
layout: post
weight: 1
---

**Jito (Re)staking** is a universal staking protocol for decentralized networks called node consensus networks (NCNs) built on Solana. It provides a framework for deploying NCNs and staking SPL tokens to them, with flexibility baked in at every level. Altogether, Jito (Re)staking is a coordination protocol for developers to build external networks that use the Solana runtime to enforce custom proof of stake protocol. 

The protocol coordinates stake and rewards between three main participants: NCNs, Vaults, and Operators. Developers register NCNs with custom rules. Operators perform arbitrary logic defined by the NCN, whether that’s processing data, orchestrating or coordination tasks, running models, or verifying offchain inputs. Vaults hold staked SPL tokens and delegate them to Operators. NCNs, Operators, and Vaults can define who they interact with and under what conditions.  

The system consists of two programs: the **Restaking Program** (`RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q`), which acts as an onchain registry for NCNs and Operators, and the **Vault Program** (`Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8`), which manages tokenized stake through Vault Receipt Tokens (VRTs) between participants. Both of these programs gives developers the flexibility to customize network operations and various administrative authorities, including appointing stake delegators, setting fees, making protocol upgrades, and updating supported tokens. Together, the Restaking program and the Vault Program  manage stake delegations across the system.

![img.png](/assets/images/ncn.png)

### Onchain Coordination, Offchain Execution

Jito (Re)staking is an onchain registry of services, stakers, and node operators on Solana. Its design separates core network services and coordination:

- All core activity (computation, validation, data collection) happens offchain.
- All coordination (stake delegation, voting, rewards distributions) is tracked, maintained, and enforced onchain.

This split enables scalability and flexibility for developers while retaining cryptoeconomic guarantees from Solana’s base layer. It makes it easier to bootstrap distributed networks of economically aligned operators and stakers, without building infrastructure from scratch or relying on high emissions. Effectively, this model creates a more efficient and cost effective security model (e.g. one set of staked tokens can secure multiple services) and allows teams to allocate moreresources toward core development.

## Core Components

### Node Consensus Network (NCN)

An NCN is a decentralized network that reaches consensus on offchain data or workloads and updates onchain state. These networks may include oracle networks, DePIN services, bridges, co-processors, or new chains. In most cases, NCNs will include their own custom onchain programs to handle proof submissions, verify work, and distribute rewards. Consensus can take place onchain or offchain.

### Vaults

Vaults serve as deposit pools that hold staked tokens (e.g. JitoSOL) and issue vault receipt tokens (VRTs) representing those positions. Vaults opt into supporting specific NCNs and delegate stake to approved operators. Users retain ownership of their stake via VRTs.

Each vault defines key parameters, including how much stake is allocated to each node operator.

### Operators

Operators are infrastructure providers that run the offchain services for NCNs and are delegated stake. They opt in to serve specific NCNs and receive stake from approved vaults. Operators have no direct control of the stake and they are usually rewarded in proportion to the stake they have. Operators can serve multiple NCNs simultaneously, enabling efficient resource usage and shared security.

### The Opt-In Handshake

Participation in Jito (Re)staking is governed on-chain by mutual consent:

1. NCNs register onchain and approve operators and vaults to participate in their network
2. Operators opt in to NCNs and must be accepted.
3. Vaults opt in to NCNs and delegate to approved operators.

This handshake guarantees that vaults, operators, or NCNs are not forced into any connection. All actively staked links are explicitly approved, creating a modular and flexible system for stake delegation and service coordination.

### Why Jito (Re)staking Matters

We built the Jito (Re)staking protocol because there are aspects of Jito Network that can benefit from incremental decentralization. And as the Solana ecosystem continues to mature, we expect other developers will eventually transition to prioritizing resiliency over product iteration speed and seek to build custom decentralized solutions that fit the needs of their protocol. The primary benefits of using the restaking protocol to bootstrap decentralized protocol include:

- **Token Utility**: The restaking protocol is completely non-custodial and requires multiple parties to opt-in and coordinate network connections, operations, and rewards distributions, unlocking a path for NCNs to build decentralized networks and install token utility.

- **Access to professional node operators**: NCNs require different hardware requirements and software competencies. Jito Network is deeply integrated with Solana’s validator ecosystem, which includes a wide range of sophisticated independent operators and institutional operators. This makes it very trivial for NCNs to connect with the industry’s best node operators to participate in their networks, regardless of the underlying network’s hardware and software requirements.

- **Wide Distribution**: JitoSOL is the largest stakepool and is deeply integrated with the Solana DeFi ecosystem. NCNs can immediately tap into Jito’s network effects without having to attract native stake from scratch. This means, by registering with the Jito (Re)staking framework, bootstrapping and building staked networks is very cost-effective and extremely trivial. 

- **Capital efficiency**: The same stake can secure multiple services. The same operators can operate multiple services.

- **Aligned incentives**: Stakers, operators, and NCN developers all benefit from performance, transparency, and modular security.

- **Instant access to Internet Capital Markets**: NCNs have instant access to [Internet Capital Markets](https://multicoin.capital/2025/01/22/the-solana-thesis-internet-capital-markets/). Vaults have the incentive to integrate VRTs across DeFi, creating market structures for native tokens. 

Jito (Re)staking greatly reduces the friction to launch, or transition existing services into, decentralized protocols with proof of stake security rooted on Solana.

## Key Roles and Responsibilities

This section focuses on the organizational roles behind the system. Each persona (whether they’re launching a network, managing capital, running infrastructure, or providing stake) has clearly defined administrative capabilities and responsibilities. This alignment is central to how Jito (Re)staking ensures trust and coordination in a modular, multi-party environment.

### NCN Admin

The NCN admin is the team or entity launching and managing the NCN. This could be a protocol team, research group, DAO, or company.

**Key responsibilities:**

- Register the NCN account, deploy the NCN program(s) including defining consensus, and configure the parameters of the NCN (e.g. accepted tokens and operator quorum)
- Approve or remove Operators that serve the network
- Launch and finalize epochs (start/close voting periods)
- Monitors results, sets fees and manages relationships between operators and vaults
- Define slashing logic including slashable behavior and subsequent penalties

This role is active and ongoing. Admins aren’t just deployers. They’re stewards of the network’s operation, performance, consensus, and upgrades.

### Vault Admin

Vault admins control how users' stake is allocated across NCNs. They manage the vault configuration and oversee delegations. Vaults may be admin-controlled or governed by token holders (e.g. via DAO voting).

**Key responsibilities:**

- Create and register Vaults that support specific SPL tokens
- Opt into selected NCNs and operators and define allocation parameters
- Delegate stake to approved Operators
- Process reward distributions
- Manage warmup/cooldown periods for stake activation and withdrawal

Vault admins allocate the capital. Their decisions influence which NCNs receive economic security and which Operators are trusted with stake.

### Operators

Operators, such as existing Solana validators, run nodes that opt in to run NCN-specific offchain workloads. They are rewarded based on performance which can include uptime, correctness, and participation, subject to the NCN. On top of this, they are penalized for underperformance or misconduct Penalties may include losing stake delegations or connections or being slashed.

**Key responsibilities:**

- Opt in to serve one or more NCNs
- Accept stake from Vaults and remain compliant to avoid slashing
- Run offchain services as specified by each NCN (e.g. compute, validate, read data)
- Stay online and responsive throughout each consensus cycle
- Participate in voting and contribute to onchain consensus

Operators form the execution layer of the network. Because they receive stake from Vaults and are rewarded on a stake-weighted basis, they are economically incentivized to perform correctly and continuously.

### Stakers

Stakers are users who deposit JitoSOL and other tokens into Vaults to secure NCNs and earn yield. Their capital is the backbone for NCNs because it provides economic security i.e. an economic incentive to follow protocol.

**Key responsibilities:**

- Choose which Vault(s) to deposit stake, based on supported NCNs and risk preferences
- Receive Vault receipt tokens (VRTs) representing their positions
- Earn native staking rewards plus additional restaking yield
- Participate in governance (if the Vault allows it) to help steer delegation choices

Stakers are the foundation of restaking, they choose where to place their economic security and their deposits give Vaults the power to delegate that economic security to NCNs.

## Lifecycle of a Node Consensus Network (NCN)

Once an NCN is initialized, it operates in continuous cycles which are coordinated across three distinct phases: setup, operation, and maintenance. This lifecycle involves initializing the network and its participants, running its offchain tasks, enforcing onchain accountability, and evolving over time based on the demands of the NCN. Each phase involves actions by multiple parties: NCN admins, vault controllers, and node operators.

### Setup Phase: Building the Foundation

The setup phase establishes the NCN’s identity, rules, and participants.

- **NCN Registration**: The NCN admin initializes an NCN account onchain by calling the restaking program, requiring both the NCN admin key and a base NCN key. This account serves as an entry in the registry and does not store detailed network configurations. Instead, configurations like accepted tokens, slashing conditions, quorum thresholds, and operator requirements are managed separately, typically via the NCN's own onchain program, CLI, or off-chain enforcement logic.
- **Vault and Operator Onboarding**: Vaults and Operators must each explicitly opt into the NCN, and the NCN must in turn approve them. Before establishing these relationships, vaults and operators must first register through the **Vault Program** and **Restaking Program**, respectively. Once registered, they can initiate connections to a specific NCN.

Once approved by the NCN, the admins should call a warm-up instruction to activate the connections. The warm-up period lasts for one full epoch, not including the current partial epoch. The stake becomes active once all three components initiate and warm up the connections. This opt-in approval process ensures that all active stake delegations are mutual and intentional.

- **Stake Delegation**: After the warm-up period, vaults can delegate stake to the operator. Stake becomes active immediately. However, when a user withdraws their stake, it must undergo a cooldown period that lasts for one full NCN-defined epoch.

By the completion of this setup phase, the NCN has established its security parameters, approved operators, and activated its initial stake. This foundation sets the stage for the operations phase, and different dynamics come into play.

### Operations Phase: Running the Network

Each NCN progresses through consensus cycles, which may be time-bound to epoch lengths or follow a custom logic defined by the NCN admin. Within each cycle, operators perform offchain tasks and submit results, which are validated and finalized onchain. This structure allows NCNs to adopt models ranging from fixed epochs to flexible, event-driven consensus.

Key steps for this include:

- **NCN Configurations:** The NCN admin registers the NCN through the restaking program and separately configures the associated onchain program and accounts. This may differ from NCN to NCN. For example, here are some configurations from the flagship TipRouter NCN:
  - `tie_breaker_admin`: The admin authorized to update the tie breaker mechanism or parameters.
  - `valid_slots_after_consensus`: Number of slots after consensus is reached where votes are still accepted
  - `epochs_before_stall`: Number of epochs to wait after consensus is reached before epoch accounts can be closed.
  - `epochs_after_consensus_before_close`: Number of epochs without reaching consensus before the cycle is considered stalled.
- **Snapshot**: The NCN admin initiates a new consensus cycle by triggering the snapshot phase, which creates a frozen, point-in-time view of the network's state. During this phase, the system captures and records several critical pieces of information: the current set of active operators, their associated vault delegations, and the weighted stake distribution across the network.

For example, the Tip Router NCN handles snapshotting as follows: This snapshot process involves creating an `EpochSnapshot` account to track the total stake weight and participant counts, individual `OperatorSnapshot` accounts for each operator to record their specific delegated stake weights, and a `WeightTable` that freezes the voting weights for each supported token. By locking this configuration at the start of the cycle, the system ensures that all subsequent votes are based on a consistent, immutable view of the network's state, preventing any manipulation of voting power through strategic stake movements or delegation changes during the active voting period.

Note: While this applies to the Tip Router and the default Template NCN, other NCNs may define different snapshot mechanisms or parameter names depending on their specific design.

- **Offchain Execution**: Operators execute the offchain service and protocol designed by the NCN. Examples include generating ZK proofs, querying external APIs, performing computations, or validating event data. In some cases, this step also involves submitting data of the results onchain or offchain from which it can be read for vote submissions.
- **Vote Submission**: Operators cast signed votes for specific results to the NCN’s onchain program. That program is responsible for tallying votes and determining whether consensus has been reached. Once consensus is finalized, it can trigger actions like reward distribution or slashing through the Restaking program.
- **Finalization and Rewarding**: Once consensus is reached, the result is sent to another program that invokes a state transition instruction for the NCN i.e. updating accounts, distributing rewards, etc. If successful, rewards are distributed to operators based on stake-weight and other logic that is codified by the NCN.
- **Slashing Enforcement (In Current Development)**: When operators fail to meet their obligations, whether through missed votes, invalid data submission, or malicious behavior, their stake is slashed by a designated slasher (set by the NCN) can submit a slashing request. The slashing mechanism ensures that economic incentives remain properly aligned throughout the system. (The slashing program is not currently live).

The operation phase is where the network’s value is produced. While all service execution happens offchain, consensus protocol and incentives are preserved onchain, creating a scalable and trust-minimized decentralized protocol.

## Critical Concepts

Jito (Re)staking introduces architectural patterns that enable scalable, secure coordination for independent decentralized services. These include:

### Stake-Weighted Voting and Slashing

Consensus in Jito (Re)staking is driven by submitting onchain votes on offchain execution. Voting power is backed by delegated **onchain** stake. Please note that the slashing functionality is still currently being developed.

During each epoch:

- Operators perform offchain services (e.g. computation, validation, data retrieval)
- They submit signed votes onchain based on their results
- These votes are weighted by how much stake they are delegated from vaults
- Finalized results require reaching a threshold of weighted agreement
- Once slashing is live, NCNs can implement custom slashing logic for missed votes or malicious behavior

### Modular and Permissioned Participation

All connections between NCNs, vaults, and operators are explicitly approved by each party—no one is auto-connected. This mutual opt-in model guarantees intentional coordination, while enabling a modular topology:

- Operators can serve multiple NCNs
- Vaults can allocate and rebalance stake freely
- NCNs evolve independently with custom rules and economic structures

### Maintenance Phase: Keeping the Network Healthy

At the end of each epoch, administrative and system-level tasks are performed to keep the NCN aligned and efficient. **A brief list includes**:

- **Account Cleanup**: To minimize rent costs, temporary accounts like vote records and delegation states are closed at epoch boundaries, reclaiming SOL for reuse.
- **Operator and Vault Updates**: The NCN admin can add or remove operators and vaults, and adjust minimum quorum thresholds. In addition, operators and vaults have the ability to opt out or move between NCNs.
- **Token Weight and Allocation Adjustments**: NCNs can refresh or rebalance token weights through their own onchain program. Vault admins can update stake allocations across NCNs or shift delegations between operators.
- **Dynamic Configuration Changes**: NCN parameters, including reward splits, penalty thresholds, and voting logic, can be revised over time, allowing the network to evolve, upgrade, and adapt to changing usage patterns.

This ongoing maintenance ensures the NCN remains flexible and cost-efficient. Administrative actions are coordinated through permissioned onchain records, making changes transparent and auditable. Additional improvements around cross-epoch delegation efficiency are being explored in the development roadmap.

## Addresses

| Network | Program   | Address                                     | Version |
| ------- | --------- | ------------------------------------------- | ------- |
| Mainnet | Restaking | RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q | 0.0.5   |
| Mainnet | Vault     | Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8 | 0.0.5   |
| Testnet | Restaking | RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q | 0.0.5   |
| Testnet | Vault     | Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8 | 0.0.5   |
| Devnet  | Restaking | RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q | 0.0.5   |
| Devnet  | Vault     | Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8 | 0.0.5   |

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE.md](https://github.com/jito-foundation/restaking/blob/master/LICENSE) file for details.
