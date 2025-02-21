#!/bin/bash
set -e

# Load .env
export $(grep -v '^#' ../.env | xargs)

echo "Running Rust hash computation..."
# Adjust the cargo command as needed so that it builds the binary from your workspace.
HASHES=$(cargo run --bin deploy_hashes --release -- $PROGRAM_SPEC_PATH)

# Parse the output and export the variables.
export PROGRAM_SPEC_HASH=$(echo "$HASHES" | grep "ProgramSpecHash=" | cut -d'=' -f2)
export BYTECODE_HASH=$(echo "$HASHES" | grep "BytecodeHash=" | cut -d'=' -f2)
export CONTEXT_DATA_HASH=$(echo "$HASHES" | grep "ContextDataHash=" | cut -d'=' -f2)

# Also ensure that the RISC0 verifier address is set (either here or in your .env file).
export RISC0_VERIFIER_ADDRESS=${RISC0_VERIFIER_ADDRESS:-"0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187"}

echo "Computed Hashes:"
echo "ProgramSpecHash: $PROGRAM_SPEC_HASH"
echo "BytecodeHash: $BYTECODE_HASH"
echo "ContextDataHash: $CONTEXT_DATA_HASH"

echo "Deploying contracts using forge script..."
# Adjust the RPC URL if necessary.
forge script ./Deploy.s.sol --broadcast --rpc-url $BLOCKCHAIN_RPC_URL --private-key $PRIVATE_KEY
