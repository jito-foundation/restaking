"use strict";
/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.COOLDOWN_NCN_VAULT_TICKET_DISCRIMINATOR = void 0;
exports.getCooldownNcnVaultTicketDiscriminatorBytes = getCooldownNcnVaultTicketDiscriminatorBytes;
exports.getCooldownNcnVaultTicketInstructionDataEncoder = getCooldownNcnVaultTicketInstructionDataEncoder;
exports.getCooldownNcnVaultTicketInstructionDataDecoder = getCooldownNcnVaultTicketInstructionDataDecoder;
exports.getCooldownNcnVaultTicketInstructionDataCodec = getCooldownNcnVaultTicketInstructionDataCodec;
exports.getCooldownNcnVaultTicketInstruction = getCooldownNcnVaultTicketInstruction;
exports.parseCooldownNcnVaultTicketInstruction = parseCooldownNcnVaultTicketInstruction;
const web3_js_1 = require("@solana/web3.js");
const programs_1 = require("../programs");
const shared_1 = require("../shared");
exports.COOLDOWN_NCN_VAULT_TICKET_DISCRIMINATOR = 8;
function getCooldownNcnVaultTicketDiscriminatorBytes() {
    return (0, web3_js_1.getU8Encoder)().encode(exports.COOLDOWN_NCN_VAULT_TICKET_DISCRIMINATOR);
}
function getCooldownNcnVaultTicketInstructionDataEncoder() {
    return (0, web3_js_1.transformEncoder)((0, web3_js_1.getStructEncoder)([['discriminator', (0, web3_js_1.getU8Encoder)()]]), (value) => ({
        ...value,
        discriminator: exports.COOLDOWN_NCN_VAULT_TICKET_DISCRIMINATOR,
    }));
}
function getCooldownNcnVaultTicketInstructionDataDecoder() {
    return (0, web3_js_1.getStructDecoder)([['discriminator', (0, web3_js_1.getU8Decoder)()]]);
}
function getCooldownNcnVaultTicketInstructionDataCodec() {
    return (0, web3_js_1.combineCodec)(getCooldownNcnVaultTicketInstructionDataEncoder(), getCooldownNcnVaultTicketInstructionDataDecoder());
}
function getCooldownNcnVaultTicketInstruction(input) {
    // Program address.
    const programAddress = programs_1.JITO_RESTAKING_PROGRAM_ADDRESS;
    // Original accounts.
    const originalAccounts = {
        config: { value: input.config ?? null, isWritable: false },
        ncn: { value: input.ncn ?? null, isWritable: false },
        vault: { value: input.vault ?? null, isWritable: false },
        ncnVaultTicket: { value: input.ncnVaultTicket ?? null, isWritable: true },
        admin: { value: input.admin ?? null, isWritable: false },
    };
    const accounts = originalAccounts;
    const getAccountMeta = (0, shared_1.getAccountMetaFactory)(programAddress, 'programId');
    const instruction = {
        accounts: [
            getAccountMeta(accounts.config),
            getAccountMeta(accounts.ncn),
            getAccountMeta(accounts.vault),
            getAccountMeta(accounts.ncnVaultTicket),
            getAccountMeta(accounts.admin),
        ],
        programAddress,
        data: getCooldownNcnVaultTicketInstructionDataEncoder().encode({}),
    };
    return instruction;
}
function parseCooldownNcnVaultTicketInstruction(instruction) {
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
            vault: getNextAccount(),
            ncnVaultTicket: getNextAccount(),
            admin: getNextAccount(),
        },
        data: getCooldownNcnVaultTicketInstructionDataDecoder().decode(instruction.data),
    };
}
