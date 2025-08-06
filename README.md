# Jito Staking Manager (JSM)

[![Build Status](https://github.com/jito-foundation/restaking/actions/workflows/ci.yaml/badge.svg?branch=master)](https://github.com/jito-foundation/restaking/actions)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![codecov](https://codecov.io/gh/jito-foundation/restaking/branch/master/graph/badge.svg?token=Q28COAGZ89)](https://codecov.io/gh/jito-foundation/restaking)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](http://makeapullrequest.com)

Jito Restaking is a next-generation restaking platform for Solana and SVM environments.

## Table of Contents

- [Features](#features)
- [Documentation](#documentation)
- [SDKs](#sdks)
- [Installation](#installation)
- [Usage](#usage)
- [Running Tests](#running-tests)
- [Releasing](#releasing)
- [Contributing](#contributing)
- [License](#license)

## Features

- Universal framework for staking, restaking, and liquid restaking
- VRT construction and management
- Flexible NCN and operator management

## Documentation

The comprehensive documentation for Stakenet has moved to [jito.network/docs/restaking/jito-restaking-overview](https://www.jito.network/docs/restaking/jito-restaking-overview/).
The source files are maintained in the [Jito Omnidocs repository](https://github.com/jito-foundation/jito-omnidocs/tree/master/restaking).

## SDKs

We provide TypeScript SDKs for interacting with the Jito Restaking system:

- 📦 [`@jito-foundation/restaking-sdk`](https://www.npmjs.com/package/@jito-foundation/restaking-sdk) – TypeScript SDK for interacting with the Jito Restaking program.
- 📦 [`@jito-foundation/vault-sdk`](https://www.npmjs.com/package/@jito-foundation/vault-sdk) – TypeScript SDK for interacting with the Jito Vault program.

## Installation

```bash
git clone https://github.com/jito-foundation/restaking.git
cd restaking
cargo-build-sbf
```

## Usage

### Quickstart

To create a vault, mint vrt, and delegate to an operator, follow this [guide](cli/getting_started.md).

### Building the software

```bash
cargo-build-sbf
```

### Building the IDLs and client code

```bash
# Build the shank CLI tool
cargo b --release -p jito-shank-cli && ./target/release/jito-shank-cli
# Generate the client code
yarn generate-clients
# Rebuild the entire project
cargo b --release
```

## Running Tests

If you haven't installed `cargo-nextest` yet, it's recommended to install it.
You find the installation instructions [here](https://nexte.st/docs/installation/from-source/).

### Outside of SVM

```bash
cargo nextest run
```

### Testing using the SVM environment

```bash
cargo-build-sbf && SBF_OUT_DIR=$(pwd)/target/sbf-solana-solana/release cargo nextest run --all-features
```

## Releasing

```bash
./release
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

- Fork the project
- Create your feature branch (git checkout -b username/feature_name)
- Commit your changes (git commit -m 'Add some feature')
- Push to the branch (git push origin username/feature_name)
- Open a Pull Request

## Security Audits

| Group    | Date       | Commit                                                                 |
|----------|------------|------------------------------------------------------------------------|
| Offside  | 2024-11-20 | [60b3884](security_audits/offside_jito_vault_audit.pdf)                |
| Ottersec | 2024-10-25 | [f04242f](security_audits/ottersec_jito_restaking_audit.pdf)           |
| Certora  | 2024-10-29 | [ecbe19a](security_audits/certora_jito_restaking_vault_audit_v1.pdf)   |
| Certora  | 2024-12-23 | [3fdcd88](security_audits/certora_jito_restaking_vault_audit_v2.pdf)   |

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.


