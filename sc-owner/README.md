# Smart Contract Deployer Crate

This Rust crate is designed to deploy two smart contracts on an EVM-compatible blockchain:

- **ImageID Contract:** An identifier contract deployed first.
- **Verifier Contract:** A contract that verifies certain conditions by leveraging a pre-deployed RISC0 verifier contract.

The deployment process incorporates a context state and a program specification (both provided as JSON files) whose hashes are computed and used during the deployment of the Verifier contract.

---

## Overview

The crate performs the following tasks:

1. **Parse CLI Arguments:**  
   It accepts several command line arguments including the private key, the RISC0 verifier contract address, and paths to the context state and program specification files.
2. **Process Input Files:**  
   Reads and parses the JSON files to compute unique hashes using helper functions.
3. **Deploy Contracts:**
   - **ImageID Contract:**  
     Reads the bytecode from `./bytecode/ImageID.bin` and deploys the contract.
   - **Verifier Contract:**  
     Reads the bytecode from `./bytecode/VerifierContract.bin` and deploys it by linking the computed hashes, the provided RISC0 verifier contract address, and the address of the deployed ImageID contract.
4. **Output Deployed Addresses:**  
   The addresses of the deployed contracts are printed to the console.

---

## Prerequisites

Before using the crate, ensure you have:

- **Rust and Cargo:**  
  Install [Rust](https://www.rust-lang.org/tools/install) to compile and run the crate.
- **EVM-Compatible Wallet and Network Access:**  
  A valid wallet private key and access to an EVM-compatible blockchain with sufficient funds for deploying contracts.
- **RISC0 Verifier Contract Address:**  
  The address of a pre-deployed RISC0 verifier contract.
- **Contract Bytecode Files:**  
  The bytecode files must be located in the `./bytecode/` directory:
  - `ImageID.bin`
  - `VerifierContract.bin`

---

## Installation

1. **Build the Crate**

   ```bash
   cargo build --release
   ```

---

## Usage

Run the crate with the required CLI arguments. For example:

`cargo run -- \   --private-key <YOUR_PRIVATE_KEY> \   --risc0-verifier-contract-address <RISC0_VERIFIER_CONTRACT_ADDRESS> \   --context-state <PATH_TO_CONTEXT_STATE_JSON> \   --program-spec <PATH_TO_PROGRAM_SPEC_JSON>`

### CLI Arguments Description

- **`--private-key`**  
   The private key of the wallet that will be used to deploy the contracts.
- **`--risc0-verifier-contract-address`**  
   The address of the pre-deployed RISC0 verifier contract.
- **`--context-state`**  
   File path to a JSON file containing the context state (a list of account data).
- **`--program-spec`**  
   File path to a JSON file containing the program specification.

---
