#![allow(clippy::all)]
#![allow(clippy::nursery)]
#![allow(clippy::integer_division)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::style)]
#![allow(clippy::perf)]
#![allow(deprecated)]
mod generated;
pub mod log;

use generated::*;

pub mod accounts {
    pub use super::generated::accounts::*;
}

pub mod instructions {
    pub use super::generated::instructions::*;
}

pub mod errors {
    pub use super::generated::errors::*;
}

pub mod types {
    pub use super::generated::types::*;
}

pub mod programs {
    pub use super::generated::programs::*;
}
