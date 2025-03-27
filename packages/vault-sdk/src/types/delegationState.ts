/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  combineCodec,
  getArrayDecoder,
  getArrayEncoder,
  getStructDecoder,
  getStructEncoder,
  getU64Decoder,
  getU64Encoder,
  getU8Decoder,
  getU8Encoder,
  type Codec,
  type Decoder,
  type Encoder,
} from '@solana/kit';

export type DelegationState = {
  stakedAmount: bigint;
  enqueuedForCooldownAmount: bigint;
  coolingDownAmount: bigint;
  reserved: Array<number>;
};

export type DelegationStateArgs = {
  stakedAmount: number | bigint;
  enqueuedForCooldownAmount: number | bigint;
  coolingDownAmount: number | bigint;
  reserved: Array<number>;
};

export function getDelegationStateEncoder(): Encoder<DelegationStateArgs> {
  return getStructEncoder([
    ['stakedAmount', getU64Encoder()],
    ['enqueuedForCooldownAmount', getU64Encoder()],
    ['coolingDownAmount', getU64Encoder()],
    ['reserved', getArrayEncoder(getU8Encoder(), { size: 256 })],
  ]);
}

export function getDelegationStateDecoder(): Decoder<DelegationState> {
  return getStructDecoder([
    ['stakedAmount', getU64Decoder()],
    ['enqueuedForCooldownAmount', getU64Decoder()],
    ['coolingDownAmount', getU64Decoder()],
    ['reserved', getArrayDecoder(getU8Decoder(), { size: 256 })],
  ]);
}

export function getDelegationStateCodec(): Codec<
  DelegationStateArgs,
  DelegationState
> {
  return combineCodec(getDelegationStateEncoder(), getDelegationStateDecoder());
}
