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

export const UPDATE_VAULT_BALANCE_DISCRIMINATOR = 25;

export function getUpdateVaultBalanceDiscriminatorBytes() {
  return getU8Encoder().encode(UPDATE_VAULT_BALANCE_DISCRIMINATOR);
}

export type UpdateVaultBalanceInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountVault extends string | IAccountMeta<string> = string,
  TAccountVaultTokenAccount extends string | IAccountMeta<string> = string,
  TAccountVrtMint extends string | IAccountMeta<string> = string,
  TAccountVaultFeeTokenAccount extends string | IAccountMeta<string> = string,
  TAccountTokenProgram extends
    | string
    | IAccountMeta<string> = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
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
      TAccountVaultTokenAccount extends string
        ? ReadonlyAccount<TAccountVaultTokenAccount>
        : TAccountVaultTokenAccount,
      TAccountVrtMint extends string
        ? WritableAccount<TAccountVrtMint>
        : TAccountVrtMint,
      TAccountVaultFeeTokenAccount extends string
        ? WritableAccount<TAccountVaultFeeTokenAccount>
        : TAccountVaultFeeTokenAccount,
      TAccountTokenProgram extends string
        ? ReadonlyAccount<TAccountTokenProgram>
        : TAccountTokenProgram,
      ...TRemainingAccounts,
    ]
  >;

export type UpdateVaultBalanceInstructionData = { discriminator: number };

export type UpdateVaultBalanceInstructionDataArgs = {};

export function getUpdateVaultBalanceInstructionDataEncoder(): Encoder<UpdateVaultBalanceInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({ ...value, discriminator: UPDATE_VAULT_BALANCE_DISCRIMINATOR })
  );
}

export function getUpdateVaultBalanceInstructionDataDecoder(): Decoder<UpdateVaultBalanceInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getUpdateVaultBalanceInstructionDataCodec(): Codec<
  UpdateVaultBalanceInstructionDataArgs,
  UpdateVaultBalanceInstructionData
> {
  return combineCodec(
    getUpdateVaultBalanceInstructionDataEncoder(),
    getUpdateVaultBalanceInstructionDataDecoder()
  );
}

export type UpdateVaultBalanceInput<
  TAccountConfig extends string = string,
  TAccountVault extends string = string,
  TAccountVaultTokenAccount extends string = string,
  TAccountVrtMint extends string = string,
  TAccountVaultFeeTokenAccount extends string = string,
  TAccountTokenProgram extends string = string,
> = {
  config: Address<TAccountConfig>;
  vault: Address<TAccountVault>;
  vaultTokenAccount: Address<TAccountVaultTokenAccount>;
  vrtMint: Address<TAccountVrtMint>;
  vaultFeeTokenAccount: Address<TAccountVaultFeeTokenAccount>;
  tokenProgram?: Address<TAccountTokenProgram>;
};

export function getUpdateVaultBalanceInstruction<
  TAccountConfig extends string,
  TAccountVault extends string,
  TAccountVaultTokenAccount extends string,
  TAccountVrtMint extends string,
  TAccountVaultFeeTokenAccount extends string,
  TAccountTokenProgram extends string,
>(
  input: UpdateVaultBalanceInput<
    TAccountConfig,
    TAccountVault,
    TAccountVaultTokenAccount,
    TAccountVrtMint,
    TAccountVaultFeeTokenAccount,
    TAccountTokenProgram
  >
): UpdateVaultBalanceInstruction<
  typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountVault,
  TAccountVaultTokenAccount,
  TAccountVrtMint,
  TAccountVaultFeeTokenAccount,
  TAccountTokenProgram
> {
  // Program address.
  const programAddress = JITO_VAULT_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: false },
    vault: { value: input.vault ?? null, isWritable: true },
    vaultTokenAccount: {
      value: input.vaultTokenAccount ?? null,
      isWritable: false,
    },
    vrtMint: { value: input.vrtMint ?? null, isWritable: true },
    vaultFeeTokenAccount: {
      value: input.vaultFeeTokenAccount ?? null,
      isWritable: true,
    },
    tokenProgram: { value: input.tokenProgram ?? null, isWritable: false },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Resolve default values.
  if (!accounts.tokenProgram.value) {
    accounts.tokenProgram.value =
      'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA' as Address<'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'>;
  }

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.config),
      getAccountMeta(accounts.vault),
      getAccountMeta(accounts.vaultTokenAccount),
      getAccountMeta(accounts.vrtMint),
      getAccountMeta(accounts.vaultFeeTokenAccount),
      getAccountMeta(accounts.tokenProgram),
    ],
    programAddress,
    data: getUpdateVaultBalanceInstructionDataEncoder().encode({}),
  } as UpdateVaultBalanceInstruction<
    typeof JITO_VAULT_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountVault,
    TAccountVaultTokenAccount,
    TAccountVrtMint,
    TAccountVaultFeeTokenAccount,
    TAccountTokenProgram
  >;

  return instruction;
}

export type ParsedUpdateVaultBalanceInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    config: TAccountMetas[0];
    vault: TAccountMetas[1];
    vaultTokenAccount: TAccountMetas[2];
    vrtMint: TAccountMetas[3];
    vaultFeeTokenAccount: TAccountMetas[4];
    tokenProgram: TAccountMetas[5];
  };
  data: UpdateVaultBalanceInstructionData;
};

export function parseUpdateVaultBalanceInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedUpdateVaultBalanceInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 6) {
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
      vaultTokenAccount: getNextAccount(),
      vrtMint: getNextAccount(),
      vaultFeeTokenAccount: getNextAccount(),
      tokenProgram: getNextAccount(),
    },
    data: getUpdateVaultBalanceInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
