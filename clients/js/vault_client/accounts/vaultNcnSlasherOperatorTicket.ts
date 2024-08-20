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
} from '@solana/web3.js';

export type VaultNcnSlasherOperatorTicket = {
  vault: Address;
  ncn: Address;
  slasher: Address;
  operator: Address;
  epoch: bigint;
  slashed: bigint;
  bump: number;
  reserved: Array<number>;
};

export type VaultNcnSlasherOperatorTicketArgs = {
  vault: Address;
  ncn: Address;
  slasher: Address;
  operator: Address;
  epoch: number | bigint;
  slashed: number | bigint;
  bump: number;
  reserved: Array<number>;
};

export function getVaultNcnSlasherOperatorTicketEncoder(): Encoder<VaultNcnSlasherOperatorTicketArgs> {
  return getStructEncoder([
    ['vault', getAddressEncoder()],
    ['ncn', getAddressEncoder()],
    ['slasher', getAddressEncoder()],
    ['operator', getAddressEncoder()],
    ['epoch', getU64Encoder()],
    ['slashed', getU64Encoder()],
    ['bump', getU8Encoder()],
    ['reserved', getArrayEncoder(getU8Encoder(), { size: 7 })],
  ]);
}

export function getVaultNcnSlasherOperatorTicketDecoder(): Decoder<VaultNcnSlasherOperatorTicket> {
  return getStructDecoder([
    ['vault', getAddressDecoder()],
    ['ncn', getAddressDecoder()],
    ['slasher', getAddressDecoder()],
    ['operator', getAddressDecoder()],
    ['epoch', getU64Decoder()],
    ['slashed', getU64Decoder()],
    ['bump', getU8Decoder()],
    ['reserved', getArrayDecoder(getU8Decoder(), { size: 7 })],
  ]);
}

export function getVaultNcnSlasherOperatorTicketCodec(): Codec<
  VaultNcnSlasherOperatorTicketArgs,
  VaultNcnSlasherOperatorTicket
> {
  return combineCodec(
    getVaultNcnSlasherOperatorTicketEncoder(),
    getVaultNcnSlasherOperatorTicketDecoder()
  );
}

export function decodeVaultNcnSlasherOperatorTicket<
  TAddress extends string = string,
>(
  encodedAccount: EncodedAccount<TAddress>
): Account<VaultNcnSlasherOperatorTicket, TAddress>;
export function decodeVaultNcnSlasherOperatorTicket<
  TAddress extends string = string,
>(
  encodedAccount: MaybeEncodedAccount<TAddress>
): MaybeAccount<VaultNcnSlasherOperatorTicket, TAddress>;
export function decodeVaultNcnSlasherOperatorTicket<
  TAddress extends string = string,
>(
  encodedAccount: EncodedAccount<TAddress> | MaybeEncodedAccount<TAddress>
):
  | Account<VaultNcnSlasherOperatorTicket, TAddress>
  | MaybeAccount<VaultNcnSlasherOperatorTicket, TAddress> {
  return decodeAccount(
    encodedAccount as MaybeEncodedAccount<TAddress>,
    getVaultNcnSlasherOperatorTicketDecoder()
  );
}

export async function fetchVaultNcnSlasherOperatorTicket<
  TAddress extends string = string,
>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<Account<VaultNcnSlasherOperatorTicket, TAddress>> {
  const maybeAccount = await fetchMaybeVaultNcnSlasherOperatorTicket(
    rpc,
    address,
    config
  );
  assertAccountExists(maybeAccount);
  return maybeAccount;
}

export async function fetchMaybeVaultNcnSlasherOperatorTicket<
  TAddress extends string = string,
>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<MaybeAccount<VaultNcnSlasherOperatorTicket, TAddress>> {
  const maybeAccount = await fetchEncodedAccount(rpc, address, config);
  return decodeVaultNcnSlasherOperatorTicket(maybeAccount);
}

export async function fetchAllVaultNcnSlasherOperatorTicket(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<Account<VaultNcnSlasherOperatorTicket>[]> {
  const maybeAccounts = await fetchAllMaybeVaultNcnSlasherOperatorTicket(
    rpc,
    addresses,
    config
  );
  assertAccountsExist(maybeAccounts);
  return maybeAccounts;
}

export async function fetchAllMaybeVaultNcnSlasherOperatorTicket(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<MaybeAccount<VaultNcnSlasherOperatorTicket>[]> {
  const maybeAccounts = await fetchEncodedAccounts(rpc, addresses, config);
  return maybeAccounts.map((maybeAccount) =>
    decodeVaultNcnSlasherOperatorTicket(maybeAccount)
  );
}

export function getVaultNcnSlasherOperatorTicketSize(): number {
  return 152;
}
