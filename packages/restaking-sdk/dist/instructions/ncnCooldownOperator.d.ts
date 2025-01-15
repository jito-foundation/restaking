/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */
import { type Address, type Codec, type Decoder, type Encoder, type IAccountMeta, type IAccountSignerMeta, type IInstruction, type IInstructionWithAccounts, type IInstructionWithData, type ReadonlyAccount, type ReadonlySignerAccount, type TransactionSigner, type WritableAccount } from '@solana/web3.js';
import { JITO_RESTAKING_PROGRAM_ADDRESS } from '../programs';
export declare const NCN_COOLDOWN_OPERATOR_DISCRIMINATOR = 10;
export declare function getNcnCooldownOperatorDiscriminatorBytes(): import("@solana/web3.js").ReadonlyUint8Array;
export type NcnCooldownOperatorInstruction<TProgram extends string = typeof JITO_RESTAKING_PROGRAM_ADDRESS, TAccountConfig extends string | IAccountMeta<string> = string, TAccountNcn extends string | IAccountMeta<string> = string, TAccountOperator extends string | IAccountMeta<string> = string, TAccountNcnOperatorState extends string | IAccountMeta<string> = string, TAccountAdmin extends string | IAccountMeta<string> = string, TRemainingAccounts extends readonly IAccountMeta<string>[] = []> = IInstruction<TProgram> & IInstructionWithData<Uint8Array> & IInstructionWithAccounts<[
    TAccountConfig extends string ? ReadonlyAccount<TAccountConfig> : TAccountConfig,
    TAccountNcn extends string ? ReadonlyAccount<TAccountNcn> : TAccountNcn,
    TAccountOperator extends string ? ReadonlyAccount<TAccountOperator> : TAccountOperator,
    TAccountNcnOperatorState extends string ? WritableAccount<TAccountNcnOperatorState> : TAccountNcnOperatorState,
    TAccountAdmin extends string ? ReadonlySignerAccount<TAccountAdmin> & IAccountSignerMeta<TAccountAdmin> : TAccountAdmin,
    ...TRemainingAccounts
]>;
export type NcnCooldownOperatorInstructionData = {
    discriminator: number;
};
export type NcnCooldownOperatorInstructionDataArgs = {};
export declare function getNcnCooldownOperatorInstructionDataEncoder(): Encoder<NcnCooldownOperatorInstructionDataArgs>;
export declare function getNcnCooldownOperatorInstructionDataDecoder(): Decoder<NcnCooldownOperatorInstructionData>;
export declare function getNcnCooldownOperatorInstructionDataCodec(): Codec<NcnCooldownOperatorInstructionDataArgs, NcnCooldownOperatorInstructionData>;
export type NcnCooldownOperatorInput<TAccountConfig extends string = string, TAccountNcn extends string = string, TAccountOperator extends string = string, TAccountNcnOperatorState extends string = string, TAccountAdmin extends string = string> = {
    config: Address<TAccountConfig>;
    ncn: Address<TAccountNcn>;
    operator: Address<TAccountOperator>;
    ncnOperatorState: Address<TAccountNcnOperatorState>;
    admin: TransactionSigner<TAccountAdmin>;
};
export declare function getNcnCooldownOperatorInstruction<TAccountConfig extends string, TAccountNcn extends string, TAccountOperator extends string, TAccountNcnOperatorState extends string, TAccountAdmin extends string>(input: NcnCooldownOperatorInput<TAccountConfig, TAccountNcn, TAccountOperator, TAccountNcnOperatorState, TAccountAdmin>): NcnCooldownOperatorInstruction<typeof JITO_RESTAKING_PROGRAM_ADDRESS, TAccountConfig, TAccountNcn, TAccountOperator, TAccountNcnOperatorState, TAccountAdmin>;
export type ParsedNcnCooldownOperatorInstruction<TProgram extends string = typeof JITO_RESTAKING_PROGRAM_ADDRESS, TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[]> = {
    programAddress: Address<TProgram>;
    accounts: {
        config: TAccountMetas[0];
        ncn: TAccountMetas[1];
        operator: TAccountMetas[2];
        ncnOperatorState: TAccountMetas[3];
        admin: TAccountMetas[4];
    };
    data: NcnCooldownOperatorInstructionData;
};
export declare function parseNcnCooldownOperatorInstruction<TProgram extends string, TAccountMetas extends readonly IAccountMeta[]>(instruction: IInstruction<TProgram> & IInstructionWithAccounts<TAccountMetas> & IInstructionWithData<Uint8Array>): ParsedNcnCooldownOperatorInstruction<TProgram, TAccountMetas>;
