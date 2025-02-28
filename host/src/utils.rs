use tiny_keccak::{Hasher, Keccak};
use clap::{Arg, Command};
use std::path::PathBuf;
use evm_runner::conditions::{Condition, FixedCondition, Operator};
use primitive_types::U256;

/// Generates the function signature in Solidity ABI format.
/// Example: `exploit(true)` -> `16112c6c0000000000000000000000000000000000000000000000000000000000000001`
pub fn generate_function_signature(function_name: &str, params: &[&str]) -> String {

    // Keccak-256 hash of the signature
    let mut hasher = Keccak::v256();
    hasher.update(function_name.as_bytes());
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);

    let selector = &output[..4];

    // Encode parameters according to ABI rules
    let encoded_params = params
        .iter()
        .map(|param| encode_abi_param(param))
        .collect::<Vec<String>>()
        .join("");

    format!("{}{}", hex::encode(selector), encoded_params)
}

/// Encodes a parameter according to Solidity ABI rules.
/// Supports basic types like `bool`, `uint256`, `address`, etc.
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


//Parse the condition string and return a Condition struct
//Example: "balance > 0" in the CLI-> Condition::Fixed(FixedCondition { k_s: "address.balance", op: Operator::Gt, v: 0 })
pub fn parse_condition(condition: &str) -> Condition {
    let parts: Vec<&str> = condition.split_whitespace().collect();
    if parts.len() != 3 {
        panic!("Invalid condition format: {}", condition);
    }

    let key = parts[0]; // e.g., "balance" or "storage.key"
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

    if key.starts_with("storage.") {
        // Handle storage conditions
        let storage_key = key.trim_start_matches("storage.");
        Condition::Fixed(FixedCondition {
            k_s: format!("4838B106FCe9647Bdf1E7877BF73cE8B0BAD5f97.storage.{}", storage_key),
            op,
            v: value,
        })
    } else {
        // Handle account field conditions (e.g., balance, nonce)
        Condition::Fixed(FixedCondition {
            k_s: format!("4838B106FCe9647Bdf1E7877BF73cE8B0BAD5f97.{}", key),
            op,
            v: value,
        })
    }
}

/// Parses CLI arguments and returns the matches.
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
        .get_matches()
}