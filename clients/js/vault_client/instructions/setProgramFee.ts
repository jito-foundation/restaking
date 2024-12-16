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
  getU16Decoder,
  getU16Encoder,
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
  type ReadonlySignerAccount,
  type TransactionSigner,
  type WritableAccount,
} from '@solana/web3.js';
import { JITO_VAULT_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export const SET_PROGRAM_FEE_DISCRIMINATOR = 20;

export function getSetProgramFeeDiscriminatorBytes() {
  return getU8Encoder().encode(SET_PROGRAM_FEE_DISCRIMINATOR);
}

export type SetProgramFeeInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountAdmin extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountConfig extends string
        ? WritableAccount<TAccountConfig>
        : TAccountConfig,
      TAccountAdmin extends string
        ? ReadonlySignerAccount<TAccountAdmin> &
            IAccountSignerMeta<TAccountAdmin>
        : TAccountAdmin,
      ...TRemainingAccounts,
    ]
  >;

export type SetProgramFeeInstructionData = {
  discriminator: number;
  newFeeBps: number;
};

export type SetProgramFeeInstructionDataArgs = { newFeeBps: number };

export function getSetProgramFeeInstructionDataEncoder(): Encoder<SetProgramFeeInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['newFeeBps', getU16Encoder()],
    ]),
    (value) => ({ ...value, discriminator: SET_PROGRAM_FEE_DISCRIMINATOR })
  );
}

export function getSetProgramFeeInstructionDataDecoder(): Decoder<SetProgramFeeInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['newFeeBps', getU16Decoder()],
  ]);
}

export function getSetProgramFeeInstructionDataCodec(): Codec<
  SetProgramFeeInstructionDataArgs,
  SetProgramFeeInstructionData
> {
  return combineCodec(
    getSetProgramFeeInstructionDataEncoder(),
    getSetProgramFeeInstructionDataDecoder()
  );
}

export type SetProgramFeeInput<
  TAccountConfig extends string = string,
  TAccountAdmin extends string = string,
> = {
  config: Address<TAccountConfig>;
  admin: TransactionSigner<TAccountAdmin>;
  newFeeBps: SetProgramFeeInstructionDataArgs['newFeeBps'];
};

export function getSetProgramFeeInstruction<
  TAccountConfig extends string,
  TAccountAdmin extends string,
>(
  input: SetProgramFeeInput<TAccountConfig, TAccountAdmin>
): SetProgramFeeInstruction<
  typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountAdmin
> {
  // Program address.
  const programAddress = JITO_VAULT_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: true },
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
    accounts: [getAccountMeta(accounts.config), getAccountMeta(accounts.admin)],
    programAddress,
    data: getSetProgramFeeInstructionDataEncoder().encode(
      args as SetProgramFeeInstructionDataArgs
    ),
  } as SetProgramFeeInstruction<
    typeof JITO_VAULT_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountAdmin
  >;

  return instruction;
}

export type ParsedSetProgramFeeInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    config: TAccountMetas[0];
    admin: TAccountMetas[1];
  };
  data: SetProgramFeeInstructionData;
};

export function parseSetProgramFeeInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedSetProgramFeeInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 2) {
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
      admin: getNextAccount(),
    },
    data: getSetProgramFeeInstructionDataDecoder().decode(instruction.data),
  };
}
