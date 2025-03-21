//PLEASE NOTE: We can create other .rs files in the shared/src/ directory and import them here or in other modules if this "template" becomes too large.


/////////////////////////////////////////////////////////////////////////////
                            //HOST MODULES//
/////////////////////////////////////////////////////////////////////////////

pub mod utils {
    use tiny_keccak::{Hasher, Keccak};
    use clap::{Arg, Command};
    use crate::conditions::{FixedCondition,Condition,Operator};
    use std::path::PathBuf;
    use primitive_types::{U256,H256};
    use std::str;

    /// -------------------------------------------
    /// Generates the function signature in Solidity ABI format.
    /// Example: `exploit(true)` -> `16112c6c0000000000000000000000000000000000000000000000000000000000000001`
    /// -------------------------------------------
    pub fn generate_function_signature(function_name: &str, params: &[&str]) -> String {
        let mut hasher = Keccak::v256();
        hasher.update(function_name.as_bytes());
        let mut output = [0u8; 32];
        hasher.finalize(&mut output);

        let selector = &output[..4];

        let encoded_params = params
            .iter()
            .map(|param| encode_abi_param(param))
            .collect::<Vec<String>>()
            .join("");

        format!("{}{}", hex::encode(selector), encoded_params)
    }

    /// -------------------------------------------
    /// Encodes a parameter according to Solidity ABI rules.
    /// Supports basic types like `bool`, `uint256`, `address`, etc.
    /// -------------------------------------------
    fn encode_abi_param(param: &str) -> String {
        if param == "true" || param == "false" {
            let value = if param == "true" { 1 } else { 0 }; //for bool
            format!("{:0>64x}", value)
        } else if param.starts_with("0x") {
            let mut padded = param[2..].to_string(); //for address
            padded = format!("{:0>64}", padded); 
            padded
        } else if let Ok(number) = param.parse::<u128>() {
            format!("{:0>64x}", number) //for numbers
        } else {
            panic!("Unsupported parameter type: {}", param);
        }
    }

    /// -------------------------------------------
    /// Parses the condition string and returns a Condition struct.
    /// Example: "balance > 0" in the CLI -> Condition::Fixed(FixedCondition { k_s: "address.balance", op: Operator::Gt, v: 0 })
    /// -------------------------------------------
    pub fn parse_condition(condition: &str, td_address: &str) -> Condition {
        let parts: Vec<&str> = condition.split_whitespace().collect();
        if parts.len() != 3 {
            panic!("Invalid condition format: {}", condition);
        }

        let key = parts[0];
        let op = match parts[1] {
            ">" => Operator::Gt,
            "<" => Operator::Lt,
            ">=" => Operator::Ge,
            "<=" => Operator::Le,
            "==" => Operator::Eq,
            "!=" => Operator::Neq,
            _ => panic!("Unsupported operator: {}", parts[1]),
        };
        let value = U256::from_dec_str(parts[2]).expect("Invalid value in condition");

        if key.starts_with("storage") {
            let storage_key = key.trim_start_matches("storage.");
            Condition::Fixed(FixedCondition {
                k_s: format!("{}.storage.{}", td_address, storage_key),
                op,
                v: value,
            })
        } else if key.starts_with("balance") {
            Condition::Fixed(FixedCondition {
                k_s: format!("{}.{}", td_address, key),
                op,
                v: value,
            })
        } else if key.starts_with("var_") {

            let var_name = key.trim_start_matches("var_"); 
            Condition::Fixed(FixedCondition {
                k_s: format!("{}.var_{}", td_address, var_name),
                op,
                v: value,
            })
        }
        else {
            panic!("Unsupported key in condition: {}", key);
        }
    }

    pub fn extract_key_from_condition(condition: &str) -> String {
        let parts: Vec<&str> = condition.split_whitespace().collect();
        if parts.len() != 3 {
            panic!("Invalid condition format: {}", condition);
        }
        let key = parts[0];
        key.to_string()
    }

    /// -------------------------------------------
    /// Hashes a string using the Keccak256 algorithm.
    /// -------------------------------------------
    pub fn keccak256(input: &str) -> H256 {
        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32];

        hasher.update(input.as_bytes());
        hasher.finalize(&mut output);
        
        H256::from(output)
    }

    /// -------------------------------------------
    /// Parses CLI arguments and returns the matches.
    /// -------------------------------------------
    pub fn parse_cli_args() -> clap::ArgMatches {
        Command::new("zkpoex-cli")
            .version("1.0")
            .author("Your Name <your.email@example.com>")
            .about("Generates zk proofs for Ethereum smart contract exploits")
            .arg(Arg::new("function")
                .short('f')
                .long("function")
                .value_name("FUNCTION")
                .help("Sets the function name (e.g., 'exploit')")
                .required(true))
            .arg(Arg::new("params") 
                .short('p')
                .long("params")
                .value_name("PARAMS")
                .help("Sets the function parameters (e.g., 'true')")
                .required(true))
            .arg(Arg::new("conditions")
                .short('c')
                .long("conditions")
                .value_name("CONDITIONS")
                .help("Sets the conditions for the exploit (e.g., 'balance > 0')")
                .required(true))
            .arg(Arg::new("contract-bytecode") 
                .short('b')
                .long("contract-bytecode")
                .value_name("BYTECODE_FILE")
                .help("Sets the contract bytecode file path")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf)))
            .arg(Arg::new("network")
                .short('n')
                .long("network")
                .value_name("NETWORK")
                .help("Specify if running on --testnet or --mainnet")
                .required(false))
            .arg(Arg::new("abi")
                .short('a')
                .long("abi")
                .value_name("ABI")
                .help("Sets the ABI file path")
                .required(false))
            .get_matches()
    }

}

pub mod evm_utils {
    use std::collections::HashMap;
    use std::process::Command as ProcessCommand;
    use std::str;
    use serde_json::Value;
    use serde_json;
    use std::fs;
    use primitive_types::H256;
    use std::collections::BTreeMap;
    use std::error::Error;
    use serde_json::from_str;
    use hex::FromHexError;
    /// -------------------------------------------
    /// Executes a cast command and returns the output as a String
    /// -------------------------------------------
    fn run_cast_command(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
        println!("--------------------------------------------------");
        println!("Executing command: cast {:?}", args);
        println!("--------------------------------------------------");
        let output = ProcessCommand::new("cast").args(args).output()?;
        if output.status.success() {
        Ok(str::from_utf8(&output.stdout)?.trim().to_string())
        } else {
            Err(format!("Command failed: {}", str::from_utf8(&output.stderr)?).into())
        }
    }

    /// -------------------------------------------
    /// Executes a solc command and returns the output as a String
    /// -------------------------------------------
    fn run_solc_command(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
        println!("--------------------------------------------------");
        println!("Executing command: solc {:?}", args);
        println!("--------------------------------------------------");
        let output = ProcessCommand::new("solc").args(args).output()?;
        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?.trim().to_string())
        } else {
            Err(format!("Command failed: {}", String::from_utf8(output.stderr)?).into())
        }
    }

    /// -------------------------------------------
    /// Get the storage layout for a given contract
    /// -------------------------------------------
    fn get_storage_layout(contract_file: &str) -> Result<HashMap<String, usize>, Box<dyn std::error::Error>> {
        let output = run_solc_command(&[
            "--storage-layout",  contract_file
        ])?;

        // Get the storage layout from the output parsing in json format
        let storage_layout_start : Vec<&str> = output.split("\n").collect();
        let json_string = storage_layout_start[2];
        let json: Value = from_str(&json_string)?;
        let storage_layout = &json["storage"];

        // Parse the storage layout and store it in a HashMap
        let mut storage_slots = HashMap::new();
        if let Some(entries) = storage_layout.as_array() {
            for entry in entries {
                if let Some(label) = entry["label"].as_str() {
                    if let Some(slot) = entry["slot"].as_str() {
                        let slot_number = slot.parse::<usize>()?;
                        storage_slots.insert(label.to_string(), slot_number);
                    }
                }
            }
        }
        Ok(storage_slots)
    }

    /// -------------------------------------------
    /// Get the storage slots for a given set of variables in a contract
    /// -------------------------------------------
    pub fn get_storage_slots_for_variables(
        contract_file: &str,
        variables: &HashMap<String, String>
    ) -> Result<HashMap<String, usize>, Box<dyn std::error::Error>> {
        let storage_layout = get_storage_layout(contract_file)?;
        let mut result = HashMap::new();
        for (var_name, _) in variables {
            if let Some(&slot) = storage_layout.get(var_name) {
                result.insert(var_name.clone(), slot);
            } else {
                return Err(format!("Variable '{}' not found in storage layout", var_name).into());
            }
        }

        Ok(result)
    }

    /// -------------------------------------------
    /// Sends a transaction to a target contract with a given calldata from a caller address
    /// -------------------------------------------
    pub fn send_transaction_with_calldata(
        target_address: &str, 
        private_key: &str, 
        calldata: &str
    ) -> Result<String, Box<dyn std::error::Error>> {
        if calldata.len() < 10 {
            return Err("Invalid calldata format".into());
        }
        let args = vec!["send",target_address, calldata, "--private-key", private_key];
        let output = run_cast_command(&args)?;
        if output.contains("transactionHash") {
            Ok(output) 
        } else {
            Err(format!("Failed to send transaction: {}", output).into())
        }
    }

    /// -------------------------------------------
    /// Retrieves the wallet address from a given private key
    /// -------------------------------------------
    pub fn get_wallet_address(private_key: &str) -> Result<String, Box<dyn std::error::Error>> {
        let output = run_cast_command(&["wallet", "address", "--private-key", private_key])?;
        Ok(output.trim().to_string())
    }


    /// -------------------------------------------
    /// Deploys a smart contract using a given RPC URL and private key
    /// -------------------------------------------
    pub fn deploy_contract(private_key: &str, bytecode: &str) -> Result<String, Box<dyn std::error::Error>> {
        let deploy_output = run_cast_command(&["send", "--private-key", private_key, "--create", bytecode])?;
        

        // Just for testing purposes...
        // Send to the contract address 1 Ether if the deployment was successful to fund the contract 
        match extract_contract_address(&deploy_output) {
            Some(contract_address) => {
                let _ = run_cast_command(&["send", &contract_address,"--private-key", private_key,  "--value", "1000000000000000000"])?;
                println!("1 Ether sent to contract: {}", contract_address);
                Ok(deploy_output)
            },
            None => Err("Failed to extract contract address from deployment output".into()),
        }
        
    }

    /// -------------------------------------------
    /// Deploys the Verifier smart contract using a given RPC URL and private key
    /// -------------------------------------------
    pub fn deploy_verifier_contract(
        private_key: &str,
        bytecode: &str,
        risc0_verifier_contract: &str,
        program_spec_hash: &str,
        context_state_hash: &str,
        image_id: &str
    ) -> Result<String, Box<dyn Error>> {

        // ABI-encode the constructor parameters to deploy a SC with constructor arguments
        let abi_encoded_params = run_cast_command(&[
            "abi-encode",
            "constructor(address,bytes32,bytes32,address)",
            risc0_verifier_contract,
            program_spec_hash,
            context_state_hash,
            image_id,
        ])?;
        println!("ABI-Encoded Parameters: {:?}", abi_encoded_params);

        // Concatenate the bytecode and ABI-encoded parameters 
        let full_bytecode = format!("{}{}", bytecode, abi_encoded_params.trim_start_matches("0x"));
        println!("Final Contract Bytecode: {:?}", full_bytecode);

        let deploy_output = run_cast_command(&[
            "send",
            "--private-key",
            private_key,
            "--create",
            &full_bytecode,
        ])?;

        Ok(deploy_output)
    }

    /// -------------------------------------------
    /// Calls the `verify()` function of the deployed VerifierContract
    /// -------------------------------------------
    pub fn call_verify_function(
        private_key: &str,
        verifier_contract_address: &str,
        public_input: &str,
        seal: &str,
    ) -> Result<String, Box<dyn Error>> {
        // Send the transaction using cast send
        let call_output = run_cast_command(&[
            "send",
            "--private-key",
            private_key,
            verifier_contract_address,
            "verify(bytes,bytes)",
            public_input,
            seal,
        ])?;

        Ok(call_output)
    }


    /// -------------------------------------------
    /// Extracts the contract address from cast output
    /// -------------------------------------------
    pub fn extract_contract_address(output: &str) -> Option<String> {
        println!("Extracting contract address from: {}", output);
        for line in output.lines() {
            if line.contains("contractAddress") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(address) = parts.get(1) {
                    return Some(address.to_string());
                }
            }
        }
        None
    }

    /// -------------------------------------------
    /// Populates contract variables from an ABI file
    /// -------------------------------------------
    pub fn populate_state_variables_from_abi(abi_file_path: std::path::PathBuf) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let abi_data = fs::read_to_string(abi_file_path)?;
        let abi: Value = serde_json::from_str(&abi_data)?;
        
        let mut state_variables = HashMap::new();

        if let Some(types) = abi.as_array() {
            for item in types {
                // Verifica se è una funzione
                if item["type"].as_str() == Some("function") {
                    // Verifica se la funzione è un getter (nessun input e un solo output)
                    if let Some(inputs) = item["inputs"].as_array() {
                        if inputs.is_empty() {
                            if let Some(outputs) = item["outputs"].as_array() {
                                if outputs.len() == 1 {
                                    if let Some(output) = outputs.get(0) {
                                        if let Some(var_type) = output["type"].as_str() {
                                            // Usa il nome della funzione come nome della variabile di stato
                                            if let Some(name) = item["name"].as_str() {
                                                state_variables.insert(name.to_string(), var_type.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Restituisce le variabili di stato dedotte
        Ok(state_variables)
    }

    /// -------------------------------------------
    /// Retrieves blockchain settings like gas price, block number, and more
    /// -------------------------------------------
    pub async fn get_blockchain_settings() -> Result<String, Box<dyn std::error::Error>> {
        let gas_price = run_cast_command(&["gas-price"])?;
        let block_number = run_cast_command(&["block-number"])?;
        let block_details_json = run_cast_command(&["block", &block_number, "--json"])?;
        let block_details: Value = serde_json::from_str(&block_details_json)?;
        let block_timestamp = block_details["timestamp"].as_str().unwrap_or("0");
        let block_difficulty = block_details["difficulty"].as_str().unwrap_or("0");
        let block_gas_limit = block_details["gasLimit"].as_str().unwrap_or("0");
        let block_coinbase = block_details["miner"].as_str().unwrap_or("0x0000000000000000000000000000000000000000");
        let block_base_fee_per_gas = block_details["baseFeePerGas"].as_str().unwrap_or("0");
        let chain_id = run_cast_command(&["chain-id"])?;
        
        let blockchain_settings = format!(
            r#"{{
                "gas_price": "{}",
                "origin": "0x0000000000000000000000000000000000000000",
                "block_hashes": "[]",
                "block_number": "{}",
                "block_coinbase": "{}",
                "block_timestamp": "{}",
                "block_difficulty": "{}",
                "block_gas_limit": "{}",
                "chain_id": "{}",
                "block_base_fee_per_gas": "{}"
            }}"#,
            gas_price,
            block_number,
            block_coinbase,
            block_timestamp,
            block_difficulty,
            block_gas_limit,
            chain_id,
            block_base_fee_per_gas
        );
        Ok(blockchain_settings)
    }

    /// -------------------------------------------
    /// Retrieves the balance of an Ethereum address
    /// -------------------------------------------
    pub async fn get_balance(address: &str) -> Result<String, Box<dyn std::error::Error>> {
        run_cast_command(&["balance", address])
    }

    /// -------------------------------------------
    /// Retrieves the nonce of an Ethereum address
    /// -------------------------------------------
    pub async fn get_nonce(address: &str) -> Result<String, Box<dyn std::error::Error>> {
        run_cast_command(&["nonce", address])
    }

    /// -------------------------------------------
    /// Retrieves the code of a contract at a given address
    /// -------------------------------------------
    pub async fn get_code(address: &str) -> Result<String, Box<dyn std::error::Error>> {
        run_cast_command(&["code", address])
    }

    /// -------------------------------------------
    /// Retrieves the storage value at a given slot for a contract
    /// -------------------------------------------
    pub fn get_storage_at(contract: &str, slot: &str) -> Result<BTreeMap<H256, H256>, FromHexError> {
        let output = run_cast_command(&["storage", contract, slot]).map_err(|_| FromHexError::InvalidStringLength)?;
        println!("Raw output: {:?}", output);
        
        let output_trimmed = output.trim();

        // If the output is empty or zero, return an empty map
        if output_trimmed.is_empty() || output_trimmed == "0x0000000000000000000000000000000000000000000000000000000000000000" {
            return Ok(BTreeMap::new());
        }

        // Decode the hexadecimal output
        let decoded_bytes = hex::decode(output_trimmed.trim_start_matches("0x"))?; 

        let value_h256 = H256::from_slice(&decoded_bytes);
        let slot_u64: u64 = slot.parse().unwrap_or(0);
        let slot_hash = H256::from_low_u64_be(slot_u64);

        println!("Slot Hash: {:?}", slot_hash);
        println!("Value H256: {:?}", value_h256);

        let mut storage_map = BTreeMap::new();
        storage_map.insert(slot_hash, value_h256); 

        Ok(storage_map)
    }
}



/////////////////////////////////////////////////////////////////////////////
                            //EVM-RUNNER MODULES//
/////////////////////////////////////////////////////////////////////////////

pub mod conditions {
    extern crate alloc;
    extern crate core;

    use alloc::string::String;
    use alloc::vec::Vec;
    use ethereum_types::{H160, H256};
    use primitive_types::U256;
    use serde::{Deserialize, Serialize};
    use sha2::Digest;
    use sha3::Keccak256;

    /// -------------------------------------------
    /// Defines the comparison operators for conditions
    /// -------------------------------------------
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum Operator {
        Eq,  // Equal
        Neq, // Not equal
        Gt,  // Greater than
        Ge,  // Greater than or equal
        Lt,  // Less than
        Le,  // Less than or equal
    }

    /// -------------------------------------------
    /// Represents a fixed condition with a state key, operator, and expected value
    /// -------------------------------------------
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct FixedCondition {
        pub k_s: String,  // State key
        pub op: Operator, // Operation
        pub v: U256,      // Expected value
    }

    /// -------------------------------------------
    /// Represents a relative condition comparing two state keys
    /// -------------------------------------------
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct RelativeCondition {
        pub k_s: String,       // State key
        pub op: Operator,      // Operation
        pub k_s_prime: String, // End state key
    }

    /// -------------------------------------------
    /// Represents a condition, either fixed or relative
    /// -------------------------------------------
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum Condition {
        Fixed(FixedCondition),
        Relative(RelativeCondition),
    }

    /// -------------------------------------------
    /// Hashes a program specification using Keccak256
    /// -------------------------------------------
    pub fn hash_program_spec(program_spec: &[(Condition, String)]) -> [u8; 32] {
        let mut hasher = Keccak256::new();

        for (cond, method) in program_spec {
            let serialized_condition = serialize_condition(cond);
            let serialized_method = method.as_bytes();
            // Concat the serialized condition and method
            let concat_condition_method = [serialized_condition, serialized_method.to_vec()].concat();
            hasher.update(concat_condition_method);
        }

        hasher.finalize().into()
    }

    /// -------------------------------------------
    /// Serializes a fixed condition into a byte vector
    /// -------------------------------------------
    fn serialize_fixed_condition(cond: &FixedCondition) -> Vec<u8> {
        let mut serialized = Vec::new();

        serialized.extend(cond.k_s.as_bytes());
        serialized.push(0); // Null terminator

        serialized.push(match cond.op {
            Operator::Eq => 0x00,
            Operator::Neq => 0x01,
            Operator::Gt => 0x02,
            Operator::Ge => 0x03,
            Operator::Lt => 0x04,
            Operator::Le => 0x05,
        });

        let mut v_bytes = [0u8; 32]; // U256 is 32 bytes
        cond.v.to_little_endian(&mut v_bytes);
        serialized.extend(&v_bytes); // 8-byte little-endian
        serialized
    }

    /// -------------------------------------------
    /// Serializes a relative condition into a byte vector
    /// -------------------------------------------
    fn serialize_relative_condition(cond: &RelativeCondition) -> Vec<u8> {
        let mut serialized = Vec::new();

        serialized.extend(cond.k_s.as_bytes());
        serialized.push(0); // Null terminator

        serialized.push(match cond.op {
            Operator::Eq => 0x00,
            Operator::Neq => 0x01,
            Operator::Gt => 0x02,
            Operator::Ge => 0x03,
            Operator::Lt => 0x04,
            Operator::Le => 0x05,
        });

        serialized.extend(cond.k_s_prime.as_bytes());
        serialized.push(0); // Null terminator
        serialized
    }

    /// -------------------------------------------
    /// Serializes a condition into a byte vector
    /// -------------------------------------------
    fn serialize_condition(cond: &Condition) -> Vec<u8> {
        match cond {
            Condition::Fixed(fixed) => serialize_fixed_condition(fixed),
            Condition::Relative(relative) => serialize_relative_condition(relative),
        }
    }

    /// -------------------------------------------
    /// Computes the storage key for a mapping in a smart contract
    /// -------------------------------------------
    pub fn compute_mapping_storage_key(key: H160, base_slot: U256) -> H256 {
        // Convert the address to a 32-byte representation (left-padded with zeros)
        let mut padded_key = [0u8; 32];
        padded_key[12..].copy_from_slice(&key.as_bytes());

        // Create a buffer to hold the bytes
        let mut base_bytes = [0u8; 32];

        // Convert the base_slot (U256) to 32 bytes (big-endian)
        base_slot.to_big_endian(&mut base_bytes);

        // Concatenate padded_key and base_bytes
        let mut hasher = Keccak256::new();
        hasher.update(&padded_key);
        hasher.update(&base_bytes);
        let result = hasher.finalize();
        H256::from_slice(&result)
    }

    /// -------------------------------------------
    /// Computes the storage key for a variable in a smart contract
    /// -------------------------------------------
    pub fn compute_storage_key(slot: u64) -> H256 {
        H256::from_low_u64_be(slot)
    }
}

pub mod input {
    extern crate alloc;
    extern crate core;
    use alloc::collections::BTreeMap;
    use alloc::string::String;
    use alloc::vec::Vec;
    use primitive_types::{H256, U256};
    use serde::{Deserialize, Serialize};
    /// -------------------------------------------
    /// Represents the data of an Ethereum account
    /// -------------------------------------------
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AccountData {
        pub address: String,
        pub nonce: U256,
        pub balance: U256,
        pub storage: BTreeMap<H256, H256>,
        pub code: Vec<u8>,
    }
}

pub mod context {
    use crate::input::AccountData;
    use ethereum_types::{H256, U256};
    use sha3::{Digest, Keccak256};
    use std::collections::BTreeMap;

    /// -------------------------------------------
    /// Defines the type of context account data
    /// -------------------------------------------
    #[derive(Debug)]
    pub enum ContextAccountDataType {
        ERC20,
    }
    const CONTEXT_ERC20_CONTRACT_ADDRESS: &str = "E4C2000000000000000000000000000000000000";
    const CONTEXT_ERC20_CONTRACT_BYTECODE: &str = include_str!("../../bytecode/ContextTemplateERC20.bin-runtime");

    /// -------------------------------------------
    /// Builds account data for a context-specific contract
    /// -------------------------------------------
    pub fn build_context_account_data(
        context_account_data_type: ContextAccountDataType,
        init_storage: Option<BTreeMap<H256, H256>>,
    ) -> AccountData {
        let storage = if let Some(storage) = init_storage {
            let mut new_storage = BTreeMap::new();
            for (key, value) in storage {
                new_storage.insert(key, value);
            }
            new_storage
        } else {
            BTreeMap::new()
        };

        return match context_account_data_type {
            ContextAccountDataType::ERC20 => AccountData {
                address: CONTEXT_ERC20_CONTRACT_ADDRESS.to_string(),
                nonce: U256::zero(),
                balance: U256::zero(),
                storage,
                code: hex::decode(CONTEXT_ERC20_CONTRACT_BYTECODE).unwrap(),
            },
        };
    }

    /// -------------------------------------------
    /// Hashes the state of a context using Keccak256
    /// -------------------------------------------
    pub fn hash_context_state(context_state: &Vec<AccountData>) -> [u8; 32] {
        let mut hasher = Keccak256::new();

        // Hash each bytecode chunk directly as raw bytes
        for cdata in context_state {
            hasher.update(&cdata.code); // Use raw bytes directly
        }

        hasher.finalize().into()
    }
}