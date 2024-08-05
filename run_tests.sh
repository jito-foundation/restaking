#!/bin/bash
set -e

TOKEN_METADATA_SO="$(pwd)/target/sbf-solana-solana/release/token_metadata.so"

if [ ! -f "$TOKEN_METADATA_SO" ]; then
	git clone https://github.com/metaplex-foundation/mpl-token-metadata
	cd mpl-token-metadata/programs/token-metadata/program/token-metadata
	cargo build-bpf
	cp target/sbf-solana-solana/release/token_metadata.so $(pwd)/target/sbf-solana-solana/release/
fi

cargo-build-sbf && SBF_OUT_DIR=$(pwd)/target/sbf-solana-solana/release cargo nextest run --all-features
