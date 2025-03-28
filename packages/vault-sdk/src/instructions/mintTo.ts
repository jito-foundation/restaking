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
  type WritableSignerAccount,
} from '@solana/kit';
import { JITO_VAULT_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export const MINT_TO_DISCRIMINATOR = 11;

export function getMintToDiscriminatorBytes() {
  return getU8Encoder().encode(MINT_TO_DISCRIMINATOR);
}

export type MintToInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountVault extends string | IAccountMeta<string> = string,
  TAccountVrtMint extends string | IAccountMeta<string> = string,
  TAccountDepositor extends string | IAccountMeta<string> = string,
  TAccountDepositorTokenAccount extends string | IAccountMeta<string> = string,
  TAccountVaultTokenAccount extends string | IAccountMeta<string> = string,
  TAccountDepositorVrtTokenAccount extends
    | string
    | IAccountMeta<string> = string,
  TAccountVaultFeeTokenAccount extends string | IAccountMeta<string> = string,
  TAccountTokenProgram extends
    | string
    | IAccountMeta<string> = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',
  TAccountMintSigner extends string | IAccountMeta<string> = string,
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
      TAccountVrtMint extends string
        ? WritableAccount<TAccountVrtMint>
        : TAccountVrtMint,
      TAccountDepositor extends string
        ? WritableSignerAccount<TAccountDepositor> &
            IAccountSignerMeta<TAccountDepositor>
        : TAccountDepositor,
      TAccountDepositorTokenAccount extends string
        ? WritableAccount<TAccountDepositorTokenAccount>
        : TAccountDepositorTokenAccount,
      TAccountVaultTokenAccount extends string
        ? WritableAccount<TAccountVaultTokenAccount>
        : TAccountVaultTokenAccount,
      TAccountDepositorVrtTokenAccount extends string
        ? WritableAccount<TAccountDepositorVrtTokenAccount>
        : TAccountDepositorVrtTokenAccount,
      TAccountVaultFeeTokenAccount extends string
        ? WritableAccount<TAccountVaultFeeTokenAccount>
        : TAccountVaultFeeTokenAccount,
      TAccountTokenProgram extends string
        ? ReadonlyAccount<TAccountTokenProgram>
        : TAccountTokenProgram,
      TAccountMintSigner extends string
        ? ReadonlySignerAccount<TAccountMintSigner> &
            IAccountSignerMeta<TAccountMintSigner>
        : TAccountMintSigner,
      ...TRemainingAccounts,
    ]
  >;

export type MintToInstructionData = {
  discriminator: number;
  amountIn: bigint;
  minAmountOut: bigint;
};

export type MintToInstructionDataArgs = {
  amountIn: number | bigint;
  minAmountOut: number | bigint;
};

export function getMintToInstructionDataEncoder(): Encoder<MintToInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['amountIn', getU64Encoder()],
      ['minAmountOut', getU64Encoder()],
    ]),
    (value) => ({ ...value, discriminator: MINT_TO_DISCRIMINATOR })
  );
}

export function getMintToInstructionDataDecoder(): Decoder<MintToInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['amountIn', getU64Decoder()],
    ['minAmountOut', getU64Decoder()],
  ]);
}

export function getMintToInstructionDataCodec(): Codec<
  MintToInstructionDataArgs,
  MintToInstructionData
> {
  return combineCodec(
    getMintToInstructionDataEncoder(),
    getMintToInstructionDataDecoder()
  );
}

export type MintToInput<
  TAccountConfig extends string = string,
  TAccountVault extends string = string,
  TAccountVrtMint extends string = string,
  TAccountDepositor extends string = string,
  TAccountDepositorTokenAccount extends string = string,
  TAccountVaultTokenAccount extends string = string,
  TAccountDepositorVrtTokenAccount extends string = string,
  TAccountVaultFeeTokenAccount extends string = string,
  TAccountTokenProgram extends string = string,
  TAccountMintSigner extends string = string,
> = {
  config: Address<TAccountConfig>;
  vault: Address<TAccountVault>;
  vrtMint: Address<TAccountVrtMint>;
  depositor: TransactionSigner<TAccountDepositor>;
  depositorTokenAccount: Address<TAccountDepositorTokenAccount>;
  vaultTokenAccount: Address<TAccountVaultTokenAccount>;
  depositorVrtTokenAccount: Address<TAccountDepositorVrtTokenAccount>;
  vaultFeeTokenAccount: Address<TAccountVaultFeeTokenAccount>;
  tokenProgram?: Address<TAccountTokenProgram>;
  /** Signer for minting */
  mintSigner?: TransactionSigner<TAccountMintSigner>;
  amountIn: MintToInstructionDataArgs['amountIn'];
  minAmountOut: MintToInstructionDataArgs['minAmountOut'];
};

export function getMintToInstruction<
  TAccountConfig extends string,
  TAccountVault extends string,
  TAccountVrtMint extends string,
  TAccountDepositor extends string,
  TAccountDepositorTokenAccount extends string,
  TAccountVaultTokenAccount extends string,
  TAccountDepositorVrtTokenAccount extends string,
  TAccountVaultFeeTokenAccount extends string,
  TAccountTokenProgram extends string,
  TAccountMintSigner extends string,
  TProgramAddress extends Address = typeof JITO_VAULT_PROGRAM_ADDRESS,
>(
  input: MintToInput<
    TAccountConfig,
    TAccountVault,
    TAccountVrtMint,
    TAccountDepositor,
    TAccountDepositorTokenAccount,
    TAccountVaultTokenAccount,
    TAccountDepositorVrtTokenAccount,
    TAccountVaultFeeTokenAccount,
    TAccountTokenProgram,
    TAccountMintSigner
  >,
  config?: { programAddress?: TProgramAddress }
): MintToInstruction<
  TProgramAddress,
  TAccountConfig,
  TAccountVault,
  TAccountVrtMint,
  TAccountDepositor,
  TAccountDepositorTokenAccount,
  TAccountVaultTokenAccount,
  TAccountDepositorVrtTokenAccount,
  TAccountVaultFeeTokenAccount,
  TAccountTokenProgram,
  TAccountMintSigner
> {
  // Program address.
  const programAddress = config?.programAddress ?? JITO_VAULT_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: false },
    vault: { value: input.vault ?? null, isWritable: true },
    vrtMint: { value: input.vrtMint ?? null, isWritable: true },
    depositor: { value: input.depositor ?? null, isWritable: true },
    depositorTokenAccount: {
      value: input.depositorTokenAccount ?? null,
      isWritable: true,
    },
    vaultTokenAccount: {
      value: input.vaultTokenAccount ?? null,
      isWritable: true,
    },
    depositorVrtTokenAccount: {
      value: input.depositorVrtTokenAccount ?? null,
      isWritable: true,
    },
    vaultFeeTokenAccount: {
      value: input.vaultFeeTokenAccount ?? null,
      isWritable: true,
    },
    tokenProgram: { value: input.tokenProgram ?? null, isWritable: false },
    mintSigner: { value: input.mintSigner ?? null, isWritable: false },
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
      getAccountMeta(accounts.config),
      getAccountMeta(accounts.vault),
      getAccountMeta(accounts.vrtMint),
      getAccountMeta(accounts.depositor),
      getAccountMeta(accounts.depositorTokenAccount),
      getAccountMeta(accounts.vaultTokenAccount),
      getAccountMeta(accounts.depositorVrtTokenAccount),
      getAccountMeta(accounts.vaultFeeTokenAccount),
      getAccountMeta(accounts.tokenProgram),
      getAccountMeta(accounts.mintSigner),
    ],
    programAddress,
    data: getMintToInstructionDataEncoder().encode(
      args as MintToInstructionDataArgs
    ),
  } as MintToInstruction<
    TProgramAddress,
    TAccountConfig,
    TAccountVault,
    TAccountVrtMint,
    TAccountDepositor,
    TAccountDepositorTokenAccount,
    TAccountVaultTokenAccount,
    TAccountDepositorVrtTokenAccount,
    TAccountVaultFeeTokenAccount,
    TAccountTokenProgram,
    TAccountMintSigner
  >;

  return instruction;
}

export type ParsedMintToInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    config: TAccountMetas[0];
    vault: TAccountMetas[1];
    vrtMint: TAccountMetas[2];
    depositor: TAccountMetas[3];
    depositorTokenAccount: TAccountMetas[4];
    vaultTokenAccount: TAccountMetas[5];
    depositorVrtTokenAccount: TAccountMetas[6];
    vaultFeeTokenAccount: TAccountMetas[7];
    tokenProgram: TAccountMetas[8];
    /** Signer for minting */
    mintSigner?: TAccountMetas[9] | undefined;
  };
  data: MintToInstructionData;
};

export function parseMintToInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedMintToInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 10) {
    // TODO: Coded error.
    throw new Error('Not enough accounts');
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts![accountIndex]!;
    accountIndex += 1;
    return accountMeta;
  };
  const getNextOptionalAccount = () => {
    const accountMeta = getNextAccount();
    return accountMeta.address === JITO_VAULT_PROGRAM_ADDRESS
      ? undefined
      : accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      config: getNextAccount(),
      vault: getNextAccount(),
      vrtMint: getNextAccount(),
      depositor: getNextAccount(),
      depositorTokenAccount: getNextAccount(),
      vaultTokenAccount: getNextAccount(),
      depositorVrtTokenAccount: getNextAccount(),
      vaultFeeTokenAccount: getNextAccount(),
      tokenProgram: getNextAccount(),
      mintSigner: getNextOptionalAccount(),
    },
    data: getMintToInstructionDataDecoder().decode(instruction.data),
  };
}
