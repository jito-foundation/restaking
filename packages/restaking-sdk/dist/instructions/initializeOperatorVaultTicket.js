"use strict";
/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.INITIALIZE_OPERATOR_VAULT_TICKET_DISCRIMINATOR = void 0;
exports.getInitializeOperatorVaultTicketDiscriminatorBytes = getInitializeOperatorVaultTicketDiscriminatorBytes;
exports.getInitializeOperatorVaultTicketInstructionDataEncoder = getInitializeOperatorVaultTicketInstructionDataEncoder;
exports.getInitializeOperatorVaultTicketInstructionDataDecoder = getInitializeOperatorVaultTicketInstructionDataDecoder;
exports.getInitializeOperatorVaultTicketInstructionDataCodec = getInitializeOperatorVaultTicketInstructionDataCodec;
exports.getInitializeOperatorVaultTicketInstruction = getInitializeOperatorVaultTicketInstruction;
exports.parseInitializeOperatorVaultTicketInstruction = parseInitializeOperatorVaultTicketInstruction;
const web3_js_1 = require("@solana/web3.js");
const programs_1 = require("../programs");
const shared_1 = require("../shared");
exports.INITIALIZE_OPERATOR_VAULT_TICKET_DISCRIMINATOR = 5;
function getInitializeOperatorVaultTicketDiscriminatorBytes() {
    return (0, web3_js_1.getU8Encoder)().encode(exports.INITIALIZE_OPERATOR_VAULT_TICKET_DISCRIMINATOR);
}
function getInitializeOperatorVaultTicketInstructionDataEncoder() {
    return (0, web3_js_1.transformEncoder)((0, web3_js_1.getStructEncoder)([['discriminator', (0, web3_js_1.getU8Encoder)()]]), (value) => ({
        ...value,
        discriminator: exports.INITIALIZE_OPERATOR_VAULT_TICKET_DISCRIMINATOR,
    }));
}
function getInitializeOperatorVaultTicketInstructionDataDecoder() {
    return (0, web3_js_1.getStructDecoder)([['discriminator', (0, web3_js_1.getU8Decoder)()]]);
}
function getInitializeOperatorVaultTicketInstructionDataCodec() {
    return (0, web3_js_1.combineCodec)(getInitializeOperatorVaultTicketInstructionDataEncoder(), getInitializeOperatorVaultTicketInstructionDataDecoder());
}
function getInitializeOperatorVaultTicketInstruction(input) {
    // Program address.
    const programAddress = programs_1.JITO_RESTAKING_PROGRAM_ADDRESS;
    // Original accounts.
    const originalAccounts = {
        config: { value: input.config ?? null, isWritable: false },
        operator: { value: input.operator ?? null, isWritable: true },
        vault: { value: input.vault ?? null, isWritable: false },
        operatorVaultTicket: {
            value: input.operatorVaultTicket ?? null,
            isWritable: true,
        },
        admin: { value: input.admin ?? null, isWritable: false },
        payer: { value: input.payer ?? null, isWritable: true },
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
            getAccountMeta(accounts.operator),
            getAccountMeta(accounts.vault),
            getAccountMeta(accounts.operatorVaultTicket),
            getAccountMeta(accounts.admin),
            getAccountMeta(accounts.payer),
            getAccountMeta(accounts.systemProgram),
        ],
        programAddress,
        data: getInitializeOperatorVaultTicketInstructionDataEncoder().encode({}),
    };
    return instruction;
}
function parseInitializeOperatorVaultTicketInstruction(instruction) {
    if (instruction.accounts.length < 7) {
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
            payer: getNextAccount(),
            systemProgram: getNextAccount(),
        },
        data: getInitializeOperatorVaultTicketInstructionDataDecoder().decode(instruction.data),
    };
}
