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
import { JITO_VAULT_PROGRAM_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export const COOLDOWN_VAULT_NCN_TICKET_DISCRIMINATOR = 8;

export function getCooldownVaultNcnTicketDiscriminatorBytes() {
  return getU8Encoder().encode(COOLDOWN_VAULT_NCN_TICKET_DISCRIMINATOR);
}

export type CooldownVaultNcnTicketInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig extends string | IAccountMeta<string> = string,
  TAccountVault extends string | IAccountMeta<string> = string,
  TAccountNcn extends string | IAccountMeta<string> = string,
  TAccountVaultNcnTicket extends string | IAccountMeta<string> = string,
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
      TAccountVaultNcnTicket extends string
        ? WritableAccount<TAccountVaultNcnTicket>
        : TAccountVaultNcnTicket,
      TAccountAdmin extends string
        ? ReadonlySignerAccount<TAccountAdmin> &
            IAccountSignerMeta<TAccountAdmin>
        : TAccountAdmin,
      ...TRemainingAccounts,
    ]
  >;

export type CooldownVaultNcnTicketInstructionData = { discriminator: number };

export type CooldownVaultNcnTicketInstructionDataArgs = {};

export function getCooldownVaultNcnTicketInstructionDataEncoder(): Encoder<CooldownVaultNcnTicketInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({
      ...value,
      discriminator: COOLDOWN_VAULT_NCN_TICKET_DISCRIMINATOR,
    })
  );
}

export function getCooldownVaultNcnTicketInstructionDataDecoder(): Decoder<CooldownVaultNcnTicketInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getCooldownVaultNcnTicketInstructionDataCodec(): Codec<
  CooldownVaultNcnTicketInstructionDataArgs,
  CooldownVaultNcnTicketInstructionData
> {
  return combineCodec(
    getCooldownVaultNcnTicketInstructionDataEncoder(),
    getCooldownVaultNcnTicketInstructionDataDecoder()
  );
}

export type CooldownVaultNcnTicketInput<
  TAccountConfig extends string = string,
  TAccountVault extends string = string,
  TAccountNcn extends string = string,
  TAccountVaultNcnTicket extends string = string,
  TAccountAdmin extends string = string,
> = {
  config: Address<TAccountConfig>;
  vault: Address<TAccountVault>;
  ncn: Address<TAccountNcn>;
  vaultNcnTicket: Address<TAccountVaultNcnTicket>;
  admin: TransactionSigner<TAccountAdmin>;
};

export function getCooldownVaultNcnTicketInstruction<
  TAccountConfig extends string,
  TAccountVault extends string,
  TAccountNcn extends string,
  TAccountVaultNcnTicket extends string,
  TAccountAdmin extends string,
>(
  input: CooldownVaultNcnTicketInput<
    TAccountConfig,
    TAccountVault,
    TAccountNcn,
    TAccountVaultNcnTicket,
    TAccountAdmin
  >
): CooldownVaultNcnTicketInstruction<
  typeof JITO_VAULT_PROGRAM_PROGRAM_ADDRESS,
  TAccountConfig,
  TAccountVault,
  TAccountNcn,
  TAccountVaultNcnTicket,
  TAccountAdmin
> {
  // Program address.
  const programAddress = JITO_VAULT_PROGRAM_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    config: { value: input.config ?? null, isWritable: false },
    vault: { value: input.vault ?? null, isWritable: false },
    ncn: { value: input.ncn ?? null, isWritable: false },
    vaultNcnTicket: { value: input.vaultNcnTicket ?? null, isWritable: true },
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
      getAccountMeta(accounts.vaultNcnTicket),
      getAccountMeta(accounts.admin),
    ],
    programAddress,
    data: getCooldownVaultNcnTicketInstructionDataEncoder().encode({}),
  } as CooldownVaultNcnTicketInstruction<
    typeof JITO_VAULT_PROGRAM_PROGRAM_ADDRESS,
    TAccountConfig,
    TAccountVault,
    TAccountNcn,
    TAccountVaultNcnTicket,
    TAccountAdmin
  >;

  return instruction;
}

export type ParsedCooldownVaultNcnTicketInstruction<
  TProgram extends string = typeof JITO_VAULT_PROGRAM_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    config: TAccountMetas[0];
    vault: TAccountMetas[1];
    ncn: TAccountMetas[2];
    vaultNcnTicket: TAccountMetas[3];
    admin: TAccountMetas[4];
  };
  data: CooldownVaultNcnTicketInstructionData;
};

export function parseCooldownVaultNcnTicketInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedCooldownVaultNcnTicketInstruction<TProgram, TAccountMetas> {
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
      vault: getNextAccount(),
      ncn: getNextAccount(),
      vaultNcnTicket: getNextAccount(),
      admin: getNextAccount(),
    },
    data: getCooldownVaultNcnTicketInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
