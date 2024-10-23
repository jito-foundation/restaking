use jito_bytemuck::Discriminator;

use crate::{
    config::Config, vault::Vault, vault_ncn_slasher_operator_ticket::VaultNcnSlasherOperatorTicket,
    vault_ncn_slasher_ticket::VaultNcnSlasherTicket, vault_ncn_ticket::VaultNcnTicket,
    vault_operator_delegation::VaultOperatorDelegation,
    vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
    vault_update_state_tracker::VaultUpdateStateTracker,
};

/// Discriminators for vault accounts
/// Values must not change as they are written on chain to determine the type of account
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultDiscriminator {
    Config = 1,
    Vault = 2,
    VaultNcnTicket = 3,
    VaultOperatorDelegation = 4,
    VaultNcnSlasherTicket = 5,
    VaultNcnSlasherOperatorTicket = 6,
    VaultStakerWithdrawalTicket = 7,
    VaultUpdateStateTracker = 8,
}

impl Discriminator for Config {
    const DISCRIMINATOR: u8 = VaultDiscriminator::Config as u8;
}

impl Discriminator for Vault {
    const DISCRIMINATOR: u8 = VaultDiscriminator::Vault as u8;
}

impl Discriminator for VaultNcnTicket {
    const DISCRIMINATOR: u8 = VaultDiscriminator::VaultNcnTicket as u8;
}

impl Discriminator for VaultOperatorDelegation {
    const DISCRIMINATOR: u8 = VaultDiscriminator::VaultOperatorDelegation as u8;
}

impl Discriminator for VaultNcnSlasherTicket {
    const DISCRIMINATOR: u8 = VaultDiscriminator::VaultNcnSlasherTicket as u8;
}

impl Discriminator for VaultNcnSlasherOperatorTicket {
    const DISCRIMINATOR: u8 = VaultDiscriminator::VaultNcnSlasherOperatorTicket as u8;
}

impl Discriminator for VaultStakerWithdrawalTicket {
    const DISCRIMINATOR: u8 = VaultDiscriminator::VaultStakerWithdrawalTicket as u8;
}

impl Discriminator for VaultUpdateStateTracker {
    const DISCRIMINATOR: u8 = VaultDiscriminator::VaultUpdateStateTracker as u8;
}
