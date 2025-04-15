extern crate alloc;

use alloc::{string::String, vec::Vec};
use evm_runner::run_evm;
use risc0_zkvm::guest::env;
use shared::conditions::MethodSpec;
use shared::input::AccountData;
use primitive_types::U256;

use alloy_sol_types::sol;
use alloy_sol_types::SolValue;
use alloy_primitives::{Address, B256};
use std::str::FromStr;

sol! {
    struct PublicInput {
        bool exploitFound;
        bytes32 programSpecHash;
        bytes32 contextStateHash;
        address proverAddress;
    }
}

fn main() {
    let start = env::cycle_count();

    let calldata: String = env::read();
    println!("\n------------- GUEST: READ CALLDATA -------------\n");
    println!("{:?}", calldata);
    println!("\n------------------------------------------------\n");

    let _context_state: String = env::read();
    let _program_spec: String = env::read();

    let blockchain_settings: String = env::read();
    println!("\n------------- GUEST: BLOCKCHAIN SETTINGS -------------\n");
    println!("{:?}", blockchain_settings);
    println!("\n------------------------------------------------\n");

    // Deserialize context state
    let context_state: Vec<AccountData> = serde_json::from_str(&_context_state).unwrap();
    println!("\n------------- GUEST: CONTEXT STATE -------------\n");
    println!("{:?}", context_state);
    println!("\n------------------------------------------------\n");

    // Deserialize program spec
    let program_spec: Vec<MethodSpec> = serde_json::from_str(&_program_spec).unwrap();
    println!("\n------------- GUEST: PROGRAM SPEC -------------\n");
    println!("{:?}", program_spec);
    println!("\n------------------------------------------------\n");

    let value: U256 = env::read();
    println!("\n------------- GUEST: VALUE -------------\n");
    println!("{:?}", value);
    println!("\n------------------------------------------------\n");

    // Log input_json
    let result = run_evm(&calldata, context_state, program_spec, &blockchain_settings, value);    

    let exploit_found: bool = result[0] == "true";
    let program_spec_hash: B256 = B256::from_str(&result[1]).expect("Invalid hex for program_spec_hash");
    let context_state_hash: B256 = B256::from_str(&result[2]).expect("Invalid hex for context_state_hash");
    let prover_address: Address = Address::from_str(&result[3]).expect("Invalid Ethereum address");

    let input = PublicInput {
        exploitFound: exploit_found,
        programSpecHash: program_spec_hash,
        contextStateHash: context_state_hash,
        proverAddress: prover_address,
    };

    let encoded = PublicInput::abi_encode(&input);
    env::commit_slice(&encoded);

    let end = env::cycle_count();
    eprintln!("my_operation_to_measure: {}", end - start);
}
