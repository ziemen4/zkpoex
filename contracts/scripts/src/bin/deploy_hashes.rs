// contracts/scripts/deploy_hashes.rs

use hex;
use sha3::{Digest, Keccak256};
use evm_runner::conditions::{hash_program_spec, Condition};
use evm_runner::context::{build_context_account_data, hash_context_data, ContextAccountDataType};
use std::fs;
use serde_json::from_str;
use std::env;

/// Compute the keccak256 hash of the ERC20 context contract's runtime bytecode.
fn compute_bytecode_hash() -> [u8; 32] {
    // Adjust the path if necessary—this expects the runtime bytecode to be at this location.
    const CONTEXT_ERC20_CONTRACT_BYTECODE: &str = include_str!("../../../../bytecode/ContextERC20.bin-runtime");
    let bytecode = hex::decode(CONTEXT_ERC20_CONTRACT_BYTECODE)
        .expect("Failed to decode ContextERC20 bytecode");
    let mut hasher = Keccak256::new();
    hasher.update(&bytecode);
    hasher.finalize().into()
}

fn load_program_spec(path: &str) -> Vec<(Condition, String)> {
    let raw = fs::read_to_string(path).unwrap();
    #[derive(serde::Deserialize)]
    struct TempSpec {
        condition: Condition,
        method: String
    }
    
    from_str::<Vec<TempSpec>>(&raw)
        .unwrap()
        .into_iter()
        .map(|item| (item.condition, item.method))
        .collect()
}

fn main() {
    println!("Computing deployment hashes for VerifierContract...");

    // 1. Compute PROGRAM_SPEC_HASH.
    let args: Vec<String> = env::args().collect();
    let loadaed_program_spec_hash = load_program_spec(&args[1]);
    let program_spec_hash = hash_program_spec(&loadaed_program_spec_hash);

    // 2. Compute BYTECODE_HASH.
    let bytecode_hash = compute_bytecode_hash();

    // 3. Compute CONTEXT_DATA_HASH.
    let context_data = build_context_account_data(ContextAccountDataType::ERC20, None);
    let context_data_hash = hash_context_data(&[context_data]);

    // Print the computed hashes in a format that our shell script can parse.
    println!("ProgramSpecHash=0x{}", hex::encode(program_spec_hash));
    println!("BytecodeHash=0x{}", hex::encode(bytecode_hash));
    println!("ContextDataHash=0x{}", hex::encode(context_data_hash));
}
