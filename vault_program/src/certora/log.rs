#![allow(unused_macros)]
#![allow(unused_imports)]

macro_rules! msg {
    ($msg:expr) => { };
    ($($arg:tt)*) => { };
}

pub(crate) use msg;
