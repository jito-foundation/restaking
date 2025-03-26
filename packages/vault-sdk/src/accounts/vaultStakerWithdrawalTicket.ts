/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  assertAccountExists,
  assertAccountsExist,
  combineCodec,
  decodeAccount,
  fetchEncodedAccount,
  fetchEncodedAccounts,
  getAddressDecoder,
  getAddressEncoder,
  getArrayDecoder,
  getArrayEncoder,
  getStructDecoder,
  getStructEncoder,
  getU64Decoder,
  getU64Encoder,
  getU8Decoder,
  getU8Encoder,
  type Account,
  type Address,
  type Codec,
  type Decoder,
  type EncodedAccount,
  type Encoder,
  type FetchAccountConfig,
  type FetchAccountsConfig,
  type MaybeAccount,
  type MaybeEncodedAccount,
} from '@solana/kit';

export type VaultStakerWithdrawalTicket = {
  discriminator: bigint;
  vault: Address;
  staker: Address;
  base: Address;
  vrtAmount: bigint;
  slotUnstaked: bigint;
  bump: number;
  reserved: Array<number>;
};

export type VaultStakerWithdrawalTicketArgs = {
  discriminator: number | bigint;
  vault: Address;
  staker: Address;
  base: Address;
  vrtAmount: number | bigint;
  slotUnstaked: number | bigint;
  bump: number;
  reserved: Array<number>;
};

export function getVaultStakerWithdrawalTicketEncoder(): Encoder<VaultStakerWithdrawalTicketArgs> {
  return getStructEncoder([
    ['discriminator', getU64Encoder()],
    ['vault', getAddressEncoder()],
    ['staker', getAddressEncoder()],
    ['base', getAddressEncoder()],
    ['vrtAmount', getU64Encoder()],
    ['slotUnstaked', getU64Encoder()],
    ['bump', getU8Encoder()],
    ['reserved', getArrayEncoder(getU8Encoder(), { size: 263 })],
  ]);
}

export function getVaultStakerWithdrawalTicketDecoder(): Decoder<VaultStakerWithdrawalTicket> {
  return getStructDecoder([
    ['discriminator', getU64Decoder()],
    ['vault', getAddressDecoder()],
    ['staker', getAddressDecoder()],
    ['base', getAddressDecoder()],
    ['vrtAmount', getU64Decoder()],
    ['slotUnstaked', getU64Decoder()],
    ['bump', getU8Decoder()],
    ['reserved', getArrayDecoder(getU8Decoder(), { size: 263 })],
  ]);
}

export function getVaultStakerWithdrawalTicketCodec(): Codec<
  VaultStakerWithdrawalTicketArgs,
  VaultStakerWithdrawalTicket
> {
  return combineCodec(
    getVaultStakerWithdrawalTicketEncoder(),
    getVaultStakerWithdrawalTicketDecoder()
  );
}

export function decodeVaultStakerWithdrawalTicket<
  TAddress extends string = string,
>(
  encodedAccount: EncodedAccount<TAddress>
): Account<VaultStakerWithdrawalTicket, TAddress>;
export function decodeVaultStakerWithdrawalTicket<
  TAddress extends string = string,
>(
  encodedAccount: MaybeEncodedAccount<TAddress>
): MaybeAccount<VaultStakerWithdrawalTicket, TAddress>;
export function decodeVaultStakerWithdrawalTicket<
  TAddress extends string = string,
>(
  encodedAccount: EncodedAccount<TAddress> | MaybeEncodedAccount<TAddress>
):
  | Account<VaultStakerWithdrawalTicket, TAddress>
  | MaybeAccount<VaultStakerWithdrawalTicket, TAddress> {
  return decodeAccount(
    encodedAccount as MaybeEncodedAccount<TAddress>,
    getVaultStakerWithdrawalTicketDecoder()
  );
}

export async function fetchVaultStakerWithdrawalTicket<
  TAddress extends string = string,
>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<Account<VaultStakerWithdrawalTicket, TAddress>> {
  const maybeAccount = await fetchMaybeVaultStakerWithdrawalTicket(
    rpc,
    address,
    config
  );
  assertAccountExists(maybeAccount);
  return maybeAccount;
}

export async function fetchMaybeVaultStakerWithdrawalTicket<
  TAddress extends string = string,
>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<MaybeAccount<VaultStakerWithdrawalTicket, TAddress>> {
  const maybeAccount = await fetchEncodedAccount(rpc, address, config);
  return decodeVaultStakerWithdrawalTicket(maybeAccount);
}

export async function fetchAllVaultStakerWithdrawalTicket(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<Account<VaultStakerWithdrawalTicket>[]> {
  const maybeAccounts = await fetchAllMaybeVaultStakerWithdrawalTicket(
    rpc,
    addresses,
    config
  );
  assertAccountsExist(maybeAccounts);
  return maybeAccounts;
}

export async function fetchAllMaybeVaultStakerWithdrawalTicket(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<MaybeAccount<VaultStakerWithdrawalTicket>[]> {
  const maybeAccounts = await fetchEncodedAccounts(rpc, addresses, config);
  return maybeAccounts.map((maybeAccount) =>
    decodeVaultStakerWithdrawalTicket(maybeAccount)
  );
}
