// contracts/scripts/deploy_hashes.rs

use hex;
use sha3::{Digest, Keccak256};
use evm_runner::conditions::{hash_program_spec, compute_mapping_storage_key, Condition, FixedCondition, Operator};
use evm_runner::context::{build_context_account_data, hash_context_data, ContextAccountDataType};
use std::collections::BTreeMap;
use std::str::FromStr;
use primitive_types::{U256, H160, H256};

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

fn main() {
    println!("Computing deployment hashes for VerifierContract...");

    // 1. Compute PROGRAM_SPEC_HASH.
    // Here we create a sample spec. In your real deployment, replace this with your actual program specification.
    let computed_storage_key = compute_mapping_storage_key(
        H160::from_str("0x4838B106FCe9647Bdf1E7877BF73cE8B0BAD5f97").unwrap(), // mapping key (address)
        U256::from(0) // the base slot for the mapping (See ContextERC20-storageLayout.json)
    );
    let mut context_erc20_init_storage: BTreeMap<H256, H256> = BTreeMap::new();
    let amount: H256 = H256::from_low_u64_be(100000);
    context_erc20_init_storage.insert(computed_storage_key, amount);

    let erc20_account_data = build_context_account_data(
        ContextAccountDataType::ERC20,
        Some(context_erc20_init_storage)
    );
	let state_path = format!("{}.storage.{}", erc20_account_data.address, hex::encode(computed_storage_key));

    let sample_spec: Vec<(Condition, String)> = vec![
        (
            Condition::Fixed(
                FixedCondition {
                    k_s: state_path,
                    op: Operator::Gt,
                    v: U256::from_dec_str("0").unwrap(),
                }
            ),
            "d92dbd19".to_string()
        ),
        (
            Condition::Fixed(
                FixedCondition {
                    k_s: "4838B106FCe9647Bdf1E7877BF73cE8B0BAD5f97.balance".to_string(),
                    op: Operator::Gt,
                    v: U256::from_dec_str("0").unwrap(),
                }
            ),
            "16112c6c".to_string()
        )
    ];
    let program_spec_hash = hash_program_spec(&sample_spec);

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
