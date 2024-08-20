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
  type IAccountSignerMeta,
  type IInstruction,
  type IInstructionWithAccounts,
  type IInstructionWithData,
  type ReadonlyAccount,
  type ReadonlySignerAccount,
  type TransactionSigner,
  type WritableAccount,
} from '@solana/web3.js';
import { JITO_RESTAKING_SDK_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export const OPERATOR_COOLDOWN_NCN_DISCRIMINATOR = 12;

export function getOperatorCooldownNcnDiscriminatorBytes() {
  return getU8Encoder().encode(OPERATOR_COOLDOWN_NCN_DISCRIMINATOR);
}

export type OperatorCooldownNcnInstruction<
  TProgram extends string = typeof JITO_RESTAKING_SDK_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountNcn extends string | IAccountMeta<string> = string,
  TAccountOperator extends string | IAccountMeta<string> = string,
  TAccountNcnOperatorState extends string | IAccountMeta<string> = string,
  TAccountAdmin extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountConfig extends string
        ? ReadonlyAccount<TAccountConfig>
        : TAccountConfig,
      TAccountNcn extends string ? ReadonlyAccount<TAccountNcn> : TAccountNcn,
      TAccountOperator extends string
        ? ReadonlyAccount<TAccountOperator>
        : TAccountOperator,
      TAccountNcnOperatorState extends string
        ? WritableAccount<TAccountNcnOperatorState>
        : TAccountNcnOperatorState,
      TAccountAdmin extends string
        ? ReadonlySignerAccount<TAccountAdmin> &
            IAccountSignerMeta<TAccountAdmin>
        : TAccountAdmin,
      ...TRemainingAccounts,
    ]
  >;

export type OperatorCooldownNcnInstructionData = { discriminator: number };

export type OperatorCooldownNcnInstructionDataArgs = {};

export function getOperatorCooldownNcnInstructionDataEncoder(): Encoder<OperatorCooldownNcnInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({
      ...value,
      discriminator: OPERATOR_COOLDOWN_NCN_DISCRIMINATOR,
    })
  );
}

export function getOperatorCooldownNcnInstructionDataDecoder(): Decoder<OperatorCooldownNcnInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getOperatorCooldownNcnInstructionDataCodec(): Codec<
  OperatorCooldownNcnInstructionDataArgs,
  OperatorCooldownNcnInstructionData
> {
  return combineCodec(
    getOperatorCooldownNcnInstructionDataEncoder(),
    getOperatorCooldownNcnInstructionDataDecoder()
  );
}

export type OperatorCooldownNcnInput<
  TAccountConfig extends string = string,
  TAccountNcn extends string = string,
  TAccountOperator extends string = string,
  TAccountNcnOperatorState extends string = string,
  TAccountAdmin extends string = string,
> = {
  config: Address<TAccountConfig>;
  ncn: Address<TAccountNcn>;
  operator: Address<TAccountOperator>;
  ncnOperatorState: Address<TAccountNcnOperatorState>;
  admin: TransactionSigner<TAccountAdmin>;
};

export function getOperatorCooldownNcnInstruction<
  TAccountConfig extends string,
  TAccountNcn extends string,
  TAccountOperator extends string,
  TAccountNcnOperatorState extends string,
  TAccountAdmin extends string,
>(
  input: OperatorCooldownNcnInput<
    TAccountConfig,
    TAccountNcn,
    TAccountOperator,
    TAccountNcnOperatorState,
    TAccountAdmin
  >
): OperatorCooldownNcnInstruction<
  typeof JITO_RESTAKING_SDK_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountNcn,
  TAccountOperator,
  TAccountNcnOperatorState,
  TAccountAdmin
> {
  // Program address.
  const programAddress = JITO_RESTAKING_SDK_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: false },
    ncn: { value: input.ncn ?? null, isWritable: false },
    operator: { value: input.operator ?? null, isWritable: false },
    ncnOperatorState: {
      value: input.ncnOperatorState ?? null,
      isWritable: true,
    },
    admin: { value: input.admin ?? null, isWritable: false },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.config),
      getAccountMeta(accounts.ncn),
      getAccountMeta(accounts.operator),
      getAccountMeta(accounts.ncnOperatorState),
      getAccountMeta(accounts.admin),
    ],
    programAddress,
    data: getOperatorCooldownNcnInstructionDataEncoder().encode({}),
  } as OperatorCooldownNcnInstruction<
    typeof JITO_RESTAKING_SDK_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountNcn,
    TAccountOperator,
    TAccountNcnOperatorState,
    TAccountAdmin
  >;

  return instruction;
}

export type ParsedOperatorCooldownNcnInstruction<
  TProgram extends string = typeof JITO_RESTAKING_SDK_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    config: TAccountMetas[0];
    ncn: TAccountMetas[1];
    operator: TAccountMetas[2];
    ncnOperatorState: TAccountMetas[3];
    admin: TAccountMetas[4];
  };
  data: OperatorCooldownNcnInstructionData;
};

export function parseOperatorCooldownNcnInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedOperatorCooldownNcnInstruction<TProgram, TAccountMetas> {
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
      ncn: getNextAccount(),
      operator: getNextAccount(),
      ncnOperatorState: getNextAccount(),
      admin: getNextAccount(),
    },
    data: getOperatorCooldownNcnInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
