// SPDX-License-Identifier: MIT

// Needed for using alloc in no-std environments (e.g., zkVM guest)
extern crate alloc;
use alloc::{string::String, vec::Vec};

// EVM execution runner (host call into EVM logic)
use evm_runner::run_evm;

// Guest environment for reading/writing data
use risc0_zkvm::guest::env;

// Shared data structures for EVM specs and context
use shared::conditions::MethodSpec;
use shared::input::AccountData;

// Alloy: Solidity ABI definitions/encoding and Ethereum primitive types
use alloy_primitives::{B256};
use alloy_sol_types::{sol, SolValue};

// Standard utility to parse from strings
use primitive_types::U256;
use std::str::FromStr;

// Solidity ABI encoding for public input
sol! {
    struct PublicInput {
        bool exploitFound;
        bytes32 programSpecHash;
        bytes32 contextStateHash;
    }
}

fn main() {
    let start = env::cycle_count();

    let calldata: String = env::read();
    let _context_state: String = env::read();
    let _program_spec: String = env::read();
    let blockchain_settings: String = env::read();

    // Deserialize context state
    let context_state: Vec<AccountData> = serde_json::from_str(&_context_state).unwrap();

    // Deserialize program spec
    let program_spec: Vec<MethodSpec> = serde_json::from_str(&_program_spec).unwrap();

    let value: U256 = env::read();

    // Log input_json
    let result = run_evm(
        &calldata,
        context_state,
        program_spec,
        &blockchain_settings,
        value,
    );

    let exploit_found: bool = result[0] == "true";
    let program_spec_hash: B256 =
        B256::from_str(&result[1]).expect("Invalid hex for program_spec_hash");
    let context_state_hash: B256 =
        B256::from_str(&result[2]).expect("Invalid hex for context_state_hash");

    let input = PublicInput {
        exploitFound: exploit_found,
        programSpecHash: program_spec_hash,
        contextStateHash: context_state_hash,
    };

    let encoded = PublicInput::abi_encode(&input);
    env::commit_slice(&encoded);

    let end = env::cycle_count();
    shared::log_warn!("my_operation_to_measure: {}", end - start);
}
