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

export enum ConfigAdminRole {
  FeeAdmin,
}

export type ConfigAdminRoleArgs = ConfigAdminRole;

export function getConfigAdminRoleEncoder(): Encoder<ConfigAdminRoleArgs> {
  return getEnumEncoder(ConfigAdminRole);
}

export function getConfigAdminRoleDecoder(): Decoder<ConfigAdminRole> {
  return getEnumDecoder(ConfigAdminRole);
}

export function getConfigAdminRoleCodec(): Codec<
  ConfigAdminRoleArgs,
  ConfigAdminRole
> {
  return combineCodec(getConfigAdminRoleEncoder(), getConfigAdminRoleDecoder());
}
