#![allow(unused_macros)]
#![allow(unused_imports)]

use {
    cvlr::nondet,
    solana_program::{account_info::AccountInfo},
};

use jito_bytemuck::{types::PodU64, AccountDeserialize, Discriminator};

macro_rules! get_ncn {
    ($ncn_info: expr) => {{
        let ncn_data = $ncn_info.data.borrow();
        let ncn = Ncn::try_from_slice_unchecked(& ncn_data).unwrap();
        *ncn
    }};
}

macro_rules! get_operator {
    ($acc_info: expr) => {{
        let data = $acc_info.data.borrow();
        let op = Operator::try_from_slice_unchecked(& data).unwrap();
        *op
    }};
}

macro_rules! get_operator_ncn_ticket {
     ($acc_info: expr) => {{
        let data = $acc_info.data.borrow();
        let operator_ncn_ticket = OperatorNcnTicket::try_from_slice_unchecked(& data).unwrap();
        *operator_ncn_ticket
    }};
}

macro_rules! get_operator_vault_ticket {
     ($acc_info: expr) => {{
        let data = $acc_info.data.borrow();
        let operator_vault_ticket = OperatorVaultTicket::try_from_slice_unchecked(& data).unwrap();
        *operator_vault_ticket
    }};
}


macro_rules! get_config {
     ($acc_info: expr) => {{
        let data = $acc_info.data.borrow();
        let config = Config::try_from_slice_unchecked(& data).unwrap();
        *config
    }};
}

pub(crate) use get_ncn;
pub(crate) use get_operator;
pub(crate) use get_operator_ncn_ticket;
pub(crate) use get_operator_vault_ticket;
pub(crate) use get_config;
