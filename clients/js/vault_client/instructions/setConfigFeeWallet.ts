/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  combineCodec,
  getAddressDecoder,
  getAddressEncoder,
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

export const SET_CONFIG_FEE_WALLET_DISCRIMINATOR = 19;

export function getSetConfigFeeWalletDiscriminatorBytes() {
  return getU8Encoder().encode(SET_CONFIG_FEE_WALLET_DISCRIMINATOR);
}

export type SetConfigFeeWalletInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountConfigFeeAdmin extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountConfig extends string
        ? WritableAccount<TAccountConfig>
        : TAccountConfig,
      TAccountConfigFeeAdmin extends string
        ? ReadonlySignerAccount<TAccountConfigFeeAdmin> &
            IAccountSignerMeta<TAccountConfigFeeAdmin>
        : TAccountConfigFeeAdmin,
      ...TRemainingAccounts,
    ]
  >;

export type SetConfigFeeWalletInstructionData = {
  discriminator: number;
  newFeeWallet: Address;
};

export type SetConfigFeeWalletInstructionDataArgs = { newFeeWallet: Address };

export function getSetConfigFeeWalletInstructionDataEncoder(): Encoder<SetConfigFeeWalletInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['newFeeWallet', getAddressEncoder()],
    ]),
    (value) => ({
      ...value,
      discriminator: SET_CONFIG_FEE_WALLET_DISCRIMINATOR,
    })
  );
}

export function getSetConfigFeeWalletInstructionDataDecoder(): Decoder<SetConfigFeeWalletInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['newFeeWallet', getAddressDecoder()],
  ]);
}

export function getSetConfigFeeWalletInstructionDataCodec(): Codec<
  SetConfigFeeWalletInstructionDataArgs,
  SetConfigFeeWalletInstructionData
> {
  return combineCodec(
    getSetConfigFeeWalletInstructionDataEncoder(),
    getSetConfigFeeWalletInstructionDataDecoder()
  );
}

export type SetConfigFeeWalletInput<
  TAccountConfig extends string = string,
  TAccountConfigFeeAdmin extends string = string,
> = {
  config: Address<TAccountConfig>;
  configFeeAdmin: TransactionSigner<TAccountConfigFeeAdmin>;
  newFeeWallet: SetConfigFeeWalletInstructionDataArgs['newFeeWallet'];
};

export function getSetConfigFeeWalletInstruction<
  TAccountConfig extends string,
  TAccountConfigFeeAdmin extends string,
>(
  input: SetConfigFeeWalletInput<TAccountConfig, TAccountConfigFeeAdmin>
): SetConfigFeeWalletInstruction<
  typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountConfigFeeAdmin
> {
  // Program address.
  const programAddress = JITO_VAULT_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: true },
    configFeeAdmin: { value: input.configFeeAdmin ?? null, isWritable: false },
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
      getAccountMeta(accounts.configFeeAdmin),
    ],
    programAddress,
    data: getSetConfigFeeWalletInstructionDataEncoder().encode(
      args as SetConfigFeeWalletInstructionDataArgs
    ),
  } as SetConfigFeeWalletInstruction<
    typeof JITO_VAULT_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountConfigFeeAdmin
  >;

  return instruction;
}

export type ParsedSetConfigFeeWalletInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    config: TAccountMetas[0];
    configFeeAdmin: TAccountMetas[1];
  };
  data: SetConfigFeeWalletInstructionData;
};

export function parseSetConfigFeeWalletInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedSetConfigFeeWalletInstruction<TProgram, TAccountMetas> {
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
      configFeeAdmin: getNextAccount(),
    },
    data: getSetConfigFeeWalletInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
