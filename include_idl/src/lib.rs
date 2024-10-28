// Inspired from https://github.com/nifty-oss/asset/tree/main/include-idl
#[cfg(feature = "parse")]
pub mod parse;

#[cfg(feature = "shrink")]
mod shrink;

#[cfg(feature = "shrink")]
pub use shrink::compress_idl;

#[macro_export]
macro_rules! include_idl {
    ($s:expr) => {
        #[cfg_attr(
            any(target_arch = "sbf", target_arch = "bpf"),
            link_section = ".solana.idl"
        )]
        #[allow(dead_code)]
        #[no_mangle]
        pub static IDL_BYTES: &[u8] = include_bytes!($s);
    };
}

#[macro_export]
macro_rules! include_kinobi_idl {
    ($s:expr) => {
        #[cfg_attr(
            any(target_arch = "sbf", target_arch = "bpf"),
            link_section = ".kinobi.idl"
        )]
        #[allow(dead_code)]
        #[no_mangle]
        pub static KINOBI_IDL_BYTES: &[u8] = include_bytes!($s);
    };
}
