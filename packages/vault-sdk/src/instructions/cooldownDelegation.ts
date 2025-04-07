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
  getU64Decoder,
  getU64Encoder,
  getU8Decoder,
  getU8Encoder,
  transformEncoder,
  type Address,
  type Codec,
  type Decoder,
  type Encoder,
  type IAccountMeta,
  type IAccountSignerMeta,
  type IInstruction,
  type IInstructionWithAccounts,
  type IInstructionWithData,
  type ReadonlyAccount,
  type ReadonlySignerAccount,
  type TransactionSigner,
  type WritableAccount,
} from '@solana/kit';
import { JITO_VAULT_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export const COOLDOWN_DELEGATION_DISCRIMINATOR = 24;

export function getCooldownDelegationDiscriminatorBytes() {
  return getU8Encoder().encode(COOLDOWN_DELEGATION_DISCRIMINATOR);
}

export type CooldownDelegationInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountVault extends string | IAccountMeta<string> = string,
  TAccountOperator extends string | IAccountMeta<string> = string,
  TAccountVaultOperatorDelegation extends
    | string
    | IAccountMeta<string> = string,
  TAccountAdmin extends string | IAccountMeta<string> = string,
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
      TAccountOperator extends string
        ? ReadonlyAccount<TAccountOperator>
        : TAccountOperator,
      TAccountVaultOperatorDelegation extends string
        ? WritableAccount<TAccountVaultOperatorDelegation>
        : TAccountVaultOperatorDelegation,
      TAccountAdmin extends string
        ? ReadonlySignerAccount<TAccountAdmin> &
            IAccountSignerMeta<TAccountAdmin>
        : TAccountAdmin,
      ...TRemainingAccounts,
    ]
  >;

export type CooldownDelegationInstructionData = {
  discriminator: number;
  amount: bigint;
};

export type CooldownDelegationInstructionDataArgs = { amount: number | bigint };

export function getCooldownDelegationInstructionDataEncoder(): Encoder<CooldownDelegationInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['amount', getU64Encoder()],
    ]),
    (value) => ({ ...value, discriminator: COOLDOWN_DELEGATION_DISCRIMINATOR })
  );
}

export function getCooldownDelegationInstructionDataDecoder(): Decoder<CooldownDelegationInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['amount', getU64Decoder()],
  ]);
}

export function getCooldownDelegationInstructionDataCodec(): Codec<
  CooldownDelegationInstructionDataArgs,
  CooldownDelegationInstructionData
> {
  return combineCodec(
    getCooldownDelegationInstructionDataEncoder(),
    getCooldownDelegationInstructionDataDecoder()
  );
}

export type CooldownDelegationInput<
  TAccountConfig extends string = string,
  TAccountVault extends string = string,
  TAccountOperator extends string = string,
  TAccountVaultOperatorDelegation extends string = string,
  TAccountAdmin extends string = string,
> = {
  config: Address<TAccountConfig>;
  vault: Address<TAccountVault>;
  operator: Address<TAccountOperator>;
  vaultOperatorDelegation: Address<TAccountVaultOperatorDelegation>;
  admin: TransactionSigner<TAccountAdmin>;
  amount: CooldownDelegationInstructionDataArgs['amount'];
};

export function getCooldownDelegationInstruction<
  TAccountConfig extends string,
  TAccountVault extends string,
  TAccountOperator extends string,
  TAccountVaultOperatorDelegation extends string,
  TAccountAdmin extends string,
  TProgramAddress extends Address = typeof JITO_VAULT_PROGRAM_ADDRESS,
>(
  input: CooldownDelegationInput<
    TAccountConfig,
    TAccountVault,
    TAccountOperator,
    TAccountVaultOperatorDelegation,
    TAccountAdmin
  >,
  config?: { programAddress?: TProgramAddress }
): CooldownDelegationInstruction<
  TProgramAddress,
  TAccountConfig,
  TAccountVault,
  TAccountOperator,
  TAccountVaultOperatorDelegation,
  TAccountAdmin
> {
  // Program address.
  const programAddress = config?.programAddress ?? JITO_VAULT_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: false },
    vault: { value: input.vault ?? null, isWritable: true },
    operator: { value: input.operator ?? null, isWritable: false },
    vaultOperatorDelegation: {
      value: input.vaultOperatorDelegation ?? null,
      isWritable: true,
    },
    admin: { value: input.admin ?? null, isWritable: false },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Original args.
  const args = { ...input };

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.config),
      getAccountMeta(accounts.vault),
      getAccountMeta(accounts.operator),
      getAccountMeta(accounts.vaultOperatorDelegation),
      getAccountMeta(accounts.admin),
    ],
    programAddress,
    data: getCooldownDelegationInstructionDataEncoder().encode(
      args as CooldownDelegationInstructionDataArgs
    ),
  } as CooldownDelegationInstruction<
    TProgramAddress,
    TAccountConfig,
    TAccountVault,
    TAccountOperator,
    TAccountVaultOperatorDelegation,
    TAccountAdmin
  >;

  return instruction;
}

export type ParsedCooldownDelegationInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    config: TAccountMetas[0];
    vault: TAccountMetas[1];
    operator: TAccountMetas[2];
    vaultOperatorDelegation: TAccountMetas[3];
    admin: TAccountMetas[4];
  };
  data: CooldownDelegationInstructionData;
};

export function parseCooldownDelegationInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedCooldownDelegationInstruction<TProgram, TAccountMetas> {
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
      operator: getNextAccount(),
      vaultOperatorDelegation: getNextAccount(),
      admin: getNextAccount(),
    },
    data: getCooldownDelegationInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
