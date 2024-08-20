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
import { JITO_RESTAKING_SDK_PROGRAM_ADDRESS } from '../programs';

/** NcnOperatorAdminInvalid: NcnOperatorAdminInvalid */
export const JITO_RESTAKING_SDK_ERROR__NCN_OPERATOR_ADMIN_INVALID = 0x3e8; // 1000
/** NcnCooldownOperatorFailed: NcnCooldownOperatorFailed */
export const JITO_RESTAKING_SDK_ERROR__NCN_COOLDOWN_OPERATOR_FAILED = 0x3e9; // 1001
/** NcnSlasherAdminInvalid: NcnSlasherAdminInvalid */
export const JITO_RESTAKING_SDK_ERROR__NCN_SLASHER_ADMIN_INVALID = 0x3ea; // 1002
/** NcnVaultAdminInvalid: NcnVaultAdminInvalid */
export const JITO_RESTAKING_SDK_ERROR__NCN_VAULT_ADMIN_INVALID = 0x3eb; // 1003
/** NcnAdminInvalid: NcnAdminInvalid */
export const JITO_RESTAKING_SDK_ERROR__NCN_ADMIN_INVALID = 0x3ec; // 1004
/** NcnWithdrawAdminInvalid: NcnWithdrawAdminInvalid */
export const JITO_RESTAKING_SDK_ERROR__NCN_WITHDRAW_ADMIN_INVALID = 0x3ed; // 1005
/** NcnVaultSlasherTicketFailedCooldown: NcnVaultSlasherTicketFailedCooldown */
export const JITO_RESTAKING_SDK_ERROR__NCN_VAULT_SLASHER_TICKET_FAILED_COOLDOWN = 0x3ee; // 1006
/** NcnVaultTicketFailedCooldown: NcnVaultTicketFailedCooldown */
export const JITO_RESTAKING_SDK_ERROR__NCN_VAULT_TICKET_FAILED_COOLDOWN = 0x3ef; // 1007
/** NcnWarmupOperatorFailed: NcnWarmupOperatorFailed */
export const JITO_RESTAKING_SDK_ERROR__NCN_WARMUP_OPERATOR_FAILED = 0x3f0; // 1008
/** NcnVaultSlasherTicketFailedWarmup: NcnVaultSlasherTicketFailedWarmup */
export const JITO_RESTAKING_SDK_ERROR__NCN_VAULT_SLASHER_TICKET_FAILED_WARMUP = 0x3f1; // 1009
/** NcnVaultTicketFailedWarmup: NcnVaultTicketFailedWarmup */
export const JITO_RESTAKING_SDK_ERROR__NCN_VAULT_TICKET_FAILED_WARMUP = 0x3f2; // 1010
/** OperatorNcnAdminInvalid: OperatorNcnAdminInvalid */
export const JITO_RESTAKING_SDK_ERROR__OPERATOR_NCN_ADMIN_INVALID = 0x7d0; // 2000
/** OperatorVaultAdminInvalid: OperatorVaultAdminInvalid */
export const JITO_RESTAKING_SDK_ERROR__OPERATOR_VAULT_ADMIN_INVALID = 0x7d1; // 2001
/** OperatorAdminInvalid: OperatorAdminInvalid */
export const JITO_RESTAKING_SDK_ERROR__OPERATOR_ADMIN_INVALID = 0x7d2; // 2002
/** OperatorWithdrawAdminInvalid: OperatorWithdrawAdminInvalid */
export const JITO_RESTAKING_SDK_ERROR__OPERATOR_WITHDRAW_ADMIN_INVALID = 0x7d3; // 2003
/** OperatorCooldownNcnFailed: OperatorCooldownNcnFailed */
export const JITO_RESTAKING_SDK_ERROR__OPERATOR_COOLDOWN_NCN_FAILED = 0x7d4; // 2004
/** OperatorVaultTicketFailedCooldown: OperatorVaultTicketFailedCooldown */
export const JITO_RESTAKING_SDK_ERROR__OPERATOR_VAULT_TICKET_FAILED_COOLDOWN = 0x7d5; // 2005
/** OperatorVaultTicketFailedWarmup: OperatorVaultTicketFailedWarmup */
export const JITO_RESTAKING_SDK_ERROR__OPERATOR_VAULT_TICKET_FAILED_WARMUP = 0x7d6; // 2006
/** OperatorWarmupNcnFailed: OperatorWarmupNcnFailed */
export const JITO_RESTAKING_SDK_ERROR__OPERATOR_WARMUP_NCN_FAILED = 0x7d7; // 2007

export type JitoRestakingSdkError =
  | typeof JITO_RESTAKING_SDK_ERROR__NCN_ADMIN_INVALID
  | typeof JITO_RESTAKING_SDK_ERROR__NCN_COOLDOWN_OPERATOR_FAILED
  | typeof JITO_RESTAKING_SDK_ERROR__NCN_OPERATOR_ADMIN_INVALID
  | typeof JITO_RESTAKING_SDK_ERROR__NCN_SLASHER_ADMIN_INVALID
  | typeof JITO_RESTAKING_SDK_ERROR__NCN_VAULT_ADMIN_INVALID
  | typeof JITO_RESTAKING_SDK_ERROR__NCN_VAULT_SLASHER_TICKET_FAILED_COOLDOWN
  | typeof JITO_RESTAKING_SDK_ERROR__NCN_VAULT_SLASHER_TICKET_FAILED_WARMUP
  | typeof JITO_RESTAKING_SDK_ERROR__NCN_VAULT_TICKET_FAILED_COOLDOWN
  | typeof JITO_RESTAKING_SDK_ERROR__NCN_VAULT_TICKET_FAILED_WARMUP
  | typeof JITO_RESTAKING_SDK_ERROR__NCN_WARMUP_OPERATOR_FAILED
  | typeof JITO_RESTAKING_SDK_ERROR__NCN_WITHDRAW_ADMIN_INVALID
  | typeof JITO_RESTAKING_SDK_ERROR__OPERATOR_ADMIN_INVALID
  | typeof JITO_RESTAKING_SDK_ERROR__OPERATOR_COOLDOWN_NCN_FAILED
  | typeof JITO_RESTAKING_SDK_ERROR__OPERATOR_NCN_ADMIN_INVALID
  | typeof JITO_RESTAKING_SDK_ERROR__OPERATOR_VAULT_ADMIN_INVALID
  | typeof JITO_RESTAKING_SDK_ERROR__OPERATOR_VAULT_TICKET_FAILED_COOLDOWN
  | typeof JITO_RESTAKING_SDK_ERROR__OPERATOR_VAULT_TICKET_FAILED_WARMUP
  | typeof JITO_RESTAKING_SDK_ERROR__OPERATOR_WARMUP_NCN_FAILED
  | typeof JITO_RESTAKING_SDK_ERROR__OPERATOR_WITHDRAW_ADMIN_INVALID;

let jitoRestakingSdkErrorMessages:
  | Record<JitoRestakingSdkError, string>
  | undefined;
if (process.env.NODE_ENV !== 'production') {
  jitoRestakingSdkErrorMessages = {
    [JITO_RESTAKING_SDK_ERROR__NCN_ADMIN_INVALID]: `NcnAdminInvalid`,
    [JITO_RESTAKING_SDK_ERROR__NCN_COOLDOWN_OPERATOR_FAILED]: `NcnCooldownOperatorFailed`,
    [JITO_RESTAKING_SDK_ERROR__NCN_OPERATOR_ADMIN_INVALID]: `NcnOperatorAdminInvalid`,
    [JITO_RESTAKING_SDK_ERROR__NCN_SLASHER_ADMIN_INVALID]: `NcnSlasherAdminInvalid`,
    [JITO_RESTAKING_SDK_ERROR__NCN_VAULT_ADMIN_INVALID]: `NcnVaultAdminInvalid`,
    [JITO_RESTAKING_SDK_ERROR__NCN_VAULT_SLASHER_TICKET_FAILED_COOLDOWN]: `NcnVaultSlasherTicketFailedCooldown`,
    [JITO_RESTAKING_SDK_ERROR__NCN_VAULT_SLASHER_TICKET_FAILED_WARMUP]: `NcnVaultSlasherTicketFailedWarmup`,
    [JITO_RESTAKING_SDK_ERROR__NCN_VAULT_TICKET_FAILED_COOLDOWN]: `NcnVaultTicketFailedCooldown`,
    [JITO_RESTAKING_SDK_ERROR__NCN_VAULT_TICKET_FAILED_WARMUP]: `NcnVaultTicketFailedWarmup`,
    [JITO_RESTAKING_SDK_ERROR__NCN_WARMUP_OPERATOR_FAILED]: `NcnWarmupOperatorFailed`,
    [JITO_RESTAKING_SDK_ERROR__NCN_WITHDRAW_ADMIN_INVALID]: `NcnWithdrawAdminInvalid`,
    [JITO_RESTAKING_SDK_ERROR__OPERATOR_ADMIN_INVALID]: `OperatorAdminInvalid`,
    [JITO_RESTAKING_SDK_ERROR__OPERATOR_COOLDOWN_NCN_FAILED]: `OperatorCooldownNcnFailed`,
    [JITO_RESTAKING_SDK_ERROR__OPERATOR_NCN_ADMIN_INVALID]: `OperatorNcnAdminInvalid`,
    [JITO_RESTAKING_SDK_ERROR__OPERATOR_VAULT_ADMIN_INVALID]: `OperatorVaultAdminInvalid`,
    [JITO_RESTAKING_SDK_ERROR__OPERATOR_VAULT_TICKET_FAILED_COOLDOWN]: `OperatorVaultTicketFailedCooldown`,
    [JITO_RESTAKING_SDK_ERROR__OPERATOR_VAULT_TICKET_FAILED_WARMUP]: `OperatorVaultTicketFailedWarmup`,
    [JITO_RESTAKING_SDK_ERROR__OPERATOR_WARMUP_NCN_FAILED]: `OperatorWarmupNcnFailed`,
    [JITO_RESTAKING_SDK_ERROR__OPERATOR_WITHDRAW_ADMIN_INVALID]: `OperatorWithdrawAdminInvalid`,
  };
}

export function getJitoRestakingSdkErrorMessage(
  code: JitoRestakingSdkError
): string {
  if (process.env.NODE_ENV !== 'production') {
    return (
      jitoRestakingSdkErrorMessages as Record<JitoRestakingSdkError, string>
    )[code];
  }

  return 'Error message not available in production bundles.';
}

export function isJitoRestakingSdkError<
  TProgramErrorCode extends JitoRestakingSdkError,
>(
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
    JITO_RESTAKING_SDK_PROGRAM_ADDRESS,
    code
  );
}
