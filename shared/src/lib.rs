// SPDX-License-Identifier: MIT

//PLEASE NOTE: We can create other .rs files in the shared/src/ directory and import them here or in other modules if this "template" becomes too large.

/////////////////////////////////////////////////////////////////////////////
//HOST MODULES//
/////////////////////////////////////////////////////////////////////////////

// Re‑export the tracing crate so macros can use `$crate::tracing::…`
pub use tracing;

pub mod log;

pub mod utils {
    use crate::conditions::{Condition, FixedCondition, Operator, Word256};
    use clap::{Arg, Command};
    use primitive_types::{H256, U256};
    use std::path::PathBuf;
    use std::str;
    use tiny_keccak::{Hasher, Keccak};

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

    /// Convert the literal on the right-hand side into our unified 256-bit type.
    ///
    /// * Decimal like `"1000"` → `Word256::Uint(1000)`  
    /// * Hex starting with `"0x"` (or `"0X"`) → `Word256::Hash(0x…)`
    fn parse_word256(lit: &str) -> Word256 {
        if lit.starts_with("0x") || lit.starts_with("0X") {
            use std::str::FromStr;
            Word256::Hash(
                H256::from_str(lit)
                    .expect("hex literal must be 0x-prefixed and 64 hex chars long"),
            )
        } else {
            Word256::Uint(
                U256::from_dec_str(lit)
                    .expect("decimal literal must fit into 256 bits"),
            )
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
        let value = parse_word256(parts[2]);

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
        } else {
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
    /// Parses CLI arguments and returns the matches for host crate.
    /// -------------------------------------------
    pub fn parse_cli_args_host() -> clap::ArgMatches {
        Command::new("zkpoex-cli")
            .version("1.0")
            .author("Ziemann, Galexela")
            .about("Generates zk proofs for Ethereum smart contract exploits")
            .arg(
                Arg::new("function")
                    .short('f')
                    .long("function")
                    .value_name("FUNCTION")
                    .help("Sets the function name (e.g., 'exploit')")
                    .required(true),
            )
            .arg(
                Arg::new("params")
                    .short('p')
                    .long("params")
                    .value_name("PARAMS")
                    .help("Sets the function parameters (e.g., 'true')")
                    .required(true),
            )
            .arg(
                Arg::new("context-state")
                    .short('c')
                    .long("context-state")
                    .value_name("CONTEXT_STATE")
                    .help("Sets the context state file path containing the state data")
                    .required(true)
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("program-spec")
                    .short('p')
                    .long("program-spec")
                    .value_name("PROGRAM_SPEC")
                    .help("Sets the program spec file path containing the method specifications")
                    .required(true)
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("value")
                    .short('v')
                    .long("value")
                    .value_name("VALUE")
                    .help("Sets the value in wei to send with the transaction")
                    .required(false),
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .value_name("VERBOSE")
                    .help("Enable verbose (debug‑level) logging")
                    .required(false)
                    .value_parser(clap::value_parser!(bool)),
            )
            .arg(
                Arg::new("onchain-verify")
                    .short('o')
                    .long("onchain-verify")
                    .value_name("ONCHAIN_VERIFY")
                    .help("Enable on-chain verification")
                    .required(false)
                    .value_parser(clap::value_parser!(bool)),
            )
            .get_matches()
    }

    /// -------------------------------------------
    /// Parses CLI arguments and returns the matches for onchain-verifier.
    /// -------------------------------------------
    pub fn parse_cli_args_onchain_verifier() -> clap::ArgMatches {
        Command::new("zkpoex-cli")
            .version("1.0")
            .author("Ziemann, Galexela")
            .about("Generates zk proofs for Ethereum smart contract exploits")
            .arg(
                Arg::new("contract-address")
                    .short('s')
                    .long("contract-address")
                    .value_name("CONTRACT_ADDRESS")
                    .help("Sets the address of the VerifierContract for the onchain verifier")
                    .required(true),
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .value_name("VERBOSE")
                    .help("Enable verbose (debug‑level) logging")
                    .required(false)
                    .value_parser(clap::value_parser!(bool)),
            )
            .get_matches()
    }

    /// -------------------------------------------
    /// Parses CLI arguments and returns the matches for sc-owner crate.
    /// -------------------------------------------
    pub fn parse_cli_args_sc_owner() -> clap::ArgMatches {
        Command::new("zkpoex-cli")
            .version("1.0")
            .author("Ziemann, Galexela")
            .about("Generates zk proofs for Ethereum smart contract exploits")
            .arg(
                Arg::new("private-key")
                    .short('p')
                    .long("private-key")
                    .value_name("WALLET_PRIV_KEY")
                    .help("Sets the private key of the deployer wallet")
                    .required(true),
            )
            .arg(
                Arg::new("risc0-verifier-contract-address")
                    .short('r')
                    .long("risc0-verifier-contract-address")
                    .value_name("RISC0_VERIFIER_CONTRACT_ADDRESS")
                    .help("Sets the address of the RISC0 verifier contract")
                    .required(true),
            )
            .arg(
                Arg::new("context-state")
                    .short('c')
                    .long("context-state")
                    .value_name("CONTEXT_STATE")
                    .help("Sets the context state file path containing the state data")
                    .required(true)
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("program-spec")
                    .short('p')
                    .long("program-spec")
                    .value_name("PROGRAM_SPEC")
                    .help("Sets the program spec file path containing the method specifications")
                    .required(true)
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .value_name("VERBOSE")
                    .help("Enable verbose (debug‑level) logging")
                    .required(false)
                    .value_parser(clap::value_parser!(bool)),
            )
            .arg(
                Arg::new("send-eth")
                    .short('s')
                    .long("send-eth")
                    .value_name("SEND_ETH")
                    .help("Sets the amount of ETH to send to the contract in wei")
                    .required(false)
                    .value_parser(clap::value_parser!(u64)),
            )
            .get_matches()
    }
}

pub mod evm_utils {
    use crate::{log_debug, log_info};
    use anyhow::bail;
    use hex::FromHexError;
    use primitive_types::H256;
    use risc0_zkvm::sha::Digestible;
    use risc0_zkvm::InnerReceipt;
    use serde_json;
    use serde_json::from_str;
    use serde_json::Value;
    use std::collections::BTreeMap;
    use std::collections::HashMap;
    use std::error::Error;
    use std::fs;
    use std::process::Command as ProcessCommand;
    use std::str;
    use std::env;

    const ANVIL_RPC_URL: &str = "http://localhost:8545";
    const SEPOLIA_RPC_URL: &str = "https://ethereum-sepolia-rpc.publicnode.com";
    const HOLESKY_RPC_URL: &str = "https://ethereum-holesky-rpc.publicnode.com";
    const MAINNET_RPC_URL: &str = "https://ethereum-rpc.publicnode.com";

    /// -------------------------------------------
    /// Executes a cast command and returns the output as a String
    /// -------------------------------------------
    fn run_cast_command(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
        log_debug!("--------------------------------------------------");
        log_debug!("Executing command: cast {:?}", args);
        log_debug!("--------------------------------------------------");
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
        log_debug!("--------------------------------------------------");
        log_debug!("Executing command: solc {:?}", args);
        log_debug!("--------------------------------------------------");
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
    fn get_storage_layout(
        contract_file: &str,
    ) -> Result<HashMap<String, usize>, Box<dyn std::error::Error>> {
        let output = run_solc_command(&["--storage-layout", contract_file])?;

        // Get the storage layout from the output parsing in json format
        let storage_layout_start: Vec<&str> = output.split("\n").collect();
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
        variables: &HashMap<String, String>,
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
        calldata: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if calldata.len() < 10 {
            return Err("Invalid calldata format".into());
        }
        let args = vec![
            "send",
            target_address,
            calldata,
            "--private-key",
            private_key,
        ];
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
    pub fn deploy_contract(
        private_key: &str,
        bytecode: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let deploy_output =
            run_cast_command(&["send", "--private-key", private_key, "--create", bytecode])?;
        Ok(deploy_output)
        // Just for testing purposes...
        /* Send to the contract address 1 Ether if the deployment was successful to fund the contract
        match extract_contract_address(&deploy_output) {
            Some(contract_address) => {
                let _ = run_cast_command(&[
                    "send",
                    &contract_address,
                    "--private-key",
                    private_key,
                    "--value",
                    "1000000000000000000",
                ])?;
                log_info!("1 Ether sent to contract: {}", contract_address);

            }
            None => Err("Failed to extract contract address from deployment output".into()),
        }*/
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
        image_id: &str,
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
        log_debug!("ABI-Encoded Parameters: {:?}", abi_encoded_params);

        // Concatenate the bytecode and ABI-encoded parameters
        let full_bytecode = format!(
            "{}{}",
            bytecode,
            abi_encoded_params.trim_start_matches("0x")
        );
        log_debug!("Final Contract Bytecode: {:?}", full_bytecode);

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
    /// Extracts the contract address from cast output
    /// -------------------------------------------
    pub fn extract_contract_address(output: &str) -> Option<String> {
        log_info!("Extracting contract address from: {}", output);
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
    pub fn populate_state_variables_from_abi(
        abi_file_path: std::path::PathBuf,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
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
                                                state_variables
                                                    .insert(name.to_string(), var_type.to_string());
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
        let block_coinbase = block_details["miner"]
            .as_str()
            .unwrap_or("0x0000000000000000000000000000000000000000");
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

    pub fn get_onchain_links(value: &str) -> String {
        let eth_rpc_url = env::var("ETH_RPC_URL").unwrap_or_else(|_| "http://localhost:8545".to_string());

        if !value.starts_with("0x") {
            panic!("Value must start with '0x' to generate onchain links");
        }

        // If the RPC URL is Anvil, return the value as is as there is no explorer link
        if eth_rpc_url == ANVIL_RPC_URL {
            return value.to_string();
        }

        if eth_rpc_url == HOLESKY_RPC_URL {
            if value.starts_with("0x") && value.len() == 42 {
                return format!("https://holesky.etherscan.io/address/{}", value);
            } else if value.starts_with("0x") && value.len() == 66 {
                return format!("https://holesky.etherscan.io/tx/{}", value);
            }
        }

        if eth_rpc_url == SEPOLIA_RPC_URL {
            if value.starts_with("0x") && value.len() == 42 {
                return format!("https://sepolia.etherscan.io/address/{}", value);
            } else if value.starts_with("0x") && value.len() == 66 {
                return format!("https://sepolia.etherscan.io/tx/{}", value);
            }
        }

        if eth_rpc_url == MAINNET_RPC_URL {
            if value.starts_with("0x") && value.len() == 42 {
                return format!("https://etherscan.io/address/{}", value);
            } else if value.starts_with("0x") && value.len() == 66 {
                return format!("https://etherscan.io/tx/{}", value);
            }
        }
        // Default case if the value doesn't match either address or txhash
        value.to_string()
    }

    /// -------------------------------------------
    /// Encode the seal of the given receipt for use with EVM smart contract verifiers.
    /// Appends the verifier selector, determined from the first 4 bytes of the verifier parameters
    /// including the Groth16 verification key and the control IDs that commit to the RISC Zero
    /// circuits.
    /// -------------------------------------------
    pub fn encode_seal(receipt: &risc0_zkvm::Receipt) -> Result<Vec<u8>, anyhow::Error> {
        let seal = match receipt.inner.clone() {
            InnerReceipt::Fake(receipt) => {
                let seal = receipt.claim.digest().as_bytes().to_vec();
                let selector = &[0xFFu8; 4];
                let mut selector_seal = Vec::with_capacity(selector.len() + seal.len());
                selector_seal.extend_from_slice(selector);
                selector_seal.extend_from_slice(&seal);
                selector_seal
            }
            InnerReceipt::Groth16(receipt) => {
                let selector = &receipt.verifier_parameters.as_bytes()[..4];
                let mut selector_seal = Vec::with_capacity(selector.len() + receipt.seal.len());
                selector_seal.extend_from_slice(selector);
                selector_seal.extend_from_slice(receipt.seal.as_ref());
                selector_seal
            }
            _ => bail!("Unsupported receipt type"),
        };
        Ok(seal)
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
    pub fn get_storage_at(
        contract: &str,
        slot: &str,
    ) -> Result<BTreeMap<H256, H256>, FromHexError> {
        let output = run_cast_command(&["storage", contract, slot])
            .map_err(|_| FromHexError::InvalidStringLength)?;

        let output_trimmed = output.trim();

        // If the output is empty or zero, return an empty map
        if output_trimmed.is_empty()
            || output_trimmed
                == "0x0000000000000000000000000000000000000000000000000000000000000000"
        {
            return Ok(BTreeMap::new());
        }

        // Decode the hexadecimal output
        let decoded_bytes = hex::decode(output_trimmed.trim_start_matches("0x"))?;

        let value_h256 = H256::from_slice(&decoded_bytes);
        let slot_u64: u64 = slot.parse().unwrap_or(0);
        let slot_hash = H256::from_low_u64_be(slot_u64);

        let mut storage_map = BTreeMap::new();
        storage_map.insert(slot_hash, value_h256);

        Ok(storage_map)
    }

    pub async fn send_eth(
        private_key: &str,
        contract_address: &str,
        value: u64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let value_str = value.to_string();
        let args = vec![
            "send",
            contract_address,
            "--private-key",
            private_key,
            "--value",
            &value_str,
        ];
        let output = run_cast_command(&args)?;
        Ok(output)
    }
}

/////////////////////////////////////////////////////////////////////////////
//EVM-RUNNER MODULES//
/////////////////////////////////////////////////////////////////////////////

pub mod conditions {
    extern crate alloc;
    extern crate core;

    use std::str::FromStr;
    use alloc::string::String;
    use alloc::vec::Vec;
    use ethereum_types::{H160, H256};
    use primitive_types::U256;
    use serde::{Serialize, Deserialize, Serializer, Deserializer};
    use sha2::Digest;
    use sha3::Keccak256;

    /// Any 256-bit word that may come out of state or user-supplied JSON.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Word256 {
        Uint(U256),
        Hash(H256),
    }

    impl From<U256> for Word256 {
        fn from(u: U256) -> Self { Word256::Uint(u) }
    }

    impl From<H256> for Word256 {
        fn from(h: H256) -> Self { Word256::Hash(h) }
    }

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
    /// Defines the arithmetic operators for conditions
    /// -------------------------------------------
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum ArithmeticOperator {
        Add, // Addition
        Sub, // Subtraction
        Mul, // Multiplication
        Div, // Division
        Mod, // Modulo
    }

    /// -------------------------------------------
    /// Represents a fixed condition with a state key, operator, and expected value
    /// val(k_s) op v
    /// -------------------------------------------
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct FixedCondition {
        pub k_s: String,  // State key
        pub op: Operator, // Operation
        pub v: Word256,      // Expected value
    }

    /// -------------------------------------------
    /// Represents a relative condition comparing two state keys and an optional value
    /// val(k_s) op (val(k_s_prime) value_op v)
    /// -------------------------------------------
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct RelativeCondition {
        pub k_s: String,                          // State key
        pub op: Operator,                         // Operation
        pub k_s_prime: String,                    // End state key
        pub value_op: Option<ArithmeticOperator>, // Optional aritmetic operator for the value
        pub v: Option<Word256>,                      // Optional value
    }

    /// -------------------------------------------
    /// Represents a fixed condition whcih depends on the input
    /// val(k_s) op input
    /// -------------------------------------------
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct InputDependantFixedCondition {
        pub k_s: String,   // State key
        pub op: Operator,  // Operation
        pub input: String, // Input
    }

    /// -------------------------------------------
    /// Represents a relative condition comparing two state keys and an optional input
    /// val(k_s) op (val(k_s_prime) input_op input)
    /// -------------------------------------------
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct InputDependantRelativeCondition {
        pub k_s: String,                  // State key
        pub op: Operator,                 // Operation
        pub k_s_prime: String,            // End state key
        pub input_op: ArithmeticOperator, // Arithmetic operation for the value
        pub input: String,                // Input
    }

    /// -------------------------------------------
    /// Represents a condition, either fixed or relative
    /// -------------------------------------------
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum Condition {
        Fixed(FixedCondition),
        Relative(RelativeCondition),
        InputDependantFixedCondition(InputDependantFixedCondition),
        InputDependantRelativeCondition(InputDependantRelativeCondition),
    }

    /// -------------------------------------------
    /// Represents a method specification for a smart contract
    /// -------------------------------------------
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct MethodSpec {
        pub method_id: String,
        pub conditions: Vec<Condition>,
        pub arguments: Vec<MethodArgument>,
    }

    /// -------------------------------------------
    /// Represents an argument for a method specification
    /// -------------------------------------------
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct MethodArgument {
        pub argument_type: String,
        pub argument_name: String,
    }

    impl Serialize for Word256 {
        fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
            match self {
                Word256::Uint(u) => ser.serialize_str(&u.to_string()),      // decimal
                Word256::Hash(h) => ser.serialize_str(&format!("{:#x}", h)), // 0x… hex
            }
        }
    }

    impl<'de> Deserialize<'de> for Word256 {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let s: &str = <&str>::deserialize(de)?;
        if let Ok(u) = U256::from_dec_str(s) {
            Ok(Word256::Uint(u))
        } else {
            // treat anything else as 0x… hex -string H256
            let h = H256::from_str(s).map_err(serde::de::Error::custom)?;
            Ok(Word256::Hash(h))
        }
    }
}

    /// -------------------------------------------
    /// Hashes a program specification using Keccak256
    /// -------------------------------------------
    pub fn hash_program_spec(program_spec: &[MethodSpec]) -> [u8; 32] {
        let mut hasher = Keccak256::new();

        for spec in program_spec {
            let method_id = spec.method_id.as_bytes();
            let serialized_conditions =
                (spec.conditions.iter().map(|cond| serialize_condition(cond)))
                    .flatten()
                    .collect::<Vec<u8>>();
            let serialized_arguments = (spec.arguments.iter().map(|arg| serialize_argument(arg)))
                .flatten()
                .collect::<Vec<u8>>();

            // Concatenate the method_id, serialized_conditions and serialized_arguments
            let concat_method_spec =
                [method_id, &serialized_conditions, &serialized_arguments].concat();
            hasher.update(&concat_method_spec);
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
        match cond.v {
            Word256::Uint(u) => u.to_little_endian(&mut v_bytes),
            Word256::Hash(h) => v_bytes.copy_from_slice(h.as_bytes()), // already 32 bytes
        }
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
    /// Serializes an input-dependant fixed condition into a byte vector
    /// -------------------------------------------
    fn serialize_input_dependant_fixed_condition(cond: &InputDependantFixedCondition) -> Vec<u8> {
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

        serialized.extend(cond.input.as_bytes());
        serialized.push(0); // Null terminator
        serialized
    }

    /// -------------------------------------------
    /// Serializes an input-dependant relative condition into a byte vector
    /// -------------------------------------------
    fn serialize_input_dependant_relative_condition(
        cond: &InputDependantRelativeCondition,
    ) -> Vec<u8> {
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

        serialized.push(match cond.input_op {
            ArithmeticOperator::Add => 0x00,
            ArithmeticOperator::Sub => 0x01,
            ArithmeticOperator::Mul => 0x02,
            ArithmeticOperator::Div => 0x03,
            ArithmeticOperator::Mod => 0x04,
        });

        serialized.extend(cond.input.as_bytes());
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
            Condition::InputDependantFixedCondition(input_fixed) => {
                serialize_input_dependant_fixed_condition(input_fixed)
            }
            Condition::InputDependantRelativeCondition(input_relative) => {
                serialize_input_dependant_relative_condition(input_relative)
            }
        }
    }

    /// -------------------------------------------
    /// Serializes a method argument into a byte vector
    /// -------------------------------------------
    fn serialize_argument(argument: &MethodArgument) -> Vec<u8> {
        let mut serialized = Vec::new();

        serialized.extend(argument.argument_type.as_bytes());
        serialized.push(0); // Null terminator

        serialized.extend(argument.argument_name.as_bytes());
        serialized.push(0); // Null terminator

        serialized
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
    use serde::de::{self, Deserializer};
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;

    /// -------------------------------------------
    /// Represents the data of an Ethereum account
    /// -------------------------------------------
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(deny_unknown_fields)]
    pub struct AccountData {
        pub address: String,
        #[serde(deserialize_with = "deserialize_u256")]
        pub nonce: U256,
        #[serde(deserialize_with = "deserialize_u256")]
        pub balance: U256,
        #[serde(deserialize_with = "deserialize_storage")]
        pub storage: BTreeMap<H256, H256>,
        #[serde(deserialize_with = "deserialize_code")]
        pub code: Vec<u8>,
        pub transaction_recipient: Option<bool>,
    }

    // Custom deserialization for U256
    fn deserialize_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        U256::from_dec_str(s).map_err(de::Error::custom)
    }

    // Custom deserialization for storage (BTreeMap<H256, H256>)
    fn deserialize_storage<'de, D>(deserializer: D) -> Result<BTreeMap<H256, H256>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: BTreeMap<String, String> = Deserialize::deserialize(deserializer)?;
        map.into_iter()
            .map(|(k, v)| {
                let key = H256::from_str(&k).map_err(de::Error::custom)?;
                let value = H256::from_str(&v).map_err(de::Error::custom)?;
                Ok((key, value))
            })
            .collect()
    }

    // Custom deserialization for code (Vec<u8>)
    fn deserialize_code<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        hex::decode(s).map_err(de::Error::custom)
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
    const CONTEXT_ERC20_CONTRACT_BYTECODE: &str =
        include_str!("../../bytecode/ContextTemplateERC20.bin-runtime");

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
                transaction_recipient: Some(false),
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
