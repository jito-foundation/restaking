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
} from '@solana/kit';
import { JITO_VAULT_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export const WARMUP_VAULT_NCN_SLASHER_TICKET_DISCRIMINATOR = 9;

export function getWarmupVaultNcnSlasherTicketDiscriminatorBytes() {
  return getU8Encoder().encode(WARMUP_VAULT_NCN_SLASHER_TICKET_DISCRIMINATOR);
}

export type WarmupVaultNcnSlasherTicketInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountVault extends string | IAccountMeta<string> = string,
  TAccountNcn extends string | IAccountMeta<string> = string,
  TAccountSlasher extends string | IAccountMeta<string> = string,
  TAccountVaultSlasherTicket extends string | IAccountMeta<string> = string,
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
        ? ReadonlyAccount<TAccountVault>
        : TAccountVault,
      TAccountNcn extends string ? ReadonlyAccount<TAccountNcn> : TAccountNcn,
      TAccountSlasher extends string
        ? ReadonlyAccount<TAccountSlasher>
        : TAccountSlasher,
      TAccountVaultSlasherTicket extends string
        ? WritableAccount<TAccountVaultSlasherTicket>
        : TAccountVaultSlasherTicket,
      TAccountAdmin extends string
        ? ReadonlySignerAccount<TAccountAdmin> &
            IAccountSignerMeta<TAccountAdmin>
        : TAccountAdmin,
      ...TRemainingAccounts,
    ]
  >;

export type WarmupVaultNcnSlasherTicketInstructionData = {
  discriminator: number;
};

export type WarmupVaultNcnSlasherTicketInstructionDataArgs = {};

export function getWarmupVaultNcnSlasherTicketInstructionDataEncoder(): Encoder<WarmupVaultNcnSlasherTicketInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({
      ...value,
      discriminator: WARMUP_VAULT_NCN_SLASHER_TICKET_DISCRIMINATOR,
    })
  );
}

export function getWarmupVaultNcnSlasherTicketInstructionDataDecoder(): Decoder<WarmupVaultNcnSlasherTicketInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getWarmupVaultNcnSlasherTicketInstructionDataCodec(): Codec<
  WarmupVaultNcnSlasherTicketInstructionDataArgs,
  WarmupVaultNcnSlasherTicketInstructionData
> {
  return combineCodec(
    getWarmupVaultNcnSlasherTicketInstructionDataEncoder(),
    getWarmupVaultNcnSlasherTicketInstructionDataDecoder()
  );
}

export type WarmupVaultNcnSlasherTicketInput<
  TAccountConfig extends string = string,
  TAccountVault extends string = string,
  TAccountNcn extends string = string,
  TAccountSlasher extends string = string,
  TAccountVaultSlasherTicket extends string = string,
  TAccountAdmin extends string = string,
> = {
  config: Address<TAccountConfig>;
  vault: Address<TAccountVault>;
  ncn: Address<TAccountNcn>;
  slasher: Address<TAccountSlasher>;
  vaultSlasherTicket: Address<TAccountVaultSlasherTicket>;
  admin: TransactionSigner<TAccountAdmin>;
};

export function getWarmupVaultNcnSlasherTicketInstruction<
  TAccountConfig extends string,
  TAccountVault extends string,
  TAccountNcn extends string,
  TAccountSlasher extends string,
  TAccountVaultSlasherTicket extends string,
  TAccountAdmin extends string,
  TProgramAddress extends Address = typeof JITO_VAULT_PROGRAM_ADDRESS,
>(
  input: WarmupVaultNcnSlasherTicketInput<
    TAccountConfig,
    TAccountVault,
    TAccountNcn,
    TAccountSlasher,
    TAccountVaultSlasherTicket,
    TAccountAdmin
  >,
  config?: { programAddress?: TProgramAddress }
): WarmupVaultNcnSlasherTicketInstruction<
  TProgramAddress,
  TAccountConfig,
  TAccountVault,
  TAccountNcn,
  TAccountSlasher,
  TAccountVaultSlasherTicket,
  TAccountAdmin
> {
  // Program address.
  const programAddress = config?.programAddress ?? JITO_VAULT_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: false },
    vault: { value: input.vault ?? null, isWritable: false },
    ncn: { value: input.ncn ?? null, isWritable: false },
    slasher: { value: input.slasher ?? null, isWritable: false },
    vaultSlasherTicket: {
      value: input.vaultSlasherTicket ?? null,
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
      getAccountMeta(accounts.vault),
      getAccountMeta(accounts.ncn),
      getAccountMeta(accounts.slasher),
      getAccountMeta(accounts.vaultSlasherTicket),
      getAccountMeta(accounts.admin),
    ],
    programAddress,
    data: getWarmupVaultNcnSlasherTicketInstructionDataEncoder().encode({}),
  } as WarmupVaultNcnSlasherTicketInstruction<
    TProgramAddress,
    TAccountConfig,
    TAccountVault,
    TAccountNcn,
    TAccountSlasher,
    TAccountVaultSlasherTicket,
    TAccountAdmin
  >;

  return instruction;
}

export type ParsedWarmupVaultNcnSlasherTicketInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    config: TAccountMetas[0];
    vault: TAccountMetas[1];
    ncn: TAccountMetas[2];
    slasher: TAccountMetas[3];
    vaultSlasherTicket: TAccountMetas[4];
    admin: TAccountMetas[5];
  };
  data: WarmupVaultNcnSlasherTicketInstructionData;
};

export function parseWarmupVaultNcnSlasherTicketInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedWarmupVaultNcnSlasherTicketInstruction<TProgram, TAccountMetas> {
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
      ncn: getNextAccount(),
      slasher: getNextAccount(),
      vaultSlasherTicket: getNextAccount(),
      admin: getNextAccount(),
    },
    data: getWarmupVaultNcnSlasherTicketInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
