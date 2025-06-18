const kinobi = require("kinobi");
const anchorIdl = require("@exo-tech-xyz/nodes-from-anchor");
const path = require("path");
const renderers = require('@exo-tech-xyz/renderers');

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
restakingKinobi.update(kinobi.bottomUpTransformerVisitor([
    {
        // PodU128 -> u128
        select: (node) => {
            return (
                kinobi.isNode(node, "structFieldTypeNode") &&
                node.type.name === "podU128"
            );
        },
        transform: (node) => {
            kinobi.assertIsNode(node, "structFieldTypeNode");
            return {
                ...node,
                type: kinobi.numberTypeNode("u128"),
            };
        },
    },
    {
        // PodU64 -> u64
        select: (node) => {
            return (
                kinobi.isNode(node, "structFieldTypeNode") &&
                node.type.name === "podU64"
            );
        },
        transform: (node) => {
            kinobi.assertIsNode(node, "structFieldTypeNode");
            return {
                ...node,
                type: kinobi.numberTypeNode("u64"),
            };
        },
    },
    {
        // PodU32 -> u32
        select: (node) => {
            return (
                kinobi.isNode(node, "structFieldTypeNode") &&
                node.type.name === "podU32"
            );
        },
        transform: (node) => {
            kinobi.assertIsNode(node, "structFieldTypeNode");
            return {
                ...node,
                type: kinobi.numberTypeNode("u32"),
            };
        },
    },
    {
        // PodU16 -> u16
        select: (node) => {
            return (
                kinobi.isNode(node, "structFieldTypeNode") &&
                node.type.name === "podU16"
            );
        },
        transform: (node) => {
            kinobi.assertIsNode(node, "structFieldTypeNode");
            return {
                ...node,
                type: kinobi.numberTypeNode("u16"),
            };
        },
    },
    // add 8 byte discriminator to accountNode
    {
        select: (node) => {
            return (
                kinobi.isNode(node, "accountNode")
            );
        },
        transform: (node) => {
            kinobi.assertIsNode(node, "accountNode");

            return {
                ...node,
                data: {
                    ...node.data,
                    fields: [
                        kinobi.structFieldTypeNode({ name: 'discriminator', type: kinobi.numberTypeNode('u64') }),
                        ...node.data.fields
                    ]
                }
            };
        },
    },
]));
restakingKinobi.accept(renderers.renderRustVisitor(path.join(rustRestakingClientDir, "src", "generated"), {
    formatCode: true,
    crateFolder: rustRestakingClientDir,
    deleteFolderBeforeRendering: true,
    toolchain: "+1.84.1"
}));
restakingKinobi.accept(renderers.renderJavaScriptVisitor(path.join(jsRestakingClientDir), {}));

// Generate the vault client in Rust and JavaScript.
const rustVaultClientDir = path.join(rustClientsDir, "vault_client");
const jsVaultClientDir = path.join(jsClientsDir, "vault_client");
const vaultRootNode = anchorIdl.rootNodeFromAnchor(require(path.join(idlDir, "jito_vault.json")));
const vaultKinobi = kinobi.createFromRoot(vaultRootNode);
vaultKinobi.update(kinobi.bottomUpTransformerVisitor([
    {
        // PodU128 -> u128
        select: (node) => {
            return (
                kinobi.isNode(node, "structFieldTypeNode") &&
                node.type.name === "podU128"
            );
        },
        transform: (node) => {
            kinobi.assertIsNode(node, "structFieldTypeNode");
            return {
                ...node,
                type: kinobi.numberTypeNode("u128"),
            };
        },
    },
    {
        // PodU64 -> u64
        select: (node) => {
            return (
                kinobi.isNode(node, "structFieldTypeNode") &&
                node.type.name === "podU64"
            );
        },
        transform: (node) => {
            kinobi.assertIsNode(node, "structFieldTypeNode");
            return {
                ...node,
                type: kinobi.numberTypeNode("u64"),
            };
        },
    },
    {
        // PodU32 -> u32
        select: (node) => {
            return (
                kinobi.isNode(node, "structFieldTypeNode") &&
                node.type.name === "podU32"
            );
        },
        transform: (node) => {
            kinobi.assertIsNode(node, "structFieldTypeNode");
            return {
                ...node,
                type: kinobi.numberTypeNode("u32"),
            };
        },
    },
    {
        // PodU16 -> u16
        select: (node) => {
            return (
                kinobi.isNode(node, "structFieldTypeNode") &&
                node.type.name === "podU16"
            );
        },
        transform: (node) => {
            kinobi.assertIsNode(node, "structFieldTypeNode");
            return {
                ...node,
                type: kinobi.numberTypeNode("u16"),
            };
        },
    },
    {
        // PodBool -> bool
        select: (node) => {
            return (
                kinobi.isNode(node, "structFieldTypeNode") &&
                node.type.name === "podBool"
            );
        },
        transform: (node) => {
            kinobi.assertIsNode(node, "structFieldTypeNode");
            return {
                ...node,
                type: kinobi.booleanTypeNode(),
            };
        },
    },
    {
        select: (node) => {
            return (
                kinobi.isNode(node, "accountNode")
            );
        },
        transform: (node) => {
            kinobi.assertIsNode(node, "accountNode");

            return {
                ...node,
                data: {
                    ...node.data,
                    fields: [
                        kinobi.structFieldTypeNode({ name: 'discriminator', type: kinobi.numberTypeNode('u64') }),
                        ...node.data.fields
                    ]
                }
            };
        },
    },
]));
vaultKinobi.accept(renderers.renderRustVisitor(path.join(rustVaultClientDir, "src", "generated"), {
    formatCode: true,
    crateFolder: rustVaultClientDir,
    deleteFolderBeforeRendering: true,
    toolchain: "+1.84.1"
}));
vaultKinobi.accept(renderers.renderJavaScriptVisitor(path.join(jsVaultClientDir), {}));
