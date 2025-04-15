# Prover Exploit Crate

This crate acts as a "prover" within a zero-knowledge proof framework, aiming to find potential exploits in a verifier contract. It leverages the RISC-V ELF binary and associated image ID (via `risc0_zkvm`) to generate an execution proof. The proof data (i.e. the journal and succinct seal) is then output for further verification or exploitation analysis.

---

## Overview

The crate performs the following tasks:

1. **CLI & Environment Setup:**

   - Loads environment variables using [dotenv](https://crates.io/crates/dotenv).
   - Parses CLI arguments including the target function, parameters, context state file, program specification file, network and bonsai config.

2. **Input Preparation:**

   - Generates function calldata dynamically from the provided function name and parameters.
   - Reads and parses the context state (account data) and program specification (JSON format).
   - Retrieves blockchain settings via asynchronous calls.

3. **Proof Generation:**

   - Constructs an input data structure that includes:
     - Calldata
     - Context state
     - Program specification
     - Blockchain settings
   - Uses `risc0_zkvm`'s default prover to execute the provided RISC-V ELF binary (`ZKPOEX_GUEST_ELF`) and generate a proof (receipt).

4. **Output Processing:**
   - Extracts the journal and succinct seal from the receipt.
   - Saves these outputs as `journal.bin` and `seal.bin`.
   - Verifies the proof against the provided guest ID (`ZKPOEX_GUEST_ID`) and prints the result.

---

## Prerequisites

- **Rust and Cargo:**  
  Install Rust from [rust-lang.org](https://www.rust-lang.org/tools/install).

- **RISC0 Environment:**  
  This crate uses [risc0_zkvm](https://docs.rs/risc0_zkvm/) for zero-knowledge proof generation. Ensure your system supports building and executing risc0-based applications.

- **Input Files:**

  - **Context State File:** JSON file containing account data.
  - **Program Specification File:** JSON file outlining program details.

- **Environment Variables:**  
  Optionally create a `.env` file to configure additional settings.

---

## Installation

1. **Build the Crate:**

   ```bash
   cargo build --release
   ```

---

## Usage

Run the crate with the required CLI arguments. The following options must be provided:
- **`{{function}}`**  
   Name of the target function.
- **`{{params}}`**  
   Parameters for the function call, passed as a string.
- **`{{context-state}}`**  
   Path to a JSON file containing the context state.
- **`{{program-spec}}`**  
   Path to a JSON file containing the program specification.
- **`{{network}}`**  
   Specifies the blockchain network to use (e.g., `mainnet`, `testnet` or `local`).
- **`{{bonsai}}`** (optional - need to set `BONSAI_API_KEY` in the `.env` file)  
   Set it to `true` if you want to use Remote Proving. By default, it's set to `false`.

### Example Command

`just prove "exploit(bool)" "true" "./shared/examples/basic-vulnerable/context_state.json" "./shared/examples/basic-vulnerable/program_spec.json" "local"`

## Testing the Existing Exploit Examples

If you want to test the existing exploit examples, simply run one of the following commands:

1. **Basic Vulnerable**:
`just example-basic-vulnerable-prove network bonsai`

2. **Under/Over-Flow Vulnerability**:
`just example-over-under-flow-prove network bonsai`

3. **Reentrancy Vulnerability**:
`just example-reentrancy-prove network bonsai`

### What the Crate Does

- **Calldata Generation:**  
   It generates the function signature and calldata from the specified function and parameters.
- **Input Construction:**  
   Reads and parses the context state and program spec files, then builds an input data structure that also includes blockchain settings.
- **Proof Generation:**  
   The crate uses the `risc0_zkvm` prover to execute the guest ELF binary and produce a receipt, which contains a journal (execution output) and a succinct seal (proof metadata).
- **Output Files:**  
   The journal and seal are saved as `journal.bin` and `seal.bin` respectively. These files can be used in subsequent verification steps or further analysis.
- **Verification:**  
   The receipt is verified against the known guest ID (`ZKPOEX_GUEST_ID`), ensuring the integrity of the generated proof.
