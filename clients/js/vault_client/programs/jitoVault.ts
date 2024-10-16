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
  type ParsedBurnInstruction,
  type ParsedBurnWithdrawalTicketInstruction,
  type ParsedChangeWithdrawalTicketOwnerInstruction,
  type ParsedCloseVaultUpdateStateTrackerInstruction,
  type ParsedCooldownDelegationInstruction,
  type ParsedCooldownVaultNcnSlasherTicketInstruction,
  type ParsedCooldownVaultNcnTicketInstruction,
  type ParsedCrankVaultUpdateStateTrackerInstruction,
  type ParsedCreateTokenMetadataInstruction,
  type ParsedDelegateTokenAccountInstruction,
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
  type ParsedSetIsPausedInstruction,
  type ParsedSetProgramFeeInstruction,
  type ParsedSetProgramFeeWalletInstruction,
  type ParsedSetSecondaryAdminInstruction,
  type ParsedSlashInstruction,
  type ParsedUpdateTokenMetadataInstruction,
  type ParsedUpdateVaultBalanceInstruction,
  type ParsedWarmupVaultNcnSlasherTicketInstruction,
  type ParsedWarmupVaultNcnTicketInstruction,
} from '../instructions';

export const JITO_VAULT_PROGRAM_ADDRESS =
  '34X2uqBhEGiWHu43RDEMwrMqXF4CpCPEZNaKdAaUS9jx' as Address<'34X2uqBhEGiWHu43RDEMwrMqXF4CpCPEZNaKdAaUS9jx'>;

export enum JitoVaultAccount {
  Config,
  Vault,
  VaultNcnSlasherOperatorTicket,
  VaultNcnSlasherTicket,
  VaultNcnTicket,
  VaultOperatorDelegation,
  VaultStakerWithdrawalTicket,
  VaultUpdateStateTracker,
}

export enum JitoVaultInstruction {
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
  BurnWithdrawalTicket,
  SetDepositCapacity,
  SetFees,
  SetProgramFee,
  SetProgramFeeWallet,
  SetIsPaused,
  DelegateTokenAccount,
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

export function identifyJitoVaultInstruction(
  instruction: { data: ReadonlyUint8Array } | ReadonlyUint8Array
): JitoVaultInstruction {
  const data = 'data' in instruction ? instruction.data : instruction;
  if (containsBytes(data, getU8Encoder().encode(0), 0)) {
    return JitoVaultInstruction.InitializeConfig;
  }
  if (containsBytes(data, getU8Encoder().encode(1), 0)) {
    return JitoVaultInstruction.InitializeVault;
  }
  if (containsBytes(data, getU8Encoder().encode(2), 0)) {
    return JitoVaultInstruction.InitializeVaultWithMint;
  }
  if (containsBytes(data, getU8Encoder().encode(3), 0)) {
    return JitoVaultInstruction.InitializeVaultOperatorDelegation;
  }
  if (containsBytes(data, getU8Encoder().encode(4), 0)) {
    return JitoVaultInstruction.InitializeVaultNcnTicket;
  }
  if (containsBytes(data, getU8Encoder().encode(5), 0)) {
    return JitoVaultInstruction.InitializeVaultNcnSlasherOperatorTicket;
  }
  if (containsBytes(data, getU8Encoder().encode(6), 0)) {
    return JitoVaultInstruction.InitializeVaultNcnSlasherTicket;
  }
  if (containsBytes(data, getU8Encoder().encode(7), 0)) {
    return JitoVaultInstruction.WarmupVaultNcnTicket;
  }
  if (containsBytes(data, getU8Encoder().encode(8), 0)) {
    return JitoVaultInstruction.CooldownVaultNcnTicket;
  }
  if (containsBytes(data, getU8Encoder().encode(9), 0)) {
    return JitoVaultInstruction.WarmupVaultNcnSlasherTicket;
  }
  if (containsBytes(data, getU8Encoder().encode(10), 0)) {
    return JitoVaultInstruction.CooldownVaultNcnSlasherTicket;
  }
  if (containsBytes(data, getU8Encoder().encode(11), 0)) {
    return JitoVaultInstruction.MintTo;
  }
  if (containsBytes(data, getU8Encoder().encode(12), 0)) {
    return JitoVaultInstruction.Burn;
  }
  if (containsBytes(data, getU8Encoder().encode(13), 0)) {
    return JitoVaultInstruction.EnqueueWithdrawal;
  }
  if (containsBytes(data, getU8Encoder().encode(14), 0)) {
    return JitoVaultInstruction.ChangeWithdrawalTicketOwner;
  }
  if (containsBytes(data, getU8Encoder().encode(15), 0)) {
    return JitoVaultInstruction.BurnWithdrawalTicket;
  }
  if (containsBytes(data, getU8Encoder().encode(16), 0)) {
    return JitoVaultInstruction.SetDepositCapacity;
  }
  if (containsBytes(data, getU8Encoder().encode(17), 0)) {
    return JitoVaultInstruction.SetFees;
  }
  if (containsBytes(data, getU8Encoder().encode(18), 0)) {
    return JitoVaultInstruction.SetProgramFee;
  }
  if (containsBytes(data, getU8Encoder().encode(19), 0)) {
    return JitoVaultInstruction.SetProgramFeeWallet;
  }
  if (containsBytes(data, getU8Encoder().encode(20), 0)) {
    return JitoVaultInstruction.SetIsPaused;
  }
  if (containsBytes(data, getU8Encoder().encode(21), 0)) {
    return JitoVaultInstruction.DelegateTokenAccount;
  }
  if (containsBytes(data, getU8Encoder().encode(22), 0)) {
    return JitoVaultInstruction.SetAdmin;
  }
  if (containsBytes(data, getU8Encoder().encode(23), 0)) {
    return JitoVaultInstruction.SetSecondaryAdmin;
  }
  if (containsBytes(data, getU8Encoder().encode(24), 0)) {
    return JitoVaultInstruction.AddDelegation;
  }
  if (containsBytes(data, getU8Encoder().encode(25), 0)) {
    return JitoVaultInstruction.CooldownDelegation;
  }
  if (containsBytes(data, getU8Encoder().encode(26), 0)) {
    return JitoVaultInstruction.UpdateVaultBalance;
  }
  if (containsBytes(data, getU8Encoder().encode(27), 0)) {
    return JitoVaultInstruction.InitializeVaultUpdateStateTracker;
  }
  if (containsBytes(data, getU8Encoder().encode(28), 0)) {
    return JitoVaultInstruction.CrankVaultUpdateStateTracker;
  }
  if (containsBytes(data, getU8Encoder().encode(29), 0)) {
    return JitoVaultInstruction.CloseVaultUpdateStateTracker;
  }
  if (containsBytes(data, getU8Encoder().encode(30), 0)) {
    return JitoVaultInstruction.CreateTokenMetadata;
  }
  if (containsBytes(data, getU8Encoder().encode(31), 0)) {
    return JitoVaultInstruction.UpdateTokenMetadata;
  }
  if (containsBytes(data, getU8Encoder().encode(32), 0)) {
    return JitoVaultInstruction.Slash;
  }
  throw new Error(
    'The provided instruction could not be identified as a jitoVault instruction.'
  );
}

export type ParsedJitoVaultInstruction<
  TProgram extends string = '34X2uqBhEGiWHu43RDEMwrMqXF4CpCPEZNaKdAaUS9jx',
> =
  | ({
      instructionType: JitoVaultInstruction.InitializeConfig;
    } & ParsedInitializeConfigInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.InitializeVault;
    } & ParsedInitializeVaultInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.InitializeVaultWithMint;
    } & ParsedInitializeVaultWithMintInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.InitializeVaultOperatorDelegation;
    } & ParsedInitializeVaultOperatorDelegationInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.InitializeVaultNcnTicket;
    } & ParsedInitializeVaultNcnTicketInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.InitializeVaultNcnSlasherOperatorTicket;
    } & ParsedInitializeVaultNcnSlasherOperatorTicketInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.InitializeVaultNcnSlasherTicket;
    } & ParsedInitializeVaultNcnSlasherTicketInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.WarmupVaultNcnTicket;
    } & ParsedWarmupVaultNcnTicketInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.CooldownVaultNcnTicket;
    } & ParsedCooldownVaultNcnTicketInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.WarmupVaultNcnSlasherTicket;
    } & ParsedWarmupVaultNcnSlasherTicketInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.CooldownVaultNcnSlasherTicket;
    } & ParsedCooldownVaultNcnSlasherTicketInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.MintTo;
    } & ParsedMintToInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.Burn;
    } & ParsedBurnInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.EnqueueWithdrawal;
    } & ParsedEnqueueWithdrawalInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.ChangeWithdrawalTicketOwner;
    } & ParsedChangeWithdrawalTicketOwnerInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.BurnWithdrawalTicket;
    } & ParsedBurnWithdrawalTicketInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.SetDepositCapacity;
    } & ParsedSetDepositCapacityInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.SetFees;
    } & ParsedSetFeesInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.SetProgramFee;
    } & ParsedSetProgramFeeInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.SetProgramFeeWallet;
    } & ParsedSetProgramFeeWalletInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.SetIsPaused;
    } & ParsedSetIsPausedInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.DelegateTokenAccount;
    } & ParsedDelegateTokenAccountInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.SetAdmin;
    } & ParsedSetAdminInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.SetSecondaryAdmin;
    } & ParsedSetSecondaryAdminInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.AddDelegation;
    } & ParsedAddDelegationInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.CooldownDelegation;
    } & ParsedCooldownDelegationInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.UpdateVaultBalance;
    } & ParsedUpdateVaultBalanceInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.InitializeVaultUpdateStateTracker;
    } & ParsedInitializeVaultUpdateStateTrackerInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.CrankVaultUpdateStateTracker;
    } & ParsedCrankVaultUpdateStateTrackerInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.CloseVaultUpdateStateTracker;
    } & ParsedCloseVaultUpdateStateTrackerInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.CreateTokenMetadata;
    } & ParsedCreateTokenMetadataInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.UpdateTokenMetadata;
    } & ParsedUpdateTokenMetadataInstruction<TProgram>)
  | ({
      instructionType: JitoVaultInstruction.Slash;
    } & ParsedSlashInstruction<TProgram>);
