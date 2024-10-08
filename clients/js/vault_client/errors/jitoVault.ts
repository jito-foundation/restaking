/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  isProgramError,
  type Address,
  type SOLANA_ERROR__INSTRUCTION_ERROR__CUSTOM,
  type SolanaError,
} from '@solana/web3.js';
import { JITO_VAULT_PROGRAM_ADDRESS } from '../programs';

/** VaultSlashUnderflow: VaultSlashUnderflow */
export const JITO_VAULT_ERROR__VAULT_SLASH_UNDERFLOW = 0x3e8; // 1000
/** VaultInsufficientFunds: VaultInsufficientFunds */
export const JITO_VAULT_ERROR__VAULT_INSUFFICIENT_FUNDS = 0x3e9; // 1001
/** VaultOverflow: VaultOverflow */
export const JITO_VAULT_ERROR__VAULT_OVERFLOW = 0x3ea; // 1002
/** VaultOperatorAdminInvalid: VaultOperatorAdminInvalid */
export const JITO_VAULT_ERROR__VAULT_OPERATOR_ADMIN_INVALID = 0x3eb; // 1003
/** VaultAdminInvalid: VaultAdminInvalid */
export const JITO_VAULT_ERROR__VAULT_ADMIN_INVALID = 0x3ec; // 1004
/** VaultCapacityAdminInvalid: VaultCapacityAdminInvalid */
export const JITO_VAULT_ERROR__VAULT_CAPACITY_ADMIN_INVALID = 0x3ed; // 1005
/** VaultMintBurnAdminInvalid: VaultMintBurnAdminInvalid */
export const JITO_VAULT_ERROR__VAULT_MINT_BURN_ADMIN_INVALID = 0x3ee; // 1006
/** VaultDelegationAdminInvalid: VaultDelegationAdminInvalid */
export const JITO_VAULT_ERROR__VAULT_DELEGATION_ADMIN_INVALID = 0x3ef; // 1007
/** VaultDelegateAssetAdminInvalid: VaultDelegateAssetAdminInvalid */
export const JITO_VAULT_ERROR__VAULT_DELEGATE_ASSET_ADMIN_INVALID = 0x3f0; // 1008
/** VaultCapacityExceeded: VaultCapacityExceeded */
export const JITO_VAULT_ERROR__VAULT_CAPACITY_EXCEEDED = 0x3f1; // 1009
/** VaultSlasherAdminInvalid: VaultSlasherAdminInvalid */
export const JITO_VAULT_ERROR__VAULT_SLASHER_ADMIN_INVALID = 0x3f2; // 1010
/** VaultNcnAdminInvalid: VaultNcnAdminInvalid */
export const JITO_VAULT_ERROR__VAULT_NCN_ADMIN_INVALID = 0x3f3; // 1011
/** VaultFeeAdminInvalid: VaultFeeAdminInvalid */
export const JITO_VAULT_ERROR__VAULT_FEE_ADMIN_INVALID = 0x3f4; // 1012
/** VaultFeeCapExceeded: VaultFeeCapExceeded */
export const JITO_VAULT_ERROR__VAULT_FEE_CAP_EXCEEDED = 0x3f5; // 1013
/** VaultFeeChangeTooSoon: VaultFeeChangeTooSoon */
export const JITO_VAULT_ERROR__VAULT_FEE_CHANGE_TOO_SOON = 0x3f6; // 1014
/** VaultFeeBumpTooLarge: VaultFeeBumpTooLarge */
export const JITO_VAULT_ERROR__VAULT_FEE_BUMP_TOO_LARGE = 0x3f7; // 1015
/** VaultUnderflow: VaultUnderflow */
export const JITO_VAULT_ERROR__VAULT_UNDERFLOW = 0x3f8; // 1016
/** VaultUpdateNeeded: VaultUpdateNeeded */
export const JITO_VAULT_ERROR__VAULT_UPDATE_NEEDED = 0x3f9; // 1017
/** VaultIsUpdated: VaultIsUpdated */
export const JITO_VAULT_ERROR__VAULT_IS_UPDATED = 0x3fa; // 1018
/** VaultOperatorDelegationUpdateNeeded: VaultOperatorDelegationUpdateNeeded */
export const JITO_VAULT_ERROR__VAULT_OPERATOR_DELEGATION_UPDATE_NEEDED = 0x3fb; // 1019
/** VaultOperatorDelegationIsUpdated: VaultOperatorDelegationIsUpdated */
export const JITO_VAULT_ERROR__VAULT_OPERATOR_DELEGATION_IS_UPDATED = 0x3fc; // 1020
/** VaultUpdateIncorrectIndex: VaultUpdateIncorrectIndex */
export const JITO_VAULT_ERROR__VAULT_UPDATE_INCORRECT_INDEX = 0x3fd; // 1021
/** VaultUpdateStateNotFinishedUpdating: VaultUpdateStateNotFinishedUpdating */
export const JITO_VAULT_ERROR__VAULT_UPDATE_STATE_NOT_FINISHED_UPDATING = 0x3fe; // 1022
/** VaultSecurityOverflow: VaultSecurityOverflow */
export const JITO_VAULT_ERROR__VAULT_SECURITY_OVERFLOW = 0x3ff; // 1023
/** VaultSlashIncomplete: VaultSlashIncomplete */
export const JITO_VAULT_ERROR__VAULT_SLASH_INCOMPLETE = 0x400; // 1024
/** VaultSecurityUnderflow: VaultSecurityUnderflow */
export const JITO_VAULT_ERROR__VAULT_SECURITY_UNDERFLOW = 0x401; // 1025
/** SlippageError: SlippageError */
export const JITO_VAULT_ERROR__SLIPPAGE_ERROR = 0x402; // 1026
/** VaultStakerWithdrawalTicketNotWithdrawable: VaultStakerWithdrawalTicketNotWithdrawable */
export const JITO_VAULT_ERROR__VAULT_STAKER_WITHDRAWAL_TICKET_NOT_WITHDRAWABLE = 0x403; // 1027
/** VaultNcnSlasherTicketFailedCooldown: VaultNcnSlasherTicketFailedCooldown */
export const JITO_VAULT_ERROR__VAULT_NCN_SLASHER_TICKET_FAILED_COOLDOWN = 0x404; // 1028
/** VaultNcnSlasherTicketFailedWarmup: VaultNcnSlasherTicketFailedWarmup */
export const JITO_VAULT_ERROR__VAULT_NCN_SLASHER_TICKET_FAILED_WARMUP = 0x405; // 1029
/** VaultNcnTicketFailedCooldown: VaultNcnTicketFailedCooldown */
export const JITO_VAULT_ERROR__VAULT_NCN_TICKET_FAILED_COOLDOWN = 0x406; // 1030
/** VaultNcnTicketFailedWarmup: VaultNcnTicketFailedWarmup */
export const JITO_VAULT_ERROR__VAULT_NCN_TICKET_FAILED_WARMUP = 0x407; // 1031
/** VaultNcnTicketUnslashable: VaultNcnTicketUnslashable */
export const JITO_VAULT_ERROR__VAULT_NCN_TICKET_UNSLASHABLE = 0x408; // 1032
/** OperatorVaultTicketUnslashable: OperatorVaultTicketUnslashable */
export const JITO_VAULT_ERROR__OPERATOR_VAULT_TICKET_UNSLASHABLE = 0x409; // 1033
/** NcnOperatorStateUnslashable: NcnOperatorStateUnslashable */
export const JITO_VAULT_ERROR__NCN_OPERATOR_STATE_UNSLASHABLE = 0x40a; // 1034
/** VaultNcnSlasherTicketUnslashable: VaultNcnSlasherTicketUnslashable */
export const JITO_VAULT_ERROR__VAULT_NCN_SLASHER_TICKET_UNSLASHABLE = 0x40b; // 1035
/** NcnVaultTicketUnslashable: NcnVaultTicketUnslashable */
export const JITO_VAULT_ERROR__NCN_VAULT_TICKET_UNSLASHABLE = 0x40c; // 1036
/** NcnVaultSlasherTicketUnslashable: NcnVaultSlasherTicketUnslashable */
export const JITO_VAULT_ERROR__NCN_VAULT_SLASHER_TICKET_UNSLASHABLE = 0x40d; // 1037
/** VaultMaxSlashedPerOperatorExceeded: VaultMaxSlashedPerOperatorExceeded */
export const JITO_VAULT_ERROR__VAULT_MAX_SLASHED_PER_OPERATOR_EXCEEDED = 0x40e; // 1038
/** VaultStakerWithdrawalTicketInvalidStaker: VaultStakerWithdrawalTicketInvalidStaker */
export const JITO_VAULT_ERROR__VAULT_STAKER_WITHDRAWAL_TICKET_INVALID_STAKER = 0x40f; // 1039
/** SlasherOverflow: SlasherOverflow */
export const JITO_VAULT_ERROR__SLASHER_OVERFLOW = 0x410; // 1040
/** NcnOverflow: NcnOverflow */
export const JITO_VAULT_ERROR__NCN_OVERFLOW = 0x411; // 1041
/** OperatorOverflow: OperatorOverflow */
export const JITO_VAULT_ERROR__OPERATOR_OVERFLOW = 0x412; // 1042
/** VaultDelegationZero: VaultDelegationZero */
export const JITO_VAULT_ERROR__VAULT_DELEGATION_ZERO = 0x413; // 1043
/** VaultCooldownZero: VaultCooldownZero */
export const JITO_VAULT_ERROR__VAULT_COOLDOWN_ZERO = 0x414; // 1044
/** VaultBurnZero: VaultBurnZero */
export const JITO_VAULT_ERROR__VAULT_BURN_ZERO = 0x415; // 1045
/** VaultEnqueueWithdrawalAmountZero: VaultEnqueueWithdrawalAmountZero */
export const JITO_VAULT_ERROR__VAULT_ENQUEUE_WITHDRAWAL_AMOUNT_ZERO = 0x416; // 1046
/** VaultMintZero: VaultMintZero */
export const JITO_VAULT_ERROR__VAULT_MINT_ZERO = 0x417; // 1047

export type JitoVaultError =
  | typeof JITO_VAULT_ERROR__NCN_OPERATOR_STATE_UNSLASHABLE
  | typeof JITO_VAULT_ERROR__NCN_OVERFLOW
  | typeof JITO_VAULT_ERROR__NCN_VAULT_SLASHER_TICKET_UNSLASHABLE
  | typeof JITO_VAULT_ERROR__NCN_VAULT_TICKET_UNSLASHABLE
  | typeof JITO_VAULT_ERROR__OPERATOR_OVERFLOW
  | typeof JITO_VAULT_ERROR__OPERATOR_VAULT_TICKET_UNSLASHABLE
  | typeof JITO_VAULT_ERROR__SLASHER_OVERFLOW
  | typeof JITO_VAULT_ERROR__SLIPPAGE_ERROR
  | typeof JITO_VAULT_ERROR__VAULT_ADMIN_INVALID
  | typeof JITO_VAULT_ERROR__VAULT_BURN_ZERO
  | typeof JITO_VAULT_ERROR__VAULT_CAPACITY_ADMIN_INVALID
  | typeof JITO_VAULT_ERROR__VAULT_CAPACITY_EXCEEDED
  | typeof JITO_VAULT_ERROR__VAULT_COOLDOWN_ZERO
  | typeof JITO_VAULT_ERROR__VAULT_DELEGATE_ASSET_ADMIN_INVALID
  | typeof JITO_VAULT_ERROR__VAULT_DELEGATION_ADMIN_INVALID
  | typeof JITO_VAULT_ERROR__VAULT_DELEGATION_ZERO
  | typeof JITO_VAULT_ERROR__VAULT_ENQUEUE_WITHDRAWAL_AMOUNT_ZERO
  | typeof JITO_VAULT_ERROR__VAULT_FEE_ADMIN_INVALID
  | typeof JITO_VAULT_ERROR__VAULT_FEE_BUMP_TOO_LARGE
  | typeof JITO_VAULT_ERROR__VAULT_FEE_CAP_EXCEEDED
  | typeof JITO_VAULT_ERROR__VAULT_FEE_CHANGE_TOO_SOON
  | typeof JITO_VAULT_ERROR__VAULT_INSUFFICIENT_FUNDS
  | typeof JITO_VAULT_ERROR__VAULT_IS_UPDATED
  | typeof JITO_VAULT_ERROR__VAULT_MAX_SLASHED_PER_OPERATOR_EXCEEDED
  | typeof JITO_VAULT_ERROR__VAULT_MINT_BURN_ADMIN_INVALID
  | typeof JITO_VAULT_ERROR__VAULT_MINT_ZERO
  | typeof JITO_VAULT_ERROR__VAULT_NCN_ADMIN_INVALID
  | typeof JITO_VAULT_ERROR__VAULT_NCN_SLASHER_TICKET_FAILED_COOLDOWN
  | typeof JITO_VAULT_ERROR__VAULT_NCN_SLASHER_TICKET_FAILED_WARMUP
  | typeof JITO_VAULT_ERROR__VAULT_NCN_SLASHER_TICKET_UNSLASHABLE
  | typeof JITO_VAULT_ERROR__VAULT_NCN_TICKET_FAILED_COOLDOWN
  | typeof JITO_VAULT_ERROR__VAULT_NCN_TICKET_FAILED_WARMUP
  | typeof JITO_VAULT_ERROR__VAULT_NCN_TICKET_UNSLASHABLE
  | typeof JITO_VAULT_ERROR__VAULT_OPERATOR_ADMIN_INVALID
  | typeof JITO_VAULT_ERROR__VAULT_OPERATOR_DELEGATION_IS_UPDATED
  | typeof JITO_VAULT_ERROR__VAULT_OPERATOR_DELEGATION_UPDATE_NEEDED
  | typeof JITO_VAULT_ERROR__VAULT_OVERFLOW
  | typeof JITO_VAULT_ERROR__VAULT_SECURITY_OVERFLOW
  | typeof JITO_VAULT_ERROR__VAULT_SECURITY_UNDERFLOW
  | typeof JITO_VAULT_ERROR__VAULT_SLASHER_ADMIN_INVALID
  | typeof JITO_VAULT_ERROR__VAULT_SLASH_INCOMPLETE
  | typeof JITO_VAULT_ERROR__VAULT_SLASH_UNDERFLOW
  | typeof JITO_VAULT_ERROR__VAULT_STAKER_WITHDRAWAL_TICKET_INVALID_STAKER
  | typeof JITO_VAULT_ERROR__VAULT_STAKER_WITHDRAWAL_TICKET_NOT_WITHDRAWABLE
  | typeof JITO_VAULT_ERROR__VAULT_UNDERFLOW
  | typeof JITO_VAULT_ERROR__VAULT_UPDATE_INCORRECT_INDEX
  | typeof JITO_VAULT_ERROR__VAULT_UPDATE_NEEDED
  | typeof JITO_VAULT_ERROR__VAULT_UPDATE_STATE_NOT_FINISHED_UPDATING;

let jitoVaultErrorMessages: Record<JitoVaultError, string> | undefined;
if (process.env.NODE_ENV !== 'production') {
  jitoVaultErrorMessages = {
    [JITO_VAULT_ERROR__NCN_OPERATOR_STATE_UNSLASHABLE]: `NcnOperatorStateUnslashable`,
    [JITO_VAULT_ERROR__NCN_OVERFLOW]: `NcnOverflow`,
    [JITO_VAULT_ERROR__NCN_VAULT_SLASHER_TICKET_UNSLASHABLE]: `NcnVaultSlasherTicketUnslashable`,
    [JITO_VAULT_ERROR__NCN_VAULT_TICKET_UNSLASHABLE]: `NcnVaultTicketUnslashable`,
    [JITO_VAULT_ERROR__OPERATOR_OVERFLOW]: `OperatorOverflow`,
    [JITO_VAULT_ERROR__OPERATOR_VAULT_TICKET_UNSLASHABLE]: `OperatorVaultTicketUnslashable`,
    [JITO_VAULT_ERROR__SLASHER_OVERFLOW]: `SlasherOverflow`,
    [JITO_VAULT_ERROR__SLIPPAGE_ERROR]: `SlippageError`,
    [JITO_VAULT_ERROR__VAULT_ADMIN_INVALID]: `VaultAdminInvalid`,
    [JITO_VAULT_ERROR__VAULT_BURN_ZERO]: `VaultBurnZero`,
    [JITO_VAULT_ERROR__VAULT_CAPACITY_ADMIN_INVALID]: `VaultCapacityAdminInvalid`,
    [JITO_VAULT_ERROR__VAULT_CAPACITY_EXCEEDED]: `VaultCapacityExceeded`,
    [JITO_VAULT_ERROR__VAULT_COOLDOWN_ZERO]: `VaultCooldownZero`,
    [JITO_VAULT_ERROR__VAULT_DELEGATE_ASSET_ADMIN_INVALID]: `VaultDelegateAssetAdminInvalid`,
    [JITO_VAULT_ERROR__VAULT_DELEGATION_ADMIN_INVALID]: `VaultDelegationAdminInvalid`,
    [JITO_VAULT_ERROR__VAULT_DELEGATION_ZERO]: `VaultDelegationZero`,
    [JITO_VAULT_ERROR__VAULT_ENQUEUE_WITHDRAWAL_AMOUNT_ZERO]: `VaultEnqueueWithdrawalAmountZero`,
    [JITO_VAULT_ERROR__VAULT_FEE_ADMIN_INVALID]: `VaultFeeAdminInvalid`,
    [JITO_VAULT_ERROR__VAULT_FEE_BUMP_TOO_LARGE]: `VaultFeeBumpTooLarge`,
    [JITO_VAULT_ERROR__VAULT_FEE_CAP_EXCEEDED]: `VaultFeeCapExceeded`,
    [JITO_VAULT_ERROR__VAULT_FEE_CHANGE_TOO_SOON]: `VaultFeeChangeTooSoon`,
    [JITO_VAULT_ERROR__VAULT_INSUFFICIENT_FUNDS]: `VaultInsufficientFunds`,
    [JITO_VAULT_ERROR__VAULT_IS_UPDATED]: `VaultIsUpdated`,
    [JITO_VAULT_ERROR__VAULT_MAX_SLASHED_PER_OPERATOR_EXCEEDED]: `VaultMaxSlashedPerOperatorExceeded`,
    [JITO_VAULT_ERROR__VAULT_MINT_BURN_ADMIN_INVALID]: `VaultMintBurnAdminInvalid`,
    [JITO_VAULT_ERROR__VAULT_MINT_ZERO]: `VaultMintZero`,
    [JITO_VAULT_ERROR__VAULT_NCN_ADMIN_INVALID]: `VaultNcnAdminInvalid`,
    [JITO_VAULT_ERROR__VAULT_NCN_SLASHER_TICKET_FAILED_COOLDOWN]: `VaultNcnSlasherTicketFailedCooldown`,
    [JITO_VAULT_ERROR__VAULT_NCN_SLASHER_TICKET_FAILED_WARMUP]: `VaultNcnSlasherTicketFailedWarmup`,
    [JITO_VAULT_ERROR__VAULT_NCN_SLASHER_TICKET_UNSLASHABLE]: `VaultNcnSlasherTicketUnslashable`,
    [JITO_VAULT_ERROR__VAULT_NCN_TICKET_FAILED_COOLDOWN]: `VaultNcnTicketFailedCooldown`,
    [JITO_VAULT_ERROR__VAULT_NCN_TICKET_FAILED_WARMUP]: `VaultNcnTicketFailedWarmup`,
    [JITO_VAULT_ERROR__VAULT_NCN_TICKET_UNSLASHABLE]: `VaultNcnTicketUnslashable`,
    [JITO_VAULT_ERROR__VAULT_OPERATOR_ADMIN_INVALID]: `VaultOperatorAdminInvalid`,
    [JITO_VAULT_ERROR__VAULT_OPERATOR_DELEGATION_IS_UPDATED]: `VaultOperatorDelegationIsUpdated`,
    [JITO_VAULT_ERROR__VAULT_OPERATOR_DELEGATION_UPDATE_NEEDED]: `VaultOperatorDelegationUpdateNeeded`,
    [JITO_VAULT_ERROR__VAULT_OVERFLOW]: `VaultOverflow`,
    [JITO_VAULT_ERROR__VAULT_SECURITY_OVERFLOW]: `VaultSecurityOverflow`,
    [JITO_VAULT_ERROR__VAULT_SECURITY_UNDERFLOW]: `VaultSecurityUnderflow`,
    [JITO_VAULT_ERROR__VAULT_SLASHER_ADMIN_INVALID]: `VaultSlasherAdminInvalid`,
    [JITO_VAULT_ERROR__VAULT_SLASH_INCOMPLETE]: `VaultSlashIncomplete`,
    [JITO_VAULT_ERROR__VAULT_SLASH_UNDERFLOW]: `VaultSlashUnderflow`,
    [JITO_VAULT_ERROR__VAULT_STAKER_WITHDRAWAL_TICKET_INVALID_STAKER]: `VaultStakerWithdrawalTicketInvalidStaker`,
    [JITO_VAULT_ERROR__VAULT_STAKER_WITHDRAWAL_TICKET_NOT_WITHDRAWABLE]: `VaultStakerWithdrawalTicketNotWithdrawable`,
    [JITO_VAULT_ERROR__VAULT_UNDERFLOW]: `VaultUnderflow`,
    [JITO_VAULT_ERROR__VAULT_UPDATE_INCORRECT_INDEX]: `VaultUpdateIncorrectIndex`,
    [JITO_VAULT_ERROR__VAULT_UPDATE_NEEDED]: `VaultUpdateNeeded`,
    [JITO_VAULT_ERROR__VAULT_UPDATE_STATE_NOT_FINISHED_UPDATING]: `VaultUpdateStateNotFinishedUpdating`,
  };
}

export function getJitoVaultErrorMessage(code: JitoVaultError): string {
  if (process.env.NODE_ENV !== 'production') {
    return (jitoVaultErrorMessages as Record<JitoVaultError, string>)[code];
  }

  return 'Error message not available in production bundles.';
}

export function isJitoVaultError<TProgramErrorCode extends JitoVaultError>(
  error: unknown,
  transactionMessage: {
    instructions: Record<number, { programAddress: Address }>;
  },
  code?: TProgramErrorCode
): error is SolanaError<typeof SOLANA_ERROR__INSTRUCTION_ERROR__CUSTOM> &
  Readonly<{ context: Readonly<{ code: TProgramErrorCode }> }> {
  return isProgramError<TProgramErrorCode>(
    error,
    transactionMessage,
    JITO_VAULT_PROGRAM_ADDRESS,
    code
  );
}
