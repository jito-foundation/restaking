"use strict";
/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.INITIALIZE_NCN_DISCRIMINATOR = void 0;
exports.getInitializeNcnDiscriminatorBytes = getInitializeNcnDiscriminatorBytes;
exports.getInitializeNcnInstructionDataEncoder = getInitializeNcnInstructionDataEncoder;
exports.getInitializeNcnInstructionDataDecoder = getInitializeNcnInstructionDataDecoder;
exports.getInitializeNcnInstructionDataCodec = getInitializeNcnInstructionDataCodec;
exports.getInitializeNcnInstruction = getInitializeNcnInstruction;
exports.parseInitializeNcnInstruction = parseInitializeNcnInstruction;
const web3_js_1 = require("@solana/web3.js");
const programs_1 = require("../programs");
const shared_1 = require("../shared");
exports.INITIALIZE_NCN_DISCRIMINATOR = 1;
function getInitializeNcnDiscriminatorBytes() {
    return (0, web3_js_1.getU8Encoder)().encode(exports.INITIALIZE_NCN_DISCRIMINATOR);
}
function getInitializeNcnInstructionDataEncoder() {
    return (0, web3_js_1.transformEncoder)((0, web3_js_1.getStructEncoder)([['discriminator', (0, web3_js_1.getU8Encoder)()]]), (value) => ({ ...value, discriminator: exports.INITIALIZE_NCN_DISCRIMINATOR }));
}
function getInitializeNcnInstructionDataDecoder() {
    return (0, web3_js_1.getStructDecoder)([['discriminator', (0, web3_js_1.getU8Decoder)()]]);
}
function getInitializeNcnInstructionDataCodec() {
    return (0, web3_js_1.combineCodec)(getInitializeNcnInstructionDataEncoder(), getInitializeNcnInstructionDataDecoder());
}
function getInitializeNcnInstruction(input) {
    // Program address.
    const programAddress = programs_1.JITO_RESTAKING_PROGRAM_ADDRESS;
    // Original accounts.
    const originalAccounts = {
        config: { value: input.config ?? null, isWritable: true },
        ncn: { value: input.ncn ?? null, isWritable: true },
        admin: { value: input.admin ?? null, isWritable: true },
        base: { value: input.base ?? null, isWritable: false },
        systemProgram: { value: input.systemProgram ?? null, isWritable: false },
    };
    const accounts = originalAccounts;
    // Resolve default values.
    if (!accounts.systemProgram.value) {
        accounts.systemProgram.value =
            '11111111111111111111111111111111';
    }
    const getAccountMeta = (0, shared_1.getAccountMetaFactory)(programAddress, 'programId');
    const instruction = {
        accounts: [
            getAccountMeta(accounts.config),
            getAccountMeta(accounts.ncn),
            getAccountMeta(accounts.admin),
            getAccountMeta(accounts.base),
            getAccountMeta(accounts.systemProgram),
        ],
        programAddress,
        data: getInitializeNcnInstructionDataEncoder().encode({}),
    };
    return instruction;
}
function parseInitializeNcnInstruction(instruction) {
    if (instruction.accounts.length < 5) {
        // TODO: Coded error.
        throw new Error('Not enough accounts');
    }
    let accountIndex = 0;
    const getNextAccount = () => {
        const accountMeta = instruction.accounts[accountIndex];
        accountIndex += 1;
        return accountMeta;
    };
    return {
        programAddress: instruction.programAddress,
        accounts: {
            config: getNextAccount(),
            ncn: getNextAccount(),
            admin: getNextAccount(),
            base: getNextAccount(),
            systemProgram: getNextAccount(),
        },
        data: getInitializeNcnInstructionDataDecoder().decode(instruction.data),
    };
}
