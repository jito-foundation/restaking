 use std::fmt::Debug;
 
 use bytemuck::{Pod, Zeroable};
 use jito_bytemuck::{
     types::{PodU16, PodU64},
     AccountDeserialize, Discriminator,
 };
 use jito_restaking_sdk::error::RestakingError;
 use shank::ShankAccount;
 use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};
 
-const RESERVED_SPACE_LEN: usize = 261;
+use crate::MAX_FEE_BPS;
+
+const RESERVED_SPACE_LEN: usize = 251;
+
+/// Sentinel for `fee_effective_epoch`. Epoch 0 is unreachable as an effective
+/// epoch (`effective = current + 1 >= 1`), so zero unambiguously encodes
+/// "no pending change". This is load-bearing for migration: the two new fields
+/// occupy the first 10 bytes of the previously-zeroed `reserved_space`, so
+/// every pre-existing account deserializes as `pending = 0, effective = 0`
+/// and resolves to its persisted `operator_fee_bps`.
+pub const NO_PENDING_FEE_CHANGE: u64 = 0;
 
 #[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize, ShankAccount)]
 #[repr(C)]
 pub struct Operator {
     pub base: Pubkey,
     pub admin: Pubkey,
     pub ncn_admin: Pubkey,
     pub vault_admin: Pubkey,
     pub delegate_admin: Pubkey,
     pub metadata_admin: Pubkey,
     pub voter: Pubkey,
     index: PodU64,
     ncn_count: PodU64,
     vault_count: PodU64,
-    pub operator_fee_bps: PodU16,
+    operator_fee_bps: PodU16,
     pub bump: u8,
-    reserved_space: [u8; 261],
+    pending_fee_bps: PodU16,
+    fee_effective_epoch: PodU64,
+    reserved_space: [u8; RESERVED_SPACE_LEN],
 }
 
 impl Operator {
     pub fn new(base: Pubkey, admin: Pubkey, index: u64, operator_fee_bps: u16, bump: u8) -> Self {
         Self {
             base,
             admin,
             ncn_admin: admin,
             vault_admin: admin,
             delegate_admin: admin,
             metadata_admin: admin,
             voter: admin,
             index: PodU64::from(index),
             ncn_count: PodU64::from(0),
             vault_count: PodU64::from(0),
             operator_fee_bps: PodU16::from(operator_fee_bps),
             bump,
+            pending_fee_bps: PodU16::from(operator_fee_bps),
+            fee_effective_epoch: PodU64::from(NO_PENDING_FEE_CHANGE),
             reserved_space: [0; RESERVED_SPACE_LEN],
         }
     }
+
+    #[inline]
+    pub fn pending_fee_bps(&self) -> u16 {
+        self.pending_fee_bps.into()
+    }
+
+    #[inline]
+    pub fn fee_effective_epoch(&self) -> u64 {
+        self.fee_effective_epoch.into()
+    }
+
+    /// Fee persisted as active. Callers pricing rewards MUST NOT use this;
+    /// use [`Self::active_fee_bps`].
+    #[inline]
+    pub fn settled_fee_bps(&self) -> u16 {
+        self.operator_fee_bps.into()
+    }
+
+    #[inline]
+    const fn pending_is_live(effective_epoch: u64, current_epoch: u64) -> bool {
+        effective_epoch != NO_PENDING_FEE_CHANGE && current_epoch >= effective_epoch
+    }
+
+    /// Sole authoritative fee resolver. Pure: a snapshot may read this without
+    /// write access to the account.
+    #[inline]
+    pub fn active_fee_bps(&self, current_epoch: u64) -> u16 {
+        if Self::pending_is_live(self.fee_effective_epoch(), current_epoch) {
+            self.pending_fee_bps()
+        } else {
+            self.settled_fee_bps()
+        }
+    }
+
+    #[inline]
+    pub fn has_unresolved_fee_change(&self, current_epoch: u64) -> bool {
+        let effective_epoch = self.fee_effective_epoch();
+        effective_epoch != NO_PENDING_FEE_CHANGE && current_epoch < effective_epoch
+    }
+
+    /// Collapses a matured pending change into the settled slot. Idempotent.
+    /// Required before queueing a new change, otherwise a matured-but-unsettled
+    /// fee would be silently reverted to the stale settled value.
+    fn settle_fee(&mut self, current_epoch: u64) {
+        if Self::pending_is_live(self.fee_effective_epoch(), current_epoch) {
+            self.operator_fee_bps = self.pending_fee_bps;
+            self.fee_effective_epoch = PodU64::from(NO_PENDING_FEE_CHANGE);
+        }
+    }
+
+    /// Queues `new_fee_bps` for `current_epoch + 1`. The fee observable at
+    /// `current_epoch` is invariant under this call.
+    pub fn set_fee_bps(
+        &mut self,
+        new_fee_bps: u16,
+        current_epoch: u64,
+    ) -> Result<u64, ProgramError> {
+        if new_fee_bps > MAX_FEE_BPS {
+            msg!("New fee exceeds maximum allowed fee");
+            return Err(RestakingError::OperatorFeeCapExceeded.into());
+        }
+
+        self.settle_fee(current_epoch);
+
+        let effective_epoch = current_epoch
+            .checked_add(1)
+            .ok_or(ProgramError::ArithmeticOverflow)?;
+
+        self.pending_fee_bps = PodU16::from(new_fee_bps);
+        self.fee_effective_epoch = PodU64::from(effective_epoch);
+
+        Ok(effective_epoch)
+    }
 }