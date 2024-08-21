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
import {
  getPodU64Decoder,
  getPodU64Encoder,
  getSlotToggleDecoder,
  getSlotToggleEncoder,
  type PodU64,
  type PodU64Args,
  type SlotToggle,
  type SlotToggleArgs,
} from '../types';

export type VaultNcnSlasherTicket = {
  vault: Address;
  ncn: Address;
  slasher: Address;
  maxSlashablePerEpoch: PodU64;
  index: PodU64;
  state: SlotToggle;
  bump: number;
  reserved: Array<number>;
};

export type VaultNcnSlasherTicketArgs = {
  vault: Address;
  ncn: Address;
  slasher: Address;
  maxSlashablePerEpoch: PodU64Args;
  index: PodU64Args;
  state: SlotToggleArgs;
  bump: number;
  reserved: Array<number>;
};

export function getVaultNcnSlasherTicketEncoder(): Encoder<VaultNcnSlasherTicketArgs> {
  return getStructEncoder([
    ['vault', getAddressEncoder()],
    ['ncn', getAddressEncoder()],
    ['slasher', getAddressEncoder()],
    ['maxSlashablePerEpoch', getPodU64Encoder()],
    ['index', getPodU64Encoder()],
    ['state', getSlotToggleEncoder()],
    ['bump', getU8Encoder()],
    ['reserved', getArrayEncoder(getU8Encoder(), { size: 7 })],
  ]);
}

export function getVaultNcnSlasherTicketDecoder(): Decoder<VaultNcnSlasherTicket> {
  return getStructDecoder([
    ['vault', getAddressDecoder()],
    ['ncn', getAddressDecoder()],
    ['slasher', getAddressDecoder()],
    ['maxSlashablePerEpoch', getPodU64Decoder()],
    ['index', getPodU64Decoder()],
    ['state', getSlotToggleDecoder()],
    ['bump', getU8Decoder()],
    ['reserved', getArrayDecoder(getU8Decoder(), { size: 7 })],
  ]);
}

export function getVaultNcnSlasherTicketCodec(): Codec<
  VaultNcnSlasherTicketArgs,
  VaultNcnSlasherTicket
> {
  return combineCodec(
    getVaultNcnSlasherTicketEncoder(),
    getVaultNcnSlasherTicketDecoder()
  );
}

export function decodeVaultNcnSlasherTicket<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress>
): Account<VaultNcnSlasherTicket, TAddress>;
export function decodeVaultNcnSlasherTicket<TAddress extends string = string>(
  encodedAccount: MaybeEncodedAccount<TAddress>
): MaybeAccount<VaultNcnSlasherTicket, TAddress>;
export function decodeVaultNcnSlasherTicket<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress> | MaybeEncodedAccount<TAddress>
):
  | Account<VaultNcnSlasherTicket, TAddress>
  | MaybeAccount<VaultNcnSlasherTicket, TAddress> {
  return decodeAccount(
    encodedAccount as MaybeEncodedAccount<TAddress>,
    getVaultNcnSlasherTicketDecoder()
  );
}

export async function fetchVaultNcnSlasherTicket<
  TAddress extends string = string,
>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<Account<VaultNcnSlasherTicket, TAddress>> {
  const maybeAccount = await fetchMaybeVaultNcnSlasherTicket(
    rpc,
    address,
    config
  );
  assertAccountExists(maybeAccount);
  return maybeAccount;
}

export async function fetchMaybeVaultNcnSlasherTicket<
  TAddress extends string = string,
>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<MaybeAccount<VaultNcnSlasherTicket, TAddress>> {
  const maybeAccount = await fetchEncodedAccount(rpc, address, config);
  return decodeVaultNcnSlasherTicket(maybeAccount);
}

export async function fetchAllVaultNcnSlasherTicket(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<Account<VaultNcnSlasherTicket>[]> {
  const maybeAccounts = await fetchAllMaybeVaultNcnSlasherTicket(
    rpc,
    addresses,
    config
  );
  assertAccountsExist(maybeAccounts);
  return maybeAccounts;
}

export async function fetchAllMaybeVaultNcnSlasherTicket(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<MaybeAccount<VaultNcnSlasherTicket>[]> {
  const maybeAccounts = await fetchEncodedAccounts(rpc, addresses, config);
  return maybeAccounts.map((maybeAccount) =>
    decodeVaultNcnSlasherTicket(maybeAccount)
  );
}

export function getVaultNcnSlasherTicketSize(): number {
  return 136;
}
