/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */
import { type Address, type Codec, type Decoder, type Encoder, type IAccountMeta, type IAccountSignerMeta, type IInstruction, type IInstructionWithAccounts, type IInstructionWithData, type ReadonlySignerAccount, type TransactionSigner, type WritableAccount } from '@solana/web3.js';
import { JITO_RESTAKING_PROGRAM_ADDRESS } from '../programs';
export declare const NCN_SET_ADMIN_DISCRIMINATOR = 17;
export declare function getNcnSetAdminDiscriminatorBytes(): import("@solana/web3.js").ReadonlyUint8Array;
export type NcnSetAdminInstruction<TProgram extends string = typeof JITO_RESTAKING_PROGRAM_ADDRESS, TAccountNcn extends string | IAccountMeta<string> = string, TAccountOldAdmin extends string | IAccountMeta<string> = string, TAccountNewAdmin extends string | IAccountMeta<string> = string, TRemainingAccounts extends readonly IAccountMeta<string>[] = []> = IInstruction<TProgram> & IInstructionWithData<Uint8Array> & IInstructionWithAccounts<[
    TAccountNcn extends string ? WritableAccount<TAccountNcn> : TAccountNcn,
    TAccountOldAdmin extends string ? ReadonlySignerAccount<TAccountOldAdmin> & IAccountSignerMeta<TAccountOldAdmin> : TAccountOldAdmin,
    TAccountNewAdmin extends string ? ReadonlySignerAccount<TAccountNewAdmin> & IAccountSignerMeta<TAccountNewAdmin> : TAccountNewAdmin,
    ...TRemainingAccounts
]>;
export type NcnSetAdminInstructionData = {
    discriminator: number;
};
export type NcnSetAdminInstructionDataArgs = {};
export declare function getNcnSetAdminInstructionDataEncoder(): Encoder<NcnSetAdminInstructionDataArgs>;
export declare function getNcnSetAdminInstructionDataDecoder(): Decoder<NcnSetAdminInstructionData>;
export declare function getNcnSetAdminInstructionDataCodec(): Codec<NcnSetAdminInstructionDataArgs, NcnSetAdminInstructionData>;
export type NcnSetAdminInput<TAccountNcn extends string = string, TAccountOldAdmin extends string = string, TAccountNewAdmin extends string = string> = {
    ncn: Address<TAccountNcn>;
    oldAdmin: TransactionSigner<TAccountOldAdmin>;
    newAdmin: TransactionSigner<TAccountNewAdmin>;
};
export declare function getNcnSetAdminInstruction<TAccountNcn extends string, TAccountOldAdmin extends string, TAccountNewAdmin extends string>(input: NcnSetAdminInput<TAccountNcn, TAccountOldAdmin, TAccountNewAdmin>): NcnSetAdminInstruction<typeof JITO_RESTAKING_PROGRAM_ADDRESS, TAccountNcn, TAccountOldAdmin, TAccountNewAdmin>;
export type ParsedNcnSetAdminInstruction<TProgram extends string = typeof JITO_RESTAKING_PROGRAM_ADDRESS, TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[]> = {
    programAddress: Address<TProgram>;
    accounts: {
        ncn: TAccountMetas[0];
        oldAdmin: TAccountMetas[1];
        newAdmin: TAccountMetas[2];
    };
    data: NcnSetAdminInstructionData;
};
export declare function parseNcnSetAdminInstruction<TProgram extends string, TAccountMetas extends readonly IAccountMeta[]>(instruction: IInstruction<TProgram> & IInstructionWithAccounts<TAccountMetas> & IInstructionWithData<Uint8Array>): ParsedNcnSetAdminInstruction<TProgram, TAccountMetas>;
