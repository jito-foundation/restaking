#! /bin/zsh

# File to modify
FILE="rust-toolchain.toml"

# Check if file exists
if [ ! -f "$FILE" ]; then
    echo "Error: $FILE does not exist."
    exit 1
fi

# Backup the original file
cp "$FILE" "${FILE}.bak"

# Remove the specified line and store the content in a variable
content=$(grep -v 'channel = "1.80.0"' "rust-toolchain.toml")
echo "$content" > "$FILE"

# Execute the commands

cargo b && ./target/debug/jito-restaking-cli --markdown-help > ./docs/_tools/00_cli.md && ./target/debug/jito-shank-cli && yarn generate-clients && cargo b
cargo sort --workspace
cargo fmt --all
cargo nextest run --all-features
cargo clippy --all-features -- -D warnings -D clippy::all -D clippy::nursery -D clippy::integer_division -D clippy::arithmetic_side_effects -D clippy::style -D clippy::perf


# # Reconstruct the file content with the line in the correct position
mv "${FILE}.bak" "$FILE"