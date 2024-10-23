use jito_bytemuck::Discriminator;

use crate::{
    config::Config, ncn::Ncn, ncn_operator_state::NcnOperatorState,
    ncn_vault_slasher_ticket::NcnVaultSlasherTicket, ncn_vault_ticket::NcnVaultTicket,
    operator::Operator, operator_vault_ticket::OperatorVaultTicket,
};

/// Discriminators for restaking accounts
/// Values must not change as they are written on chain to determine the type of account
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestakingDiscriminator {
    Config = 1,
    Ncn = 2,
    Operator = 3,
    NcnOperatorState = 4,
    OperatorVaultTicket = 5,
    NcnVaultTicket = 6,
    NcnVaultSlasherTicket = 7,
}

impl Discriminator for Config {
    const DISCRIMINATOR: u8 = RestakingDiscriminator::Config as u8;
}

impl Discriminator for Ncn {
    const DISCRIMINATOR: u8 = RestakingDiscriminator::Ncn as u8;
}

impl Discriminator for Operator {
    const DISCRIMINATOR: u8 = RestakingDiscriminator::Operator as u8;
}

impl Discriminator for NcnOperatorState {
    const DISCRIMINATOR: u8 = RestakingDiscriminator::NcnOperatorState as u8;
}

impl Discriminator for OperatorVaultTicket {
    const DISCRIMINATOR: u8 = RestakingDiscriminator::OperatorVaultTicket as u8;
}

impl Discriminator for NcnVaultTicket {
    const DISCRIMINATOR: u8 = RestakingDiscriminator::NcnVaultTicket as u8;
}

impl Discriminator for NcnVaultSlasherTicket {
    const DISCRIMINATOR: u8 = RestakingDiscriminator::NcnVaultSlasherTicket as u8;
}
