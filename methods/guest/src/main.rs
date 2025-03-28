extern crate alloc;

use alloc::{string::String, vec::Vec};
use evm_runner::run_evm;
use risc0_zkvm::guest::env;
use shared::conditions::Condition;
use shared::input::AccountData;

fn main() {
    let start = env::cycle_count();

    let calldata: String = env::read();
    println!("\n------------- GUEST: READ CALLDATA -------------\n");
    println!("{:?}", calldata);
    println!("\n------------------------------------------------\n");
    // TODO: See if we can deserialize directly into Vec<AccountData>, instead of String
    let context_state: Vec<AccountData> = env::read();
    println!("\n------------- GUEST: CONTEXT STATE -------------\n");
    println!("Deserialized {} account states", context_state.len());
    println!("\n------------------------------------------------\n");

    // TODO: See if we can deserialize directly into Vec<Condition>, instead of String
    let _program_spec: String = env::read();
    println!("\n------------- GUEST: PROGRAM SPEC -------------\n");
    println!("{:?}", _program_spec);
    println!("\n------------------------------------------------\n");

    let blockchain_settings: String = env::read();
    println!("\n------------- GUEST: BLOCKCHAIN SETTINGS -------------\n");
    println!("{:?}", blockchain_settings);
    println!("\n------------------------------------------------\n");

    let program_spec: Vec<(Condition, String)> = serde_json::from_str(&_program_spec).unwrap();
    println!("\n------------- GUEST: PROGRAM SPEC -------------\n");
    println!("{:?}", program_spec);
    println!("\n------------------------------------------------\n");

    // Log input_json
    let result = run_evm(&calldata, context_state, program_spec, &blockchain_settings);
    env::commit(&result);

    let end = env::cycle_count();
    eprintln!("my_operation_to_measure: {}", end - start);
}
