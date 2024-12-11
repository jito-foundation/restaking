const kinobi = require("codama");
const anchorIdl = require("@codama/nodes-from-anchor");
const path = require("path");
const renderers = require('@codama/renderers');

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
        // PodU64 -> u64
        select: (nodes) => {
            for (let i = 0; i < nodes.length; i++) {
                if (
                    kinobi.isNode(nodes[i], "structFieldTypeNode") &&
                    nodes[i].type.name === "podU64"
                ) {
                    return true;
                };
            }

            return false;
        },
        transform: (node) => {
            kinobi.assertIsNode(node, ["structFieldTypeNode", "definedTypeLinkNode"]);
            return {
                ...node,
                type: kinobi.numberTypeNode("u64"),
            };
        },
    },
    {
        // PodU32 -> u32
        select: (nodes) => {
            for (let i = 0; i < nodes.length; i++) {
                if (
                    kinobi.isNode(nodes[i], "structFieldTypeNode") &&
                    nodes[i].type.name === "podU32"
                ) {
                    return true;
                }
            }

            return false;
        },
        transform: (node) => {
            kinobi.assertIsNode(node, ["structFieldTypeNode", "definedTypeLinkNode"]);
            return {
                ...node,
                type: kinobi.numberTypeNode("u32"),
            };
        },
    },
    {
        // PodU16 -> u16
        select: (nodes) => {
            for (let i = 0; i < nodes.length; i++) {
                if (
                    kinobi.isNode(nodes[i], "structFieldTypeNode") &&
                    nodes[i].type.name === "podU16"
                ) {
                    return true;
                }
            }

            return false;
        },
        transform: (node) => {
            kinobi.assertIsNode(node, ["structFieldTypeNode", "definedTypeLinkNode"]);
            return {
                ...node,
                type: kinobi.numberTypeNode("u16"),
            };
        },
    },
    // add 8 byte discriminator to accountNode
    {
        select: '[accountNode]',
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
const traitOptions = {
    baseDefaults: [
        'borsh::BorshSerialize',
        'borsh::BorshDeserialize',
        'serde::Serialize',
        'serde::Deserialize',
        'Clone',
        'Debug',
        'Eq',
        'PartialEq',
    ],
    dataEnumDefaults: [],
    scalarEnumDefaults: ['Copy', 'Hash', 'num_derive::FromPrimitive', 'clap::ValueEnum'],
    structDefaults: [],
};
restakingKinobi.accept(renderers.renderRustVisitor(path.join(rustRestakingClientDir, "src", "generated"), {
    formatCode: true,
    crateFolder: rustRestakingClientDir,
    deleteFolderBeforeRendering: true,
    toolchain: "+nightly-2024-07-25",
    traitOptions,
}));
restakingKinobi.accept(renderers.renderJavaScriptVisitor(path.join(jsRestakingClientDir), {}));

// Generate the vault client in Rust and JavaScript.
const rustVaultClientDir = path.join(rustClientsDir, "vault_client");
const jsVaultClientDir = path.join(jsClientsDir, "vault_client");
const vaultRootNode = anchorIdl.rootNodeFromAnchor(require(path.join(idlDir, "jito_vault.json")));
const vaultKinobi = kinobi.createFromRoot(vaultRootNode);
vaultKinobi.update(kinobi.bottomUpTransformerVisitor([
    {
        // PodU64 -> u64
        select: (nodes) => {
            for (let i = 0; i < nodes.length; i++) {
                if (
                    kinobi.isNode(nodes[i], "structFieldTypeNode") &&
                    nodes[i].type.name === "podU64"
                ) {
                    return true;
                };
            }

            return false;
        },
        transform: (node) => {
            kinobi.assertIsNode(node, ["structFieldTypeNode", "definedTypeLinkNode"]);
            return {
                ...node,
                type: kinobi.numberTypeNode("u64"),
            };
        },
    },
    {
        // PodU32 -> u32
        select: (nodes) => {
            for (let i = 0; i < nodes.length; i++) {
                if (
                    kinobi.isNode(nodes[i], "structFieldTypeNode") &&
                    nodes[i].type.name === "podU32"
                ) {
                    return true;
                }
            }

            return false;
        },
        transform: (node) => {
            kinobi.assertIsNode(node, ["structFieldTypeNode", "definedTypeLinkNode"]);
            return {
                ...node,
                type: kinobi.numberTypeNode("u32"),
            };
        },
    },
    {
        // PodU16 -> u16
        select: (nodes) => {
            for (let i = 0; i < nodes.length; i++) {
                if (
                    kinobi.isNode(nodes[i], "structFieldTypeNode") &&
                    nodes[i].type.name === "podU16"
                ) {
                    return true;
                }
            }

            return false;
        },
        transform: (node) => {
            kinobi.assertIsNode(node, ["structFieldTypeNode", "definedTypeLinkNode"]);
            return {
                ...node,
                type: kinobi.numberTypeNode("u16"),
            };
        },
    },
    {
        // PodBool -> bool
        select: (nodes) => {
            for (let i = 0; i < nodes.length; i++) {
                if (
                    kinobi.isNode(nodes[i], "structFieldTypeNode") &&
                    nodes[i].type.name === "podBool"
                ) {
                    return true;
                }
            }

            return false;
        },
        transform: (node) => {
            kinobi.assertIsNode(node, ["structFieldTypeNode", "definedTypeLinkNode"]);
            return {
                ...node,
                type: kinobi.numberTypeNode("bool"),
            };
        },
    },
    // add 8 byte discriminator to accountNode
    {
        select: '[accountNode]',
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
    toolchain: "+nightly-2024-07-25",
    traitOptions,
}));
vaultKinobi.accept(renderers.renderJavaScriptVisitor(path.join(jsVaultClientDir), {}));
