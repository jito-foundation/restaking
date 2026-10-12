# Makefile for Jito Restaking Project
# Commands
CARGO := cargo
CARGO_SORT := cargo sort
CARGO_CLIPPY := cargo clippy
CARGO_FMT := cargo +nightly fmt
# sbpfv3 needs platform-tools >= v1.57; the cargo-build-sbf 3.1.x default (v1.52) has no sbpfv3 sysroot
CARGO_SBF := cargo build-sbf --arch v3 --tools-version v1.57
SBF_PROGRAMS := restaking_program vault_program
CARGO_NEXTEST := cargo nextest
YARN := yarn
SHANK_CLI := ./target/release/jito-shank-cli

# Environment and paths
ENV_PATH := ./config/program.env
IDL_OUTPUT_PATH := ./idl

# Default target
.PHONY: all
all: build test

# Linting
.PHONY: lint
lint:
	$(CARGO_SORT) --workspace --check
	$(CARGO_FMT) --all --check
	$(CARGO_CLIPPY) --all-features -- -D warnings -D clippy::all -D clippy::nursery -D clippy::integer_division -D clippy::arithmetic_side_effects -D clippy::style -D clippy::perf

# Code generation
.PHONY: generate-code
generate-code: build-release generate-idl
	$(YARN) install
	$(YARN) generate-clients
	$(YARN) update-dependencies
	$(YARN) update-rust-borsh
	# Stable rustfmt, matching the `cargo fmt --all --check` gate in CI. The
	# nightly-only options in rustfmt.toml are commented out, so $(CARGO_FMT)
	# would produce identical output while requiring a toolchain CI lacks.
	$(CARGO) fmt --all

# Generate IDL files
.PHONY: generate-idl
generate-idl:
	$(SHANK_CLI) \
		--program-env-path $(ENV_PATH) \
		--output-idl-path $(IDL_OUTPUT_PATH) \
		generate \
		--program-id-key "RESTAKING_PROGRAM_ID" \
		--idl-name jito_restaking \
		--module-paths "restaking_sdk" \
		--module-paths "restaking_core" \
		--module-paths "restaking_program" \
		--module-paths "core"

	$(SHANK_CLI) \
		--program-env-path $(ENV_PATH) \
		--output-idl-path $(IDL_OUTPUT_PATH) \
		generate \
		--program-id-key "VAULT_PROGRAM_ID" \
		--idl-name jito_vault \
		--module-paths "vault_sdk" \
		--module-paths "vault_core" \
		--module-paths "vault_program" \
		--module-paths "core"

# Build debug
.PHONY: build
build:
	$(CARGO) build

# Build release
.PHONY: build-release
build-release:
	$(CARGO) build --release

# Build Solana BPF/SBF programs
.PHONY: build-sbf
build-sbf:
	# Build per program: a workspace-wide build pulls host-only deps (cli, tests) into the SBF target
	for p in $(SBF_PROGRAMS); do $(CARGO_SBF) --manifest-path $$p/Cargo.toml || exit 1; done

# Run tests
.PHONY: test
test:
	$(CARGO_NEXTEST) run --all-features

# Format code
.PHONY: format
format:
	$(CARGO_SORT) --workspace
	$(CARGO_FMT) --all
