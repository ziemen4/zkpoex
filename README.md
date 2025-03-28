# zkpoex

**zkpoex** is a Rust-based toolkit for proving exploits using zero-knowledge proofs. Built on top of [risc0](https://risc0.com/), zkpoex leverages advanced cryptographic techniques to verify exploit execution without revealing sensitive details. The project is structured as a Cargo workspace with three main members: **host**, **methods**, and **evm-runner**.

## Table of Contents

- [zkpoex](#zkpoex)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Setup \& Build](#setup--build)
  - [Usage](#usage)
    - [Compiling Contracts](#compiling-contracts)
    - [Running Tests for the EVM Runner](#running-tests-for-the-evm-runner)
    - [Proving an Exploit](#proving-an-exploit)
  - [Project Structure](#project-structure)
  - [License](#license)

## Features

- **Zero-Knowledge Proofs:** Use risc0 to generate verifiable zk proofs of exploit execution.
- **EVM Integration:** Interact with an Ethereum Virtual Machine interpreter for proving.
- **Modular Design:** Workspace split into multiple packages for clear separation of concerns.

## Prerequisites

- **Rust Toolchain:** Install the latest version from [rustup.rs](https://rustup.rs/).
- **Solidity Compiler (solc):** Required for compiling smart contracts.
- **Just:** A command runner to streamline common tasks. Install via [Justfile instructions](https://github.com/casey/just).
- **RISC0 SDK:** Follow the installation instructions on [risc0.com](https://risc0.com/).

## Installation

1. **Clone the Repository:**

   ```sh
   git clone https://github.com/your-org/zkpoex.git
   cd zkpoex
   ```

2. **Install Dependencies:**

   Ensure you have the Rust toolchain and solc installed. Then, update your Rust dependencies:

   ```sh
   cargo update
   ```

## Setup & Build

The project is configured as a Cargo workspace.

The current configuration ensures optimized builds for faster execution of proofs.

## Usage

### Compiling Contracts

Contracts are written in Solidity. Use the provided `justfile` commands to compile them:

```sh
just compile-contract
```

This command will:

- Compile `BasicVulnerable.sol` and other contracts. The output is stored in the `bytecode` directory.

### Running Tests for the EVM Runner

After compiling contracts, run the tests for the `evm-runner` package:

```sh
just test-evm
```

### Proving an Exploit

To generate a zero-knowledge proof of an exploit, you can run:

```sh
just prove
```

This command sets the `RUST_BACKTRACE=full` environment variable and runs the `host` package in release mode.

Alternatively, if you have a bonsai key you can use

```sh
just prove-bonsai
```

**Important**: Beware since the prover has very high requirements

## Project Structure

```
zkpoex/
├── Cargo.toml            # Workspace manifest
├── justfile              # Command runner instructions
├── host/                 # Main package for host functionalities (zk proving)
├── methods/              # Package for various zk methods
├── evm-runner/           # Package to run EVM-related tasks and tests
├── contracts/            # Solidity contracts and outputs (bytecode, storage layout)
├── bytecode/			  # Bytecode from the contracts
├── docs/				  # Documentations for zkpoex releases
└── README.md             # This file
```

## License

[MIT License](LICENSE)
