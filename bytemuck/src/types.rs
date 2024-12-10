use std::fmt::{Debug, Formatter};

// https://github.com/solana-labs/solana-program-library/tree/master/libraries/pod
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// The standard `bool` is not a `Pod`, define a replacement that is
#[derive(Clone, Copy, Default, PartialEq, Eq, Pod, Zeroable, Serialize, Deserialize)]
#[repr(transparent)]
pub struct PodBool(pub u8);
impl PodBool {
    pub const fn from_bool(b: bool) -> Self {
        Self(if b { 1 } else { 0 })
    }
}

impl From<bool> for PodBool {
    fn from(b: bool) -> Self {
        Self::from_bool(b)
    }
}

impl From<&bool> for PodBool {
    fn from(b: &bool) -> Self {
        Self(if *b { 1 } else { 0 })
    }
}

impl From<&PodBool> for bool {
    fn from(b: &PodBool) -> Self {
        b.0 != 0
    }
}

impl From<PodBool> for bool {
    fn from(b: PodBool) -> Self {
        b.0 != 0
    }
}

impl Debug for PodBool {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v: bool = bool::from(self);
        f.debug_tuple("PodBool").field(&v).finish()
    }
}

/// Simple macro for implementing conversion functions between Pod* ints and
/// standard ints.
///
/// The standard int types can cause alignment issues when placed in a `Pod`,
/// so these replacements are usable in all `Pod`s.
#[macro_export]
macro_rules! impl_int_conversion {
    ($P:ty, $I:ty) => {
        impl From<$I> for $P {
            fn from(n: $I) -> Self {
                Self(n.to_le_bytes())
            }
        }
        impl From<&$I> for $P {
            fn from(n: &$I) -> Self {
                Self(n.to_le_bytes())
            }
        }
        impl From<$P> for $I {
            fn from(pod: $P) -> Self {
                Self::from_le_bytes(pod.0)
            }
        }
        impl From<&$P> for $I {
            fn from(pod: &$P) -> Self {
                Self::from_le_bytes(pod.0)
            }
        }
    };
}

#[derive(Clone, Copy, Default, PartialEq, Pod, Zeroable, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct PodU16([u8; 2]);
impl_int_conversion!(PodU16, u16);

impl Debug for PodU16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v: u16 = u16::from(self);
        f.debug_tuple("PodU16").field(&v).finish()
    }
}

#[derive(Clone, Copy, Default, PartialEq, Pod, Zeroable, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct PodU32([u8; 4]);
impl_int_conversion!(PodU32, u32);

impl Debug for PodU32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v: u32 = u32::from(self);
        f.debug_tuple("PodU32").field(&v).finish()
    }
}

#[derive(Clone, Copy, Default, PartialEq, Pod, Zeroable, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct PodU64([u8; 8]);
impl_int_conversion!(PodU64, u64);

impl Debug for PodU64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v: u64 = u64::from(self);
        f.debug_tuple("PodU64").field(&v).finish()
    }
}

#[derive(Clone, Copy, Default, PartialEq, Pod, Zeroable, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct PodU128([u8; 16]);
impl_int_conversion!(PodU128, u128);

impl Debug for PodU128 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v: u128 = u128::from(self);
        f.debug_tuple("PodU128").field(&v).finish()
    }
}
