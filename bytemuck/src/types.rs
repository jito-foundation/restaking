use std::fmt::{Debug, Formatter};

// https://github.com/solana-labs/solana-program-library/tree/master/libraries/pod
use bytemuck::{Pod, Zeroable};

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

#[derive(Clone, Copy, Default, PartialEq, Pod, Zeroable, Eq)]
#[repr(transparent)]
pub struct PodU16([u8; 2]);
impl_int_conversion!(PodU16, u16);

impl Debug for PodU16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v: u16 = u16::from(self);
        f.debug_tuple("PodU16").field(&v).finish()
    }
}

#[derive(Clone, Copy, Default, PartialEq, Pod, Zeroable, Eq)]
#[repr(transparent)]
pub struct PodU32([u8; 4]);
impl_int_conversion!(PodU32, u32);

impl Debug for PodU32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v: u32 = u32::from(self);
        f.debug_tuple("PodU32").field(&v).finish()
    }
}

#[derive(Clone, Copy, Default, PartialEq, Pod, Zeroable, Eq)]
#[repr(transparent)]
pub struct PodU64([u8; 8]);
impl_int_conversion!(PodU64, u64);

impl Debug for PodU64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v: u64 = u64::from(self);
        f.debug_tuple("PodU64").field(&v).finish()
    }
}
