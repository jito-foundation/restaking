---
title: NCN Implementation Guide
category: Jekyll
layout: post
weight: 1
---

## Introduction

The Node Consensus Network (NCN) is a robust blockchain consensus system built on Solana. It enables network participants to agree on critical network decisions using a secure, stake-weighted voting mechanism. This system utilizes Jito's restaking infrastructure, allowing operators with delegated tokens to vote on network parameters and states.

This tutorial will focus on a [pre-built NCN program](https://github.com/Unboxed-Software/ncn-program-template) that acts like a template or base that you can use to create your own NCN program. To help you understand how it works, we will walk through building a simulation test that covers the majority of its setup and functionality. We do not recommend most NCN developers build an NCN from scratch. Rather, we suggest using this prebuilt program as a starting point and customizing it according to your needs.

By following the simulation test setup in this guide, you will gain hands-on experience with the entire NCN lifecycle: initializing vaults and operators using Jito's restaking and vault programs, configuring the NCN program, and executing the full voting process.

### The purpose of NCNs

Decentralized networks require reliable mechanisms for participants to reach consensus without central authorities. The NCN addresses this need by:

1. Providing a secure voting framework where influence is proportional to the amount of stake held.
2. Supporting multiple token types with configurable voting weights, allowing flexibility in how voting power is assigned.
3. Creating verifiable and immutable records of consensus decisions on the blockchain.
4. Establishing a solid foundation for network governance and parameter setting.

### NCN components

To run an NCN, you need one or more of each of the following three components, which interact with each other: Vaults, Operators, and the NCN Program itself.

#### 1. Vaults

Vaults are accounts that hold tokens and delegate them to operators. They play a crucial role in the NCN by:

1. Holding the tokens used for staking.
2. Delegating stake (voting power) to chosen operators.
3. Enabling stake-weighted participation in the network's governance.

#### 2. Operators

Operators are accounts that receive delegated stake from vaults and actively participate in the voting process. Their key functions are:

1. Receiving stake delegations from one or more vaults.
2. Casting votes on behalf of the delegated stake during consensus rounds.
3. Forming the network of active participants who drive the consensus process.

#### 3. NCN program

The NCN Program is the core on-chain component of the system. It's the smart contract that NCN developers build and deploy. Its main responsibilities are:

1. Storing the global configuration parameters for the NCN instance.
2. Maintaining the registry of participating vaults and supported token types.
3. Managing the state for each voting epoch (consensus cycle).

### NCN Lifecycle

The Node Consensus Network operates in a well-defined lifecycle that consists of three main phases:

1. **Initial Setup (One-time)**: This phase involves establishing the foundational infrastructure of the NCN. It includes:

   - Configuring the NCN parameters
   - Initializing the vault registry
   - Registering supported token types and assigning weights

     The initial setup is performed only once when the NCN is first deployed, with occasional administrative updates as needed (such as adjusting token weights or adding new supported tokens).

2. **Snapshotting (Recurring)**: At the beginning of each consensus cycle (epoch), the system captures the current state of all participants:

   - Creating epoch state and weight tables
   - Taking snapshots of operator stake weights
   - Recording vault-operator delegations
   - Calculating total voting power distribution

     This phase ensures that voting is based on a consistent, point-in-time view of the network, preventing manipulation during the voting process.

3. **Voting (Recurring)**: After snapshotting is complete, operators can cast their votes:
   - Operators submit their choices (e.g., weather status)
   - Votes are weighted according to the operator's stake

## Get to know the program template

Our example NCN Program facilitates consensus on a simple "weather status" using a stake-weighted voting mechanism. It operates in distinct time periods called epochs (your NCN's epochs do not have to be equivalent to a Solana epoch). The program uses a weight-based system to determine the influence (voting power) of different operators. Consensus is achieved when votes representing at least 66% of the total participating stake weight agree on the same outcome (ballot).

### Key components

The program uses several types of accounts:

1.  **Global Accounts**: Initialized once at the start and updated infrequently.
    - **[`Config`](#config)**: Stores global settings like epoch timing parameters (`epochs_before_stall`, `epochs_after_consensus_before_close`) and voting validity periods (`valid_slots_after_consensus`).
    - **[`VaultRegistry`](#vaultregistry)**: Manages the list of registered vaults and the different types of stake tokens (mints) the NCN supports.
    - **[`AccountPayer`](#accountpayer)**: An empty PDA account used to hold SOL temporarily for paying rent during account creation or reallocation.
2.  **Per-Consensus Cycle Accounts**: Initialized at the beginning of each epoch and usually closed shortly after the cycle ends.
    - **[`WeightTable`](#weighttable)**: Stores the specific voting weights assigned to different stake tokens for the current epoch.
    - **[`EpochState`](#epochaccountstatus)**: Tracks the status and progress of the current epoch's consensus cycle.
    - **[`BallotBox`](#ballotbox)**: Handles the collection and stake-weighted tallying of votes for the current epoch's decision (e.g., weather status).
    - **[`EpochSnapshot`](#epochsnapshot)**: Captures the state of stake delegations at the beginning of the epoch to ensure consistent voting weights throughout the cycle.
    - **[`OperatorSnapshot`](#operatorsnapshot)**: Records each operator's total stake weight and delegation breakdown for the current epoch.
    - **[`ConsensusResult`](#consensusresult)**: Stores the final outcome (the winning ballot and associated details) for the completed epoch.
    - **[`EpochMarker`](#epochmarker)**: A marker account created when all temporary accounts for an epoch have been successfully closed.
3.  **Component Structures**: These are not separate accounts but important data structures used within the accounts above.
    - **[`Ballot`](#ballot)**: Represents a single potential outcome in the consensus process.
    - **[`BallotTally`](#ballottally)**: Aggregates votes and stake weight for a specific ballot.
    - **[`OperatorVote`](#operatorvote)**: Records a vote cast by a single operator.
    - **[`VaultOperatorStakeWeight`](#vaultoperatorstakeweight)**: Tracks the weighted stake from a specific vault to an operator.
    - **[`StMintEntry`](#stmintentry)**: Represents a supported token mint and its voting weight in the VaultRegistry.
    - **[`VaultEntry`](#vaultentry)**: Represents a registered vault in the VaultRegistry.

### Weather status system

The goal of the NCN program is to come to consensus on the weather in Solana Beach. For the purposes of keeping this tutorial simple, our weather statuses are as follows:

1. **Sunny (0)**: Represents clear, sunny weather.
2. **Cloudy (1)**: Represents cloudy weather conditions.
3. **Rainy (2)**: Represents rainy weather conditions.

Operators vote on these status values. The program tallies the votes, weighting each vote by the operator's associated stake weight, to determine the final consensus result. Leveraging the final result of this NCN, we can build onchain programs whose behavior is dependent on the weather in Solana Beach.

### Consensus mechanism

The consensus process follows these steps:

1. Operators cast votes, choosing a specific weather status (Sunny, Cloudy, or Rainy).
2. Each vote's influence is determined by the operator's total stake weight, calculated from delegations received.
3. Votes are collected and tallied within the `BallotBox` account for the current epoch.
4. Consensus is reached when one weather status receives votes representing ≥66% of the total stake weight participating in that epoch.
5. The final consensus result (winning status, total weight supporting it, etc.) is recorded in the `ConsensusResult` account.

### Onchain program overview

The onchain program is written in Rust (without using the Anchor framework) and consists of several instructions that can be called to perform various actions within the NCN. The instruction logic resides in the `/program` directory, while shared core logic is located in the `/core` directory.

The instructions are broadly categorized:

1.  **Admin Instructions**: These require administrator privileges and are used for initial setup and configuration.
    - `admin_initialize_config`: Initializes the main `Config` account.
    - `admin_register_st_mint`: Registers a new type of stake token (ST) the NCN will support.
    - `admin_set_new_admin`: Transfers administrative control to a new keypair.
    - `admin_set_parameters`: Updates parameters within the `Config` account.
    - `admin_set_st_mint`: Updates details for an existing supported token mint (Deprecated/Redundant? Check `admin_register_st_mint` and `admin_set_weight`).
    - `admin_set_tie_breaker`: Configures the tie-breaking mechanism or authority.
    - `admin_set_weight`: Sets or updates the voting weight for a specific supported token mint.
2.  **Permissionless Keeper Instructions**: These are permissionless instructions, meaning anyone can call them to advance the state of the NCN, typically moving between epoch phases. They ensure the NCN progresses correctly.
    - `initialize_epoch_state`: Creates the `EpochState` account for a new epoch.
    - `initialize_vault_registry`: Creates the initial `VaultRegistry` account.
    - `realloc_vault_registry`: Increases the size of the `VaultRegistry` account, to reach the desired size. Solana has a limitation when it comes to the size of the account that you can allocate in one call, so when you have a larger account, you will need to call realloc on it multiple times to reach the desired size.
    - `initialize_weight_table`: Creates the `WeightTable` account for an epoch.
    - `realloc_weight_table`: Increases the size of the `WeightTable` account.
    - `initialize_epoch_snapshot`: Creates the main `EpochSnapshot` account.
    - `initialize_operator_snapshot`: Creates an `OperatorSnapshot` account for a specific operator within an epoch.
    - `set_epoch_weights`: Populates the `WeightTable` with weights from the `VaultRegistry`.
    - `snapshot_vault_operator_delegation`: Records the weighted stake from a specific vault delegation into the relevant `OperatorSnapshot`.
    - `initialize_ballot_box`: Creates the `BallotBox` account for voting in an epoch.
    - `realloc_ballot_box`: Increases the size of the `BallotBox` account.
    - `register_vault`: Registers a vault (that has already been approved via Jito handshake) with the NCN program's `VaultRegistry`.
    - `close_epoch_account`: Closes temporary epoch-specific accounts (like `EpochState`, `BallotBox`, etc.) after they are no longer needed, reclaiming rent.
3.  **Operator Instruction**: This is the primary action taken by participants during a consensus cycle.
    - `cast_vote`: Allows an operator (using their admin key) to submit their vote for the current epoch.

For more details, you can always check the source code or the API documentation [here](https://github.com/Unboxed-Software/ncn-program-template).

## Build and run the simulation test

This section will walk through building a simulation test of our example NCN program. The test represents a comprehensive scenario designed to mimic a complete NCN system. It involves multiple operators, vaults, and different types of tokens. The test covers the entire workflow, from the initial setup of participants and the NCN program itself, through the voting process, and finally to reaching and verifying consensus. It heavily utilizes Jito's restaking and vault infrastructure alongside the custom NCN voting logic.

The NCN program used can be found [here](https://github.com/Unboxed-Software/ncn-program-template). By creating a simulation test of this NCN, you'll be better prepared to use it as a template or base that you can adapt to create your own NCN program. Just a reminder: we do not recommend most NCN developers build their NCN from scratch. Rather, we suggest using this prebuilt program as a starting point and customizing it according to your needs.

The simulation test we'll be creating below can also be found in the [example NCN repository](https://github.com/Unboxed-Software/ncn-program-template). However, you'll understand the system better if you write the test along with us, so feel free to clone the repository, delete the test file `./integration_tests/test/ncn_program/simulation_test.rs`, and follow along. This will give you hands-on experience with the entire NCN lifecycle: initializing vaults and operators using Jito's restaking and vault programs, configuring the NCN program, and executing the full voting process.

### Prerequisites

Before running the simulation test, ensure you have completed the following setup steps:

1.  Build the NCN onchain program using Cargo: `cargo build-sbf --manifest-path program/Cargo.toml --sbf-out-dir integration_tests/tests/fixtures`
2.  Ensure you have the correct versions installed:
    - Solana CLI: 2.2.6 (recommended)
    - Rust/Cargo: 1.81 or newer

### Building the Simulation Test

Let's build the simulation test step by step.

#### 1. Create a new file

You can start with a blank file. Create a new file named `simulation_test.rs` inside the `integration_tests/tests` folder. Copy and paste the following boilerplate code at the bottom of your test function:

```rust
#[cfg(test)]
mod tests {
    use crate::fixtures::{test_builder::TestBuilder, TestResult};
    use jito_restaking_core::{config::Config, ncn_vault_ticket::NcnVaultTicket};
    use ncn_program_core::{ballot_box::WeatherStatus, constants::WEIGHT};
    use solana_sdk::{msg, signature::Keypair, signer::Signer};

    #[tokio::test]
    async fn simulation_test() -> TestResult<()> {
        // YOUR TEST CODE WILL GO HERE
        // 2. ENVIRONMENT SETUP

        // 3. NCN SETUP

        // 4. OPERATORS AND VAULTS SETUP

        // 5. NCN PROGRAM CONFIGURATION

        // 6. Epoch Snapshot and Voting Preparation

        // 7. VOTING

        // 8. VERIFICATION

        // 9. CLEANUP

        Ok(())
    }
}
```

Unless otherwise specified, all of the code snippets provided in this guide represent code that should go inside the `simulation_test` test function, in the order provided.

Next, you need to make this new test discoverable. Copy and paste the following line into the `integration_tests/tests/mod.rs` file to declare the new module:

```rust
// integration_tests/tests/mod.rs
mod simulation_test;
```

Now, you can run this specific test using the following command:

```bash
SBF_OUT_DIR=integration_tests/tests/fixtures cargo test -p ncn-program-integration-tests --test tests simulation_test
```

This command targets the `ncn-program-integration-tests` package and runs only the `simulation_test` test function. If you want to run all tests in the suite, simply remove the test name filter (`-p ncn-program-integration-tests --test tests simulation_test`) from the command.

Currently, the test will pass because it doesn't contain any logic yet. You should see output similar to this:

```bash
running 1 test
test ncn_program::simulation_test::tests::simulation_test ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 54 filtered out; finished in 0.00s
```

#### 2. Environment Setup

The first step within our test function is to set up the testing environment using the `TestBuilder`. Copy and paste the following code at the bottom of your test function:

```rust
let mut fixture = TestBuilder::new().await;
```

The `TestBuilder` is a test utility that encapsulates and simplifies the setup process for NCN program testing. It provides:

1. A local test validator environment with pre-loaded programs
2. Clients for interacting with the NCN, Vault, and Restaking programs
3. Helper methods for common operations (creating operators, vaults, advancing clock time)
4. Management of test accounts, keypairs, and token mints

This and other utility functions (like `add_operators_to_test_ncn`, `add_vaults_to_test_ncn`) abstract away much of the complex, repetitive setup code, allowing tests to focus on the specific behaviors being verified rather than boilerplate infrastructure.

Since we are running this test locally against a test ledger, we need to initialize the Jito Restaking and Vault programs on the ledger. In a real network environment (devnet, mainnet), these programs would already be deployed and configured.

Copy and paste the following code at the bottom of your test function:

```rust
fixture.initialize_restaking_and_vault_programs().await?;
```

Finally, let's prepare some client objects and configuration variables we'll use throughout the test.

Copy and paste the following code at the bottom of your test function:

```rust
let ncn_program_client = fixture.ncn_program_client();
let vault_program_client = fixture.vault_client();
let restaking_client = fixture.restaking_program_client();

// Define test parameters
const OPERATOR_COUNT: usize = 13; // Number of operators to simulate
let mints = vec![
   (Keypair::new(), WEIGHT),     // Alice: Base weight
   (Keypair::new(), WEIGHT * 2), // Bob: Double weight
   (Keypair::new(), WEIGHT * 3), // Charlie: Triple weight
   (Keypair::new(), WEIGHT * 4), // Dave: Quadruple weight
];
let delegations = [
   1,                  // Minimum delegation amount (e.g., 1 lamport)
   10_000_000_000,     // 10 tokens (assuming 9 decimals)
   100_000_000_000,    // 100 tokens
   1_000_000_000_000,  // 1,000 tokens
   10_000_000_000_000, // 10,000 tokens
];
```

This code does the following:

1.  Gets client handles for interacting with the NCN, Vault, and Restaking programs.
2.  Defines `OPERATOR_COUNT` to specify how many operators we'll create.
3.  Sets up `mints`: a list of keypairs representing different SPL token mints and their corresponding voting weights. We use different weights to test the stake-weighting mechanism. `WEIGHT` is likely a constant representing the base unit of weight.
4.  Defines `delegations`: an array of different token amounts (in lamports, assuming 9 decimals for typical SPL tokens) that vaults will delegate to operators.

#### 3. NCN Setup

Now, let's create the NCN account using the Jito Restaking program. The `create_test_ncn` helper function handles the necessary instruction calls.

Copy and paste the following code at the bottom of your test function:

```rust
let mut test_ncn = fixture.create_test_ncn().await?;
let ncn_pubkey = test_ncn.ncn_root.ncn_pubkey;
```

This step:

- Calls the Jito Restaking program to create a new Node Consensus Network (NCN) account and its associated administrative structures.
- Stores the public key (`ncn_pubkey`) of the newly created NCN, which we'll need to interact with it later.

If you run the test at this point (`cargo test ... simulation_test`), you should see transaction logs in the output, indicating that the NCN creation instructions were executed successfully.

#### 4. Operators and Vaults Setup

This phase is crucial for simulating a realistic network. We will create the operators who vote and the vaults that provide the stake (voting power).

##### 4.1 Operator Creation and NCN Connection

We'll add the specified number of operators (`OPERATOR_COUNT`) to our NCN using another helper function.

Copy and paste the following code at the bottom of your test function:

```rust
fixture
   .add_operators_to_test_ncn(&mut test_ncn, OPERATOR_COUNT, Some(100))
   .await?;
```

This `add_operators_to_test_ncn` function performs several actions by calling instructions in the Jito Restaking program:

- Creates `OPERATOR_COUNT` (13 in our case) separate operator accounts.
- Sets an optional operator fee (here, 100 basis points = 1%).
- Establishes a secure, bidirectional "handshake" between each newly created operator and the NCN.

The handshake process involves multiple steps:

1.  Creating the operator account itself, managed by its unique admin keypair.
2.  Initializing the state that tracks the relationship between the NCN and the operator (`do_initialize_ncn_operator_state`).
3.  Warming up the connection from the NCN's perspective (`do_ncn_warmup_operator`).
4.  Warming up the connection from the operator's perspective (`do_operator_warmup_ncn`).

For more information on this, please read the guide [here](/)

This handshake is essential for security. It ensures that operators must explicitly connect to the NCN (and vice-versa) and potentially wait through an activation period before they can participate in voting.

##### 4.2 Vault Creation

Next, we create vaults to hold the different types of tokens we defined earlier. We'll distribute them across the token types.
Note that you can have more than one vault with the same ST Mint (Support Token Mint).

Copy and paste the following code at the bottom of your test function:

```rust
// Create vaults associated with different token mints
{
    // Create 3 vaults for Alice (base weight)
    fixture
        .add_vaults_to_test_ncn(&mut test_ncn, 3, Some(mints[0].0.insecure_clone()))
        .await?;
    // Create 2 vaults for Bob (double weight)
    fixture
        .add_vaults_to_test_ncn(&mut test_ncn, 2, Some(mints[1].0.insecure_clone()))
        .await?;
    // Create 1 vault for Charlie (triple weight)
    fixture
        .add_vaults_to_test_ncn(&mut test_ncn, 1, Some(mints[2].0.insecure_clone()))
        .await?;
    // Create 1 vault for Dave (quadruple weight)
    fixture
        .add_vaults_to_test_ncn(&mut test_ncn, 1, Some(mints[3].0.insecure_clone()))
        .await?;
}
```

The `add_vaults_to_test_ncn` helper function orchestrates calls to both the Jito Vault and Jito Restaking programs to:

- Create a total of 7 vaults (3 + 2 + 1 + 1).
- Associate each group of vaults with one of our predefined token mints (`mints[0]`, `mints[1]`, etc.).
- Initialize the vault accounts using the Jito Vault program (setting zero fees, which is common for testing).
- Mint tokens for the vaults if needed (though here we provide the mints).
- Establish bidirectional handshakes "Tickets" between each vault and the NCN using specific Jito Restaking instructions (`do_initialize_ncn_vault_ticket`, `do_warmup_ncn_vault_ticket`).
- Establish corresponding handshakes "Tickets" using Jito Vault program instructions (`do_initialize_vault_ncn_ticket`, `do_warmup_vault_ncn_ticket`).
- Establish bidirectional handshakes "Tickets" between each new vault and _all_ existing operators using Jito Restaking (`do_initialize_operator_vault_ticket`, `do_warmup_operator_vault_ticket`) and Jito Vault (`do_initialize_vault_operator_delegation`) instructions. Note that `do_initialize_vault_operator_delegation` only sets up the _potential_ for delegation; no actual tokens are delegated yet.
- Advance the simulated clock (`fixture.advance_slots`) after handshakes "Tickets" to ensure the relationships become active, simulating the necessary waiting period.

Creating vaults with different token types allows us to test how the NCN handles varying voting power based on token weights.

##### 4.3 Delegation Setup

This is where vaults actually delegate their tokens (stake) to operators, granting them voting power. We'll iterate through operators and vaults to create delegations.

Copy and paste the following code at the bottom of your test function:

```rust
// Vaults delegate stake to operators
{
    // Iterate through all operators except the last one
    for (index, operator_root) in test_ncn
        .operators
        .iter()
        .take(OPERATOR_COUNT - 1)
        .enumerate()
    {
        // Each vault delegates to this operator
        for vault_root in test_ncn.vaults.iter() {
            // Cycle through the predefined delegation amounts
            let delegation_amount = delegations[index % delegations.len()];

            if delegation_amount > 0 {
                // Call the Vault program to add the delegation
                vault_program_client
                    .do_add_delegation(
                        vault_root, // The vault delegating
                        &operator_root.operator_pubkey, // The operator receiving
                        delegation_amount, // The amount to delegate
                    )
                    .await
                    .unwrap();
            }
        }
    }
}
```

The delegation process is where voting power is established. Each vault delegates tokens to operators, which determines:

1. How much voting power each operator has
2. How token weights multiply that power
3. The distribution of influence across the network

Key aspects of the delegation setup:

- Every vault delegates to every operator (except the last one for this example)

* Note that vaults can choose whom to delegate to, they don't have to delegate to all operators

- Delegation amounts cycle through the `delegations` array to test different scenarios
- The last operator intentionally receives zero delegation to test the system's handling of operators without stake
- The delegation is performed directly through the vault program using `do_add_delegation` which will call a specific instruction in the vault program to do that

Each operator accumulates voting power from all the different delegations they receive. The total voting power for an operator is the sum of the weighted values of each delegation.

**Example:**

- Vault A (holding Alice, weight W) delegates 100 tokens to Operator X. Power contribution: 100 \* W.
- Vault B (holding Bob, weight 2W) delegates 50 tokens to Operator X. Power contribution: 50 _ 2W = 100 _ W.
- Operator X's total voting power would be (100 _ W) + (50 _ 2W) = 200 \* W.

This distributed delegation model enables testing complex scenarios where:

- Operators have vastly different amounts of influence.
- Tokens with higher weights contribute disproportionately more voting power.
- The distribution of delegations affects consensus outcomes.

The deliberate omission of delegation to the last operator creates a control case to verify that operators with zero stake cannot influence the voting process, which is a critical security feature.

You can run the test now and see the output.

##### 4.4 Delegation Architecture and Voting Power Calculation

The delegation architecture follows a multiplication relationship:

- **Operator Voting Power = Sum of (Delegation Amount × Delegated Token's Weight)**

Each operator accumulates voting power from all the different delegations they receive. The total voting power for an operator is the sum of the weighted values of each delegation.

**Example:**

- Vault A (holding TKN1, weight W) delegates 100 tokens to Operator X. Power contribution: 100 \* W.
- Vault B (holding TKN2, weight 2W) delegates 50 tokens to Operator X. Power contribution: 50 _ 2W = 100 _ W.
- Operator X's total voting power would be (100 _ W) + (50 _ 2W) = 200 \* W.

This distributed delegation model enables testing complex scenarios where:

- Operators have vastly different amounts of influence.
- Tokens with higher weights contribute disproportionately more voting power.
- The distribution of delegations affects consensus outcomes.

The deliberate omission of delegation to the last operator creates a control case to verify that operators with zero stake cannot influence the voting process, which is a critical security feature.

You can run the test now and see the output.

#### 5. NCN Program Configuration

Until now, all the code we've written uses the Jito restaking program and Jito vault program. Now we will start using the example NCN program that you will have to deploy.

The NCN Program Configuration phase establishes the on-chain infrastructure necessary for the voting and consensus mechanisms. This includes setting up configuration parameters, creating data structures, and registering the token types and vaults that will participate in the system.

##### 5.1 Program Configuration Initialization

First, we initialize the main configuration account for our NCN instance.

Copy and paste the following code at the bottom of your test function:

```rust
// Initialize the main Config account for the NCN program
ncn_program_client
    .do_initialize_config(test_ncn.ncn_root.ncn_pubkey, &test_ncn.ncn_root.ncn_admin)
    .await?;
```

This step initializes the core configuration for the NCN program with critical parameters:

- **NCN Admin**: The authority that can modify configuration settings, this admin has to be the same admin for the NCN account from Jito restaking program side.
- **Epochs Before Stall**: How many epochs before a non-completed consensus cycle is considered stalled (default: 3)
- **Epochs After Consensus Before Close**: How long to wait after consensus before closing epoch data (default: 10)
- **Valid Slots After Consensus**: How many slots votes are still accepted after consensus is reached (default: 10000)

Under the hood, this creates an `NcnConfig` account that stores these parameters and serves as the authoritative configuration for this NCN instance.

##### 5.2 Vault Registry Initialization

The vault registry account is a large one, so it is not possible to initialize it in one call due to Solana network limitations. We will have to call the NCN program multiple times to get to the full size. The first call will be an init call to the instruction `admin_initialize_vault_registry`. After that, we will call a realloc instruction `admin_realloc_vault_registry` to increase the size of the account. This will be done in a loop until the account is the correct size.

The realloc will take care of assigning the default values to the vault registry account once the desirable size is reached. In our example, we will do that by calling one function `do_full_initialize_vault_registry`. If you want to learn more about this, you can check the [source code](https://github.com/Unboxed-Software/ncn-program-template).

Copy and paste the following code at the bottom of your test function:

```rust
// Initialize the VaultRegistry account (handles potential reallocations)
ncn_program_client
    .do_full_initialize_vault_registry(test_ncn.ncn_root.ncn_pubkey)
    .await?;
```

The vault registry is a critical data structure that:

- Tracks all supported vault accounts
- Maintains the list of supported token mints (token types)
- Records the weight assigned to each token type
- Serves as the source of truth for vault and token configurations

Note that this is only initializing the vault registry. The vaults and the supported tokens will be registered in the next steps.

Check out the vault registry struct [here](#vaultregistry)

##### 5.3 Activating Relationships with Time Advancement

Next, we advance the simulation clock to ensure that all previously established handshake relationships (NCN-Operator, NCN-Vault, Operator-Vault) become active, as Jito's restaking infrastructure often includes activation periods.

Copy and paste the following code at the bottom of your test function:

```rust
// Fast-forward time to simulate a full epoch passing
// This is needed for all the relationships to get activated
let restaking_config_address =
    Config::find_program_address(&jito_restaking_program::id()).0;
let restaking_config = restaking_client
    .get_config(&restaking_config_address)
    .await?;
let epoch_length = restaking_config.epoch_length();
fixture
    .warp_slot_incremental(epoch_length * 2)
    .await
    .unwrap();
```

This section:

1. Retrieves the epoch length from the restaking program configuration
2. Advances the simulation time by two full epochs
3. Ensures all handshake relationships between NCN, operators, and vaults become active

The time advancement is necessary because Jito's restaking infrastructure uses an activation period for security. This prevents malicious actors from quickly creating and voting with fake operators or vaults by enforcing a waiting period before they can participate.

Now it is time to register the supported tokens with the NCN program and assign weights to each mint for voting power calculations.

Copy and paste the following code at the bottom of your test function:

```rust
// Register each Supported Token (ST) mint and its weight in the NCN's VaultRegistry
for (mint, weight) in mints.iter() {
    ncn_program_client
        .do_admin_register_st_mint(ncn_pubkey, mint.pubkey(), *weight)
        .await?;
}
```

This step registers each Supported Token (ST) mint with the NCN program and assigns the appropriate weight:

- Each token mint (Alice, Bob, Charlie, Dave) is registered with its corresponding weight
- The weights determine the voting power multiplier for delegations in that token
- Only the NCN admin has the authority to register tokens, ensuring trust in the system
- Registration involves updating the vault registry with each token's data
- The NCN admin can update the weights of the tokens at any time, which will affect the voting power of the delegations in the next consensus cycle

The weight assignment is fundamental to the design, allowing different tokens to have varying influence on the voting process based on their economic significance or other criteria determined by the NCN administrators.

It's good to know that in real-life examples, NCNs will probably want to set the token weights based on the token's price or market cap. To do so, you will have to use an oracle to get the price of the token and then set the weight based on that. In this case, you will have to store the feed of the price in this step instead of the weight.

##### 5.5 Vault Registration

Registering a vault is a permissionless operation. The reason is the admin has already given permission to the vault to be part of the NCN in the vault registration step earlier, so this step is just to register the vault in the NCN program.

Copy and paste the following code at the bottom of your test function:

```rust
// Register all the vaults in the ncn program
for vault in test_ncn.vaults.iter() {
    let vault = vault.vault_pubkey;
    let (ncn_vault_ticket, _, _) = NcnVaultTicket::find_program_address(
        &jito_restaking_program::id(),
        &ncn_pubkey,
        &vault,
    );

    ncn_program_client
        .do_register_vault(ncn_pubkey, vault, ncn_vault_ticket)
        .await?;
}
```

The final configuration step registers each vault with the NCN program:

1.  For each vault created earlier, the system finds its NCN vault ticket PDA (Program Derived Address)
2.  The vault is registered in the NCN program's vault registry
3.  This creates the association between the vault and its supported token type
4.  The registration enables the NCN program to track vault delegations for voting power calculation

This registration process establishes the complete set of vaults that can contribute to the voting system, creating a closed ecosystem of verified participants.

##### 5.6 NCN Architecture and Security Considerations

##### 5.5 Architecture Considerations

The NCN program configuration establishes a multi-layered security model:

1.  **Authentication Layer**: Only the NCN admin can initialize configuration and register tokens
2.  **Relationship Layer**: Only vaults and operators with established, active handshakes can participate
3.  **Time Security Layer**: Enforced waiting periods prevent quick creation and use of malicious actors
4.  **Registry Layer**: All participants must be registered and tracked in on-chain registries

This layered approach ensures the integrity of the voting system by validating the identity and relationships of all participants before they can influence the consensus process.

The configuration phase completes the preparation of the system's infrastructure, setting the stage for the actual voting mechanics to begin in subsequent phases.

#### 6. Epoch Snapshot and Voting Preparation

The Epoch Snapshot and Voting Preparation phase is where the system captures the current state of all participants and prepares the infrastructure for voting. This is an essential component of the architecture as it ensures voting is based on a consistent, verifiable snapshot of the network state at a specific moment in time.

The upcoming section is a keeper task (with the exception of the voting). This means that it is permissionless and can be done by anyone.

##### 6.1 Epoch State Initialization

To begin a new consensus cycle (epoch), we first initialize an `EpochState` account for our NCN, which will track the progress of this epoch.

Copy and paste the following code at the bottom of your test function:

```rust
// Initialize the epoch state for the current epoch
fixture.add_epoch_state_for_test_ncn(&test_ncn).await?;
```

This step initializes the **Epoch State** for the current consensus cycle:

- It creates an `EpochState` account tied to the specific NCN and epoch.
- This account tracks the progress through each stage of the consensus cycle.
- It maintains flags for each phase (weight setting, snapshot taking, voting, closing).
- The epoch state provides protection against out-of-sequence operations.
- It stores metadata like the current epoch, slot information, and participant counts.

Once initialized, the `EpochState` account becomes the authoritative record of where the system is in the voting process, preventing operations from happening out of order or in duplicate.

You can take a look at the epoch state struct [here](#epochaccountstatus).

##### 6.2 Weight Table Initialization and Population

For the current epoch, we initialize a `WeightTable` and populate it by copying the token weights from the `VaultRegistry`, effectively freezing these weights for the duration of this consensus cycle.

Copy and paste the following code at the bottom of your test function:

```rust
// Initialize the weight table to track voting weights
let clock = fixture.clock().await;
let epoch = clock.epoch;
ncn_program_client
    .do_full_initialize_weight_table(test_ncn.ncn_root.ncn_pubkey, epoch)
    .await?;

// Take a snapshot of weights for each token mint
ncn_program_client
    .do_set_epoch_weights(test_ncn.ncn_root.ncn_pubkey, epoch)
    .await?;
```

The weight table mechanism handles the token weights for the current epoch in two stages:

1.  **Weight Table Initialization**:

    - Creates a [`WeightTable`](#weighttable) account for the specific epoch using `do_full_initialize_weight_table`. This may involve multiple calls internally to allocate sufficient space.
    - Allocates space based on the number of supported tokens registered in the [`VaultRegistry`](#vaultregistry).
    - Links the table to the NCN and current epoch.
    - Initializes the table structure with empty entries.

2.  **Weight Setting**:
    - Populates the [`WeightTable`](#weighttable) by calling `do_set_epoch_weights`
    - Copies the current weights from the [`VaultRegistry`](#vaultregistry) to the epoch-specific `WeightTable`.
    - "Freezes" these weights for the duration of the consensus cycle.
    - Updates the [`EpochState`](#epochaccountstatus) to mark weight setting as complete.
    - Creates an immutable record of token weights that will be used for voting.

This two-step process is critical for the integrity of the system as it:

- Creates a permanent record of weights at the time voting begins.
- Prevents weight changes during a consensus cycle from affecting ongoing votes.
- Allows transparent verification of the weights used for a particular vote.
- Enables historical auditing of how weights changed over time.

##### 6.3 Epoch Snapshot Creation

We then create an `EpochSnapshot` account to record the overall state for this epoch, such as total operator and vault counts, and to accumulate total stake weight.

Copy and paste the following code at the bottom of your test function:

```rust
// Take the epoch snapshot
fixture.add_epoch_snapshot_to_test_ncn(&test_ncn).await?;
```

The epoch snapshot captures the aggregate state of the entire system:

- Creates an [`EpochSnapshot`](#epochsnapshot) account for the NCN and epoch.
- Records the total number of operators and vaults expected to participate.
- Captures the total potential stake weight across all participants (initialized to zero).
- Stores important metadata like the snapshot creation slot.
- Serves as the reference point for total voting power calculations, acting as the denominator for consensus thresholds.

##### 6.4 Operator Snapshots

Next, individual `OperatorSnapshot` accounts are created for each participating operator, capturing their state and expected delegations for the epoch.

Copy and paste the following code at the bottom of your test function:

```rust
// 2.b. Initialize the operators using the Jito Restaking program, and initiate the
//   handshake relationship between the NCN <> operators
{
    for _ in 0..OPERATOR_COUNT {
        // Set operator fee to 100 basis points (1%)
        let operator_fees_bps: Option<u16> = Some(100);

        // Initialize a new operator account with the specified fee
        let operator_root = restaking_client
            .do_initialize_operator(operator_fees_bps)
            .await?;

        // Establish bidirectional handshake between NCN and operator:
        // 1. Initialize the NCN's state tracking (the NCN operator ticket) for this operator
        restaking_client
            .do_initialize_ncn_operator_state(
                &test_ncn.ncn_root,
                &operator_root.operator_pubkey,
            )
            .await?;

        // 2. Advance slot to satisfy timing requirements
        fixture.warp_slot_incremental(1).await.unwrap();

        // 3. NCN warms up to operator - creates NCN's half of the handshake
        restaking_client
            .do_ncn_warmup_operator(&test_ncn.ncn_root, &operator_root.operator_pubkey)
            .await?;

        // 4. Operator warms up to NCN - completes operator's half of the handshake
        restaking_client
            .do_operator_warmup_ncn(&operator_root, &test_ncn.ncn_root.ncn_pubkey)
            .await?;

        // Add the initialized operator to our test NCN's operator list
        test_ncn.operators.push(operator_root);
    }
}
```

This step creates an individual snapshot for each operator in the system:

- For each operator, it creates an [`OperatorSnapshot`](#operatorsnapshot) account linked to the operator, NCN, and epoch.
- Records the operator's total delegated stake weight at this moment (initialized to zero).
- Captures the expected number of vault delegations for the operator.
- Verifies the operator has active handshakes with the NCN.
- Validates the operator's eligibility to participate in voting.

These snapshots establish each operator's baseline for the current epoch. The actual voting power will be populated in the next step based on individual delegations. This ensures that later delegation changes cannot alter voting weight once the snapshot phase is complete.

##### 6.5 Vault-Operator Delegation Snapshots

With operator snapshots ready, we now record the weighted stake from each specific vault-to-operator delegation into the relevant `OperatorSnapshot` and update the total stake in the `EpochSnapshot`.

Copy and paste the following code at the bottom of your test function:

```rust
// Record all vault-to-operator delegations
fixture
    .add_vault_operator_delegation_snapshots_to_test_ncn(&test_ncn)
    .await?;
```

This crucial step iterates through each active vault-to-operator delegation and records its contribution to the operator's voting power:

- For each valid delegation found in the Jito Vault program:
  - Retrieves the corresponding token weight from the epoch's [`WeightTable`](#weighttable).
  - Calculates the weighted stake for that delegation (delegation amount \* token weight).
  - Updates the relevant [`OperatorSnapshot`](#operatorsnapshot) by adding the calculated stake weight.
  - Stores detailed information about the weighted delegation within the [`OperatorSnapshot`](#operatorsnapshot)'s `vault_operator_stake_weight` array.
  - Increments the total stake weight in the global [`EpochSnapshot`](#epochsnapshot).
  - Creates a [`VaultOperatorDelegationSnapshot`](#vaultoperatordelegationsnapshot) account for detailed auditing.

These granular snapshots serve multiple purposes:

- They populate the [`OperatorSnapshot`](#operatorsnapshot) accounts with the actual stake weights used for voting.
- They update the [`EpochSnapshot`](#epochsnapshot) with the total voting power present in the system for this epoch.
- They provide detailed audit trails of exactly where each operator's voting power originates.
- They enable verification of correct weight calculation for each delegation.
- They prevent retroactive manipulation of the voting power distribution.

##### 6.6 Ballot Box Initialization

To prepare for voting, we initialize a `BallotBox` account for the current epoch, which will collect and tally all operator votes.

Copy and paste the following code at the bottom of your test function:

```rust
// Initialize the ballot box for collecting votes
fixture.add_ballot_box_to_test_ncn(&test_ncn).await?;
```

The final preparation step creates the ballot box:

- Initializes a [`BallotBox`](#ballotbox) account linked to the NCN and epoch using `do_full_initialize_ballot_box`. Similar to the weight table, this may require multiple allocation calls internally.
- Creates arrays to track operator votes ([`OperatorVote`](#operatorvote)) and ballot tallies ([`BallotTally`](#ballottally)).
- Sets up the data structures for recording and counting votes.
- Prepares the consensus tracking mechanism.
- Links the ballot box to the [`EpochState`](#epochaccountstatus) for progress tracking.

The [`BallotBox`](#ballotbox) becomes the central repository where all votes are recorded and tallied during the voting process. It is designed to efficiently track:

- Which operators have voted and what they voted for.
- The cumulative stake weight behind each voting option (ballot).
- The current winning ballot (if any).
- Whether consensus has been reached.

##### 6.7 Snapshot Architecture and Security Considerations

The snapshot system implements several key architectural principles:

1.  **Point-in-Time Consistency**: All snapshots capture the system state relative to the start of the epoch, creating a consistent view based on frozen weights and delegations present at that time.
2.  **Immutability**: Once taken and populated, snapshots cannot be modified, ensuring the integrity of the voting weights used.
3.  **Layered Verification**: The system enables verification at multiple levels:
    - Aggregate level (`EpochSnapshot`)
    - Participant level (`OperatorSnapshot`)
    - Relationship level (individual weighted delegations within `OperatorSnapshot`, optionally `VaultOperatorDelegationSnapshot`)
4.  **Defense Against Time-Based Attacks**: By freezing the state (weights and relevant delegations) before voting begins, the system prevents:
    - Late stake additions influencing outcomes within the _current_ epoch.
    - Strategic withdrawals affecting voting power _after_ the snapshot.
    - Any form of "stake voting power front-running" within the epoch.
5.  **Separation of State and Process**:
    - The state (snapshots, weights) is captured separately from the process (voting).
    - This clear separation simplifies reasoning about the system.
    - It enables more effective testing and verification.

The comprehensive snapshot approach ensures that voting occurs on a well-defined, verifiable view of the network's state, establishing a solid foundation for the actual voting process to follow.

#### 7. Voting Process

The Voting Process is the core functionality of the NCN system, where operators express their preferences on the network state (represented by the "weather status" in this simulation). This process leverages the infrastructure and snapshots created in previous steps to ensure secure, verifiable, and stake-weighted consensus.

##### 7.1 Setting the Expected Outcome

In our simulation, we'll predefine an expected winning outcome for verification purposes.

Copy and paste the following code at the bottom of your test function:

```rust
// Define the expected winning weather status
let winning_weather_status = WeatherStatus::Sunny as u8;
```

For testing purposes, the system defines an expected outcome (`WeatherStatus::Sunny`). In a production environment, the winning outcome would be determined organically through actual operator votes based on real-world data or criteria. The weather status enum (`Sunny`, `Cloudy`, `Rainy`) serves as a simplified proxy for any on-chain decision that requires consensus.

##### 7.2 Casting Votes from Different Operators

Operators now cast their votes. We'll simulate a few operators voting, some for the expected outcome and some against, to test the tallying logic.

Copy and paste the following code at the bottom of your test function:

```rust
// Cast votes from operators
{
    let epoch = fixture.clock().await.epoch;

    let first_operator = &test_ncn.operators[0];
    let second_operator = &test_ncn.operators[1];
    let third_operator = &test_ncn.operators[2];

    // First operator votes for Cloudy
    ncn_program_client
        .do_cast_vote(
            ncn_pubkey,
            first_operator.operator_pubkey,
            &first_operator.operator_admin,
            WeatherStatus::Cloudy as u8,
            epoch,
        )
        .await?;

    // Second and third operators vote for Sunny (expected winner)
    ncn_program_client
        .do_cast_vote(
            ncn_pubkey,
            second_operator.operator_pubkey,
            &second_operator.operator_admin,
            winning_weather_status,
            epoch,
        )
        .await?;
    ncn_program_client
        .do_cast_vote(
            ncn_pubkey,
            third_operator.operator_pubkey,
            &third_operator.operator_admin,
            winning_weather_status,
            epoch,
        )
        .await?;
}
```

This section demonstrates the system's ability to handle diverse voting preferences using the `do_cast_vote` helper, which calls the `cast_vote` instruction:

- The first operator votes for "Cloudy" (representing a minority view).
- The second and third operators vote for "Sunny" (the presumed majority view).
- Each `do_cast_vote` call invokes the NCN program with the operator's choice and admin signature.

Under the hood, each vote triggers several key operations within the `cast_vote` instruction:

- **Verification**:
  - Verifies the operator admin's signature.
  - Checks that the operator hasn't already voted in this epoch using the [`BallotBox`](#ballotbox).
  - Retrieves the operator's [`OperatorSnapshot`](#operatorsnapshot) to confirm eligibility and get its total stake weight.
  - Ensures the [`EpochState`](#epochaccountstatus) indicates voting is currently allowed.
- **Recording**:
  - Records the vote details (operator, slot, stake weight, ballot choice) in the `operator_votes` array within the [`BallotBox`](#ballotbox).
  - Marks the operator as having voted.
- **Tallying**:
  - Finds or creates a [`BallotTally`](#ballottally) for the chosen weather status in the `ballot_tallies` array.
  - Adds the operator's full stake weight (from the snapshot) to this tally.
  - Increments the raw vote count for this tally.
- **Consensus Check**:
  - Compares the updated tally's stake weight against the total stake weight recorded in the [`EpochSnapshot`](#epochsnapshot).
  - If the tally now exceeds the consensus threshold (e.g., 66%), it marks consensus as reached in the [`BallotBox`](#ballotbox) and records the current slot.

##### 7.3 Establishing Consensus Through Majority Voting

To ensure consensus is reached for our test, the remaining eligible operators will now vote for the predefined winning weather status.

Copy and paste the following code at the bottom of your test function:

```rust
// All remaining operators vote for Sunny to form a majority
for operator_root in test_ncn.operators.iter().take(OPERATOR_COUNT).skip(3) {
    ncn_program_client
        .do_cast_vote(
            ncn_pubkey,
            operator_root.operator_pubkey,
            &operator_root.operator_admin,
            winning_weather_status,
            epoch,
        )
        .await?;
}
```

The consensus mechanism works as follows:

- The system maintains a running [`BallotTally`](#ballottally) for each unique option voted on.
- After each vote, it recalculates the total stake weight supporting the voted option.
- It compares this stake weight to the total stake weight available in the [`EpochSnapshot`](#epochsnapshot).
- If an option's stake weight reaches the consensus threshold (e.g., >= 66%), the system:
  - Marks that `Ballot` as the `winning_ballot` in the [`BallotBox`](#ballotbox).
  - Records the current `slot` in `slot_consensus_reached`.
  - Updates the `EpochState`.
  - Creates a persistent [`ConsensusResult`](#consensusresult) account (discussed in Verification).
- Consensus requires a supermajority to ensure decisions have strong, verifiable support across the network's weighted stake.

##### 7.4 Vote Processing Architecture

When an operator casts a vote via the `cast_vote` instruction, the system performs several critical operations:

- **Authentication**: Verifies the transaction is signed by the correct `operator_admin` keypair associated with the `operator` account.
- **Authorization & Preconditions**: Confirms that:
  - The operator exists, is registered with the NCN, and has an active [`OperatorSnapshot`](#operatorsnapshot) for the current `epoch`.
  - The operator has not already voted in this epoch (checked via [`BallotBox`](#ballotbox)).
  - The operator has non-zero stake weight in their [`OperatorSnapshot`](#operatorsnapshot).
  - The [`EpochState`](#epochaccountstatus) confirms that the snapshotting phase is complete and voting is open.
- **Vote Recording**:
  - Locates an empty slot or confirms the operator hasn't voted in the `operator_votes` array within the [`BallotBox`](#ballotbox).
  - Stores the `operator` pubkey, current `slot`, the operator's total `stake_weights` (from [`OperatorSnapshot`](#operatorsnapshot)), and the index corresponding to the chosen ballot within the `ballot_tallies` array.
  - Increments the `operators_voted` counter in the [`BallotBox`](#ballotbox).
- **Ballot Processing & Tallying**:
  - Searches the `ballot_tallies` array for an existing entry matching the `weather_status`.
  - If found: Adds the operator's `stake_weights` to the `stake_weights` field of the existing [`BallotTally`](#ballottally) and increments the raw `tally` counter.
  - If not found: Initializes a new `BallotTally` entry with the `weather_status`, the operator's `stake_weights`, and a `tally` of 1. Increments `unique_ballots`.
- **Consensus Calculation & Result Creation**:
  - Retrieves the total `stake_weights` from the `EpochSnapshot`.
  - Compares the winning ballot's accumulated `stake_weights` against the total.
  - If the threshold is met _and_ consensus hasn't already been marked:
    - Sets the `winning_ballot` field in the `BallotBox`.
    - Records the current `slot` in `slot_consensus_reached`.
    - Updates the `EpochState`.
    - Invokes an instruction (likely via CPI or separate transaction) to create the `ConsensusResult` account, storing the winning status, epoch, weights, and slot.
- **Cross-Validation**: Implicitly ensures the vote aligns with the correct `ncn` and `epoch` through the PDAs used for the involved accounts (`BallotBox`, `OperatorSnapshot`, `EpochState`).

This multi-layered architecture ensures votes are processed securely, tallied correctly using the snapshotted weights, and that consensus is determined accurately based on stake-weighted participation.

##### 7.5 Security Considerations in the Voting Process

The voting process incorporates several key security features:

- **Sybil Attack Prevention**:
  - Voting power is derived directly from snapshotted stake weight, not operator count.
  - Operators with zero snapshotted stake weight cannot vote, preventing attacks based on creating numerous fake operators.
- **Replay Protection**:
  - The [`BallotBox`](#ballotbox) tracks which operators have voted (`operator_votes` array).
  - Attempts by an operator to vote more than once within the same epoch are rejected.
- **Time-Bound Voting**:
  - Votes are only accepted if the [`EpochState`](#epochaccountstatus) indicates the voting phase is active for the specified `epoch`.
  - While votes might be accepted slightly after consensus is reached (within `valid_slots_after_consensus`), they won't change the already determined outcome.
- **Authority**: Requires `operator_admin` signature.
- **Tamper-Proof Tallying**: Uses immutable snapshotted data created _before_ voting began.
- **Consistent Threshold**: Calculated based on the total stake weight recorded in the [`EpochSnapshot`](#epochsnapshot), providing a fixed target for the epoch.

These security measures ensure the voting process remains resilient against various attack vectors and manipulation attempts, maintaining the integrity of the consensus mechanism.

#### 8. Verification

The Verification phase validates that the voting process completed successfully and that the expected consensus was achieved. This critical step confirms the integrity of the entire system by examining the on-chain data structures ([`BallotBox`](#ballotbox) and [`ConsensusResult`](#consensusresult)) and verifying they contain the expected results.

##### 8.1 Ballot Box Verification

After voting concludes, we first verify the `BallotBox` to ensure it correctly reflects that consensus was reached and identifies the expected winning ballot.

Copy and paste the following code at the bottom of your test function:

```rust
// Verify the results recorded in the BallotBox
{
    let epoch = fixture.clock().await.epoch;
    let ballot_box = ncn_program_client.get_ballot_box(ncn_pubkey, epoch).await?;

    assert!(ballot_box.has_winning_ballot());
    assert!(ballot_box.is_consensus_reached());
    assert_eq!(ballot_box.get_winning_ballot().unwrap().weather_status(), winning_weather_status);
}
```

The first verification step examines the `BallotBox` account for the completed epoch:

- **Winning Ballot Check**:
  - `has_winning_ballot()` confirms that the `winning_ballot` field within the `BallotBox` structure is marked as valid.
- **Consensus Status Check**:

* **Winning Ballot Check**:
  - `has_winning_ballot()` confirms that the `winning_ballot` field within the `BallotBox` structure is marked as valid.

2.  **Consensus Status Check**:
    - `is_consensus_reached()` checks if the `slot_consensus_reached` field is greater than zero, indicating the consensus condition was met during the voting process.

- **Outcome Verification**:
  - The test retrieves the `winning_ballot` struct and asserts that its `weather_status` field matches the `winning_weather_status` defined earlier (`WeatherStatus::Sunny`). This confirms the correct outcome was identified based on the stake-weighted tally.

Verifying the `BallotBox` ensures the core voting and tallying mechanism functioned correctly during the active epoch.

##### 8.2 Consensus Result Account Verification

Next, we verify the permanently stored `ConsensusResult` account to confirm it accurately records the winning outcome, epoch details, and vote weights, consistent with the `BallotBox`.

Copy and paste the following code at the bottom of your test function:

```rust
// Fetch and verify the consensus_result account
{
    let epoch = fixture.clock().await.epoch;
    let consensus_result = ncn_program_client
        .get_consensus_result(ncn_pubkey, epoch)
        .await?;

    assert!(consensus_result.is_consensus_reached());
    assert_eq!(consensus_result.epoch(), epoch);
    assert_eq!(consensus_result.weather_status(), winning_weather_status);

    let ballot_box = ncn_program_client.get_ballot_box(ncn_pubkey, epoch).await?;
    let winning_ballot_tally = ballot_box.get_winning_ballot_tally().unwrap();

    assert_eq!(consensus_result.vote_weight(), winning_ballot_tally.stake_weights().stake_weight() as u64);

    println!(
        "✅ Consensus Result Verified - Weather Status: {}, Vote Weight: {}, Total Weight: {}, Recorder: {}",
        consensus_result.weather_status(),
        consensus_result.vote_weight(),
        consensus_result.total_vote_weight(),
        consensus_result.consensus_recorder()
    );
}
```

The second verification step examines the `ConsensusResult` account, which serves as the permanent, immutable record of the voting outcome:

- **Consensus Result Existence & Fetching**:
  - The test successfully fetches the `ConsensusResult` account using its PDA derived from the NCN pubkey and epoch. Its existence implies consensus was reached and the account was created.
- **Consensus Status Validation**:

* **Consensus Result Existence & Fetching**:
  - The test successfully fetches the `ConsensusResult` account using its PDA derived from the NCN pubkey and epoch. Its existence implies consensus was reached and the account was created.

2.  **Consensus Status Validation**:
    - `is_consensus_reached()` checks an internal flag derived from stored values (like `consensus_slot` > 0), confirming the outcome is officially recognized.

- **Metadata Verification**:
  - Asserts that the `epoch` field matches the current epoch.
  - Asserts that the `weather_status` matches the expected `winning_weather_status`.
- **Cross-Account Consistency Check**:
  - Fetches the `BallotBox` again.
  - Retrieves the `BallotTally` corresponding to the winning ballot from the `BallotBox`.
  - Asserts that the `vote_weight` stored in the `ConsensusResult` exactly matches the `stake_weight` recorded in the winning `BallotTally` within the `BallotBox`. This ensures consistency between the temporary voting record and the permanent result.
- **Detailed Reporting**:
  - Prints key details from the verified `ConsensusResult` account for confirmation.

Verifying the `ConsensusResult` confirms that the outcome was durably stored with the correct details and consistent with the voting process itself.

##### 8.3 Architecture of Verification and Result Persistence

The verification phase highlights several important architectural features:

1.  **Dual Records**:
    - The system temporarily uses the `BallotBox` during the epoch for active voting and tallying.
    - Upon reaching consensus, it creates a separate, permanent `ConsensusResult` account.
    - This redundancy allows for cleanup while preserving the essential outcome.
2.  **Separation of Process and Outcome**:
    - The `BallotBox` (process) can eventually be closed to reclaim rent.
    - The `ConsensusResult` (outcome) persists indefinitely as the historical record.
3.  **Automated Result Creation**:
    - The `ConsensusResult` account is typically created automatically within the `cast_vote` instruction when the consensus threshold is first met. This ensures timely recording without requiring a separate administrative action.
4.  **Result Immutability**:
    - The `ConsensusResult` account, once created, is designed to be immutable. It stores the outcome based on the state when consensus was reached.
5.  **Time and Slot Tracking**:
    - Both `BallotBox` and `ConsensusResult` store key timing information (`slot_consensus_reached`, `epoch`). This metadata is crucial for auditing and understanding the system's behavior over time.

##### 8.4 Verification Techniques and Best Practices

The verification approach demonstrates several best practices:

1.  **Multi-Level Verification**: Testing both the ephemeral process account (`BallotBox`) and the persistent outcome account (`ConsensusResult`) provides comprehensive validation.
2.  **State Assertions**: Using dedicated helper functions on the deserialized accounts (`has_winning_ballot()`, `is_consensus_reached()`) makes tests more readable and robust against internal representation changes.
3.  **Equality Assertions**: Using strict equality (`assert_eq!`) for key outcome data (winning status, epoch, weights) ensures exactness.
4.  **Cross-Structure Validation**: Comparing critical values (like `vote_weight`) between the `BallotBox` and `ConsensusResult` confirms data consistency across different parts of the system.
5.  **Complete Outcome Validation**: Checking not just the winning choice but also associated metadata (epoch, weights, consensus flags) catches more subtle errors.
6.  **Clear Reporting**: Outputting verified data (`println!`) provides immediate feedback during test runs.

This rigorous verification ensures the NCN system reliably achieves and records stake-weighted consensus according to its design.

#### 9. Cleanup

After the core functionality has been tested and verified for a given epoch, the temporary accounts associated with that epoch can be closed to reclaim the SOL locked for rent. The persistent `ConsensusResult` account remains.

Copy and paste the following code at the bottom of your test function:

```rust
// Close epoch accounts but keep consensus result
let epoch_before_closing_account = fixture.clock().await.epoch;
fixture.close_epoch_accounts_for_test_ncn(&test_ncn).await?;

// Verify that consensus_result account is not closed
{
    let consensus_result = ncn_program_client
        .get_consensus_result(ncn_pubkey, epoch_before_closing_account)
        .await?;

    assert!(consensus_result.is_consensus_reached());
    assert_eq!(consensus_result.epoch(), epoch_before_closing_account);
}
```

This cleanup process involves:

- **Identifying Epoch**: Recording the current epoch (`epoch_before_closing_account`) just before initiating closure.
- **Closing Accounts**: Calling `fixture.close_epoch_accounts_for_test_ncn`, which likely iterates through epoch-specific accounts and invokes a `close_epoch_account` instruction for each.
- **Verifying Persistence**: After the cleanup function returns, the test attempts to fetch the `ConsensusResult` account for the _same_ `epoch_before_closing_account`.
- **Confirming Data**: It asserts that the fetched `ConsensusResult` still exists and retains its key data (`is_consensus_reached`, `epoch`), confirming it was _not_ closed during the cleanup process.

This demonstrates a crucial design feature:

- **Resource Management**: Temporary accounts are removed, preventing indefinite accumulation of rent-paying accounts.
- **Outcome Preservation**: The final, critical outcome (`ConsensusResult`) is preserved as a permanent on-chain record, suitable for historical lookups or use by other programs.

This efficient cleanup mechanism allows the NCN system to operate continuously over many epochs without unbounded growth in account storage requirements.

Now you can save the file and run the test to see the result.

## Core struct definitions

Here are the definitions for the core data structures used by the NCN program, typically found in the `/core/src` directory. Understanding these structures is key to understanding the program's logic.

#### Config

file: `config.rs`

- **Purpose**: Stores global, long-lived configuration parameters for the NCN program instance.
- **Definition**:

```rust
#[derive(Debug, Clone, Copy, Zeroable, Pod, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct Config {
    /// The Pubkey of the associated Jito Restaking NCN account this config belongs to.
    pub ncn: Pubkey,
    /// The admin authorized to update the tie breaker mechanism or parameters.
    pub tie_breaker_admin: Pubkey,
    /// Number of slots after consensus is reached where votes are still accepted
    /// (though they won't change the outcome).
    pub valid_slots_after_consensus: PodU64,
    /// Number of epochs without reaching consensus before the cycle is considered stalled.
    pub epochs_before_stall: PodU64,
    /// Number of epochs to wait after consensus is reached before epoch accounts can be closed.
    pub epochs_after_consensus_before_close: PodU64,
    /// The first epoch number for which voting is considered valid.
    pub starting_valid_epoch: PodU64,
    /// Bump seed for the PDA
    pub bump: u8,
}
```

- **Explanation**: Holds the associated `ncn`, the `tie_breaker_admin`, and various timing/threshold parameters (`valid_slots_after_consensus`, `epochs_before_stall`, `epochs_after_consensus_before_close`, `starting_valid_epoch`).

#### Ballot

file: `ballot_box.rs`

- **Purpose**: Represents a single potential outcome in the consensus process, specifically a weather status in this example.
- **Definition**:

```rust
#[derive(Debug, Clone, Copy, Zeroable, ShankType, Pod)]
#[repr(C)]
pub struct Ballot {
    /// The weather status value
    weather_status: u8,
    /// Whether the ballot is valid
    is_valid: PodBool,
}
```

- **Explanation**: Holds the numeric `weather_status` being voted on and a boolean `is_valid` flag to ensure it corresponds to a known status.

#### BallotTally

file: `ballot_box.rs`

- **Purpose**: Aggregates votes and stake weight for a specific `Ballot`.
- **Definition**:

```rust
#[derive(Debug, Clone, Copy, Zeroable, ShankType, Pod)]
#[repr(C)]
pub struct BallotTally {
    /// Index of the tally within the ballot_tallies
    index: PodU16,
    /// The ballot being tallied
    ballot: Ballot,
    /// Breakdown of all of the stake weights that contribute to the vote
    stake_weights: StakeWeights,
    /// The number of votes for this ballot
    tally: PodU64,
}
```

- **Explanation**: Tracks which `ballot` this tally is for, its `index` in the main array, the total `stake_weights` supporting it, and the raw `tally` (count) of votes.

#### OperatorVote

file: `ballot_box.rs`

- **Purpose**: Records the vote cast by a single operator within a specific epoch's `BallotBox`.
- **Definition**:

```rust
#[derive(Debug, Clone, Copy, Zeroable, ShankType, Pod)]
#[repr(C)]
pub struct OperatorVote {
    /// The operator that cast the vote
    operator: Pubkey,
    /// The slot when the operator voted
    slot_voted: PodU64,
    /// The stake weights of the operator
    stake_weights: StakeWeights,
    /// The index of the ballot in the ballot_tallies array
    ballot_index: PodU16,
}
```

- **Explanation**: Stores the `operator` pubkey, the current `slot`, their `stake_weights`, and the `ballot_index` they voted for.

#### BallotBox

file: `ballot_box.rs`

- **Purpose**: The central account for managing the voting process within a specific epoch.
- **Definition**:

```rust
#[derive(Debug, Clone, Copy, Zeroable, Pod, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct BallotBox {
    /// The Pubkey of the NCN this ballot box is for
    ncn: Pubkey,
    /// The epoch this ballot box is for
    epoch: PodU64,
    /// Bump seed for the PDA
    bump: u8,
    /// Slot when this ballot box was created
    slot_created: PodU64,
    /// Slot when consensus was reached
    slot_consensus_reached: PodU64,
    /// Number of operators that have voted
    operators_voted: PodU64,
    /// Number of unique ballots
    unique_ballots: PodU64,
    /// The ballot that got at least 66% of votes
    winning_ballot: Ballot,
    /// Operator votes
    operator_votes: [OperatorVote; MAX_OPERATORS],
    /// Mapping of ballots votes to stake weight
    ballot_tallies: [BallotTally; MAX_OPERATORS],
}
```

- **Explanation**: Holds metadata (`ncn`, `epoch`, timestamps), vote counts, and arrays for individual operator votes and aggregated tallies.

#### ConsensusResult

file: `consensus_result.rs`

- **Purpose**: A persistent account storing the final, immutable outcome of a consensus cycle for a specific epoch.
- **Definition**:

```rust
#[derive(Debug, Clone, Copy, Zeroable, Pod, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct ConsensusResult {
    /// The Pubkey of the NCN this consensus result is for
    ncn: Pubkey,
    /// The epoch this consensus result is for
    epoch: PodU64,
    /// The vote weight that supported the winning status
    vote_weight: PodU64,
    /// The total vote weight in the ballot box
    total_vote_weight: PodU64,
    /// The slot at which consensus was reached
    consensus_slot: PodU64,
    /// Bump seed for the PDA
    bump: u8,
    /// The winning weather status that reached consensus
    weather_status: u8,
}
```

- **Explanation**: Stores the `ncn`, `epoch`, the winning `weather_status`, and the `consensus_slot`.

#### AccountPayer

file: `account_payer.rs`

- **Purpose**: An empty, uninitialized system account used solely as a Program Derived Address (PDA) to hold SOL temporarily for paying rent during account creation or reallocation within the NCN program.
- **Definition**:

```rust
pub struct AccountPayer {}
```

- **Explanation**: This is a marker struct with no fields. Its associated functions handle deriving the PDA and performing SOL transfers for rent payments using `invoke_signed`.

#### EpochMarker

file: `epoch_marker.rs`

- **Purpose**: An empty account created as a marker to signify that all temporary accounts associated with a specific NCN epoch have been successfully closed.
- **Definition**:

```rust
#[derive(Debug, Clone, Copy, Zeroable, ShankType, Pod, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct EpochMarker {
    ncn: Pubkey,
    epoch: PodU64,
    slot_closed: PodU64,
}
```

- **Explanation**: Contains the `ncn`, the `epoch` that was closed, and the `slot_closed`. Its existence confirms cleanup completion for that epoch.

#### EpochSnapshot

file: `epoch_snapshot.rs`

- **Purpose**: Captures the aggregated state of the NCN system at the beginning of a specific epoch snapshot phase.
- **Definition**:

```rust
#[derive(Debug, Clone, Copy, Zeroable, Pod, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct EpochSnapshot {
    /// The Pubkey of the NCN this snapshot is for
    ncn: Pubkey,
    /// The epoch this snapshot is for
    epoch: PodU64,
    /// Bump seed for the PDA
    bump: u8,
    /// Slot when this EpochSnapshot account was created
    slot_created: PodU64,
    /// Slot when the snapshotting process (including all operator delegations) was completed
    slot_finalized: PodU64,
    /// Number of operators in the epoch
    operator_count: PodU64,
    /// Number of vaults in the epoch
    vault_count: PodU64,
    /// Keeps track of the number of completed operator registration through `snapshot_vault_operator_delegation` and `initialize_operator_snapshot`
    operators_registered: PodU64,
    /// Keeps track of the number of valid operator vault delegations
    valid_operator_vault_delegations: PodU64,
    /// Tallies the total stake weights for all vault operator delegations
    stake_weights: StakeWeights,
}
```

- **Explanation**: Stores metadata (`ncn`, `epoch`, timestamps), counts (`operator_count`, `vault_count`), progress trackers, and the total aggregated `stake_weights` for the epoch.

#### OperatorSnapshot

file: `epoch_snapshot.rs`

- **Purpose**: Captures the state of a single operator for a specific epoch, including their total delegated stake weight and a breakdown of contributions from each vault.
- **Definition**:

```rust
#[derive(Debug, Clone, Copy, Zeroable, Pod, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct OperatorSnapshot {
    operator: Pubkey,
    ncn: Pubkey,
    ncn_epoch: PodU64,
    bump: u8,
    slot_created: PodU64,
    slot_finalized: PodU64,
    is_active: PodBool,
    ncn_operator_index: PodU64,
    operator_index: PodU64,
    operator_fee_bps: PodU16,
    vault_operator_delegation_count: PodU64,
    vault_operator_delegations_registered: PodU64,
    valid_operator_vault_delegations: PodU64,
    stake_weights: StakeWeights,
    vault_operator_stake_weight: [VaultOperatorStakeWeight; MAX_VAULTS],
}
```

- **Explanation**: Contains operator/NCN identifiers, timestamps, status, indices, `operator_fee_bps`, delegation counts/progress, the operator's total `stake_weights`, and a detailed breakdown in `vault_operator_stake_weight`.

#### VaultOperatorStakeWeight

file: `epoch_snapshot.rs`

- **Purpose**: A helper struct within `OperatorSnapshot` to store the calculated stake weight originating from one specific vault's delegation to that operator.
- **Definition**:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Zeroable, Pod)]
pub struct VaultOperatorStakeWeight {
    vault: Pubkey,
    vault_index: PodU64,
    stake_weight: StakeWeights,
}
```

- **Explanation**: Links a `vault` pubkey and `vault_index` to the specific `stake_weight` derived from its delegation to the parent `OperatorSnapshot`.

#### StMintEntry

file: `vault_registry.rs`

- **Purpose**: Represents a supported token mint within the `VaultRegistry`, storing its address and associated voting weight.
- **Definition**:

```rust
#[derive(Debug, Clone, Copy, Zeroable, ShankType, Pod)]
#[repr(C)]
pub struct StMintEntry {
    /// The supported token ( ST ) mint
    st_mint: Pubkey,
    // Either a switchboard feed or a weight must be set
    /// The switchboard feed for the mint
    reserve_switchboard_feed: [u8; 32],
    /// The weight
    weight: PodU128,
}
```

- **Explanation**: Stores the `st_mint` address and its assigned voting `weight`. `reserve_switchboard_feed` is unused here.

#### VaultEntry

file: `vault_registry.rs`

- **Purpose**: Represents a registered vault within the `VaultRegistry`.
- **Definition**:

```rust
#[derive(Debug, Clone, Copy, Zeroable, ShankType, Pod)]
#[repr(C)]
pub struct VaultEntry {
    /// The vault account
    vault: Pubkey,
    /// The supported token ( ST ) mint of the vault
    st_mint: Pubkey,
    /// The index of the vault in respect to the NCN account
    vault_index: PodU64,
    /// The slot the vault was registered
    slot_registered: PodU64,
}
```

- **Explanation**: Stores the `vault` address, the `st_mint` it holds, its assigned `vault_index`, and the `slot_registered`.

#### VaultRegistry

file: `vault_registry.rs`

- **Purpose**: A global account for the NCN program instance that maintains the list of all supported token mints (`StMintEntry`) and all registered vaults (`VaultEntry`).
- **Definition**:

```rust
#[derive(Debug, Clone, Copy, Zeroable, Pod, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct VaultRegistry {
    /// The Pubkey of the associated NCN account this registry belongs to.
    pub ncn: Pubkey,
    /// Bump seed for the PDA
    pub bump: u8,
    /// Array storing entries for each supported token mint
    pub st_mint_list: [StMintEntry; MAX_ST_MINTS],
    /// Array storing entries for each vault
    pub vault_list: [VaultEntry; MAX_VAULTS],
}
```

- **Explanation**: Holds the `ncn` identifier, `bump`, and arrays for `st_mint_list` and `vault_list`.

#### WeightTable

file: `weight_table.rs`

- **Purpose**: An epoch-specific account that snapshots the weights of all supported tokens at the beginning of the epoch.
- **Definition**:

```rust
#[derive(Debug, Clone, Copy, Zeroable, Pod, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct WeightTable {
    /// The Pubkey of the associated NCN account this account is for.
    ncn: Pubkey,
    /// The epoch this account is for.
    epoch: PodU64,
    /// Slot when this WeightTable account was created.
    slot_created: PodU64,
    /// Number of vaults in tracked mints at the time of creation
    vault_count: PodU64,
    /// Bump seed for the PDA
    bump: u8,
    /// A snapshot copy of the relevant vault entries from the VaultRegistry
    vault_registry: [VaultEntry; MAX_VAULTS],
    /// The weight table
    table: [WeightEntry; MAX_ST_MINTS],
}
```

- **Explanation**: Contains metadata (`ncn`, `epoch`, `slot_created`, `vault_count`), a snapshot of the `vault_registry`, and the main `table` holding `WeightEntry` structs with the frozen weights for the epoch.

#### EpochAccountStatus

file: `epoch_state.rs`

- **Purpose**: A helper struct within `EpochState` used to track the lifecycle status of various temporary accounts associated with a specific epoch.
- **Definition**:

```rust
#[derive(Debug, Clone, Copy, Zeroable, ShankType, Pod)]
#[repr(C)]
pub struct EpochAccountStatus {
    /// Status of the main EpochState account itself.
    epoch_state: u8,
    /// Status of the WeightTable account for this epoch.
    weight_table: u8,
    /// Status of the main EpochSnapshot account for this epoch.
    epoch_snapshot: u8,
    /// Status array for each individual OperatorSnapshot account.
    operator_snapshot: [u8; MAX_OPERATORS],
    /// Status of the BallotBox account for this epoch.
    ballot_box: u8,
}
```

- **Explanation**: Uses `u8` fields to represent the status of various temporary accounts associated with a specific epoch.
