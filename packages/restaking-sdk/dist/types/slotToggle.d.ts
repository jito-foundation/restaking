/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */
import { type Codec, type Decoder, type Encoder, type ReadonlyUint8Array } from '@solana/web3.js';
export type SlotToggle = {
    slotAdded: bigint;
    slotRemoved: bigint;
    reserved: ReadonlyUint8Array;
};
export type SlotToggleArgs = {
    slotAdded: number | bigint;
    slotRemoved: number | bigint;
    reserved: ReadonlyUint8Array;
};
export declare function getSlotToggleEncoder(): Encoder<SlotToggleArgs>;
export declare function getSlotToggleDecoder(): Decoder<SlotToggle>;
export declare function getSlotToggleCodec(): Codec<SlotToggleArgs, SlotToggle>;
