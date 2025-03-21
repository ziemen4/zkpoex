use evm::backend::Basic;
use shared::context;
use shared::conditions;
use shared::input::AccountData;
extern crate alloc;
extern crate core;

use crate::alloc::string::ToString;
use core::str::FromStr;

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use conditions::{Condition, FixedCondition, Operator, RelativeCondition};
use evm::{
    backend::{Backend, MemoryAccount, MemoryBackend, MemoryVicinity},
    executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata},
    Config, ExitReason, ExitSucceed,
};
use hex::encode;
use hpke::kem::X25519HkdfSha256;
use hpke::{Kem, Serializable};
use primitive_types::{H160, H256, U256, U512};
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::Deserialize;
use serde_json::from_str;

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

// Address `0x7A46E70000000000000000000000000000000000` is reserved for the `Target` contract.
const TARGET_ADDRESS: &str = "7A46E70000000000000000000000000000000000";
// Address `0xCA11E40000000000000000000000000000000000` is reserved for the `Caller`
const CALLER_ADDRESS: &str = "CA11E40000000000000000000000000000000000";
/*
Arbitrary contracts (ranging through 000 to fff, in total 4096 addresses) are reserved for the prover 
to deploy arbitrary contracts.
From `0x1000000000000000000000000000000000000000` to `0x1000000000000000000000000000000000000fff`:
*/
// Instead of defining each arbitrary address, just provide the gap (they are sequential).
const RESERVED_ARBITRARY_ADDRESSES: (&str,  &str) = (
    "1000000000000000000000000000000000000000",
    "1000000000000000000000000000000000000fff"
);

// Address `0xE4C2000000000000000000000000000000000000` is reserved for the `ContextTemplateERC20` contract.
const RESERVED_TEMPLATE_ADDRESSES: [&str; 1] = ["E4C2000000000000000000000000000000000000"];

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

fn check_condition_op<T: PartialOrd + std::fmt::Debug>(operator: &Operator, first_val: T, second_val: T) -> bool {
    println!("FIRST VAL: {:?}", first_val);
    println!("SECOND VAL: {:?}", second_val);
    println!("OPERATOR: {:?}", operator);
    match operator {
        Operator::Eq => first_val == second_val,
        Operator::Neq => first_val != second_val,
        Operator::Gt => first_val > second_val,
        Operator::Ge => first_val >= second_val,
        Operator::Lt => first_val < second_val,
        Operator::Le => first_val <= second_val,
    }
}

fn get_storage_value(
    state: &MemoryStackState<MemoryBackend>,
    state_key: Vec<&str>,
) -> U256 {
    // We expect a format like: <account_address>.storage.<storage_key>
    let address = H160::from_str(state_key[0]).unwrap();
    if state_key[1] != "storage" {
        panic!(
            "Expected 'storage' keyword in state key, got: {}",
            state_key[1]
        );
    }

    // Parse the third part as the storage key (which should be a 32-byte hex string)
    let storage_key =
        H256::from_str(state_key[2]).expect("Invalid storage key hex provided in state key");

    // Retrieve the storage value from the state.
    // (Assume your MemoryStackState has a method similar to `storage` that takes an account and a key)
    let storage_value = state.storage(address, storage_key);

    // TODO: See how to handle properly taking into account that we may want to compare non-numeric things
    let storage_value_as_u256 = U256::from_big_endian(&storage_value[..]);

    storage_value_as_u256    
}

fn get_state_value(
    state: &MemoryStackState<MemoryBackend>,
    state_key: Vec<&str>,
    account: &Basic,
) -> U256 {
    // If the state key vector has length 2, then we are accessing the account fields directly
    if state_key.len() == 2 {
        match state_key[1] {
            "nonce" => account.nonce,
            "balance" => account.balance,
            _ => {
                panic!("Invalid state key: {}", state_key[1]);
            }
        }
    } else if state_key.len() == 3 {
        // If the state key vector has length 3, then we expect a format like: <account_address>.storage.<storage_key>
        get_storage_value(
            state,
            state_key,
        )
    } else {
        panic!("Invalid state key format");
    }
}

fn check_relative_condition(
    // TODO: See if its possible to use the MemoryStackState, for now use the BTreeMap instead
    pre_state: &MemoryStackState<MemoryBackend>,
    post_state: &MemoryStackState<MemoryBackend>,
    relative_condition: &RelativeCondition,
) -> bool {
    // Pre state key is joined by '.', so split it
    let pre_state_key = relative_condition.k_s.split('.').collect::<Vec<&str>>();

    // Post state key is joined by '.', so split it
    let post_state_key = relative_condition
        .k_s_prime
        .split('.')
        .collect::<Vec<&str>>();

    // The first state key is always the address of the account
    let pre_account_address = H160::from_str(pre_state_key[0]).unwrap();
    let post_account_address = H160::from_str(post_state_key[0]).unwrap();

    // If account does not exist, panic
    if !post_state.exists(post_account_address) {
        panic!("Account does not exist: {}", post_account_address);
    }

    if !pre_state.exists(pre_account_address) {
        panic!("Account does not exist: {}", pre_account_address);
    }

    // Get the account from the state
    let pre_account_basic = pre_state.basic(pre_account_address);
    let post_account_basic = post_state.basic(post_account_address);

    println!("PRE ACCOUNT BASIC BALANCE {:?}", pre_account_basic);
	println!("POST ACCOUNT BASIC BALANCE {:?}", post_account_basic);

    let pre_state_val = get_state_value(
        pre_state,
        pre_state_key,
        &pre_account_basic,
    );
    let post_state_val = get_state_value(
        post_state,
        post_state_key,
        &post_account_basic,
    );

    check_condition_op(&relative_condition.op, pre_state_val, post_state_val)
}

fn check_fixed_condition(
    state: &MemoryStackState<MemoryBackend>,
    fixed_condition: &FixedCondition,
) -> bool {
    // State key is joined by '.', so split it
    let state_key = fixed_condition.k_s.split('.').collect::<Vec<&str>>();
    println!("\n\n---------------------------\n\n");
    println!("State key: {:?}", state_key);
    // The first state key is always the address of the account
    let account_address = H160::from_str(state_key[0]).unwrap();

    // If account does not exist, panic
    if !state.exists(account_address) {
        panic!("Account does not exist: {}", account_address);
    }

    // Get the account from the state
    let account_basic = state.basic(account_address);
    println!("Account basic: {:?}", account_basic);
    println!("Value of fixed_cond {:?}", fixed_condition.v,);
    println!("\n\n---------------------------\n\n");

    let state_value = get_state_value(
        state,
        state_key,
        &account_basic,
    );

    check_condition_op(
        &fixed_condition.op,
        state_value,
        fixed_condition.v,
    )
}

fn from_deserialized_vicinity(deserialized_vicinity: DeserializeMemoryVicinity) -> MemoryVicinity {
    MemoryVicinity {
        gas_price: U256::from_str(&deserialized_vicinity.gas_price).unwrap(),
        origin: H160::from_str(&deserialized_vicinity.origin).unwrap(),
        chain_id: U256::from_str(&deserialized_vicinity.chain_id).unwrap(),
        block_hashes: serde_json::from_str::<Vec<String>>(&deserialized_vicinity.block_hashes)
            .unwrap()
            .into_iter()
            .map(|s| H256::from_str(&s).unwrap())
            .collect(),
        block_number: U256::from_str(&deserialized_vicinity.block_number).unwrap(),
        block_coinbase: H160::from_str(&deserialized_vicinity.block_coinbase).unwrap(),
        block_timestamp: U256::from_str(&deserialized_vicinity.block_timestamp).unwrap(),
        block_difficulty: U256::from_str(&deserialized_vicinity.block_difficulty).unwrap(),
        block_gas_limit: U256::from_str(&deserialized_vicinity.block_gas_limit).unwrap(),
        block_base_fee_per_gas: U256::from_str(&deserialized_vicinity.block_base_fee_per_gas)
            .unwrap(),
        block_randomness: H256::from_str("0x0").ok(), // TODO: Check if this is correct
    }
}

fn build_global_state(
    global_state: &mut BTreeMap<H160, MemoryAccount>,
    context_state: &Vec<AccountData>,
) {
    for cdata in context_state {
        global_state.insert(
            H160::from_str(&cdata.address).unwrap(),
            MemoryAccount {
                nonce: cdata.nonce,
                balance: cdata.balance,
                storage: cdata.storage.clone(),
                code: cdata.code.clone(),
            },
        );
    }
}

fn filter_program_spec(program_spec: &Vec<(Condition, String)>, method_id: &str) -> Vec<Condition> {
    program_spec
        .iter()
        .filter_map(|(condition, method)| {
            if method[..] == method_id[..] {
                Some(condition.clone())
            } else {
                None
            }
        })
        .collect()
}

fn verify_pre_state(
    state: &MemoryStackState<MemoryBackend>,
    program_spec: &Vec<(Condition, String)>,
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
    pre_state: &MemoryStackState<MemoryBackend>,
    post_state: &MemoryStackState<MemoryBackend>,
    method_conditions: &Vec<Condition>,
) -> bool {
    let mut exit_succeed = false;

    // First check if there is an exploit
    for condition in method_conditions {
        match condition {
            Condition::Fixed(condition) => {
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

fn build_memory_stack_state<'a>(
    context_state: &'a Vec<AccountData>,
    config: &'a Config,
    backend: &'a mut MemoryBackend<'a>,
) -> MemoryStackState<'a, 'a, MemoryBackend<'a>> {
    let mut global_state: BTreeMap<H160, MemoryAccount> = BTreeMap::new();
    build_global_state(&mut global_state, &context_state);

    let metadata = StackSubstateMetadata::new(u64::MAX, &config);
    let state = MemoryStackState::new(metadata, backend);

    state
}

pub fn run_evm(
    calldata: &str,
    context_state: Vec<AccountData>,
    program_spec: Vec<(Condition, String)>,
    blockchain_settings: &str
) -> Vec<String> {
    // 0. Preliminaries
    // 0.1 Initialize the EVM config
    let config = Config::cancun();
    
    // Deserialize vicinity
    let deserialize_vicinity: DeserializeMemoryVicinity = from_str(blockchain_settings).unwrap();
    let vicinity: MemoryVicinity = from_deserialized_vicinity(deserialize_vicinity);
    println!("Vicinity: {:?}", vicinity);

    // 0.1 Obtain the caller, target and context data
    let target_data = context_state[0].clone();
    let caller_data = context_state[1].clone();

    // 0.2 Pre-checks
    assert!(caller_data.address == CALLER_ADDRESS);
    assert!(target_data.address == TARGET_ADDRESS);
    
    // TODO: Alternatively, if this is too costly, we could allow the program spec
    //       to specify "any()" as the address, and have the prover just show that some address
    //       satisfies the condition.
    let lower_bound_reserved_address: U256 = U256::from_str(&RESERVED_ARBITRARY_ADDRESSES.0[2..]).unwrap();
    let upper_bound_reserved_address: U256 = U256::from_str(&RESERVED_ARBITRARY_ADDRESSES.1[2..]).unwrap();

    for account_data in context_state.clone() {
        let account_address = H160::from_str(&account_data.address).unwrap();
        let account_address_int = U256::from_big_endian(&account_address[..]);
        if account_address_int >= lower_bound_reserved_address || account_address_int <= upper_bound_reserved_address {
            continue;
        }
        assert!(!RESERVED_TEMPLATE_ADDRESSES.contains(&account_data.address.as_str()));
        // TODO: For each reserved template address, assert that the correct bytecode is present
    }

    // 1. Setup global state from caller_data and target_data
    let mut global_state: BTreeMap<H160, MemoryAccount> = BTreeMap::new();
    build_global_state(&mut global_state, &context_state);
    let mut backend = MemoryBackend::new(&vicinity, global_state);
    let pre_state = build_memory_stack_state(
        &context_state,
        &config,
        &mut backend,
    );

    // 1.1 Create a snapshot of the pre state for further use
    // TOOD: See if there is a better way to avoid cloning the global state from scratch
    let mut snapshot_global_state: BTreeMap<H160, MemoryAccount> = BTreeMap::new();
    build_global_state(&mut snapshot_global_state, &context_state);
    let mut snapshot_backend = MemoryBackend::new(&vicinity, snapshot_global_state);
    let pre_state_snapshot = build_memory_stack_state(
        &context_state,
        &config,
        &mut snapshot_backend,
    );

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
    println!("Exit reason: {:?}", exit_reason);

    assert!(matches!(exit_reason, ExitReason::Succeed(ExitSucceed::Stopped) | ExitReason::Succeed(ExitSucceed::Returned)));
    
    // 4. Prove that the final state is invalid wrt the program specification
    let post_state = executor.state();
    let method_id = &calldata[..8];
    let method_conditions: Vec<Condition> = filter_program_spec(&program_spec, method_id);
    println!("Method conditions: {:?}", method_conditions);

    // 4.1 Depending on the returning boolean value, we can determine if an exploit was found
    let exploit_found = prove_final_state(&pre_state_snapshot, &post_state, &method_conditions);

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

    // 6.2 The hash of context state data $SHA3((a_1, b_1) || ... || (a_n, b_n))$
    let hashed_context_state = context::hash_context_state(&context_state);
    outputs.push(hex::encode(hashed_context_state));

    // 6.3 The address of the prover
    // TODO: For now we use the caller, but we could use a different address sent as a parameter
    // specially when supporting contract callers
    let prover_address = H160::from_str(&caller_data.address).unwrap();
    outputs.push(prover_address.to_string());

    outputs
}

#[cfg(test)]
mod tests {
    use super::*;
    use conditions::compute_mapping_storage_key;
    use context::{build_context_account_data, ContextAccountDataType};
    use shared::conditions::compute_storage_key;

    pub const BASIC_VULNERABLE_CONTRACT_BYTECODE: &str = include_str!("../../bytecode/BasicVulnerable.bin-runtime");
    pub const OUFLOW_CONTRACT_BYTECODE: &str = include_str!("../../bytecode/OverUnderFlowVulnerable.bin-runtime");

    #[test]
    fn evm_find_new_exploit_target_contract_works() {
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
                Condition::Fixed(FixedCondition {
                    k_s: "7A46E70000000000000000000000000000000000.balance".to_string(),
                    op: Operator::Gt,
                    v: U256::from_dec_str("0").unwrap(),
                }),
                "16112c6c".to_string(),
            ),
        ];
        let context_state = vec![
            AccountData {
                address: "7A46E70000000000000000000000000000000000".to_string(),
                nonce: U256::one(),
                balance: U256::from_dec_str("1000000000000000000").unwrap(),
                storage: BTreeMap::new(),
                code: hex::decode(BASIC_VULNERABLE_CONTRACT_BYTECODE).unwrap(),
            },
			AccountData {
                address: "CA11E40000000000000000000000000000000000".to_string(),
                nonce: U256::one(),
                balance: U256::from_dec_str("10000000000000000000").unwrap(),
                storage: BTreeMap::new(),
                code: vec![],
            },
        ];

        let result = run_evm(
            calldata,
            context_state.clone(),
            program_spec.clone(),
            blockchain_settings,
        );
        println!("Result: {:?}", result);
        assert_eq!(result[0], "true"); // exploit should be found

        let hashed_program_spec = conditions::hash_program_spec(&program_spec);
        assert_eq!(result[1], hex::encode(hashed_program_spec));

        let hashed_context_data = context::hash_context_state(&context_state);
        assert_eq!(result[2], hex::encode(hashed_context_data));

        let prover_address = H160::from_str("CA11E40000000000000000000000000000000000").unwrap();
        assert_eq!(result[3], prover_address.to_string());
    }

    
    #[test]
    fn evm_find_new_exploit_target_contract_erc20_works() {
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
            H160::from_str("0x7A46E70000000000000000000000000000000000").unwrap(), // mapping key (address)
            U256::from(0), // the base slot for the mapping (See ContextTemplateERC20-storageLayout.json)
        );

        // Initialize the init storage such that the target address has a balance of 100
        let mut context_erc20_init_storage: BTreeMap<H256, H256> = BTreeMap::new();
        let amount: H256 = H256::from_low_u64_be(100000);
        context_erc20_init_storage.insert(computed_storage_key, amount);

        let erc20_account_data = build_context_account_data(
            ContextAccountDataType::ERC20,
            Some(context_erc20_init_storage),
        );
        let state_path = format!(
            "{}.storage.{}",
            erc20_account_data.address,
            hex::encode(computed_storage_key)
        );

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
                    },
                ),
                "d92dbd19".to_string(),
            ),
        ];

        let context_state = vec![
            AccountData {
                address: "7A46E70000000000000000000000000000000000".to_string(),
                nonce: U256::one(),
                balance: U256::from_dec_str("0").unwrap(),
                storage: BTreeMap::new(),
                code: hex::decode(BASIC_VULNERABLE_CONTRACT_BYTECODE).unwrap(),
            },
			AccountData {
                address: "CA11E40000000000000000000000000000000000".to_string(),
                nonce: U256::one(),
                balance: U256::from_dec_str("10000000000000000000").unwrap(),
                storage: BTreeMap::new(),
                code: vec![],
            },
            erc20_account_data,
        ];
        let result = run_evm(
            calldata,
            context_state.clone(),
            program_spec.clone(),
            blockchain_settings,
        );

        println!("Result: {:?}", result);
        assert_eq!(result[0], "true"); // exploit should be found

        let hashed_program_spec = conditions::hash_program_spec(&program_spec);
        assert_eq!(result[1], hex::encode(hashed_program_spec));

        let hashed_context_state = context::hash_context_state(&context_state);
        assert_eq!(result[2], hex::encode(hashed_context_state));

        let prover_address = H160::from_str("CA11E40000000000000000000000000000000000").unwrap();
        assert_eq!(result[3], prover_address.to_string());
    }

    #[test]
    fn evm_find_new_exploit_over_under_flow_storage_works() {
        let calldata = "2e1a7d4d00000000000000000000000000000000000000000000000000000000000003e9"; // withdraw(1001) -> 3e9 at the end does not work. With withdraw(1) yes, seems like balance is always 0
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
        let computed_storage_key = compute_storage_key(0); // the "balance" variable is at slot 0

        // Initialize the init storage such that the target address has a balance of 1000
        let mut over_underflow_init_storage: BTreeMap<H256, H256> = BTreeMap::new();
        let amount: H256 = H256::from_low_u64_be(1000);
        over_underflow_init_storage.insert(computed_storage_key, amount);

        let state_path = format!(
            "{}.storage.{}",
            "7A46E70000000000000000000000000000000000",
            hex::encode(computed_storage_key)
        );

        let program_spec = vec![
            // Program specification is a list of (condition, method) pairs
            // Where method is defined by its method id
            // and condition is a list of conditions that must be satisfied for the method to be executed
            (
                Condition::Fixed(FixedCondition {
                    k_s: state_path,
                    op: Operator::Neq,
                    v: U256::from_dec_str("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap(), //maximum representable value of uint256
                }),
                "2e1a7d4d".to_string(),
            ),
        ];
        
        let context_state = vec![
            AccountData {
                address: "7A46E70000000000000000000000000000000000".to_string(),
                nonce: U256::one(),
                balance: U256::from_dec_str("0").unwrap(),
                storage: over_underflow_init_storage,
                code: hex::decode(OUFLOW_CONTRACT_BYTECODE).unwrap(),
            },
			AccountData {
                address: "CA11E40000000000000000000000000000000000".to_string(),
                nonce: U256::one(),
                balance: U256::from_dec_str("10000000000000000000").unwrap(),
                storage: BTreeMap::new(),
                code: vec![],
            },
        ];

        let result = run_evm(
            calldata,
            context_state.clone(),
            program_spec.clone(),
            blockchain_settings,
        );
        println!("Result: {:?}", result);
        assert_eq!(result[0], "true"); // exploit should be found

        let hashed_program_spec = conditions::hash_program_spec(&program_spec);
        assert_eq!(result[1], hex::encode(hashed_program_spec));

        let hashed_context_data = context::hash_context_state(&context_state);
        assert_eq!(result[2], hex::encode(hashed_context_data));

        let prover_address = H160::from_str("CA11E40000000000000000000000000000000000").unwrap();
        assert_eq!(result[3], prover_address.to_string());
    }
}
