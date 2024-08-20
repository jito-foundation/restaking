/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  containsBytes,
  getU8Encoder,
  type Address,
  type ReadonlyUint8Array,
} from '@solana/web3.js';
import {
  type ParsedAddDelegationInstruction,
  type ParsedAdminWithdrawInstruction,
  type ParsedBurnInstruction,
  type ParsedBurnWithdrawTicketInstruction,
  type ParsedChangeWithdrawalTicketOwnerInstruction,
  type ParsedCloseVaultUpdateStateTrackerInstruction,
  type ParsedCooldownDelegationInstruction,
  type ParsedCooldownVaultNcnSlasherTicketInstruction,
  type ParsedCooldownVaultNcnTicketInstruction,
  type ParsedCrankVaultUpdateStateTrackerInstruction,
  type ParsedCreateTokenMetadataInstruction,
  type ParsedEnqueueWithdrawalInstruction,
  type ParsedInitializeConfigInstruction,
  type ParsedInitializeVaultInstruction,
  type ParsedInitializeVaultNcnSlasherOperatorTicketInstruction,
  type ParsedInitializeVaultNcnSlasherTicketInstruction,
  type ParsedInitializeVaultNcnTicketInstruction,
  type ParsedInitializeVaultOperatorDelegationInstruction,
  type ParsedInitializeVaultUpdateStateTrackerInstruction,
  type ParsedInitializeVaultWithMintInstruction,
  type ParsedMintToInstruction,
  type ParsedSetAdminInstruction,
  type ParsedSetDepositCapacityInstruction,
  type ParsedSetFeesInstruction,
  type ParsedSetSecondaryAdminInstruction,
  type ParsedSlashInstruction,
  type ParsedUpdateTokenMetadataInstruction,
  type ParsedUpdateVaultBalanceInstruction,
  type ParsedWarmupVaultNcnSlasherTicketInstruction,
  type ParsedWarmupVaultNcnTicketInstruction,
} from '../instructions';

export const JITO_VAULT_SDK_PROGRAM_ADDRESS =
  'F7kL131bqDJB7jKs4nhNUu3KCLwcgkiKQKyBHJfkCjWy' as Address<'F7kL131bqDJB7jKs4nhNUu3KCLwcgkiKQKyBHJfkCjWy'>;

export enum JitoVaultSdkInstruction {
  InitializeConfig,
  InitializeVault,
  InitializeVaultWithMint,
  InitializeVaultOperatorDelegation,
  InitializeVaultNcnTicket,
  InitializeVaultNcnSlasherOperatorTicket,
  InitializeVaultNcnSlasherTicket,
  WarmupVaultNcnTicket,
  CooldownVaultNcnTicket,
  WarmupVaultNcnSlasherTicket,
  CooldownVaultNcnSlasherTicket,
  MintTo,
  Burn,
  EnqueueWithdrawal,
  ChangeWithdrawalTicketOwner,
  BurnWithdrawTicket,
  SetDepositCapacity,
  SetFees,
  AdminWithdraw,
  SetAdmin,
  SetSecondaryAdmin,
  AddDelegation,
  CooldownDelegation,
  UpdateVaultBalance,
  InitializeVaultUpdateStateTracker,
  CrankVaultUpdateStateTracker,
  CloseVaultUpdateStateTracker,
  CreateTokenMetadata,
  UpdateTokenMetadata,
  Slash,
}

export function identifyJitoVaultSdkInstruction(
  instruction: { data: ReadonlyUint8Array } | ReadonlyUint8Array
): JitoVaultSdkInstruction {
  const data = 'data' in instruction ? instruction.data : instruction;
  if (containsBytes(data, getU8Encoder().encode(0), 0)) {
    return JitoVaultSdkInstruction.InitializeConfig;
  }
  if (containsBytes(data, getU8Encoder().encode(1), 0)) {
    return JitoVaultSdkInstruction.InitializeVault;
  }
  if (containsBytes(data, getU8Encoder().encode(2), 0)) {
    return JitoVaultSdkInstruction.InitializeVaultWithMint;
  }
  if (containsBytes(data, getU8Encoder().encode(3), 0)) {
    return JitoVaultSdkInstruction.InitializeVaultOperatorDelegation;
  }
  if (containsBytes(data, getU8Encoder().encode(4), 0)) {
    return JitoVaultSdkInstruction.InitializeVaultNcnTicket;
  }
  if (containsBytes(data, getU8Encoder().encode(5), 0)) {
    return JitoVaultSdkInstruction.InitializeVaultNcnSlasherOperatorTicket;
  }
  if (containsBytes(data, getU8Encoder().encode(6), 0)) {
    return JitoVaultSdkInstruction.InitializeVaultNcnSlasherTicket;
  }
  if (containsBytes(data, getU8Encoder().encode(7), 0)) {
    return JitoVaultSdkInstruction.WarmupVaultNcnTicket;
  }
  if (containsBytes(data, getU8Encoder().encode(8), 0)) {
    return JitoVaultSdkInstruction.CooldownVaultNcnTicket;
  }
  if (containsBytes(data, getU8Encoder().encode(9), 0)) {
    return JitoVaultSdkInstruction.WarmupVaultNcnSlasherTicket;
  }
  if (containsBytes(data, getU8Encoder().encode(10), 0)) {
    return JitoVaultSdkInstruction.CooldownVaultNcnSlasherTicket;
  }
  if (containsBytes(data, getU8Encoder().encode(11), 0)) {
    return JitoVaultSdkInstruction.MintTo;
  }
  if (containsBytes(data, getU8Encoder().encode(12), 0)) {
    return JitoVaultSdkInstruction.Burn;
  }
  if (containsBytes(data, getU8Encoder().encode(13), 0)) {
    return JitoVaultSdkInstruction.EnqueueWithdrawal;
  }
  if (containsBytes(data, getU8Encoder().encode(14), 0)) {
    return JitoVaultSdkInstruction.ChangeWithdrawalTicketOwner;
  }
  if (containsBytes(data, getU8Encoder().encode(15), 0)) {
    return JitoVaultSdkInstruction.BurnWithdrawTicket;
  }
  if (containsBytes(data, getU8Encoder().encode(16), 0)) {
    return JitoVaultSdkInstruction.SetDepositCapacity;
  }
  if (containsBytes(data, getU8Encoder().encode(17), 0)) {
    return JitoVaultSdkInstruction.SetFees;
  }
  if (containsBytes(data, getU8Encoder().encode(18), 0)) {
    return JitoVaultSdkInstruction.AdminWithdraw;
  }
  if (containsBytes(data, getU8Encoder().encode(19), 0)) {
    return JitoVaultSdkInstruction.SetAdmin;
  }
  if (containsBytes(data, getU8Encoder().encode(20), 0)) {
    return JitoVaultSdkInstruction.SetSecondaryAdmin;
  }
  if (containsBytes(data, getU8Encoder().encode(21), 0)) {
    return JitoVaultSdkInstruction.AddDelegation;
  }
  if (containsBytes(data, getU8Encoder().encode(22), 0)) {
    return JitoVaultSdkInstruction.CooldownDelegation;
  }
  if (containsBytes(data, getU8Encoder().encode(23), 0)) {
    return JitoVaultSdkInstruction.UpdateVaultBalance;
  }
  if (containsBytes(data, getU8Encoder().encode(24), 0)) {
    return JitoVaultSdkInstruction.InitializeVaultUpdateStateTracker;
  }
  if (containsBytes(data, getU8Encoder().encode(25), 0)) {
    return JitoVaultSdkInstruction.CrankVaultUpdateStateTracker;
  }
  if (containsBytes(data, getU8Encoder().encode(26), 0)) {
    return JitoVaultSdkInstruction.CloseVaultUpdateStateTracker;
  }
  if (containsBytes(data, getU8Encoder().encode(27), 0)) {
    return JitoVaultSdkInstruction.CreateTokenMetadata;
  }
  if (containsBytes(data, getU8Encoder().encode(28), 0)) {
    return JitoVaultSdkInstruction.UpdateTokenMetadata;
  }
  if (containsBytes(data, getU8Encoder().encode(29), 0)) {
    return JitoVaultSdkInstruction.Slash;
  }
  throw new Error(
    'The provided instruction could not be identified as a jitoVaultSdk instruction.'
  );
}

export type ParsedJitoVaultSdkInstruction<
  TProgram extends string = 'F7kL131bqDJB7jKs4nhNUu3KCLwcgkiKQKyBHJfkCjWy',
> =
  | ({
      instructionType: JitoVaultSdkInstruction.InitializeConfig;
    } & ParsedInitializeConfigInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.InitializeVault;
    } & ParsedInitializeVaultInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.InitializeVaultWithMint;
    } & ParsedInitializeVaultWithMintInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.InitializeVaultOperatorDelegation;
    } & ParsedInitializeVaultOperatorDelegationInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.InitializeVaultNcnTicket;
    } & ParsedInitializeVaultNcnTicketInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.InitializeVaultNcnSlasherOperatorTicket;
    } & ParsedInitializeVaultNcnSlasherOperatorTicketInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.InitializeVaultNcnSlasherTicket;
    } & ParsedInitializeVaultNcnSlasherTicketInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.WarmupVaultNcnTicket;
    } & ParsedWarmupVaultNcnTicketInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.CooldownVaultNcnTicket;
    } & ParsedCooldownVaultNcnTicketInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.WarmupVaultNcnSlasherTicket;
    } & ParsedWarmupVaultNcnSlasherTicketInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.CooldownVaultNcnSlasherTicket;
    } & ParsedCooldownVaultNcnSlasherTicketInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.MintTo;
    } & ParsedMintToInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.Burn;
    } & ParsedBurnInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.EnqueueWithdrawal;
    } & ParsedEnqueueWithdrawalInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.ChangeWithdrawalTicketOwner;
    } & ParsedChangeWithdrawalTicketOwnerInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.BurnWithdrawTicket;
    } & ParsedBurnWithdrawTicketInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.SetDepositCapacity;
    } & ParsedSetDepositCapacityInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.SetFees;
    } & ParsedSetFeesInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.AdminWithdraw;
    } & ParsedAdminWithdrawInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.SetAdmin;
    } & ParsedSetAdminInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.SetSecondaryAdmin;
    } & ParsedSetSecondaryAdminInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.AddDelegation;
    } & ParsedAddDelegationInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.CooldownDelegation;
    } & ParsedCooldownDelegationInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.UpdateVaultBalance;
    } & ParsedUpdateVaultBalanceInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.InitializeVaultUpdateStateTracker;
    } & ParsedInitializeVaultUpdateStateTrackerInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.CrankVaultUpdateStateTracker;
    } & ParsedCrankVaultUpdateStateTrackerInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.CloseVaultUpdateStateTracker;
    } & ParsedCloseVaultUpdateStateTrackerInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.CreateTokenMetadata;
    } & ParsedCreateTokenMetadataInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.UpdateTokenMetadata;
    } & ParsedUpdateTokenMetadataInstruction<TProgram>)
  | ({
      instructionType: JitoVaultSdkInstruction.Slash;
    } & ParsedSlashInstruction<TProgram>);
