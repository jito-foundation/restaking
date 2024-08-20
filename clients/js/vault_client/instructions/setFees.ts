/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  combineCodec,
  getOptionDecoder,
  getOptionEncoder,
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
  type Option,
  type OptionOrNullable,
  type ReadonlyAccount,
  type ReadonlySignerAccount,
  type TransactionSigner,
  type WritableAccount,
} from '@solana/web3.js';
import { JITO_VAULT_SDK_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export const SET_FEES_DISCRIMINATOR = 17;

export function getSetFeesDiscriminatorBytes() {
  return getU8Encoder().encode(SET_FEES_DISCRIMINATOR);
}

export type SetFeesInstruction<
  TProgram extends string = typeof JITO_VAULT_SDK_PROGRAM_ADDRESS,
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

export type SetFeesInstructionData = {
  discriminator: number;
  depositFeeBps: Option<number>;
  withdrawalFeeBps: Option<number>;
  rewardFeeBps: Option<number>;
};

export type SetFeesInstructionDataArgs = {
  depositFeeBps: OptionOrNullable<number>;
  withdrawalFeeBps: OptionOrNullable<number>;
  rewardFeeBps: OptionOrNullable<number>;
};

export function getSetFeesInstructionDataEncoder(): Encoder<SetFeesInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['depositFeeBps', getOptionEncoder(getU16Encoder())],
      ['withdrawalFeeBps', getOptionEncoder(getU16Encoder())],
      ['rewardFeeBps', getOptionEncoder(getU16Encoder())],
    ]),
    (value) => ({ ...value, discriminator: SET_FEES_DISCRIMINATOR })
  );
}

export function getSetFeesInstructionDataDecoder(): Decoder<SetFeesInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['depositFeeBps', getOptionDecoder(getU16Decoder())],
    ['withdrawalFeeBps', getOptionDecoder(getU16Decoder())],
    ['rewardFeeBps', getOptionDecoder(getU16Decoder())],
  ]);
}

export function getSetFeesInstructionDataCodec(): Codec<
  SetFeesInstructionDataArgs,
  SetFeesInstructionData
> {
  return combineCodec(
    getSetFeesInstructionDataEncoder(),
    getSetFeesInstructionDataDecoder()
  );
}

export type SetFeesInput<
  TAccountConfig extends string = string,
  TAccountVault extends string = string,
  TAccountAdmin extends string = string,
> = {
  config: Address<TAccountConfig>;
  vault: Address<TAccountVault>;
  admin: TransactionSigner<TAccountAdmin>;
  depositFeeBps: SetFeesInstructionDataArgs['depositFeeBps'];
  withdrawalFeeBps: SetFeesInstructionDataArgs['withdrawalFeeBps'];
  rewardFeeBps: SetFeesInstructionDataArgs['rewardFeeBps'];
};

export function getSetFeesInstruction<
  TAccountConfig extends string,
  TAccountVault extends string,
  TAccountAdmin extends string,
>(
  input: SetFeesInput<TAccountConfig, TAccountVault, TAccountAdmin>
): SetFeesInstruction<
  typeof JITO_VAULT_SDK_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountVault,
  TAccountAdmin
> {
  // Program address.
  const programAddress = JITO_VAULT_SDK_PROGRAM_ADDRESS;

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
    data: getSetFeesInstructionDataEncoder().encode(
      args as SetFeesInstructionDataArgs
    ),
  } as SetFeesInstruction<
    typeof JITO_VAULT_SDK_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountVault,
    TAccountAdmin
  >;

  return instruction;
}

export type ParsedSetFeesInstruction<
  TProgram extends string = typeof JITO_VAULT_SDK_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    config: TAccountMetas[0];
    vault: TAccountMetas[1];
    admin: TAccountMetas[2];
  };
  data: SetFeesInstructionData;
};

export function parseSetFeesInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedSetFeesInstruction<TProgram, TAccountMetas> {
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
    data: getSetFeesInstructionDataDecoder().decode(instruction.data),
  };
}
