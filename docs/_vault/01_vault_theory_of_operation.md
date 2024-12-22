---
title: Vault Theory Of Operation
category: Jekyll
layout: post
weight: 2
---

# 1. Introduction

The Vault Program is a crucial component of Jito's Liquid Restaking system, designed to manage and facilitate the staking and unstaking of tokens within the Solana ecosystem. This program is responsible for holding tokens, minting and burning Vault Receipt Tokens (VRT), and managing various administrative functions related to the vault's operation.

Key features of the vault program include:
- Token management: Securely holding deposited tokens and minting corresponding VRT tokens.
- Fee handling: Managing deposit, withdrawal, and reward fees.
- Administrative controls: Allowing authorized parties to manage vault parameters and perform administrative actions.
- Delegation support: Facilitating delegation to operators and managing relationships with Node Consensus Networks (NCNs).
- State tracking: Maintaining and updating the vault's state to ensure accurate token representation.

The Vault Program is designed to be flexible and extensible, allowing for various configurations and supporting multiple roles such as admins, operators, and NCNs. It interacts with other components of the Solana ecosystem, including the System Program and Token Program, to provide a comprehensive liquid staking solution.

In the following sections, we will delve into the theory of operation of the Vault Program.

# 2. Vault Initialization

Vault initialization is a crucial step in setting up the Vault Program for Jito's Liquid (Re)staking system.

Vaults are initialized with several parameters that are used to configure the vault's behavior:
- `base`: The base address of the vault. This is used to derive the vault's PDA.
- `vrt_mint`: The mint address of the Vault Receipt Token (VRT).
- `supported_mint`: The mint address of the SPL token that the vault supports.
- `admin`: The admin address of the vault. This is the address that will be used to manage the vault.
- `deposit_fee_bps`: The deposit fee of the vault in basis points (bps).
- `withdrawal_fee_bps`: The withdrawal fee of the vault in basis points (bps).
- `reward_fee_bps`: The reward fee of the vault in basis points (bps).

# 3. Vault Admins

Here's a list of different admins that control the vault:

- `admin`: Control the vault's overall management.
- `delegation_admin`: Add and removal of delegations to operators.
- `operator_admin`: Add and removal of operators.
- `ncn_admin`: Add and removal of NCNs.
- `slasher_admin`: Add and removal of slashers.
- `capacity_admin`: Set the vault's max token capacity.
- `fee_admin`: Set and adjust deposit, withdrawal, and reward fees.
- `withdrawal_admin`: Initiate token withdrawals from the vault.
- `mint_burn_admin`: An optional admin for minting and burning operations.

# 4. Vault Configuration

The vault has several configurable parameters by different admins mentioned above:

- `capacity`: The maximum amount of tokens that can be deposited into the vault.
- `deposit_fee_bps`: The fee charged on deposits in the VRT token, in basis points.
- `withdrawal_fee_bps`: The fee charged on withdrawals in the VRT token, in basis points.
- `reward_fee_bps`: The fee charged on rewards in the VRT token, in basis points.

These parameters allow for fine-tuning of the vault's behavior and economics.

# 5. Vault State Tracking
The Vault Program maintains several state variables to track various aspects of its operation. Here's an overview of some key state variables:

1. `vrt_supply`: Tracks the total number of Vault Receipt Tokens (VRT) in circulation. This is updated whenever VRTs are minted or burned. It shall always match the total supply of VRTs tracked by the token mint.
2. `tokens_deposited`: Represents the total number of supported tokens currently held by the vault. This is increased when users deposit tokens and decreased when tokens are withdrawn.
3. `delegation_state`: A complex state that tracks the aggregate delegation of tokens to operators. It includes:
   - `total_security`: The total amount of tokens delegated as security.
   - `enqueued_for_cooldown_amount`: Tokens that are scheduled to begin the cooldown process.
   - `cooling_down_amount`: Tokens currently in the cooldown period.
4. `vrt_enqueued_for_cooldown_amount`: Tracks the amount of VRT tokens that have been enqueued for cooldown but haven't started the cooldown process yet.
5. `vrt_cooling_down_amount`: Represents the amount of VRT tokens currently in the cooldown period.
6. `vrt_ready_to_claim_amount`: Tracks the amount of VRT tokens that have completed the cooldown period and are ready to be claimed by users.
7. `last_full_state_update_slot`: Records the last Solana slot when the full state of the vault was updated. This is crucial for maintaining accurate state across epochs.
8. `capacity`: The maximum amount of tokens that can be deposited into the vault.
9. `last_fee_change_slot`: Tracks the last Solana slot when fee parameters were changed.

These state variables work together to provide a comprehensive view of the vault's current status, including its token holdings, VRT circulation, delegation status, and cooldown processes. The vault regularly updates these states to ensure accurate representation of its assets and liabilities.

# 6. Minting

Minting is the process of depositing tokens into the vault and receiving Vault Receipt Tokens (VRT) in return. Here's a high-level overview of the minting process:

1. User initiates a mint transaction, specifying the amount of tokens to deposit.
2. The program checks if the vault has sufficient capacity to accept the deposit.
3. The program calculates the amount of VRT to mint based on the current exchange rate between the deposited token and VRT.
4. A deposit fee is calculated and deducted from the VRT amount.
5. The program transfers the deposited tokens from the user's account to the vault's token account.
6. The program mints new VRT tokens:
   - The majority goes to the user's VRT account.
   - A small portion (the fee) goes to an ATA owned by the fee wallet.
7. The vault's state is updated:
   - The `tokens_deposited` amount is increased.
   - The `vrt_supply` is increased by the total amount of VRT minted.
8. If the mint is successful, the user now holds VRT tokens representing their share of the vault's assets.

The minting process ensures that the relationship between deposited tokens and minted VRT remains consistent, maintaining the integrity of the liquid staking system. The inclusion of fees during minting allows the protocol to generate revenue and incentivize various participants in the ecosystem.

Some other details:
- Note that there is an optional mint burn admin that can be set by the admin. If set, only the mint burn admin can mint and burn VRT tokens. This can be a useful feature for the admin to control the supply of VRT tokens through a multi-sig, hot wallet, or a required CPI call.
- The vault state must be updated before calling, which is detailed more below.

# 7. NCN & Operator Support

## 7.1. Adding & Removing NCNs

The vault supports Node Consensus Networks (NCNs) through a process managed by the vault's NCN admin. Here's a high-level overview of how NCNs are added:

1. The vault NCN admin initiates the process by calling the `InitializeVaultNcnTicket` instruction.
2. The instruction creates a `VaultNcnTicket` account, which is a Program Derived Address (PDA) based on the vault and NCN public keys.
3. The `VaultNcnTicket` account tracks the relationship between the vault and the NCN. It contains:
   - The vault's public key
   - The NCN's public key
   - An index (the count of NCNs supported by the vault)
   - A state toggle (to enable/disable the NCN support over time)
   - A bump seed for the PDA
4. The vault's NCN count is incremented to reflect the addition of the new NCN.
5. This process allows the vault to support multiple NCNs, each tracked by its own `VaultNcnTicket`.
6. After initialization, the vault's NCN admin can warmup or cooldown support for NCNs.

Key points:
- Only the vault's NCN admin can add new NCNs.
- The vault's state must be up-to-date before adding an NCN.
- An NCN at some point must have an active `NcnVaultTicket` account, indicating it supports the vault. This does not mean the NCN supports the vault at this point in time.
- By keeping track of indexes, programs can load the vault account to find the number of NCNs supported by the vault and deterministically iterate through NCNs to run aggregate operations.

## 7.2. Adding & Removing Operators

The vault supports operators through a process managed by the vault's operator admin. Here's a high-level overview of how operators are added:

1. The vault operator admin initiates the process by calling the `InitializeVaultOperatorDelegation` instruction.
2. The instruction creates a `VaultOperatorDelegation` account, which is a Program Derived Address (PDA) based on the vault and operator PDAs.
3. The `VaultOperatorDelegation` account tracks the relationship between the vault and the operator. It contains:
   - The vault's public key
   - The operator's public key
   - A delegation state (to track the status of delegation)
   - The last update slot
   - An index (the count of operators supported by the vault)
   - A bump seed for the PDA
4. The vault's operator count is incremented to reflect the addition of the new operator.
5. This process allows the vault to support multiple operators, each tracked by its own `VaultOperatorDelegation` account.

Key points:
- Only the vault's operator admin can add new operators.
- The vault's state must be up-to-date before adding an operator.
- An operator at some point must have an active `OperatorVaultTicket` account, indicating it supports the vault. This does not mean the operator supports the vault at this point in time.
- This process initializes the operator support but does not immediately delegate stake to the operator. Delegation is a separate process managed through the `OperatorVaultTicket` account's state.
- By keeping track of indexes, programs can load the vault account to find the number of operators supported by the vault and deterministically iterate through operators to run aggregate operations.

# 8. Delegations

## 8.1. Adding Delegations
1. The vault delegation admin initiates the process by calling the `AddDelegation` instruction. The delegation instruction needs the vault, operator, `VaultOperatorDelegation` and other accounts.
2. The system performs several checks:
   - Verifies that the vault delegation admin is the signer of the transaction
   - Ensures that the vault's state is up-to-date before adding delegation
3. After the above checks, the vault attempts to increment its internal delegation state, running the following checks before doing so:
   - The system calculates the amount available for delegation, considering:
     - The total tokens deposited in the vault
     - The current total assets already delegated
     - A reserve amount for VRTs pending withdrawal (calculated based on the current VRT supply and token deposits)
   - If the amount to delegate is less than or equal to the amount available for delegation, the delegation is executed by updating the vault's `delegation_state`
4. The `delegate` method in the vault updates the internal `delegation_state`, increasing the `staked_amount`.

Key points:
- Only the vault delegation admin can add new delegations.
- The vault's state must be up-to-date before adding a delegation.
- Delegation increases the `staked_amount` for a specific operator, in addition to the vault's aggregate `delegation_state`.
- The vault will not delegate more tokens than it has available, ensuring it can always meet its obligations. The amount available to delegate includes the total amount of tokens deposited minus the amount already delegated minus the converted amount of VRTs pending withdrawal. This provides protections against the vault over-delegating assets and not being able to meet its withdrawal obligations.

## 8.2. Cooling Down Delegations

1. The vault delegation admin initiates the process by calling the `CooldownDelegation` instruction.

2. The system performs several checks:
   - Verifies that the vault delegation admin is the signer of the transaction
   - Ensures that the vault's state is up-to-date before cooling down delegation

3. If all checks pass, the cooldown process proceeds:
   - The `VaultOperatorDelegation` account's `delegation_state` is updated to reflect the cooldown:
     - The specified amount is moved from `staked_amount` to `enqueued_for_cooldown_amount`
   - The vault's aggregate `delegation_state` is also updated to reflect this change

4. The cooldown process follows a two-epoch cycle:
   - In the current epoch, the amount is marked as `enqueued_for_cooldown_amount`
   - In the next epoch, it becomes `cooling_down_amount`
   - After another epoch, it's considered fully cooled down and available for withdrawal or re-delegation

Key points:
- Only the vault delegation admin can initiate the cooldown process.
- The vault's state must be up-to-date before cooling down a delegation.
- Cooldown decreases the `staked_amount` for a specific operator, in addition to the vault's aggregate `delegation_state` by moving the amount to `cooling_down_amount`.

## 8.3. DelegationState

Both the vault and the operator delegation account keep track of the delegation state. The vault's delegation state shall reflect the aggregate delegation state of all operators. The `DelegationState` struct keeps track of three key amounts:
- `staked_amount`: The amount of stake currently active
- `enqueued_for_cooldown_amount`: Any stake deactivated in the current epoch
- `cooling_down_amount`: Any stake deactivated in the previous epoch, available for re-delegation in the current epoch + 1

# 9. Withdrawal Enqueueing

The withdrawal enqueueing process is a crucial part of the vault's operation, allowing stakers to initiate the withdrawal of their funds. This process involves creating a VaultStakerWithdrawalTicket and transferring the staker's VRT to a holding account. After one full epoch, the staker can complete the withdrawal process in a separate transaction. Here's a high-level description of how the enqueueing logic works:

1. The staker initiates the withdrawal process by calling the `EnqueueWithdrawal` instruction.

2. The system performs several checks:
   - Ensures that the vault's state is up-to-date before enqueuing a withdrawal
   - If the vault has a mint burn admin, it must be set and be the signer of the transaction

3. The `VaultStakerWithdrawalTicket` is created.

4. The vault's `vrt_enqueued_for_cooldown_amount` is incremented by the VRT amount deposited in the ticket.

5. The staker's VRT is transferred to an ATA owned by the `VaultStakerWithdrawalTicket`.

Key points:
- The vault keeps track of all the enqueued withdrawals in `vrt_enqueued_for_cooldown_amount`, `vrt_cooling_down_amount` and `vrt_ready_to_claim_amount` amounts. This is a safeguard to ensure the vault can meet its withdrawal obligations.
- Withdrawals are not immediately available for withdrawal. They must complete the cooldown period of one full epoch before they can be withdrawn.
- Anyone can complete the withdrawal process by calling the `BurnWithdrawalTicket` instruction.
  - This ensures that squatters can't prevent delegation by holding VRTs that can be withdrawn but aren't.
- The amount of VRTs cooling down is tracked in `vrt_cooling_down_amount`, as opposed to assets equal to the redemption price at the time of withdrawal. This is because the redemption price at the time of withdrawal is unknown at the time of enqueuing. This attempts to guarantee that the vault can meet its withdrawal obligations even if the redemption price at the time of withdrawal is lower than the redemption price at the time of enqueuing.

# 10. Epoch Processing

The vault program is designed to be epoch processed. This means that at the end of each epoch, the vault will perform a number of checks and updates to ensure the integrity of the vault's state. This includes summing up the `delegation_state` and updating the enqueued VRT amounts, among other things.

The epoch processing of the vault is facilitated by the `VaultUpdateStateTracker`, which is initialized, cranked, and then closed at the end of each epoch. Here's a high-level overview of this process:

1. Initialize `VaultUpdateStateTracker`:
   - At the beginning of an epoch, a `VaultUpdateStateTracker` is created for the vault.
   - It stores information about the current epoch, the vault it's associated with, and initializes tracking for delegations and withdrawals.
   - The additional assets that need unstaking for withdrawals are calculated and stored.
2. Crank `VaultUpdateStateTracker`:
   - Throughout the epoch, the `VaultUpdateStateTracker` is "cranked" for each `VaultOperatorDelegation`.
   - This process updates the tracker with the current state of each delegation, including staked amounts and cooldowns.
   - It accumulates the delegation states across all operators.
   - If using a greedy withdrawal allocation method, it may force cooldowns to meet withdrawal demands.
3. Close `VaultUpdateStateTracker`:
   - At the end of the epoch, after all delegations have been processed, the `VaultUpdateStateTracker` is closed.
   - The accumulated state from the tracker is copied back to the vault.
   - This final state represents the total delegations, cooldowns, and withdrawal requirements for the vault.
   - The `VaultUpdateStateTracker` account is then closed, and its lamports are typically returned to the payer.

## 10.1. Last Look for VRT Withdrawals

During the vault update process, there's a final opportunity to account for pending VRT withdrawals. This "last look" mechanism ensures that the vault has sufficient assets to cover all withdrawal requests before finalizing the epoch update. Ideally, the vault delegation admin can manage setting aside enough assets to cover the VRT withdrawals. However, in the case that doesn't happen, the last look mechanism ensures that the vault can still meet its withdrawal obligations.

Here's how it works:

1. When initializing the `VaultUpdateStateTracker`, the vault calculates the additional assets needed for withdrawals based on the amount returned by `Vault::calculate_assets_needed_for_withdrawals`.

2. Throughout the update process, the vault may force cooldowns on delegations to meet these withdrawal demands, especially if using the greedy withdrawal allocation method.

This last look ensures that the vault remains responsive to withdrawal requests made throughout the epoch, even up to the last moment before the update is finalized. It helps maintain the vault's liquidity and ability to meet its obligations to VRT holders.

# 11. Burning

## 11.1. Burning VRT Withdrawal Tickets

Burning a VRT Withdrawal Ticket is the process of finalizing a withdrawal from the vault. VRTs that have been withdrawn for more than one full epoch can be burned to receive the underlying assets.

Here's a high-level overview of the process:

1. The user initiates the burn process by calling the `BurnWithdrawalTicket` instruction.
2. The vault performs several checks:
   - Ensures the VRT mint is correct
   - Checks if a mint burn admin is required and present
   - Ensures the vault state doesn't need an update
   - Checks if the withdrawal ticket is withdrawable
3. The vault calculates the burn summary, which includes:
   - The fee amount to be collected
   - The amount of VRT to be burned
   - The amount of underlying assets to be returned to the staker
4. The specified amount of VRT is burned from the user's account.
5. The fee amount of VRT is transferred to the vault's fee account.
6. The calculated amount of underlying assets is transferred from the vault to the user's account.
7. The vault's internal state is updated to reflect the completed withdrawal.
8. The withdrawal ticket is closed, with its lamports returned to the staker.

Key points:
- The burn process includes slippage protection to guard against unexpected price movements.

