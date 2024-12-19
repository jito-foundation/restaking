use anchor_lang::prelude::AccountInfo;
use anchor_lang::prelude::AccountLoader;
use anchor_lang::prelude::ProgramError;
use jito_bytemuck::Discriminator;
use once_cell::sync::Lazy;
use paste::paste;
use solana_program::pubkey::Pubkey;
use std::str::FromStr;

pub const RESTAKING_PROGRAM_ID: Lazy<Pubkey> =
    Lazy::new(|| Pubkey::from_str(env!("RESTAKING_PROGRAM_ID")).unwrap());

macro_rules! jito_anchor_adapter {
    ($name:ident, $inner:ty, $parent:ident) => {
        #[repr(C)]
        #[derive(Clone, Copy, shrinkwraprs::Shrinkwrap, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct $name(pub $inner);

        impl anchor_lang::ZeroCopy for $name {}

        impl anchor_lang::Discriminator for $name {
            const DISCRIMINATOR: [u8; 8] = [<$inner>::DISCRIMINATOR, 0, 0, 0, 0, 0, 0, 0];
        }

        impl anchor_lang::Owner for $name {
            fn owner() -> Pubkey {
                $parent.clone()
            }
        }

        impl $name {
            pub fn try_from<'a>(
                account: &'a AccountInfo<'a>,
            ) -> Result<AccountLoader<'a, Self>, ProgramError> {
                Ok(AccountLoader::try_from(account)?)
            }
        }
    };
}

macro_rules! jito_restaking_adapter {
    ($name:ident) => {
        paste! {
            jito_anchor_adapter!($name, jito_restaking_core::[<$name:snake>]::$name, RESTAKING_PROGRAM_ID);
        }
    };
}

jito_restaking_adapter!(Config);
jito_restaking_adapter!(Operator);
jito_restaking_adapter!(Ncn);
jito_restaking_adapter!(NcnOperatorState);
jito_restaking_adapter!(NcnVaultSlasherTicket);
jito_restaking_adapter!(NcnVaultTicket);
jito_restaking_adapter!(OperatorVaultTicket);

#[derive(Debug, Clone)]
pub struct JitoRestakingProgram<'info>(pub AccountInfo<'info>);
impl anchor_lang::Id for JitoRestakingProgram<'_> {
    fn id() -> Pubkey {
        *RESTAKING_PROGRAM_ID
    }
}
