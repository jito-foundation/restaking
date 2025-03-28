const fs = require('fs');
const path = require('path');

/**
 * Updates @solana/web3.js to @solana/kit in code files
 * 
 * @param {string} filePath - Path to the file to update
 */
function updateSolanaImports(filePath) {
  console.log(`Processing file: ${filePath}`);
  
  try {
    // Read the file content
    let content = fs.readFileSync(filePath, 'utf8');
    
    // Check if the file contains @solana/web3.js imports
    if (content.includes('@solana/web3.js')) {
      // Replace import statements
      const updatedContent = content.replace(/@solana\/web3\.js/g, '@solana/kit');
      
      // Write the updated content back to the file
      fs.writeFileSync(filePath, updatedContent, 'utf8');
      console.log(`✅ Updated imports in: ${filePath}`);
      return true;
    } else {
      console.log(`❗ No @solana/web3.js imports found in: ${filePath}`);
      return false;
    }
  } catch (error) {
    console.error(`❌ Error processing file ${filePath}:`, error);
    return false;
  }
}

/**
 * Updates imports in all JS/TS files in a directory (recursive)
 * 
 * @param {string} directory - Directory containing the files to update
 * @returns {number} - Number of files updated
 */
function updateImportsInDirectory(directory) {
  console.log(`\nScanning directory: ${directory}`);
  
  let updatedFilesCount = 0;
  
  // Get all files in the directory recursively
  const getAllFiles = function(dirPath, arrayOfFiles = []) {
    try {
      const files = fs.readdirSync(dirPath);
      
      files.forEach(file => {
        const filePath = path.join(dirPath, file);
        const stats = fs.statSync(filePath);
        
        if (stats.isDirectory()) {
          arrayOfFiles = getAllFiles(filePath, arrayOfFiles);
        } else if (filePath.endsWith('.js') || filePath.endsWith('.ts')) {
          arrayOfFiles.push(filePath);
        }
      });
    } catch (error) {
      console.error(`Error reading directory ${dirPath}:`, error);
    }
    
    return arrayOfFiles;
  };
  
  const files = getAllFiles(directory);
  console.log(`Found ${files.length} JavaScript/TypeScript files`);
  
  files.forEach(filePath => {
    if (updateSolanaImports(filePath)) {
      updatedFilesCount++;
    }
  });
  
  return updatedFilesCount;
}

// Main function to update both client directories
function updateClientDirectories() {
  // Get the project root directory (assuming this script is at the project root)
  const projectRoot = process.cwd();
  
  // Define the client directories to update
  const clientDirs = [
    path.join(projectRoot, 'clients', 'js', 'restaking_client', 'src'),
    path.join(projectRoot, 'clients', 'js', 'vault_client', 'src')
  ];
  
  let totalUpdatedFiles = 0;
  
  // Process each client directory
  clientDirs.forEach(dir => {
    console.log(`\n===== Processing directory: ${dir} =====`);
    
    // Check if directory exists
    if (!fs.existsSync(dir)) {
      console.error(`❌ Directory does not exist: ${dir}`);
      return;
    }
    
    // Update imports in the directory
    const updatedCount = updateImportsInDirectory(dir);
    totalUpdatedFiles += updatedCount;
    
    console.log(`Updated ${updatedCount} files in ${dir}`);
  });
  
  console.log(`\n✅ Total files updated: ${totalUpdatedFiles}`);
}

// Run the update
updateClientDirectories();