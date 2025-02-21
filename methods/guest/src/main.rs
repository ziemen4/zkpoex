extern crate alloc;

use risc0_zkvm::guest::env;
use evm_runner::run_evm;
use evm_runner::conditions::Condition;
use evm_runner::input::AccountData;
use primitive_types::{U256, H160, H256};
use alloc::{vec::Vec, collections::BTreeMap, string::String, format};
use evm::{
	Config,ExitReason, ExitSucceed,
	backend::{
		MemoryVicinity, MemoryAccount, MemoryBackend
	},
	executor::stack::{
		StackSubstateMetadata, MemoryStackState, StackExecutor
	}, Handler,
};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;
use serde_json::Value;

fn main() {
    let start = env::cycle_count();

    let calldata: String = env::read();
    println!("Read calldata successfully");
    println!("{:?}", calldata);
    
    // TODO: See if we can deserialize directly into Vec<AccountData>, instead of String
    let _context_state: String = env::read();
    println!("Read context_state successfully");
    println!("{:?}", _context_state);

    // TODO: See if we can deserialize directly into Vec<Condition>, instead of String
    let _program_spec: String = env::read();
    println!("Read program_spec successfully");
    println!("{:?}", _program_spec);

    let blockchain_settings: String = env::read();
    println!("Read blockchain_settings successfully");
    println!("{:?}", blockchain_settings);

    let program_spec: Vec<(Condition, String)> = serde_json::from_str(&_program_spec).unwrap();
    println!("Converted program_spec successfully");
    println!("{:?}", program_spec);

    let context_state: Vec<AccountData> = serde_json::from_str(&_context_state).unwrap();
    println!("Converted context_data successfully");
    println!("{:?}", context_data);

    // Log input_json
    let result = run_evm(
        &calldata, 
        context_state,
        program_spec, 
        &blockchain_settings
    );
    env::commit(&result);

    let end = env::cycle_count();
    eprintln!("my_operation_to_measure: {}", end - start);
}
