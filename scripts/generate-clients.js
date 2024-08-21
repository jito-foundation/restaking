const kinobi = require("kinobi");
const anchorIdl = require("@kinobi-so/nodes-from-anchor");
const path = require("path");
const renderers = require('@kinobi-so/renderers');

// Paths.
const projectRoot = path.join(__dirname, "..");

const idlDir = path.join(projectRoot, "idl");

const rustClientsDir = path.join(__dirname, "..", "clients", "rust");
const jsClientsDir = path.join(__dirname, "..", "clients", "js");

// Generate the restaking client in Rust and JavaScript.
const rustRestakingClientDir = path.join(rustClientsDir, "restaking_client");
const jsRestakingClientDir = path.join(jsClientsDir, "restaking_client");
const restakingRootNode = anchorIdl.rootNodeFromAnchor(require(path.join(idlDir, "jito_restaking.json")));
const restakingKinobi = kinobi.createFromRoot(restakingRootNode);
restakingKinobi.update(kinobi.updateProgramsVisitor({}));
restakingKinobi.accept(renderers.renderRustVisitor(path.join(rustRestakingClientDir, "src", "generated"), {
    formatCode: true,
    crateFolder: rustRestakingClientDir,
    deleteFolderBeforeRendering: true,
    toolchain: "+nightly-2024-07-25"
}));
restakingKinobi.accept(renderers.renderJavaScriptVisitor(path.join(jsRestakingClientDir), {}));

// Generate the vault client in Rust and JavaScript.
const rustVaultClientDir = path.join(rustClientsDir, "vault_client");
const jsVaultClientDir = path.join(jsClientsDir, "vault_client");
const vaultRootNode = anchorIdl.rootNodeFromAnchor(require(path.join(idlDir, "jito_vault.json")));
const vaultKinobi = kinobi.createFromRoot(vaultRootNode);
vaultKinobi.update(kinobi.updateProgramsVisitor({}));
vaultKinobi.accept(renderers.renderRustVisitor(path.join(rustVaultClientDir, "src", "generated"), {
    formatCode: true,
    crateFolder: rustVaultClientDir,
    deleteFolderBeforeRendering: true,
    toolchain: "+nightly-2024-07-25"
}));
vaultKinobi.accept(renderers.renderJavaScriptVisitor(path.join(jsVaultClientDir), {}));