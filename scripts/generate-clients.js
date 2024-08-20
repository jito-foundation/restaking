const kinobi = require("kinobi");
const anchorIdl = require("@kinobi-so/nodes-from-anchor");
const path = require("path");
const renderers = require('@kinobi-so/renderers');

// Paths.
const projectRoot = path.join(__dirname, "..");

const idlDir = path.join(projectRoot, "idl");

const rustClientsDir = path.join(__dirname, "..", "clients", "rust");
const jsClientsDir = path.join(__dirname, "..", "clients", "js");

const rustRestakingClientDir = path.join(rustClientsDir, "restaking_client");
const jsRestakingClientDir = path.join(jsClientsDir, "restaking_client");
const restakingRootNode = anchorIdl.rootNodeFromAnchor(require(path.join(idlDir, "jito_restaking_sdk.json")));
const restakingKinobi = kinobi.createFromRoot(restakingRootNode);
restakingKinobi.update(kinobi.updateProgramsVisitor({
    assetProgram: {name: "jito_restaking_program"},
}));
restakingKinobi.accept(renderers.renderRustVisitor(path.join(rustRestakingClientDir, "src", "generated"), {
    formatCode: true, crateFolder: rustRestakingClientDir, deleteFolderBeforeRendering: true
}));
restakingKinobi.accept(renderers.renderJavaScriptVisitor(path.join(jsRestakingClientDir), {}));
