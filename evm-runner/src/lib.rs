#![cfg_attr(not(test),)]
pub mod conditions;
pub mod input;
pub mod context;

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
	}, Config, ExitReason, ExitSucceed
};
use input::AccountData;
use primitive_types::{U256, H160, H256};
use serde_json::from_str;
use serde::Deserialize;
use sha2::{Sha256, Digest};
use hpke::{Serializable,Kem};
use hpke::kem::X25519HkdfSha256;
use rand::rngs::StdRng;
use rand::SeedableRng;
use hex::encode;

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

// This function generates a new keypair (private key and public key) for the chosen KEM.
pub fn generate_keypair(seed: &[u8; 32]) -> (String, String) {
	// Create a deterministic RNG from the seed
	let mut rng = StdRng::from_seed(*seed);

	// Generate a key pair
	let (sk, pk) = X25519HkdfSha256::gen_keypair(&mut rng);

	// Convert to base64 for easy encoding
	let sk_b64 = encode(sk.to_bytes());
	let pk_b64 = encode(pk.to_bytes());

	(sk_b64, pk_b64)
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
	// TODO: See if its possible to use the MemoryStackState, for now use the BTreeMap instead
	pre_state: &BTreeMap<H160, MemoryAccount>,
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
	if !post_state.exists(pre_account_address) {
		panic!("Account does not exist: {}", pre_account_address);
	}

	// Get the account from the state
	let pre_account_basic = pre_state.get(&pre_account_address).unwrap();
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
	} else if state_key.len() == 3 {
        // We expect a format like: <account_address>.storage.<storage_key>
        if state_key[1] != "storage" {
            panic!("Expected 'storage' keyword in state key, got: {}", state_key[1]);
        }

        // Parse the third part as the storage key (which should be a 32-byte hex string)
        let storage_key = H256::from_str(state_key[2])
            .expect("Invalid storage key hex provided in state key");

        // Retrieve the storage value from the state.
        // (Assume your MemoryStackState has a method similar to `storage` that takes an account and a key)
        let storage_value = state.storage(account_address, storage_key);

		// TODO: See how to handle properly taking into account that we may want to compare non-numeric things
		let storage_value_as_u256 = U256::from_big_endian(&storage_value[..]);
        
        // Compare the storage value with the expected value using your condition operator.
        check_condition_op(&fixed_condition.op, storage_value_as_u256, fixed_condition.v)
    } else {
        panic!("Invalid state key format");
	}
}

fn from_deserialized_vicinity(deserialized_vicinity: DeserializeMemoryVicinity) -> MemoryVicinity {
	MemoryVicinity {
		gas_price: U256::from_str(&deserialized_vicinity.gas_price).unwrap(),
		origin: H160::from_str(&deserialized_vicinity.origin).unwrap(),
		chain_id: U256::from_str(&deserialized_vicinity.chain_id).unwrap(),
		block_hashes: serde_json::from_str::<Vec<String>>(&deserialized_vicinity.block_hashes).unwrap().into_iter().map(|s| H256::from_str(&s).unwrap()).collect(),
		block_number: U256::from_str(&deserialized_vicinity.block_number).unwrap(),
		block_coinbase: H160::from_str(&deserialized_vicinity.block_coinbase).unwrap(),
		block_timestamp: U256::from_str(&deserialized_vicinity.block_timestamp).unwrap(),
		block_difficulty: U256::from_str(&deserialized_vicinity.block_difficulty).unwrap(),
		block_gas_limit: U256::from_str(&deserialized_vicinity.block_gas_limit).unwrap(),
		block_base_fee_per_gas: U256::from_str(&deserialized_vicinity.block_base_fee_per_gas).unwrap(),
		block_randomness: H256::from_str("0x0").ok(), // TODO: Check if this is correct
	}
}

// TODO: Think if everything should be a vec<accountdata> directly
fn build_global_state(
	global_state: &mut BTreeMap<H160, MemoryAccount>,
	caller_data: &AccountData,
	target_data: &AccountData,
	context_data: &Vec<AccountData>,
) {
	// Target contract
	global_state.insert(
		H160::from_str(&target_data.address).unwrap(),
		MemoryAccount {
			nonce: target_data.nonce,
			balance: target_data.balance,
			storage: target_data.storage.clone(),
			code: target_data.code.clone(),
		}
	);

	// Caller EOA (TODO: Support contract callers)
	global_state.insert(
		H160::from_str(&caller_data.address).unwrap(),
		MemoryAccount {
			nonce: caller_data.nonce,
			balance: caller_data.balance,
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

	// Context data (other contracts)
	for cdata in context_data {
		global_state.insert(
			H160::from_str(&cdata.address).unwrap(),
			MemoryAccount {
				nonce: cdata.nonce,
				balance: cdata.balance,
				storage: cdata.storage.clone(),
				code: cdata.code.clone(),
			}
		);
	}
}

fn filter_program_spec(program_spec: &Vec<(Condition, String)>, method_id: &str) -> Vec<Condition> {
	program_spec.iter().filter_map(|(condition, method)| {
		if method[..] == method_id[..] {
			Some(condition.clone())
		} else {
			None
		}
	}).collect()
}

fn verify_pre_state(
	state: &MemoryStackState<MemoryBackend>,
	program_spec: &Vec<(Condition, String)>
) {
	for (condition, _) in program_spec {
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
}

fn prove_final_state(
	pre_state: &BTreeMap<H160, MemoryAccount>,
	post_state: &MemoryStackState<MemoryBackend>,
	method_conditions: &Vec<Condition>
) -> bool {
	let mut exit_succeed  = false;

	// First check if there is an exploit
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
				println!("Checking relative condition: {:?}", condition);
				let result = check_relative_condition(&pre_state, &post_state, &condition);
				if !result {
					println!("Relative condition failed: {:?}", condition);
					exit_succeed = true;
					break;
				}
			}
		}		
	}

	return exit_succeed;

}

fn hash(data: &Vec<u8>) -> [u8; 32] {
    let mut hasher = Sha256::new();
	hasher.update(data);
    hasher.finalize().into()
}

pub fn run_evm(
	calldata: &str,
	caller_data: AccountData,
	target_data: AccountData,
	context_data: Vec<AccountData>,
	program_spec: Vec<(Condition, String)>,
	blockchain_settings: &str
) -> Vec<String> {
	// 0. Preliminaries
	// 0.1 Initialize the EVM config
	let config = Config::cancun();
	// Deserialize vicinity
	let deserialize_vicinity: DeserializeMemoryVicinity = from_str(blockchain_settings).unwrap();
	let vicinity: MemoryVicinity = from_deserialized_vicinity(deserialize_vicinity);

	// 0.2 Sanity checks (TODO: Think of more sanity checks if needed)
	assert!(caller_data.address != target_data.address);

	// 1. Setup global state from caller_data and target_data
	let mut global_state: BTreeMap<H160, MemoryAccount> = BTreeMap::new();
	build_global_state(&mut global_state, &caller_data, &target_data, &context_data);
	let pre_state_tree = global_state.clone();

	let mut backend = MemoryBackend::new(&vicinity, global_state);
	let metadata = StackSubstateMetadata::new(u64::MAX, &config);
	let pre_state = MemoryStackState::new(metadata, &mut backend);

	// 2. Prove that the initial state is valid wrt the program specification
	// 2.1 Verify that the program specification is valid
	verify_pre_state(&pre_state, &program_spec);

	// 2.2 Verify that the contract code is in the state
	let target_address = H160::from_str(&target_data.address).unwrap();
	let state_target_bytecode = pre_state.code(target_address);
	assert!(!state_target_bytecode.is_empty());

	// 3. Execute the transaction (TODO: Contract call if caller is a contract)
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(pre_state, &config, &precompiles);

	let (exit_reason, _) = executor.transact_call(
		H160::from_str(&caller_data.address).unwrap(),
		H160::from_str(&target_data.address).unwrap(),
		U256::from_dec_str("0").unwrap(),
		hex::decode(calldata).unwrap(),
		u64::MAX,
		Vec::new(),
	);
	assert!(exit_reason == ExitReason::Succeed(ExitSucceed::Stopped));

	// 4. Prove that the final state is invalid wrt the program specification
	let post_state = executor.state();
	let method_id = &calldata[..8];
	let method_conditions: Vec<Condition> = filter_program_spec(&program_spec, method_id);

	// 4.1 Depending on the returning boolean value, we can determine if an exploit was found
	let exploit_found = prove_final_state(
		&pre_state_tree, 
		&post_state, 
		&method_conditions
	);
	
	println!("Exploit found: {:?}", exploit_found);

	if !exploit_found {
		panic!("No exploit found");
	}

	// 6. Prepare outputs
	let mut outputs = Vec::new();

	// 6.0 Exploit found and new condition found flag
	outputs.push(exploit_found.to_string());

	// 6.1 The hash of the program's specification used $SHA3(S)$
	let hashed_program_spec = conditions::hash_program_spec(&program_spec);
	outputs.push(hex::encode(hashed_program_spec));

	// 6.2 The hash of the bytecode used $SHA3(b)$
	let hashed_bytecode = hash(&target_data.code);
	outputs.push(hex::encode(hashed_bytecode));

	// 6.3 The hash of other used bytecodes $SHA3((b_1) || ... || (a_m, b_m))$
	let mut bytecode_list: Vec<String> = Vec::new();
	for cdata in context_data {
		bytecode_list.push(hex::encode(cdata.code));
	}
	let joined_context_data_hashes = bytecode_list.join("");
	let hashed_list = hash(&joined_context_data_hashes.as_bytes().to_vec());
	outputs.push(hex::encode(hashed_list));

	// 6.4 The address of the prover
	// TODO: For now we use the caller, but we could use a different address sent as a parameter
	//		 specially when supporting contract callers
	let prover_address = H160::from_str(&caller_data.address).unwrap();
	outputs.push(prover_address.to_string());

	outputs
}

#[cfg(test)]
mod tests {
    use conditions::compute_mapping_storage_key;
    use context::{build_context_account_data, ContextAccountDataType};
    use super::*;

	#[test]
	fn evm_find_new_exploit_balance_works() {
		/*
		(**Finding a new exploit**) Calling the method ```exploit``` and showing that if there existed a condition $C_j$ (where currently $C_j \notin S$), then the program specification would not comply with the end state $s'$
		 */
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

		let program_spec = vec![
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
		];

		let result = run_evm(
			calldata,
			AccountData { 
				address: "E94f1fa4F27D9d288FFeA234bB62E1fBC086CA0c".to_string(),
				nonce: U256::one(),
				balance: U256::from_dec_str("10000000000000000000").unwrap(),
				storage: BTreeMap::new(),
				code: vec![],
			},
			AccountData {
				address: "4838B106FCe9647Bdf1E7877BF73cE8B0BAD5f97".to_string(),
				nonce: U256::one(),
				balance: U256::from_dec_str("1000000000000000000").unwrap(),
				storage: BTreeMap::new(),
				code: hex::decode(TARGET_CONTRACT_BYTECODE).unwrap(),
			},
			vec![],
			program_spec.clone(),
			blockchain_settings
		);
		println!("Result [Balance before tx, Balance after tx]: {:?}", result);
		assert_eq!(result[0], "true"); // exploit should be found

		let hashed_program_spec = conditions::hash_program_spec(&program_spec);
		assert_eq!(result[1], hex::encode(hashed_program_spec));

		let hashed_bytecode = hash(&hex::decode(TARGET_CONTRACT_BYTECODE).unwrap());
		assert_eq!(result[2], hex::encode(hashed_bytecode));

		let hashed_context_data = hash(&"".as_bytes().to_vec());
		assert_eq!(result[3], hex::encode(hashed_context_data));

		let prover_address = H160::from_str("E94f1fa4F27D9d288FFeA234bB62E1fBC086CA0c").unwrap();
		assert_eq!(result[4], prover_address.to_string());
		
	}

	#[test]
	fn evm_find_new_exploit_erc20_works() {
		/*
		(**Finding a new exploit**) Calling the method ```exploit_erc20``` and showing that if there existed a condition $C_j$ (where currently $C_j \notin S$), then the program specification would not comply with the end state $s'$
		 */
		 let calldata = "d92dbd190000000000000000000000000000000000000000000000000000000000000001"; // exploit_erc20(true)
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

		 // First, compute the storage key for the balances mapping at the target address
		 let computed_storage_key = compute_mapping_storage_key(
			H160::from_str("0x4838B106FCe9647Bdf1E7877BF73cE8B0BAD5f97").unwrap(), // mapping key (address)
			U256::from(0) // the base slot for the mapping (See ContextERC20-storageLayout.json)
		);

		 // Initialize the init storage such that the target address has a balance of 100
		 let mut context_erc20_init_storage: BTreeMap<H256, H256> = BTreeMap::new();
		 let amount: H256 = H256::from_low_u64_be(100000);
		 context_erc20_init_storage.insert(computed_storage_key, amount);
		 
		 let erc20_account_data = build_context_account_data(
			 ContextAccountDataType::ERC20,
			 Some(context_erc20_init_storage)
		 );
		let state_path = format!("{}.storage.{}", erc20_account_data.address, hex::encode(computed_storage_key));
		
		let program_spec = vec![
			 // Program specification is a list of (condition, method) pairs
			 // Where method is defined by its method id
			 // and condition is a list of conditions that must be satisfied for the method to be executed
			 (
				 Condition::Fixed(
					// TODO: Fix this, see https://chatgpt.com/share/67b111b2-313c-800e-9131-e08bc93175bb
					 FixedCondition {
						 k_s: state_path,
						 op: Operator::Gt,
						 v: U256::from_dec_str("0").unwrap(),
					 }
				 ),
				 "d92dbd19".to_string()
			 )
		 ];
			 
		let context_data = vec![erc20_account_data];
		 let result = run_evm(
			 calldata,
			 AccountData { 
				 address: "E94f1fa4F27D9d288FFeA234bB62E1fBC086CA0c".to_string(),
				 nonce: U256::one(),
				 balance: U256::from_dec_str("10000000000000000000").unwrap(),
				 storage: BTreeMap::new(),
				 code: vec![],
			 },
			 AccountData {
				 address: "4838B106FCe9647Bdf1E7877BF73cE8B0BAD5f97".to_string(),
				 nonce: U256::one(),
				 balance: U256::from_dec_str("0").unwrap(),
				 storage: BTreeMap::new(),
				 code: hex::decode(TARGET_CONTRACT_BYTECODE).unwrap(),
			 },
			 context_data.clone(),
			 program_spec.clone(),
			 blockchain_settings
		 );

		 println!("Result [Balance before tx, Balance after tx]: {:?}", result);
		 assert_eq!(result[0], "true"); // exploit should be found
 
		 let hashed_program_spec = conditions::hash_program_spec(&program_spec);
		 assert_eq!(result[1], hex::encode(hashed_program_spec));
 
		 let hashed_bytecode = hash(&hex::decode(TARGET_CONTRACT_BYTECODE).unwrap());
		 assert_eq!(result[2], hex::encode(hashed_bytecode));
 
		let mut context_data_bytecode_list: Vec<String> = Vec::new();
		for cdata in context_data.clone() {
			context_data_bytecode_list.push(hex::encode(cdata.code));
		}
		let joined_context_data_hashes = context_data_bytecode_list.join("");
		let hashed_list = hash(&joined_context_data_hashes.as_bytes().to_vec());
			let hashed_context_data = hex::encode(hashed_list);
			assert_eq!(result[3], hashed_context_data);

		let prover_address = H160::from_str("E94f1fa4F27D9d288FFeA234bB62E1fBC086CA0c").unwrap();
		assert_eq!(result[4], prover_address.to_string());
			
	}
}
