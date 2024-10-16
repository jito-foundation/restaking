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
import { JITO_VAULT_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export const SLASH_DISCRIMINATOR = 32;

export function getSlashDiscriminatorBytes() {
  return getU8Encoder().encode(SLASH_DISCRIMINATOR);
}

export type SlashInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountVault extends string | IAccountMeta<string> = string,
  TAccountNcn extends string | IAccountMeta<string> = string,
  TAccountOperator extends string | IAccountMeta<string> = string,
  TAccountSlasher extends string | IAccountMeta<string> = string,
  TAccountNcnOperatorState extends string | IAccountMeta<string> = string,
  TAccountNcnVaultTicket extends string | IAccountMeta<string> = string,
  TAccountOperatorVaultTicket extends string | IAccountMeta<string> = string,
  TAccountVaultNcnTicket extends string | IAccountMeta<string> = string,
  TAccountVaultOperatorDelegation extends
    | string
    | IAccountMeta<string> = string,
  TAccountNcnVaultSlasherTicket extends string | IAccountMeta<string> = string,
  TAccountVaultNcnSlasherTicket extends string | IAccountMeta<string> = string,
  TAccountVaultNcnSlasherOperatorTicket extends
    | string
    | IAccountMeta<string> = string,
  TAccountVaultTokenAccount extends string | IAccountMeta<string> = string,
  TAccountSlasherTokenAccount extends string | IAccountMeta<string> = string,
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
      TAccountNcn extends string ? ReadonlyAccount<TAccountNcn> : TAccountNcn,
      TAccountOperator extends string
        ? ReadonlyAccount<TAccountOperator>
        : TAccountOperator,
      TAccountSlasher extends string
        ? ReadonlySignerAccount<TAccountSlasher> &
            IAccountSignerMeta<TAccountSlasher>
        : TAccountSlasher,
      TAccountNcnOperatorState extends string
        ? ReadonlyAccount<TAccountNcnOperatorState>
        : TAccountNcnOperatorState,
      TAccountNcnVaultTicket extends string
        ? ReadonlyAccount<TAccountNcnVaultTicket>
        : TAccountNcnVaultTicket,
      TAccountOperatorVaultTicket extends string
        ? ReadonlyAccount<TAccountOperatorVaultTicket>
        : TAccountOperatorVaultTicket,
      TAccountVaultNcnTicket extends string
        ? ReadonlyAccount<TAccountVaultNcnTicket>
        : TAccountVaultNcnTicket,
      TAccountVaultOperatorDelegation extends string
        ? WritableAccount<TAccountVaultOperatorDelegation>
        : TAccountVaultOperatorDelegation,
      TAccountNcnVaultSlasherTicket extends string
        ? ReadonlyAccount<TAccountNcnVaultSlasherTicket>
        : TAccountNcnVaultSlasherTicket,
      TAccountVaultNcnSlasherTicket extends string
        ? ReadonlyAccount<TAccountVaultNcnSlasherTicket>
        : TAccountVaultNcnSlasherTicket,
      TAccountVaultNcnSlasherOperatorTicket extends string
        ? WritableAccount<TAccountVaultNcnSlasherOperatorTicket>
        : TAccountVaultNcnSlasherOperatorTicket,
      TAccountVaultTokenAccount extends string
        ? WritableAccount<TAccountVaultTokenAccount>
        : TAccountVaultTokenAccount,
      TAccountSlasherTokenAccount extends string
        ? ReadonlyAccount<TAccountSlasherTokenAccount>
        : TAccountSlasherTokenAccount,
      TAccountTokenProgram extends string
        ? ReadonlyAccount<TAccountTokenProgram>
        : TAccountTokenProgram,
      ...TRemainingAccounts,
    ]
  >;

export type SlashInstructionData = { discriminator: number; amount: bigint };

export type SlashInstructionDataArgs = { amount: number | bigint };

export function getSlashInstructionDataEncoder(): Encoder<SlashInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['amount', getU64Encoder()],
    ]),
    (value) => ({ ...value, discriminator: SLASH_DISCRIMINATOR })
  );
}

export function getSlashInstructionDataDecoder(): Decoder<SlashInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['amount', getU64Decoder()],
  ]);
}

export function getSlashInstructionDataCodec(): Codec<
  SlashInstructionDataArgs,
  SlashInstructionData
> {
  return combineCodec(
    getSlashInstructionDataEncoder(),
    getSlashInstructionDataDecoder()
  );
}

export type SlashInput<
  TAccountConfig extends string = string,
  TAccountVault extends string = string,
  TAccountNcn extends string = string,
  TAccountOperator extends string = string,
  TAccountSlasher extends string = string,
  TAccountNcnOperatorState extends string = string,
  TAccountNcnVaultTicket extends string = string,
  TAccountOperatorVaultTicket extends string = string,
  TAccountVaultNcnTicket extends string = string,
  TAccountVaultOperatorDelegation extends string = string,
  TAccountNcnVaultSlasherTicket extends string = string,
  TAccountVaultNcnSlasherTicket extends string = string,
  TAccountVaultNcnSlasherOperatorTicket extends string = string,
  TAccountVaultTokenAccount extends string = string,
  TAccountSlasherTokenAccount extends string = string,
  TAccountTokenProgram extends string = string,
> = {
  config: Address<TAccountConfig>;
  vault: Address<TAccountVault>;
  ncn: Address<TAccountNcn>;
  operator: Address<TAccountOperator>;
  slasher: TransactionSigner<TAccountSlasher>;
  ncnOperatorState: Address<TAccountNcnOperatorState>;
  ncnVaultTicket: Address<TAccountNcnVaultTicket>;
  operatorVaultTicket: Address<TAccountOperatorVaultTicket>;
  vaultNcnTicket: Address<TAccountVaultNcnTicket>;
  vaultOperatorDelegation: Address<TAccountVaultOperatorDelegation>;
  ncnVaultSlasherTicket: Address<TAccountNcnVaultSlasherTicket>;
  vaultNcnSlasherTicket: Address<TAccountVaultNcnSlasherTicket>;
  vaultNcnSlasherOperatorTicket: Address<TAccountVaultNcnSlasherOperatorTicket>;
  vaultTokenAccount: Address<TAccountVaultTokenAccount>;
  slasherTokenAccount: Address<TAccountSlasherTokenAccount>;
  tokenProgram?: Address<TAccountTokenProgram>;
  amount: SlashInstructionDataArgs['amount'];
};

export function getSlashInstruction<
  TAccountConfig extends string,
  TAccountVault extends string,
  TAccountNcn extends string,
  TAccountOperator extends string,
  TAccountSlasher extends string,
  TAccountNcnOperatorState extends string,
  TAccountNcnVaultTicket extends string,
  TAccountOperatorVaultTicket extends string,
  TAccountVaultNcnTicket extends string,
  TAccountVaultOperatorDelegation extends string,
  TAccountNcnVaultSlasherTicket extends string,
  TAccountVaultNcnSlasherTicket extends string,
  TAccountVaultNcnSlasherOperatorTicket extends string,
  TAccountVaultTokenAccount extends string,
  TAccountSlasherTokenAccount extends string,
  TAccountTokenProgram extends string,
>(
  input: SlashInput<
    TAccountConfig,
    TAccountVault,
    TAccountNcn,
    TAccountOperator,
    TAccountSlasher,
    TAccountNcnOperatorState,
    TAccountNcnVaultTicket,
    TAccountOperatorVaultTicket,
    TAccountVaultNcnTicket,
    TAccountVaultOperatorDelegation,
    TAccountNcnVaultSlasherTicket,
    TAccountVaultNcnSlasherTicket,
    TAccountVaultNcnSlasherOperatorTicket,
    TAccountVaultTokenAccount,
    TAccountSlasherTokenAccount,
    TAccountTokenProgram
  >
): SlashInstruction<
  typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountVault,
  TAccountNcn,
  TAccountOperator,
  TAccountSlasher,
  TAccountNcnOperatorState,
  TAccountNcnVaultTicket,
  TAccountOperatorVaultTicket,
  TAccountVaultNcnTicket,
  TAccountVaultOperatorDelegation,
  TAccountNcnVaultSlasherTicket,
  TAccountVaultNcnSlasherTicket,
  TAccountVaultNcnSlasherOperatorTicket,
  TAccountVaultTokenAccount,
  TAccountSlasherTokenAccount,
  TAccountTokenProgram
> {
  // Program address.
  const programAddress = JITO_VAULT_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: false },
    vault: { value: input.vault ?? null, isWritable: true },
    ncn: { value: input.ncn ?? null, isWritable: false },
    operator: { value: input.operator ?? null, isWritable: false },
    slasher: { value: input.slasher ?? null, isWritable: false },
    ncnOperatorState: {
      value: input.ncnOperatorState ?? null,
      isWritable: false,
    },
    ncnVaultTicket: { value: input.ncnVaultTicket ?? null, isWritable: false },
    operatorVaultTicket: {
      value: input.operatorVaultTicket ?? null,
      isWritable: false,
    },
    vaultNcnTicket: { value: input.vaultNcnTicket ?? null, isWritable: false },
    vaultOperatorDelegation: {
      value: input.vaultOperatorDelegation ?? null,
      isWritable: true,
    },
    ncnVaultSlasherTicket: {
      value: input.ncnVaultSlasherTicket ?? null,
      isWritable: false,
    },
    vaultNcnSlasherTicket: {
      value: input.vaultNcnSlasherTicket ?? null,
      isWritable: false,
    },
    vaultNcnSlasherOperatorTicket: {
      value: input.vaultNcnSlasherOperatorTicket ?? null,
      isWritable: true,
    },
    vaultTokenAccount: {
      value: input.vaultTokenAccount ?? null,
      isWritable: true,
    },
    slasherTokenAccount: {
      value: input.slasherTokenAccount ?? null,
      isWritable: false,
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
      getAccountMeta(accounts.config),
      getAccountMeta(accounts.vault),
      getAccountMeta(accounts.ncn),
      getAccountMeta(accounts.operator),
      getAccountMeta(accounts.slasher),
      getAccountMeta(accounts.ncnOperatorState),
      getAccountMeta(accounts.ncnVaultTicket),
      getAccountMeta(accounts.operatorVaultTicket),
      getAccountMeta(accounts.vaultNcnTicket),
      getAccountMeta(accounts.vaultOperatorDelegation),
      getAccountMeta(accounts.ncnVaultSlasherTicket),
      getAccountMeta(accounts.vaultNcnSlasherTicket),
      getAccountMeta(accounts.vaultNcnSlasherOperatorTicket),
      getAccountMeta(accounts.vaultTokenAccount),
      getAccountMeta(accounts.slasherTokenAccount),
      getAccountMeta(accounts.tokenProgram),
    ],
    programAddress,
    data: getSlashInstructionDataEncoder().encode(
      args as SlashInstructionDataArgs
    ),
  } as SlashInstruction<
    typeof JITO_VAULT_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountVault,
    TAccountNcn,
    TAccountOperator,
    TAccountSlasher,
    TAccountNcnOperatorState,
    TAccountNcnVaultTicket,
    TAccountOperatorVaultTicket,
    TAccountVaultNcnTicket,
    TAccountVaultOperatorDelegation,
    TAccountNcnVaultSlasherTicket,
    TAccountVaultNcnSlasherTicket,
    TAccountVaultNcnSlasherOperatorTicket,
    TAccountVaultTokenAccount,
    TAccountSlasherTokenAccount,
    TAccountTokenProgram
  >;

  return instruction;
}

export type ParsedSlashInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    config: TAccountMetas[0];
    vault: TAccountMetas[1];
    ncn: TAccountMetas[2];
    operator: TAccountMetas[3];
    slasher: TAccountMetas[4];
    ncnOperatorState: TAccountMetas[5];
    ncnVaultTicket: TAccountMetas[6];
    operatorVaultTicket: TAccountMetas[7];
    vaultNcnTicket: TAccountMetas[8];
    vaultOperatorDelegation: TAccountMetas[9];
    ncnVaultSlasherTicket: TAccountMetas[10];
    vaultNcnSlasherTicket: TAccountMetas[11];
    vaultNcnSlasherOperatorTicket: TAccountMetas[12];
    vaultTokenAccount: TAccountMetas[13];
    slasherTokenAccount: TAccountMetas[14];
    tokenProgram: TAccountMetas[15];
  };
  data: SlashInstructionData;
};

export function parseSlashInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedSlashInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 16) {
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
      ncn: getNextAccount(),
      operator: getNextAccount(),
      slasher: getNextAccount(),
      ncnOperatorState: getNextAccount(),
      ncnVaultTicket: getNextAccount(),
      operatorVaultTicket: getNextAccount(),
      vaultNcnTicket: getNextAccount(),
      vaultOperatorDelegation: getNextAccount(),
      ncnVaultSlasherTicket: getNextAccount(),
      vaultNcnSlasherTicket: getNextAccount(),
      vaultNcnSlasherOperatorTicket: getNextAccount(),
      vaultTokenAccount: getNextAccount(),
      slasherTokenAccount: getNextAccount(),
      tokenProgram: getNextAccount(),
    },
    data: getSlashInstructionDataDecoder().decode(instruction.data),
  };
}
