---
title: Jito (Re)Staking Overview
category: Jekyll
layout: post
weight: 1
---

At its core, **Jito (Re)Staking** is a framework to build decentralized protocols using onchain consensus and any SPL token as economic security. In other words, it’s a framework for developers to use the Solana runtime for any arbitrary proof of stake system and accelerate decentralization i.e. you can build mini-Solanas on Solana!

The protocol involves three main personas: Node Consensus Networks (NCNs), Vaults, and Operators, and the Restaking program and the Vault Program act as the coordination layer between each persona, making state transitions transparent, auditable, and enforceable. Altogether, it’s a platform where (1) NCN developers can build and register decentralized services, and (2) Vaults (stakers) and Node Operators can discover and participate in said services.

![img.png](/assets/images/ncn.png)

### Onchain Coordination, Offchain Execution

Jito (Re)Staking is an onchain registry of services, stakers, and node operators, on Solana. The defining feature of Jito restaking is: services are executed offchain, and consensus is maintained and enforced onchain.

- All core activity (computation, validation, data collection) happens offchain (like a cloud service).
- All coordination (stake delegation, voting, slashing, rewards) is tracked, maintained, and enforced onchain.

This split enables scalability and flexibility for developers while retaining cryptoeconomic guarantees from Solana’s base layer. The design makes it much easier for projects to access and bootstrap distributed networks of economically aligned operators and stakers to provide security and consensus, without needing to build infrastructure from scratch or distribute extremely high emissions. Effectively, this model creates a more efficient and cost effective security model (i.e. one set of staked tokens can secure multiple services) and unlocks more effective capital allocation toward core business development.

## Core Components

### Node Consensus Network (NCN)

An NCN is a decentralized service or network that reaches onchain consensus on offchain data or workloads. This may include oracles, DePIN services, bridges, co-processors, or new chains. The NCN is registered onchain using the Jito (Re)Staking program (`RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q`) and interacts with the Vault Program (`Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8`) to source security from staked SPL tokens without needing to build a validator set or depend on a native token.

### Vaults

Vaults serve as deposit pools that hold staked tokens (e.g. JitoSOL) and issue vault receipt tokens (VRTs) representing those positions. Vaults opt into supporting specific NCNs and delegate stake to approved operators. Users retain ownership of their stake via VRTs.

Each vault defines key parameters, including how much stake is allocated to each node operator. Vaults also define cooldown periods for withdrawing stake, which apply at the vault level.

### Operators

Operators are infrastructure providers that run the offchain services for NCNs and either hold or are delegated stake. They opt in to serve specific NCNs and receive stake from approved vaults. Since their stake and subsequent rewards are at risk, they hold an economic incentivize to behave as expected. Operators can serve multiple NCNs simultaneously, enabling efficient resource usage and shared security.

### The Opt-In Handshake

Participation in Jito (Re)Staking is governed on-chain by mutual consent:

1. NCNs register onchain and approve operators and vaults to participate in their network
2. Operators opt in to NCNs and must be accepted.
3. Vaults opt in to NCNs and delegate to approved operators.

This handshake guarantees that vaults, operators, or NCNs are not forced into any connection. All actively staked links are explicitly approved, creating a modular and flexible system for stake delegation and service coordination.

### Why Jito (Re)Staking Matters

We built the Jito (Re)Staking protocol because there are aspects of Jito Network that can benefit from incremental decentralization. And as the Solana ecosystem continues to mature, we expect other developers will eventually transition to prioritizing resiliency over product iteration speed and seek to build custom decentralized solutions that fits the needs of their protocol. The primary benefits of using the restaking protocol to bootstrap decentralized protocol include:

- **Wide Distribution**: Jito Network is deeply integrated with Solana’s existing validator set and stakers. NCNs can immediately tap into Jito’s network effects without having to kickstart their own validator set or attract native stake from scratch.
- **Cost Effective**: NCNs don’t need to leverage native tokens to emit high inflationary rewards or lock staked tokens to bootstrap stakers or validators. Instead, by registering with the Jito (Re)Staking framework, they tap into existing stakers, token holders, and validators on Solana—all of which have already underwritten costs of capital. .
- **Capital efficiency**: The same stake can secure multiple services. The same operators can operate multiple services.
- **Aligned incentives**: Stakers, operators, and NCN developers all benefit from performance, transparency, and modular security.
- **Solana’s feature set**: Solana offers developers a unique combination of high-throughput, low-latency execution with sub-second finality, which is ideal for protocols requiring fast state updates. The platform enables parallel transaction processing (via Sealevel runtime), maintains composability within a single runtime environment and provides developer-friendly tools like the Anchor framework that streamline smart contract development.

Jito (Re)Staking greatly reduces the friction to launch, or transition existing services into, decentralized protocols with proof of stake security rooted on Solana.

## Lifecycle of a Node Consensus Network (NCN)

Once an NCN is initialized, it operates in continuous cycles which are coordinated across three distinct phases: setup, operation, and maintenance. This lifecycle involves initializing the network and its participants, running its offchain tasks, enforcing onchain accountability, and evolving over time based on the demands of the NCN. Each phase involves actions by multiple parties: NCN admins, vault controllers, and node operators.

### Setup Phase: Building the Foundation

The setup phase establishes the NCN’s identity, rules, and participants.

- **NCN Registration**: The NCN admin initializes an NCN account onchain by calling the restaking program, requiring both the NCN admin key and a base NCN key. This account serves as an entry in the registry and does not store detailed network configurations. Instead, configurations like accepted tokens, slashing conditions, quorum thresholds, and operator requirements are managed separately, typically via the NCN's own onchain program, CLI, or off-chain enforcement logic.
- **Vault and Operator Onboarding**: Vaults and Operators must each explicitly opt into the NCN, and the NCN must in turn approve them. Before establishing these relationships, vaults and operators must first register through the **Vault Program** and **Restaking Program**, respectively. Once registered, they can initiate connections to a specific NCN.

Once approved by the NCN, the admins should call a warm-up instruction to activate the connections. The stake becomes active once all three components initiate and warm up the connections. This opt-in approval process ensures that all active stake delegations are mutual and intentional.

- **Stake Delegation**: After the warm-up period, vaults can delegate stake to the operator. Stake becomes active immediately. However, when a user withdraws their stake, it must undergo a cooldown period that lasts for one full NCN-defined epoch.

By the completion of this setup phase, the NCN has established its security parameters, approved operators, and activated its initial stake. This foundation sets the stage for the operations phase, and different dynamics come into play.

### Operations Phase: Running the Network

Each NCN progresses through consensus cycles, which may be time-bound to epoch lengths or follow a custom logic defined by the NCN admin. Within each cycle, operators perform offchain tasks and submit results, which are validated and finalized onchain. This structure allows NCNs to adopt models ranging from fixed epochs to flexible, event-driven consensus.

Key steps for this include:

- **NCN Configurations:** The NCN admin initiates the NCN program configurations by setting the important parameters such as:
  - `tie_breaker_admin`: The admin authorized to update the tie breaker mechanism or parameters.
  - `valid_slots_after_consensus`: Number of slots after consensus is reached where votes are still accepted
  - `epochs_before_stall`: Number of epochs to wait after consensus is reached before epoch accounts can be closed.
  - `epochs_after_consensus_before_close`: Number of epochs without reaching consensus before the cycle is considered stalled.
  - `epochs_before_stall`: Number of epochs to wait after consensus is reached before epoch accounts can be closed.
- **Snapshot**: The NCN admin initiates a new consensus cycle by triggering the snapshot phase, which creates a frozen, point-in-time view of the network's state. During this phase, the system captures and records several critical pieces of information: the current set of active operators, their associated vault delegations, and the weighted stake distribution across the network.

This snapshot process involves creating an `EpochSnapshot` account to track the total stake weight and participant counts, individual `OperatorSnapshot` accounts for each operator to record their specific delegated stake weights, and a `WeightTable` that freezes the voting weights for each supported token. By locking this configuration at the start of the cycle, the system ensures that all subsequent votes are based on a consistent, immutable view of the network's state, preventing any manipulation of voting power through strategic stake movements or delegation changes during the active voting period.

- **Offchain Execution**: Operators execute the offchain service and protocol assigned by the NCN. Examples include generating ZK proofs, querying external APIs, performing computations, or validating event data. In some cases, this step also involves submitting data of the results onchain or offchain from which it can be read for vote submissions.
- **Vote Submission**: Operators cast signed votes for specific results to the NCN’s onchain program. That program is responsible for tallying votes and determining whether consensus has been reached. Once consensus is finalized, it can trigger actions like reward distribution or slashing through the Restaking program.
- **Finalization and Rewarding**: Once consensus is reached, the result is sent to another program that invokes a state transition instruction for the NCN i.e. updating accounts, distributing rewards, etc. If successful, rewards are distributed to operators based on stake-weight and other logic that is codified by the NCN.
- **Slashing Enforcement**: When operators fail to meet their obligations, whether through missed votes, invalid data submission, or malicious behavior, their stake is slashed by a designated slasher (set by the NCN) can submit a slashing request.. The slashing mechanism ensures that economic incentives remain properly aligned throughout the system. (The slashing program is not currently live).

The operation phase is where the network’s value is produced. While all service execution happens offchain, consensus protocol and incentives are preserved onchain, creating a scalable and trust-minimized decentralized protocol .

### Maintenance Phase: Keeping the Network Healthy

At the end of each epoch, administrative and system-level tasks are performed to keep the NCN aligned and efficient. **A brief list includes**:

- **Account Cleanup**: To minimize rent costs, temporary accounts like vote records and delegation states are closed at epoch boundaries, reclaiming SOL for reuse.
- **Operator and Vault Updates**: The NCN admin can add or remove operators and vaults, and adjust minimum quorum thresholds. In addition, operators and vaults have the ability to opt out or move between NCNs.
- **Token Weight and Allocation Adjustments**: NCNs can refresh or rebalance token weights through their own onchain program. Vault admins can update stake allocations across NCNs or shift delegations between operators.
- **Dynamic Configuration Changes**: NCN parameters, including reward splits, penalty thresholds, and voting logic, can be revised over time, allowing the network to evolve, upgrade, and adapt to changing usage patterns.

This ongoing maintenance ensures the NCN remains flexible and cost-efficient. Administrative actions are coordinated through permissioned onchain records, making changes transparent and auditable. Additional improvements around cross-epoch delegation efficiency are being explored in the development roadmap.

## Key Roles and Responsibilities

While the previous sections outline the architecture and lifecycle of NCNs, this section focuses on the organizational roles behind the system. Each persona (whether they’re launching a network, managing capital, running infrastructure, or providing stake) has clearly defined administrative capabilities and responsibilities. This alignment is central to how Jito (Re)Staking ensures trust and coordination in a modular, multi-party environment.

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

Operators, such as existing Solana validators, run nodes who opt in to run NCN-specific offchain workloads. They are rewarded based on performance which can include uptime, correctness, and participation, subject to the NCN. On top of this, they are penalized for underperformance or misconduct Penalties may include losing stake delegations or connections or getting slashed.

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

## Critical Concepts

Jito (Re)Staking introduces architectural patterns that enable scalable, secure coordination for independent decentralized services. These include:

### Stake-Weighted Voting and Slashing

Consensus in Jito (Re)Staking is driven by submitting onchain votes on offchain execution. Voting power is backed by delegated **onchain** stake. During each epoch:

- Operators perform offchain services (e.g. computation, validation, data retrieval)
- They submit signed votes onchain based on their results
- These votes are weighted by how much stake they are delegated from vaults
- Finalized results require reaching a threshold of weighted agreement
- NCNs can implement custom slashing logic for missed votes or malicious behavior

### Modular and Permissioned Participation

All connections between NCNs, vaults, and operators are explicitly approved by each party—no one is auto-connected. This mutual opt-in model guarantees intentional coordination, while enabling a modular topology:

- Operators can serve multiple NCNs
- Vaults can allocate and rebalance stake freely
- NCNs evolve independently with custom rules and economic structures

## Addresses

| Network | Program   | Address                                     | Version |
| ------- | --------- | ------------------------------------------- | ------- |
| Mainnet | Restaking | RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q | 0.0.2   |
| Mainnet | Vault     | Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8 | 0.0.2   |
| Testnet | Restaking | RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q | 0.0.2   |
| Testnet | Vault     | Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8 | 0.0.2   |
| Devnet  | Restaking | RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q | 0.0.2   |
| Devnet  | Vault     | Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8 | 0.0.2   |

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE.md](https://github.com/jito-foundation/restaking/blob/master/LICENSE) file for details.
