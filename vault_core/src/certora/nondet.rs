//! Nondet trait impl
 
use {cvlr::prelude::*, cvlr_solana::cvlr_nondet_pubkey};

impl cvlr::nondet::Nondet for crate::vault::Vault {
    fn nondet() -> Self {
        return crate::vault::Vault::new(
            cvlr_nondet_pubkey(),
            cvlr_nondet_pubkey(),
            cvlr_nondet_pubkey(),
            nondet::<u64>(),
            cvlr_nondet_pubkey(),
            nondet::<u16>(),
            nondet::<u16>(),
            nondet::<u16>(),
            nondet::<u16>(),
            nondet::<u8>(),
            nondet::<u64>(),
        )
        .unwrap();
    }
}
