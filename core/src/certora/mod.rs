pub mod mocks;

pub mod utils {
    mod rt_decl {
        extern "C" {
            pub fn sol_set_clock_sysvar(clk: solana_program::clock::Clock);
        }
    }

    pub fn cvlr_advance_clock_slot() {
        use solana_program::sysvar::Sysvar as _;
        let mut clk = solana_program::clock::Clock::get().unwrap();
        clk.slot = clk.slot + cvlr::nondet::nondet_with(|x: &u64| *x > 0);
        unsafe {
            rt_decl::sol_set_clock_sysvar(clk);
        }
    }
}
