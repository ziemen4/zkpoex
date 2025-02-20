# Contracts

This directory contains the Solidity contracts along with a suite of scripts to deploy them using [Foundry (forge)](https://github.com/foundry-rs/foundry). The deployment process is integrated with a Rust-based tool that computes necessary hash values from our existing evm-runner code. These hash values are then passed into the contract constructor during deployment.

## Overview

The deployment suite consists of three main components:

1. **Rust Hash Computation Script:**  
   Located at `scripts/deploy_hashes.rs`, this binary computes:
   - `ProgramSpecHash`
   - `BytecodeHash`
   - `ContextDataHash`  
   These values are printed in a format that can be parsed and exported as environment variables.

2. **Foundry Deployment Script:**  
   The Solidity script `scripts/Deploy.s.sol` uses Foundry's cheatcodes to read the hash values (and other parameters) from environment variables and deploys the contracts on Sepolia.

3. **Shell Deployment Script:**  
   The Bash script `scripts/deploy.sh` ties everything together:
   - It runs the Rust hash computation script.
   - Parses and exports the hash values.
   - Executes the Foundry deployment script using `forge`.

## Prerequisites

- **Rust:** Install via [rustup](https://rustup.rs/).
- **Foundry:** Install Foundry by following the [Foundry book](https://book.getfoundry.sh/getting-started/installation).
- **Solidity Compiler (solc):** Ensure you have `solc` installed.
- **Node.js (optional):** If you use additional tooling around deployment.
- **Environment Variables:**  
  Create a `.env` file in your project root (or export manually) with at least:
  ```env
  RISC0_VERIFIER_ADDRESS=0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187
  SEPOLIA_RPC_URL=https://sepolia.infura.io/v3/YOUR_INFURA_PROJECT_ID
  ```
  The hash variables (`PROGRAM_SPEC_HASH`, `BYTECODE_HASH`, `CONTEXT_DATA_HASH`) are automatically computed.

## Directory Structure

```
contracts/
├── src/                     # Solidity contract source files
│   ├── VerifierContract.sol
│   └── TargetContract.sol
├── bytecode/                # Compiled contract bytecode is stored here
├── scripts/
│   ├── Deploy.s.sol         # Foundry deployment script (Solidity)
│   ├── deploy_hashes.rs     # Rust binary to compute required hash values
│   └── deploy.sh            # Shell script to run the entire deployment workflow
└── README.md                # This file
```

## Setup & Build

1. **Compile Contracts (Optional):**  
   You can compile your Solidity contracts separately using Foundry:
   ```sh
   forge build
   ```

2. **Build the Rust Hash Script:**  
   Make sure your Rust environment is set up and that the `deploy_hashes` binary is configured in your Cargo.toml (or workspace configuration).  
   From the project root (or within the contracts folder if configured), you can run:
   ```sh
   cargo build --bin deploy_hashes
   ```

## Deployment Workflow

The deployment process is automated via the `deploy.sh` script. Here’s how it works:

1. **Compute Hashes:**  
   The script runs the Rust binary `deploy_hashes` (located in `scripts/`) which outputs the following:
   - `ProgramSpecHash`
   - `BytecodeHash`
   - `ContextDataHash`

2. **Export Environment Variables:**  
   The script parses the output of `deploy_hashes` and exports these values as environment variables.

3. **Deploy Contracts via Foundry:**  
   The script then calls the Foundry deployment script (`Deploy.s.sol`) using:
   ```sh
   forge script scripts/Deploy.s.sol --broadcast --verify --rpc-url $SEPOLIA_RPC_URL
   ```

## How to Run the Deployment

1. **Set Environment Variables:**  
   Ensure your `.env` file is configured or manually export the following in your shell:
   ```sh
   export RISC0_VERIFIER_ADDRESS=0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187
   export SEPOLIA_RPC_URL=https://sepolia.infura.io/v3/YOUR_INFURA_PROJECT_ID
   ```
   
2. **Run the Shell Script:**  
   From within the `contracts` directory, execute:
   ```sh
   ./scripts/deploy.sh
   ```
   This will:
   - Compute the hash values using the Rust script.
   - Set the necessary environment variables.
   - Deploy the contracts to Sepolia via Foundry.

## Troubleshooting

- **Missing Dependencies:**  
  Ensure Foundry is installed by running `forge --version` and that your Rust toolchain is up to date.
- **Environment Variables:**  
  Double-check that your `.env` file (or exported variables) contains valid RPC URLs and addresses.
- **Compilation Issues:**  
  If the Rust script fails to build, verify your Cargo.toml dependencies and file paths (especially for the bytecode file).