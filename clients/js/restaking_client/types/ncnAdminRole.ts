/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  combineCodec,
  getEnumDecoder,
  getEnumEncoder,
  type Codec,
  type Decoder,
  type Encoder,
} from '@solana/web3.js';

export enum NcnAdminRole {
  OperatorAdmin,
  VaultAdmin,
  SlasherAdmin,
  DelegateAdmin,
  MetadataAdmin,
}

export type NcnAdminRoleArgs = NcnAdminRole;

export function getNcnAdminRoleEncoder(): Encoder<NcnAdminRoleArgs> {
  return getEnumEncoder(NcnAdminRole);
}

export function getNcnAdminRoleDecoder(): Decoder<NcnAdminRole> {
  return getEnumDecoder(NcnAdminRole);
}

export function getNcnAdminRoleCodec(): Codec<NcnAdminRoleArgs, NcnAdminRole> {
  return combineCodec(getNcnAdminRoleEncoder(), getNcnAdminRoleDecoder());
}
