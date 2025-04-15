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
  The hash variables (`PROGRAM_SPEC_HASH`, `BYTECODE_HASH`, `CONTEXT_DATA_HASH`) are automatically computed.

## Directory Structure

```
contracts/
├── src/                                 # Solidity contract source files
│   ├── context/
│   │   └── ContextTemplateERC20.sol
│   ├── examples                         # Dir with Exploits Examples 
│   │   ├── reentrancy/
│   │   │   ├── AttackContract.sol
│   │   │   └── ReentrancyVulnerable.sol
│   │   ├── BasicVulnerable.sol
│   │   └── OverUnderFlowVulnerable.sol
│   ├── VerifierContract.sol             # VerifierContract.sol to verify a proof onchain
│   └── ImageID.sol                      # ImageID.sol defines the unique image ID hash of the zkVM guest code.
├── bytecode/                            # Compiled contract bytecode is stored here (.abi, .bin and .bin-runtime format)
├── test/
│   ├── mocks/
│   │   └── MockRiscZeroVerifier.sol     # Mock used in VerifierContract.t.sol
│   └── VerifierContract.t.sol           # Test file for VerifierContract.sol
└── README.md                            # This file
```

## Setup & Build

**Compile Contracts (Optional):**  
   You can compile your Solidity contracts separately using Foundry:
   ```sh
   forge build
   ```
   This command is optional because when you run `just prove` the compilation of the contracts is automatic

## Troubleshooting

- **Missing Dependencies:**  
  Ensure Foundry is installed by running `forge --version` and that your Rust toolchain is up to date.
- **Compilation Issues:**  
  If the Rust script fails to build, verify your Cargo.toml dependencies and file paths (especially for the bytecode file).