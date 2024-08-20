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

export const COOLDOWN_NCN_VAULT_SLASHER_TICKET_DISCRIMINATOR = 14;

export function getCooldownNcnVaultSlasherTicketDiscriminatorBytes() {
  return getU8Encoder().encode(COOLDOWN_NCN_VAULT_SLASHER_TICKET_DISCRIMINATOR);
}

export type CooldownNcnVaultSlasherTicketInstruction<
  TProgram extends string = typeof JITO_RESTAKING_SDK_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountNcn extends string | IAccountMeta<string> = string,
  TAccountVault extends string | IAccountMeta<string> = string,
  TAccountSlasher extends string | IAccountMeta<string> = string,
  TAccountNcnVaultSlasherTicket extends string | IAccountMeta<string> = string,
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
      TAccountVault extends string
        ? ReadonlyAccount<TAccountVault>
        : TAccountVault,
      TAccountSlasher extends string
        ? ReadonlyAccount<TAccountSlasher>
        : TAccountSlasher,
      TAccountNcnVaultSlasherTicket extends string
        ? WritableAccount<TAccountNcnVaultSlasherTicket>
        : TAccountNcnVaultSlasherTicket,
      TAccountAdmin extends string
        ? ReadonlySignerAccount<TAccountAdmin> &
            IAccountSignerMeta<TAccountAdmin>
        : TAccountAdmin,
      ...TRemainingAccounts,
    ]
  >;

export type CooldownNcnVaultSlasherTicketInstructionData = {
  discriminator: number;
};

export type CooldownNcnVaultSlasherTicketInstructionDataArgs = {};

export function getCooldownNcnVaultSlasherTicketInstructionDataEncoder(): Encoder<CooldownNcnVaultSlasherTicketInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({
      ...value,
      discriminator: COOLDOWN_NCN_VAULT_SLASHER_TICKET_DISCRIMINATOR,
    })
  );
}

export function getCooldownNcnVaultSlasherTicketInstructionDataDecoder(): Decoder<CooldownNcnVaultSlasherTicketInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getCooldownNcnVaultSlasherTicketInstructionDataCodec(): Codec<
  CooldownNcnVaultSlasherTicketInstructionDataArgs,
  CooldownNcnVaultSlasherTicketInstructionData
> {
  return combineCodec(
    getCooldownNcnVaultSlasherTicketInstructionDataEncoder(),
    getCooldownNcnVaultSlasherTicketInstructionDataDecoder()
  );
}

export type CooldownNcnVaultSlasherTicketInput<
  TAccountConfig extends string = string,
  TAccountNcn extends string = string,
  TAccountVault extends string = string,
  TAccountSlasher extends string = string,
  TAccountNcnVaultSlasherTicket extends string = string,
  TAccountAdmin extends string = string,
> = {
  config: Address<TAccountConfig>;
  ncn: Address<TAccountNcn>;
  vault: Address<TAccountVault>;
  slasher: Address<TAccountSlasher>;
  ncnVaultSlasherTicket: Address<TAccountNcnVaultSlasherTicket>;
  admin: TransactionSigner<TAccountAdmin>;
};

export function getCooldownNcnVaultSlasherTicketInstruction<
  TAccountConfig extends string,
  TAccountNcn extends string,
  TAccountVault extends string,
  TAccountSlasher extends string,
  TAccountNcnVaultSlasherTicket extends string,
  TAccountAdmin extends string,
>(
  input: CooldownNcnVaultSlasherTicketInput<
    TAccountConfig,
    TAccountNcn,
    TAccountVault,
    TAccountSlasher,
    TAccountNcnVaultSlasherTicket,
    TAccountAdmin
  >
): CooldownNcnVaultSlasherTicketInstruction<
  typeof JITO_RESTAKING_SDK_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountNcn,
  TAccountVault,
  TAccountSlasher,
  TAccountNcnVaultSlasherTicket,
  TAccountAdmin
> {
  // Program address.
  const programAddress = JITO_RESTAKING_SDK_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: false },
    ncn: { value: input.ncn ?? null, isWritable: false },
    vault: { value: input.vault ?? null, isWritable: false },
    slasher: { value: input.slasher ?? null, isWritable: false },
    ncnVaultSlasherTicket: {
      value: input.ncnVaultSlasherTicket ?? null,
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
      getAccountMeta(accounts.vault),
      getAccountMeta(accounts.slasher),
      getAccountMeta(accounts.ncnVaultSlasherTicket),
      getAccountMeta(accounts.admin),
    ],
    programAddress,
    data: getCooldownNcnVaultSlasherTicketInstructionDataEncoder().encode({}),
  } as CooldownNcnVaultSlasherTicketInstruction<
    typeof JITO_RESTAKING_SDK_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountNcn,
    TAccountVault,
    TAccountSlasher,
    TAccountNcnVaultSlasherTicket,
    TAccountAdmin
  >;

  return instruction;
}

export type ParsedCooldownNcnVaultSlasherTicketInstruction<
  TProgram extends string = typeof JITO_RESTAKING_SDK_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    config: TAccountMetas[0];
    ncn: TAccountMetas[1];
    vault: TAccountMetas[2];
    slasher: TAccountMetas[3];
    ncnVaultSlasherTicket: TAccountMetas[4];
    admin: TAccountMetas[5];
  };
  data: CooldownNcnVaultSlasherTicketInstructionData;
};

export function parseCooldownNcnVaultSlasherTicketInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedCooldownNcnVaultSlasherTicketInstruction<TProgram, TAccountMetas> {
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
      ncn: getNextAccount(),
      vault: getNextAccount(),
      slasher: getNextAccount(),
      ncnVaultSlasherTicket: getNextAccount(),
      admin: getNextAccount(),
    },
    data: getCooldownNcnVaultSlasherTicketInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
