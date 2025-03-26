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

export const SET_DEPOSIT_CAPACITY_DISCRIMINATOR = 15;

export function getSetDepositCapacityDiscriminatorBytes() {
  return getU8Encoder().encode(SET_DEPOSIT_CAPACITY_DISCRIMINATOR);
}

export type SetDepositCapacityInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountVault extends string | IAccountMeta<string> = string,
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
      TAccountAdmin extends string
        ? ReadonlySignerAccount<TAccountAdmin> &
            IAccountSignerMeta<TAccountAdmin>
        : TAccountAdmin,
      ...TRemainingAccounts,
    ]
  >;

export type SetDepositCapacityInstructionData = {
  discriminator: number;
  amount: bigint;
};

export type SetDepositCapacityInstructionDataArgs = { amount: number | bigint };

export function getSetDepositCapacityInstructionDataEncoder(): Encoder<SetDepositCapacityInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['amount', getU64Encoder()],
    ]),
    (value) => ({ ...value, discriminator: SET_DEPOSIT_CAPACITY_DISCRIMINATOR })
  );
}

export function getSetDepositCapacityInstructionDataDecoder(): Decoder<SetDepositCapacityInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['amount', getU64Decoder()],
  ]);
}

export function getSetDepositCapacityInstructionDataCodec(): Codec<
  SetDepositCapacityInstructionDataArgs,
  SetDepositCapacityInstructionData
> {
  return combineCodec(
    getSetDepositCapacityInstructionDataEncoder(),
    getSetDepositCapacityInstructionDataDecoder()
  );
}

export type SetDepositCapacityInput<
  TAccountConfig extends string = string,
  TAccountVault extends string = string,
  TAccountAdmin extends string = string,
> = {
  config: Address<TAccountConfig>;
  vault: Address<TAccountVault>;
  admin: TransactionSigner<TAccountAdmin>;
  amount: SetDepositCapacityInstructionDataArgs['amount'];
};

export function getSetDepositCapacityInstruction<
  TAccountConfig extends string,
  TAccountVault extends string,
  TAccountAdmin extends string,
  TProgramAddress extends Address = typeof JITO_VAULT_PROGRAM_ADDRESS,
>(
  input: SetDepositCapacityInput<TAccountConfig, TAccountVault, TAccountAdmin>,
  config?: { programAddress?: TProgramAddress }
): SetDepositCapacityInstruction<
  TProgramAddress,
  TAccountConfig,
  TAccountVault,
  TAccountAdmin
> {
  // Program address.
  const programAddress = config?.programAddress ?? JITO_VAULT_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: false },
    vault: { value: input.vault ?? null, isWritable: true },
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
      getAccountMeta(accounts.admin),
    ],
    programAddress,
    data: getSetDepositCapacityInstructionDataEncoder().encode(
      args as SetDepositCapacityInstructionDataArgs
    ),
  } as SetDepositCapacityInstruction<
    TProgramAddress,
    TAccountConfig,
    TAccountVault,
    TAccountAdmin
  >;

  return instruction;
}

export type ParsedSetDepositCapacityInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    config: TAccountMetas[0];
    vault: TAccountMetas[1];
    admin: TAccountMetas[2];
  };
  data: SetDepositCapacityInstructionData;
};

export function parseSetDepositCapacityInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedSetDepositCapacityInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 3) {
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
      admin: getNextAccount(),
    },
    data: getSetDepositCapacityInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
