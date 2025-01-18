#![cfg_attr(not(test),)]
pub mod conditions;
pub mod input;

extern crate alloc;
extern crate core;

use core::str::FromStr;
use crate::alloc::string::ToString;

use conditions::{Condition, FixedCondition, RelativeCondition, Operator};
use alloc::{vec::Vec, collections::BTreeMap, string::String};
use evm::{
	backend::{
		Backend, MemoryAccount, MemoryBackend, MemoryVicinity
	}, executor::stack::{
		MemoryStackState, StackExecutor, StackSubstateMetadata
	}, Config, ExitReason, ExitSucceed, Handler
};
use input::{CallerData, TargetData};
use primitive_types::{U256, H160, H256};
use serde_json::from_str;
use serde::Deserialize;

pub const TARGET_CONTRACT_BYTECODE: &str = include_str!("../../bytecode/TargetContract.bin-runtime");

#[derive(Debug, Deserialize)]
pub struct DeserializeMemoryVicinity {
    pub gas_price: String,
    pub origin: String,
    pub chain_id: String,
    pub block_hashes: String,
    pub block_number: String,
    pub block_coinbase: String,
    pub block_timestamp: String,
    pub block_difficulty: String,
    pub block_gas_limit: String,
    pub block_base_fee_per_gas: String,
}

fn check_condition_op<T: PartialOrd>(operator: &Operator, first_val: T, second_val: T) -> bool {
	match operator {
		Operator::Eq => first_val == second_val,
		Operator::Neq => first_val != second_val,
		Operator::Gt => first_val > second_val,
		Operator::Ge => first_val >= second_val,
		Operator::Lt => first_val < second_val,
		Operator::Le => first_val <= second_val,
	}
}

fn check_relative_condition(
	pre_state: &MemoryStackState<MemoryBackend>,
	post_state: &MemoryStackState<MemoryBackend>,
	relative_condition: &RelativeCondition
) -> bool {
	// Pre state key is joined by '.', so split it
	let pre_state_key = relative_condition.k_s.split('.').collect::<Vec<&str>>();

	// Post state key is joined by '.', so split it
	let post_state_key = relative_condition.k_s_prime.split('.').collect::<Vec<&str>>();

	// The first state key is always the address of the account
	let pre_account_address = H160::from_str(pre_state_key[0]).unwrap();
	let post_account_address = H160::from_str(post_state_key[0]).unwrap();

	// Check that the addresses are the same
	assert!(pre_account_address == post_account_address);

	// If account does not exist, panic
	if !pre_state.exists(pre_account_address) {
		panic!("Account does not exist: {}", pre_account_address);
	}

	// Get the account from the state
	let pre_account_basic = pre_state.basic(pre_account_address);
	let post_account_basic = post_state.basic(post_account_address);

	// If the state key vector has length 2, then we are accessing the account fields directly
	if pre_state_key.len() == 2 {
		match pre_state_key[1] {
			"nonce" => {
				check_condition_op(&relative_condition.op, pre_account_basic.nonce, post_account_basic.nonce)
			}
			"balance" => {
				check_condition_op(&relative_condition.op, pre_account_basic.balance, post_account_basic.balance)
			}
			_ => {
				panic!("Invalid state key: {}", pre_state_key[1]);
			}
		}
	} else {
		// TODO: Figure out how to compare storage values correctly (type stuff)
		// If the state key vector has length 3, then we are accessing the storage fields
		//let storage_key = H256::from_str(state_key[1]).unwrap();
		//let storage_value = state.storage.(account_address, storage_key);
		//assert!(check_condition_op(fixed_condition.op, storage_value, fixed_condition.v));
		false
	}
}

fn check_fixed_condition(
	state: &MemoryStackState<MemoryBackend>,
	fixed_condition: &FixedCondition
) -> bool {
	// State key is joined by '.', so split it
	let state_key = fixed_condition.k_s.split('.').collect::<Vec<&str>>();

	// The first state key is always the address of the account
	let account_address = H160::from_str(state_key[0]).unwrap();

	// If account does not exist, panic
	if !state.exists(account_address) {
		panic!("Account does not exist: {}", account_address);
	}

	// Get the account from the state
    let account_basic = state.basic(account_address);

	println!("Account basic: {:?}", account_basic);
	println!("Fixed condition: {:?}", fixed_condition);
	// If the state key vector has length 2, then we are accessing the account fields directly
	if state_key.len() == 2 {
		match state_key[1] {
			"nonce" => {
				check_condition_op(&fixed_condition.op, account_basic.nonce, fixed_condition.v)
			}
			"balance" => {
				check_condition_op(&fixed_condition.op, account_basic.balance, fixed_condition.v)
			}
			_ => {
				panic!("Invalid state key: {}", state_key[1]);
			}
		}
	} else {
		// TODO: Figure out how to compare storage values correctly (type stuff)
		// If the state key vector has length 3, then we are accessing the storage fields
		//let storage_key = H256::from_str(state_key[1]).unwrap();
		//let storage_value = state.storage.(account_address, storage_key);
		//assert!(check_condition_op(fixed_condition.op, storage_value, fixed_condition.v));
		false
	}
}

pub fn run_evm(
	calldata: &str,
	caller_data: CallerData,
	target_data: TargetData,
	program_spec: Vec<(Condition, String)>,
	blockchain_settings: &str,
) -> Vec<String> {
	// 0. Initialize and setup the EVM
	let config = Config::istanbul();
	// deserialize vicinity
	let deserialize_vicinity: DeserializeMemoryVicinity = from_str(blockchain_settings).unwrap();
	
	let vicinity = MemoryVicinity {
		gas_price: U256::from_str(&deserialize_vicinity.gas_price).unwrap(),
		origin: H160::from_str(&deserialize_vicinity.origin).unwrap(),
		chain_id: U256::from_str(&deserialize_vicinity.chain_id).unwrap(),
		block_hashes: serde_json::from_str::<Vec<String>>(&deserialize_vicinity.block_hashes).unwrap().into_iter().map(|s| H256::from_str(&s).unwrap()).collect(),
		block_number: U256::from_str(&deserialize_vicinity.block_number).unwrap(),
		block_coinbase: H160::from_str(&deserialize_vicinity.block_coinbase).unwrap(),
		block_timestamp: U256::from_str(&deserialize_vicinity.block_timestamp).unwrap(),
		block_difficulty: U256::from_str(&deserialize_vicinity.block_difficulty).unwrap(),
		block_gas_limit: U256::from_str(&deserialize_vicinity.block_gas_limit).unwrap(),
		block_base_fee_per_gas: U256::from_str(&deserialize_vicinity.block_base_fee_per_gas).unwrap(),
		block_randomness: H256::from_str("0x0").ok(), // TODO: Check if this is correct
	};

	// 1. Setup global state from caller_data and target_data
	let mut global_state: BTreeMap<H160, MemoryAccount> = BTreeMap::new();

	// Target contract
	global_state.insert(
		H160::from_str(&target_data.address).unwrap(),
		MemoryAccount {
			nonce: target_data.nonce,
			balance: target_data.balance,
			storage: target_data.storage,
			code: hex::decode(target_data.bytecode).unwrap(),
		}
	);

	// Caller EOA (TODO: Support contract callers)
	global_state.insert(
		H160::from_str(&caller_data.address).unwrap(),
		MemoryAccount {
			nonce: caller_data.nonce,
			balance: caller_data.balance,
			// TODO: Actually use the storage from caller_storage, if EOA just empty BTreeMap, if not put the storage
			storage: BTreeMap::new(),
			// TODO: Actually use the code from caller_data, if EOA just empty Vec, if not put the code
			code: Vec::new(),
		},
	);

	let mut backend = MemoryBackend::new(&vicinity, global_state);
	let metadata = StackSubstateMetadata::new(u64::MAX, &config);
	let state = MemoryStackState::new(metadata, &mut backend);

	// 2. Prove that the initial state is valid wrt the program specification
	// 2.1 Verify that the program specification is valid
	for (condition, _) in &program_spec {
		// First, check that the fixed conditions are satisfied
		match condition {
			Condition::Fixed(condition) => {
				let result = check_fixed_condition(&state, &condition);
				assert!(result == true);
			}
			Condition::Relative(_) => {
				// Continue iterating, this doesnt need to be checked
			}
		}
	}

	// 2.2 Verify that the contract code is in the state
	let target_address = H160::from_str(&target_data.address).unwrap();
	let state_target_bytecode = state.code(target_address);
	assert!(!state_target_bytecode.is_empty());

	// 3. Execute the transaction (TODO: Contract call if caller is a contract)
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	let before = executor.balance(H160::from_str(&target_data.address).unwrap());
	let (exit_reason, _) = executor.transact_call(
		H160::from_str(&caller_data.address).unwrap(),
		H160::from_str(&target_data.address).unwrap(),
		U256::from_dec_str("0").unwrap(),
		hex::decode(calldata).unwrap(),
		u64::MAX,
		Vec::new(),
	);
	let after = executor.balance(H160::from_str(&target_data.address).unwrap());
	assert!(caller_data.address != target_data.address);

	// 4. Prove that the final state is invalid wrt the program specification
	let post_state = executor.state();

	// Get only conditions associated with the method_id called (first 4 bytes)
	println!("Program spec: {:?}", program_spec);
	let method_conditions: Vec<Condition> = program_spec.iter().filter_map(|(condition, method_id)| {
		println!("Method id: {:?}", method_id);
		println!("Calldata: {:?}", calldata);
		if method_id[..] == calldata[..8] {
			Some(condition.clone())
		} else {
			None
		}
	}).collect();

	println!("Method conditions: {:?}", method_conditions);
	// 4.1 If exit_succeed is true, then we have found an exploit
	let mut exit_succeed = false;
	for condition in method_conditions {
		match condition {
			Condition::Fixed(condition) => {
				println!("Checking fixed condition: {:?}", condition);
				let result = check_fixed_condition(&post_state, &condition);
				if !result {
					println!("Fixed condition failed: {:?}", condition);
					exit_succeed = true;
					break;
				}
			}
			Condition::Relative(condition) => {
				// TODO: Somehow use pre-state. Since executor "moved" the state we cannot borrow it again, but we need it to check the relative condition
				// 	 	 Perhaps create a copy of the state before the transaction and use that to check the relative condition. 
				//		 It seems that clone() is not implemented for MemoryStackState, so we need to figure out how to do this
				let result = check_relative_condition(&post_state, &post_state, &condition);
				if !result {
					exit_succeed = true;
					break;
				}
			}
		}		
	}

	if !exit_succeed {
		// TODO: Support "Finding a new condition" use case
		panic!("No exploit found");
	}

	// 5. Encrypt the calldata with the public key  
	// TODO: Implement this
	let encrypted_calldata = hex::encode(calldata);

	// 6. Prepare outputs
	/*
	Public inputs (can be seen as outputs, but for the zkVM are inputs)
		1. The public key of the protocol: $pk$
		2. The contract's address: $a$
		3. The encrypted calldata $Enc(c)$
		4. The hash of the program's specification used $SHA3(S)$
		5. The hash of the bytecode used $SHA3(b)$
		6. **(Optionally)** The hash of other used addresses and bytecodes $SHA3((a_1, b_1) || ... || (a_m, b_m))$
	*/
	// TODO: Implement this
	let mut outputs = Vec::new();
	outputs.push("pk".to_string());
	outputs.push(target_data.address);
	outputs.push(encrypted_calldata);
	outputs.push("SHA3(S)".to_string());
	outputs.push("SHA3(b)".to_string());
	outputs.push("SHA3((a_1, b_1) || ... || (a_m, b_m))".to_string());


	// constraint: transaction succeeded
	assert!(exit_reason == ExitReason::Succeed(ExitSucceed::Stopped));
	
	// println!("AFTER: {:?}", after);

	// simulataion outputs: the before and after hack balance of ETH of the target
	let mut outputs = Vec::new();
	
	outputs.push(before.to_string());
	outputs.push(after.to_string());

	outputs
}

#[cfg(test)]
mod tests {
    use super::*;
	
	#[test]
	fn evm_exploit_works() {
		let calldata = "16112c6c0000000000000000000000000000000000000000000000000000000000000001"; // exploit(true)
		let blockchain_settings = r#"
        {
			"gas_price": "0",
			"origin": "0x0000000000000000000000000000000000000000",
			"block_hashes": "[]",
			"block_number": "0",
			"block_coinbase": "0x0000000000000000000000000000000000000000",
			"block_timestamp": "0",
			"block_difficulty": "0",
			"block_gas_limit": "0",
			"chain_id": "1",
			"block_base_fee_per_gas": "0"
		}
    	"#;

		let result = run_evm(
			calldata,
			CallerData { 
				address: "E94f1fa4F27D9d288FFeA234bB62E1fBC086CA0c".to_string(),
				nonce: U256::one(),
				balance: U256::from_dec_str("10000000000000000000").unwrap(),
				storage: BTreeMap::new(),
				code: vec![],
			},
			TargetData {
				bytecode: TARGET_CONTRACT_BYTECODE.to_string(),
				address: "4838B106FCe9647Bdf1E7877BF73cE8B0BAD5f97".to_string(),
				nonce: U256::one(),
				balance: U256::from_dec_str("1000000000000000000").unwrap(),
				storage: BTreeMap::new(),
				code: vec![],
			},
			vec![
				// Program specification is a list of (condition, method) pairs
				// Where method is defined by its method id
				// and condition is a list of conditions that must be satisfied for the method to be executed
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
			],
			blockchain_settings
		);
		println!("Result [Balance before tx, Balance after tx]: {:?}", result);
		assert_eq!(result[0], "1000000000000000000"); // target should have 1 ethers before the exploit
		assert_eq!(result[1], "0"); // target should have 0 after the exploit
	}

}
