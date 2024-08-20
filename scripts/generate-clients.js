const kinobi = require("kinobi");
const anchorIdl = require("@kinobi-so/nodes-from-anchor");
const path = require("path");
const renderers = require('@kinobi-so/renderers');

// Paths.
const projectRoot = path.join(__dirname, "..");

const idlDir = path.join(projectRoot, "idl");

const rustRestakingModule = path.join(__dirname, "..", "restaking_client");
const rustRestakingGeneratedDir = path.join(rustRestakingModule, "src", "generated");

const restaking_idl = require(path.join(idlDir, "jito_restaking_sdk.json"));
const restaking_program = anchorIdl.rootNodeFromAnchor(restaking_idl);
const restaking_kinobi = kinobi.createFromRoot(restaking_program)

restaking_kinobi.update(kinobi.updateProgramsVisitor({
    jitoRestakingSdk: {name: "jito_restaking_program"},
}))

restaking_kinobi.accept(renderers.renderRustVisitor(path.join(rustRestakingGeneratedDir), {
    formatCode: true, crateFolder: rustRestakingModule, deleteFolderBeforeRendering: true
}));