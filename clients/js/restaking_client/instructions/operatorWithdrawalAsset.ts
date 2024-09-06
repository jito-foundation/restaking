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
} from '@solana/web3.js';
import { JITO_RESTAKING_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export const OPERATOR_WITHDRAWAL_ASSET_DISCRIMINATOR = 22;

export function getOperatorWithdrawalAssetDiscriminatorBytes() {
  return getU8Encoder().encode(OPERATOR_WITHDRAWAL_ASSET_DISCRIMINATOR);
}

export type OperatorWithdrawalAssetInstruction<
  TProgram extends string = typeof JITO_RESTAKING_PROGRAM_ADDRESS,
  TAccountOperator extends string | IAccountMeta<string> = string,
  TAccountAdmin extends string | IAccountMeta<string> = string,
  TAccountMint extends string | IAccountMeta<string> = string,
  TAccountOperatorTokenAccount extends string | IAccountMeta<string> = string,
  TAccountReceiverTokenAccount extends string | IAccountMeta<string> = string,
  TAccountTokenProgram extends
    | string
    | IAccountMeta<string> = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountOperator extends string
        ? ReadonlyAccount<TAccountOperator>
        : TAccountOperator,
      TAccountAdmin extends string
        ? ReadonlySignerAccount<TAccountAdmin> &
            IAccountSignerMeta<TAccountAdmin>
        : TAccountAdmin,
      TAccountMint extends string
        ? ReadonlyAccount<TAccountMint>
        : TAccountMint,
      TAccountOperatorTokenAccount extends string
        ? WritableAccount<TAccountOperatorTokenAccount>
        : TAccountOperatorTokenAccount,
      TAccountReceiverTokenAccount extends string
        ? WritableAccount<TAccountReceiverTokenAccount>
        : TAccountReceiverTokenAccount,
      TAccountTokenProgram extends string
        ? ReadonlyAccount<TAccountTokenProgram>
        : TAccountTokenProgram,
      ...TRemainingAccounts,
    ]
  >;

export type OperatorWithdrawalAssetInstructionData = {
  discriminator: number;
  amount: bigint;
};

export type OperatorWithdrawalAssetInstructionDataArgs = {
  amount: number | bigint;
};

export function getOperatorWithdrawalAssetInstructionDataEncoder(): Encoder<OperatorWithdrawalAssetInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['amount', getU64Encoder()],
    ]),
    (value) => ({
      ...value,
      discriminator: OPERATOR_WITHDRAWAL_ASSET_DISCRIMINATOR,
    })
  );
}

export function getOperatorWithdrawalAssetInstructionDataDecoder(): Decoder<OperatorWithdrawalAssetInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['amount', getU64Decoder()],
  ]);
}

export function getOperatorWithdrawalAssetInstructionDataCodec(): Codec<
  OperatorWithdrawalAssetInstructionDataArgs,
  OperatorWithdrawalAssetInstructionData
> {
  return combineCodec(
    getOperatorWithdrawalAssetInstructionDataEncoder(),
    getOperatorWithdrawalAssetInstructionDataDecoder()
  );
}

export type OperatorWithdrawalAssetInput<
  TAccountOperator extends string = string,
  TAccountAdmin extends string = string,
  TAccountMint extends string = string,
  TAccountOperatorTokenAccount extends string = string,
  TAccountReceiverTokenAccount extends string = string,
  TAccountTokenProgram extends string = string,
> = {
  operator: Address<TAccountOperator>;
  admin: TransactionSigner<TAccountAdmin>;
  mint: Address<TAccountMint>;
  operatorTokenAccount: Address<TAccountOperatorTokenAccount>;
  receiverTokenAccount: Address<TAccountReceiverTokenAccount>;
  tokenProgram?: Address<TAccountTokenProgram>;
  amount: OperatorWithdrawalAssetInstructionDataArgs['amount'];
};

export function getOperatorWithdrawalAssetInstruction<
  TAccountOperator extends string,
  TAccountAdmin extends string,
  TAccountMint extends string,
  TAccountOperatorTokenAccount extends string,
  TAccountReceiverTokenAccount extends string,
  TAccountTokenProgram extends string,
>(
  input: OperatorWithdrawalAssetInput<
    TAccountOperator,
    TAccountAdmin,
    TAccountMint,
    TAccountOperatorTokenAccount,
    TAccountReceiverTokenAccount,
    TAccountTokenProgram
  >
): OperatorWithdrawalAssetInstruction<
  typeof JITO_RESTAKING_PROGRAM_ADDRESS,
  TAccountOperator,
  TAccountAdmin,
  TAccountMint,
  TAccountOperatorTokenAccount,
  TAccountReceiverTokenAccount,
  TAccountTokenProgram
> {
  // Program address.
  const programAddress = JITO_RESTAKING_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    operator: { value: input.operator ?? null, isWritable: false },
    admin: { value: input.admin ?? null, isWritable: false },
    mint: { value: input.mint ?? null, isWritable: false },
    operatorTokenAccount: {
      value: input.operatorTokenAccount ?? null,
      isWritable: true,
    },
    receiverTokenAccount: {
      value: input.receiverTokenAccount ?? null,
      isWritable: true,
    },
    tokenProgram: { value: input.tokenProgram ?? null, isWritable: false },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Original args.
  const args = { ...input };

  // Resolve default values.
  if (!accounts.tokenProgram.value) {
    accounts.tokenProgram.value =
      'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA' as Address<'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'>;
  }

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.operator),
      getAccountMeta(accounts.admin),
      getAccountMeta(accounts.mint),
      getAccountMeta(accounts.operatorTokenAccount),
      getAccountMeta(accounts.receiverTokenAccount),
      getAccountMeta(accounts.tokenProgram),
    ],
    programAddress,
    data: getOperatorWithdrawalAssetInstructionDataEncoder().encode(
      args as OperatorWithdrawalAssetInstructionDataArgs
    ),
  } as OperatorWithdrawalAssetInstruction<
    typeof JITO_RESTAKING_PROGRAM_ADDRESS,
    TAccountOperator,
    TAccountAdmin,
    TAccountMint,
    TAccountOperatorTokenAccount,
    TAccountReceiverTokenAccount,
    TAccountTokenProgram
  >;

  return instruction;
}

export type ParsedOperatorWithdrawalAssetInstruction<
  TProgram extends string = typeof JITO_RESTAKING_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    operator: TAccountMetas[0];
    admin: TAccountMetas[1];
    mint: TAccountMetas[2];
    operatorTokenAccount: TAccountMetas[3];
    receiverTokenAccount: TAccountMetas[4];
    tokenProgram: TAccountMetas[5];
  };
  data: OperatorWithdrawalAssetInstructionData;
};

export function parseOperatorWithdrawalAssetInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedOperatorWithdrawalAssetInstruction<TProgram, TAccountMetas> {
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
      operator: getNextAccount(),
      admin: getNextAccount(),
      mint: getNextAccount(),
      operatorTokenAccount: getNextAccount(),
      receiverTokenAccount: getNextAccount(),
      tokenProgram: getNextAccount(),
    },
    data: getOperatorWithdrawalAssetInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
