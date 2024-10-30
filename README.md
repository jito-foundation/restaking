# Jito Staking Manager (JSM)

[![Build Status](https://github.com/jito-foundation/restaking/actions/workflows/ci.yaml/badge.svg?branch=master)](https://github.com/jito-foundation/restaking/actions)
[![License](https://img.shields.io/badge/License-BSL%201.1-blue.svg)](https://mariadb.com/bsl11/)
[![codecov](https://codecov.io/gh/jito-foundation/restaking/branch/master/graph/badge.svg?token=Q28COAGZ89)](https://codecov.io/gh/jito-foundation/restaking)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](http://makeapullrequest.com)

Jito Restaking is a next-generation restaking platform for Solana and SVM environments.

**This project is currently under development and is not yet ready for production use.
Expect breaking changes.**

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Running Tests](#running-tests)
- [Contributing](#contributing)
- [License](#license)

## Features

- Universal framework for staking, restaking, and liquid restaking
- VRT construction and management
- Customizable slashing conditions
- Flexible NCN and operator management

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

## License

This project is licensed under the Business Source License 1.1 - see the [LICENSE](./LICENSE.md) file for details.
