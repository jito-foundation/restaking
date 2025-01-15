"use strict";
/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.WARMUP_OPERATOR_VAULT_TICKET_DISCRIMINATOR = void 0;
exports.getWarmupOperatorVaultTicketDiscriminatorBytes = getWarmupOperatorVaultTicketDiscriminatorBytes;
exports.getWarmupOperatorVaultTicketInstructionDataEncoder = getWarmupOperatorVaultTicketInstructionDataEncoder;
exports.getWarmupOperatorVaultTicketInstructionDataDecoder = getWarmupOperatorVaultTicketInstructionDataDecoder;
exports.getWarmupOperatorVaultTicketInstructionDataCodec = getWarmupOperatorVaultTicketInstructionDataCodec;
exports.getWarmupOperatorVaultTicketInstruction = getWarmupOperatorVaultTicketInstruction;
exports.parseWarmupOperatorVaultTicketInstruction = parseWarmupOperatorVaultTicketInstruction;
const web3_js_1 = require("@solana/web3.js");
const programs_1 = require("../programs");
const shared_1 = require("../shared");
exports.WARMUP_OPERATOR_VAULT_TICKET_DISCRIMINATOR = 15;
function getWarmupOperatorVaultTicketDiscriminatorBytes() {
    return (0, web3_js_1.getU8Encoder)().encode(exports.WARMUP_OPERATOR_VAULT_TICKET_DISCRIMINATOR);
}
function getWarmupOperatorVaultTicketInstructionDataEncoder() {
    return (0, web3_js_1.transformEncoder)((0, web3_js_1.getStructEncoder)([['discriminator', (0, web3_js_1.getU8Encoder)()]]), (value) => ({
        ...value,
        discriminator: exports.WARMUP_OPERATOR_VAULT_TICKET_DISCRIMINATOR,
    }));
}
function getWarmupOperatorVaultTicketInstructionDataDecoder() {
    return (0, web3_js_1.getStructDecoder)([['discriminator', (0, web3_js_1.getU8Decoder)()]]);
}
function getWarmupOperatorVaultTicketInstructionDataCodec() {
    return (0, web3_js_1.combineCodec)(getWarmupOperatorVaultTicketInstructionDataEncoder(), getWarmupOperatorVaultTicketInstructionDataDecoder());
}
function getWarmupOperatorVaultTicketInstruction(input) {
    // Program address.
    const programAddress = programs_1.JITO_RESTAKING_PROGRAM_ADDRESS;
    // Original accounts.
    const originalAccounts = {
        config: { value: input.config ?? null, isWritable: false },
        operator: { value: input.operator ?? null, isWritable: false },
        vault: { value: input.vault ?? null, isWritable: false },
        operatorVaultTicket: {
            value: input.operatorVaultTicket ?? null,
            isWritable: true,
        },
        admin: { value: input.admin ?? null, isWritable: false },
    };
    const accounts = originalAccounts;
    const getAccountMeta = (0, shared_1.getAccountMetaFactory)(programAddress, 'programId');
    const instruction = {
        accounts: [
            getAccountMeta(accounts.config),
            getAccountMeta(accounts.operator),
            getAccountMeta(accounts.vault),
            getAccountMeta(accounts.operatorVaultTicket),
            getAccountMeta(accounts.admin),
        ],
        programAddress,
        data: getWarmupOperatorVaultTicketInstructionDataEncoder().encode({}),
    };
    return instruction;
}
function parseWarmupOperatorVaultTicketInstruction(instruction) {
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
            operator: getNextAccount(),
            vault: getNextAccount(),
            operatorVaultTicket: getNextAccount(),
            admin: getNextAccount(),
        },
        data: getWarmupOperatorVaultTicketInstructionDataDecoder().decode(instruction.data),
    };
}
