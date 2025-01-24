/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  combineCodec,
  getStructDecoder,
  getStructEncoder,
  getU8Decoder,
  getU8Encoder,
  transformEncoder,
  type Address,
  type Codec,
  type Decoder,
  type Encoder,
  type IAccountMeta,
  type IInstruction,
  type IInstructionWithAccounts,
  type IInstructionWithData,
  type ReadonlyAccount,
  type WritableAccount,
} from '@solana/web3.js';
import { JITO_VAULT_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';
import {
  getWithdrawalAllocationMethodDecoder,
  getWithdrawalAllocationMethodEncoder,
  type WithdrawalAllocationMethod,
  type WithdrawalAllocationMethodArgs,
} from '../types';

export const INITIALIZE_VAULT_UPDATE_STATE_TRACKER_DISCRIMINATOR = 27;

export function getInitializeVaultUpdateStateTrackerDiscriminatorBytes() {
  return getU8Encoder().encode(
    INITIALIZE_VAULT_UPDATE_STATE_TRACKER_DISCRIMINATOR
  );
}

export type InitializeVaultUpdateStateTrackerInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountVault extends string | IAccountMeta<string> = string,
  TAccountVaultUpdateStateTracker extends
    | string
    | IAccountMeta<string> = string,
  TAccountPayer extends string | IAccountMeta<string> = string,
  TAccountSystemProgram extends
    | string
    | IAccountMeta<string> = '11111111111111111111111111111111',
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountConfig extends string
        ? ReadonlyAccount<TAccountConfig>
        : TAccountConfig,
      TAccountVault extends string
        ? WritableAccount<TAccountVault>
        : TAccountVault,
      TAccountVaultUpdateStateTracker extends string
        ? WritableAccount<TAccountVaultUpdateStateTracker>
        : TAccountVaultUpdateStateTracker,
      TAccountPayer extends string
        ? WritableAccount<TAccountPayer>
        : TAccountPayer,
      TAccountSystemProgram extends string
        ? ReadonlyAccount<TAccountSystemProgram>
        : TAccountSystemProgram,
      ...TRemainingAccounts,
    ]
  >;

export type InitializeVaultUpdateStateTrackerInstructionData = {
  discriminator: number;
  withdrawalAllocationMethod: WithdrawalAllocationMethod;
};

export type InitializeVaultUpdateStateTrackerInstructionDataArgs = {
  withdrawalAllocationMethod: WithdrawalAllocationMethodArgs;
};

export function getInitializeVaultUpdateStateTrackerInstructionDataEncoder(): Encoder<InitializeVaultUpdateStateTrackerInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['withdrawalAllocationMethod', getWithdrawalAllocationMethodEncoder()],
    ]),
    (value) => ({
      ...value,
      discriminator: INITIALIZE_VAULT_UPDATE_STATE_TRACKER_DISCRIMINATOR,
    })
  );
}

export function getInitializeVaultUpdateStateTrackerInstructionDataDecoder(): Decoder<InitializeVaultUpdateStateTrackerInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['withdrawalAllocationMethod', getWithdrawalAllocationMethodDecoder()],
  ]);
}

export function getInitializeVaultUpdateStateTrackerInstructionDataCodec(): Codec<
  InitializeVaultUpdateStateTrackerInstructionDataArgs,
  InitializeVaultUpdateStateTrackerInstructionData
> {
  return combineCodec(
    getInitializeVaultUpdateStateTrackerInstructionDataEncoder(),
    getInitializeVaultUpdateStateTrackerInstructionDataDecoder()
  );
}

export type InitializeVaultUpdateStateTrackerInput<
  TAccountConfig extends string = string,
  TAccountVault extends string = string,
  TAccountVaultUpdateStateTracker extends string = string,
  TAccountPayer extends string = string,
  TAccountSystemProgram extends string = string,
> = {
  config: Address<TAccountConfig>;
  vault: Address<TAccountVault>;
  vaultUpdateStateTracker: Address<TAccountVaultUpdateStateTracker>;
  payer: Address<TAccountPayer>;
  systemProgram?: Address<TAccountSystemProgram>;
  withdrawalAllocationMethod: InitializeVaultUpdateStateTrackerInstructionDataArgs['withdrawalAllocationMethod'];
};

export function getInitializeVaultUpdateStateTrackerInstruction<
  TAccountConfig extends string,
  TAccountVault extends string,
  TAccountVaultUpdateStateTracker extends string,
  TAccountPayer extends string,
  TAccountSystemProgram extends string,
>(
  input: InitializeVaultUpdateStateTrackerInput<
    TAccountConfig,
    TAccountVault,
    TAccountVaultUpdateStateTracker,
    TAccountPayer,
    TAccountSystemProgram
  >
): InitializeVaultUpdateStateTrackerInstruction<
  typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountVault,
  TAccountVaultUpdateStateTracker,
  TAccountPayer,
  TAccountSystemProgram
> {
  // Program address.
  const programAddress = JITO_VAULT_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: false },
    vault: { value: input.vault ?? null, isWritable: true },
    vaultUpdateStateTracker: {
      value: input.vaultUpdateStateTracker ?? null,
      isWritable: true,
    },
    payer: { value: input.payer ?? null, isWritable: true },
    systemProgram: { value: input.systemProgram ?? null, isWritable: false },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Original args.
  const args = { ...input };

  // Resolve default values.
  if (!accounts.systemProgram.value) {
    accounts.systemProgram.value =
      '11111111111111111111111111111111' as Address<'11111111111111111111111111111111'>;
  }

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.config),
      getAccountMeta(accounts.vault),
      getAccountMeta(accounts.vaultUpdateStateTracker),
      getAccountMeta(accounts.payer),
      getAccountMeta(accounts.systemProgram),
    ],
    programAddress,
    data: getInitializeVaultUpdateStateTrackerInstructionDataEncoder().encode(
      args as InitializeVaultUpdateStateTrackerInstructionDataArgs
    ),
  } as InitializeVaultUpdateStateTrackerInstruction<
    typeof JITO_VAULT_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountVault,
    TAccountVaultUpdateStateTracker,
    TAccountPayer,
    TAccountSystemProgram
  >;

  return instruction;
}

export type ParsedInitializeVaultUpdateStateTrackerInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    config: TAccountMetas[0];
    vault: TAccountMetas[1];
    vaultUpdateStateTracker: TAccountMetas[2];
    payer: TAccountMetas[3];
    systemProgram: TAccountMetas[4];
  };
  data: InitializeVaultUpdateStateTrackerInstructionData;
};

export function parseInitializeVaultUpdateStateTrackerInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedInitializeVaultUpdateStateTrackerInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 5) {
    // TODO: Coded error.
    throw new Error('Not enough accounts');
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts![accountIndex]!;
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      config: getNextAccount(),
      vault: getNextAccount(),
      vaultUpdateStateTracker: getNextAccount(),
      payer: getNextAccount(),
      systemProgram: getNextAccount(),
    },
    data: getInitializeVaultUpdateStateTrackerInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
