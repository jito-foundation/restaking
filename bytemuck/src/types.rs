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

#[derive(Clone, Copy, Default, PartialEq, Pod, Zeroable, Eq)]
#[repr(transparent)]
pub struct PodU128([u8; 16]);
impl_int_conversion!(PodU128, u128);

impl Debug for PodU128 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v: u128 = u128::from(self);
        f.debug_tuple("PodU128").field(&v).finish()
    }
}

/// The standard `bool` is not a `Pod`, define a replacement that is
#[derive(Clone, Copy, Default, PartialEq, Eq, Pod, Zeroable)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pod_u16() {
        // Test zero
        let zero = PodU16::default();
        assert_eq!(u16::from(zero), 0);

        // Test conversion from u16
        let n: u16 = 12345;
        let pod = PodU16::from(n);
        assert_eq!(u16::from(pod), n);

        // Test reference conversion
        let pod_from_ref = PodU16::from(&n);
        assert_eq!(u16::from(pod_from_ref), n);

        // Test max value
        let max = PodU16::from(u16::MAX);
        assert_eq!(u16::from(max), u16::MAX);
    }

    #[test]
    fn test_pod_u32() {
        // Test zero
        let zero = PodU32::default();
        assert_eq!(u32::from(zero), 0);

        // Test conversion from u32
        let n: u32 = 305_419_896;
        let pod = PodU32::from(n);
        assert_eq!(u32::from(pod), n);

        // Test reference conversion
        let pod_from_ref = PodU32::from(&n);
        assert_eq!(u32::from(pod_from_ref), n);

        // Test max value
        let max = PodU32::from(u32::MAX);
        assert_eq!(u32::from(max), u32::MAX);
    }

    #[test]
    fn test_pod_u64() {
        // Test zero
        let zero = PodU64::default();
        assert_eq!(u64::from(zero), 0);

        // Test conversion from u64
        let n: u64 = 1_311_768_467_294_899_695;
        let pod = PodU64::from(n);
        assert_eq!(u64::from(pod), n);

        // Test reference conversion
        let pod_from_ref = PodU64::from(&n);
        assert_eq!(u64::from(pod_from_ref), n);

        // Test max value
        let max = PodU64::from(u64::MAX);
        assert_eq!(u64::from(max), u64::MAX);
    }

    #[test]
    fn test_pod_u128() {
        // Test zero
        let zero = PodU128::default();
        assert_eq!(u128::from(zero), 0);

        // Test conversion from u128
        let n: u128 = 170_141_183_460_469_231_731_687_303_715_884_105_727;
        let pod = PodU128::from(n);
        assert_eq!(u128::from(pod), n);

        // Test reference conversion
        let pod_from_ref = PodU128::from(&n);
        assert_eq!(u128::from(pod_from_ref), n);

        // Test max value
        let max = PodU128::from(u128::MAX);
        assert_eq!(u128::from(max), u128::MAX);
    }

    #[test]
    fn test_pod_bool() {
        // Test default is false
        let default_bool = PodBool::default();
        assert_eq!(bool::from(default_bool), false);

        // Test true conversion
        let true_bool = PodBool::from(true);
        assert_eq!(bool::from(true_bool), true);

        // Test false conversion
        let false_bool = PodBool::from(false);
        assert_eq!(bool::from(false_bool), false);

        // Test reference conversion
        let true_ref = true;
        let pod_from_ref = PodBool::from(&true_ref);
        assert_eq!(bool::from(pod_from_ref), true);

        // Test non-zero values are true
        let non_zero = PodBool(2);
        assert_eq!(bool::from(non_zero), true);
    }

    #[test]
    fn test_debug_formatting() {
        // Test debug formatting for all types
        assert_eq!(format!("{:?}", PodU16::from(12345)), "PodU16(12345)");
        assert_eq!(
            format!("{:?}", PodU32::from(305419896)),
            "PodU32(305419896)"
        );
        assert_eq!(
            format!("{:?}", PodU64::from(1311768467294899695)),
            "PodU64(1311768467294899695)"
        );
        assert_eq!(
            format!(
                "{:?}",
                PodU128::from(170141183460469231731687303715884105727)
            ),
            "PodU128(170141183460469231731687303715884105727)"
        );
        assert_eq!(format!("{:?}", PodBool::from(true)), "PodBool(true)");
        assert_eq!(format!("{:?}", PodBool::from(false)), "PodBool(false)");
    }

    #[test]
    fn test_byte_representation() {
        // Test that the byte representation is correct (little-endian)
        let n: u16 = 0x1234;
        let pod = PodU16::from(n);
        assert_eq!(pod.0, [0x34, 0x12]);

        let n: u32 = 0x12345678;
        let pod = PodU32::from(n);
        assert_eq!(pod.0, [0x78, 0x56, 0x34, 0x12]);

        let n: u64 = 0x1234567890ABCDEF;
        let pod = PodU64::from(n);
        assert_eq!(pod.0, [0xEF, 0xCD, 0xAB, 0x90, 0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_pod_properties() {
        // Test that Pod types implement expected traits
        fn assert_pod<T: Pod + Default + Copy + Clone + PartialEq + Eq>() {}

        assert_pod::<PodU16>();
        assert_pod::<PodU32>();
        assert_pod::<PodU64>();
        assert_pod::<PodU128>();
        assert_pod::<PodBool>();
    }
}
