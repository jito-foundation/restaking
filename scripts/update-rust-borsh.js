const fs = require('fs');
const path = require('path');

/**
 * Rewrites borsh 0.10 serialization calls to their borsh 1.x equivalent.
 *
 * `@exo-tech-xyz/renderers-rust` is pinned at 0.21.8, its only published
 * version, and still emits `value.try_to_vec()`. That inherent method was
 * removed in borsh 1.0 in favour of the free function `borsh::to_vec(&value)`.
 *
 * @param {string} contents - File contents to rewrite
 * @param {string} filePath - Path the contents came from, for error messages
 * @returns {string} - The rewritten contents
 */
function rewriteBorshCalls(contents, filePath) {
  if (!contents.includes('try_to_vec')) {
    return contents;
  }

  // The renderer breaks long chains across lines, so pull `.try_to_vec()`
  // back onto the same line as its receiver before matching. Formatting is
  // restored by `cargo fmt` at the end of code generation.
  const collapsed = contents.replace(/\r?\n\s*\.try_to_vec\(\)/g, '.try_to_vec()');

  // A receiver is a path (`args`, `self.__args`) optionally ending in a
  // constructor call (`FooInstructionData::new()`).
  const receiver =
    '[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*(?:\\(\\))?(?:\\.[A-Za-z_][A-Za-z0-9_]*)*';
  const updated = collapsed.replace(
    new RegExp(`(${receiver})\\.try_to_vec\\(\\)`, 'g'),
    'borsh::to_vec(&$1)'
  );

  if (updated.includes('try_to_vec')) {
    throw new Error(
      `unconverted try_to_vec() call remains in ${filePath}; the renderer output shape changed`
    );
  }

  return updated;
}

/**
 * Drops the rendered `PrintProgramError` impl.
 *
 * The trait was removed from `solana-program-error` in 3.0, so the impl the
 * renderer emits for every error enum no longer resolves.
 *
 * @param {string} contents - File contents to rewrite
 * @returns {string} - The rewritten contents
 */
function removePrintProgramError(contents) {
  return contents.replace(
    /\n?impl solana_program::program_error::PrintProgramError for \w+ \{[\s\S]*?\n\}\n/g,
    ''
  );
}

/**
 * Applies every post-generation rewrite to one file.
 *
 * Rewriting after generation keeps `make generate-code` idempotent, which the
 * `code_gen` CI job checks.
 *
 * @param {string} filePath - Path to the file to update
 * @returns {boolean} - Whether the file was changed
 */
function updateGeneratedFile(filePath) {
  try {
    const original = fs.readFileSync(filePath, 'utf8');
    const updated = removePrintProgramError(rewriteBorshCalls(original, filePath));

    if (updated === original) {
      return false;
    }

    fs.writeFileSync(filePath, updated, 'utf8');
    console.log(`✅ Updated: ${filePath}`);
    return true;
  } catch (error) {
    console.error(`❌ Error processing file ${filePath}:`, error);
    throw error;
  }
}

/**
 * Collects every `.rs` file under a directory, recursively.
 *
 * @param {string} dirPath - Directory to walk
 * @param {string[]} arrayOfFiles - Accumulator
 * @returns {string[]} - Absolute paths of the Rust files found
 */
function getAllRustFiles(dirPath, arrayOfFiles = []) {
  for (const entry of fs.readdirSync(dirPath)) {
    const filePath = path.join(dirPath, entry);

    if (fs.statSync(filePath).isDirectory()) {
      getAllRustFiles(filePath, arrayOfFiles);
    } else if (filePath.endsWith('.rs')) {
      arrayOfFiles.push(filePath);
    }
  }

  return arrayOfFiles;
}

// Main function to update both generated client directories
function updateGeneratedClients() {
  const projectRoot = process.cwd();

  const generatedDirs = [
    path.join(projectRoot, 'clients', 'rust', 'restaking_client', 'src', 'generated'),
    path.join(projectRoot, 'clients', 'rust', 'vault_client', 'src', 'generated'),
  ];

  let totalUpdatedFiles = 0;

  generatedDirs.forEach((dir) => {
    console.log(`\n===== Processing directory: ${dir} =====`);

    if (!fs.existsSync(dir)) {
      console.error(`❌ Directory does not exist: ${dir}`);
      return;
    }

    const files = getAllRustFiles(dir);
    console.log(`Found ${files.length} Rust files`);

    const updatedCount = files.filter(updateGeneratedFile).length;
    totalUpdatedFiles += updatedCount;

    console.log(`Updated ${updatedCount} files in ${dir}`);
  });

  console.log(`\n✅ Total files updated: ${totalUpdatedFiles}`);
}

// Run the update
updateGeneratedClients();
