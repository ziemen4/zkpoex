use dotenv::dotenv;
use hex;
use serde_json;
use shared::conditions::hash_program_spec;
use shared::conditions::MethodSpec;
use shared::context::hash_context_state;
use shared::evm_utils;
use shared::input::AccountData;
use shared::utils;
use std::fs;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    // Parse CLI arguments
    let matches = utils::parse_cli_args_sc_owner();

    // CLI Arguments are as follows
    // --private-key: Private key of the wallet
    // --risc0-verifier-contract-address: Address of the RISC0 verifier contract
    // --context-state: Path to the context state file
    // --program-spec: Path to the program specification file
    let private_key = matches.get_one::<String>("private-key").unwrap();
    let risc0_verifier_contract_address = matches
        .get_one::<String>("risc0-verifier-contract-address")
        .unwrap();
    let context_state_file = matches
        .get_one::<std::path::PathBuf>("context-state")
        .unwrap();
    let program_spec_file = matches
        .get_one::<std::path::PathBuf>("program-spec")
        .unwrap();

    // Read the context state and program specification files
    let context_state_json = fs::read_to_string(context_state_file).expect("Failed to read file");
    let context_state: Vec<AccountData> = serde_json::from_str(&context_state_json)?;
    let context_state_hash: String = hex::encode(hash_context_state(&context_state));

    let program_spec_json = fs::read_to_string(program_spec_file).expect("Failed to read file");
    let program_spec: Vec<MethodSpec> = serde_json::from_str(&program_spec_json)?;
    let program_spec_hash: String = hex::encode(hash_program_spec(&program_spec));

    // Deploy the ImageID contract
    let contract_bytecode_file_imageid = "./bytecode/ImageID.bin";
    let contract_bytecode_deployment_imageid = fs::read_to_string(contract_bytecode_file_imageid)
        .expect("Failed to read contract bytecode file");
    let imageid_deploy_output =
        evm_utils::deploy_contract(&private_key, &contract_bytecode_deployment_imageid)?;
    let image_id = evm_utils::extract_contract_address(&imageid_deploy_output)
        .expect("Failed to extract contract address")
        .to_string();
    println!("ImageID contract deployed at address: {}", image_id);

    // Deploy the Verifier contract
    let contract_bytecode_file_verifier = "./bytecode/VerifierContract.bin";
    let contract_bytecode_deployment_verifier = fs::read_to_string(contract_bytecode_file_verifier)
        .expect("Failed to read contract bytecode file");
    let output = evm_utils::deploy_verifier_contract(
        &private_key,
        &contract_bytecode_deployment_verifier,
        &risc0_verifier_contract_address,
        &program_spec_hash,
        &context_state_hash,
        &image_id,
    )?;

    let verifier_address = evm_utils::extract_contract_address(&output)
        .expect("Failed to extract contract address")
        .to_string()
        .trim_start_matches("0x")
        .to_string();
    println!(
        "\nVerifier contract deployed at address: 0x{}",
        verifier_address
    );

    Ok(())
}


#[cfg(test)]
mod tests {
    use shared::evm_utils;
    pub const VERIFIER_TESTNET_HOLESKY_ADDRESS : &str = "0xb94aA3E7a1CEFd86B5F439d0Ca34aA9D2c612bd9";
    use dotenv::dotenv;
    use ethers::abi::{AbiDecode, encode, Token};
    use sha2::{Digest, Sha256};
    use hex::encode as hencode;
    use std::fs;
    use std::str::FromStr;
    use alloy_sol_types::{SolType};
    use alloy_primitives::{Address, B256};
    use hex::FromHex;

    fn parse_journal_manual(bytes: &[u8]) -> Result<(bool, [u8; 32], [u8; 32], Address), String> {
        let mut cursor = 0;

        if bytes.len() < 4 {
            return Err("journal too short".to_string());
        }

        let array_len = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
        cursor += 4;

        if array_len != 4 {
            return Err(format!("Expected 4 elements, found {}", array_len));
        }

        let mut strings = Vec::new();

        for _ in 0..array_len {
            if cursor + 4 > bytes.len() {
                return Err("unexpected EOF while reading length".to_string());
            }

            let str_len = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap()) as usize;
            cursor += 4;

            if cursor + str_len > bytes.len() {
                return Err("unexpected EOF while reading string".to_string());
            }

            let s = std::str::from_utf8(&bytes[cursor..cursor + str_len])
                .map_err(|e| format!("invalid UTF-8 string: {}", e))?
                .to_string();

            strings.push(s);
            cursor += str_len;
        }

        // guest is sending an array of 4 values
        // strings[0] = "true" | "false"
        // strings[1] = hashed_program_spec ("0x...")
        // strings[2] = hashed_context_state ("0x...")
        // strings[3] = prover_address ("0x...")

        let exploit_found = match strings[0].as_str() {
            "true" => true,
            "false" => false,
            other => return Err(format!("invalid bool string: {}", other)),
        };
        println!("Exploit found: {:?}", strings[0]);
        println!("Exploit found: {}", exploit_found);

        let mut hashed_program_spec = [0u8; 32];
        hashed_program_spec.copy_from_slice(
            &<[u8; 32]>::from_hex(strings[1].trim_start_matches("0x"))
                .map_err(|e| format!("invalid hex for hashed_program_spec: {e}"))?
        );
        println!("Hashed program spec: {:?}", strings[1]);
        println!("Hashed program spec: {:?}", hashed_program_spec);
        
        let mut hashed_context_state = [0u8; 32];
        hashed_context_state.copy_from_slice(
            &<[u8; 32]>::from_hex(strings[2].trim_start_matches("0x"))
                .map_err(|e| format!("invalid hex for hashed_context_state: {e}"))?
        );
        println!("Hashed context state: {:?}", strings[2]);
        println!("Hashed context state: {:?}", hashed_context_state);

        

        let addr_bytes = <[u8; 20]>::from_hex("CA11E40000000000000000000000000000000000")
            .map_err(|e| format!("invalid address: {e}"))?;
        let prover_address = Address::from_slice(&addr_bytes);

        println!("Prover address: {:?}", strings[3]);
        println!("Prover address: {:?}", prover_address);

        Ok((exploit_found, hashed_program_spec, hashed_context_state, prover_address))
    }

    fn encode_public_input_manual(
        exploit_found: bool,
        hashed_program_spec: &[u8; 32],
        hashed_context_state: &[u8; 32],
        prover_address: Address,
    ) -> Vec<u8> {
        let mut encoded = Vec::with_capacity(32 * 4);

        let mut bool_word = [0u8; 32];
        bool_word[31] = if exploit_found { 1 } else { 0 };
        encoded.extend_from_slice(&bool_word);

        encoded.extend_from_slice(hashed_program_spec);
        encoded.extend_from_slice(hashed_context_state);

        let mut addr_word = [0u8; 32];
        addr_word[12..].copy_from_slice(prover_address.as_slice());
        encoded.extend_from_slice(&addr_word);

        encoded
    }

    #[test]
    fn test_onchain_verify_basic_vuln() -> Result<(), Box<dyn std::error::Error>>{
        dotenv().ok();
        let private_key = std::env::var("WALLET_PRIV_KEY").expect("PRIVATE_KEY must be set in the environment");
        let journal= std::fs::read("../journal.bin")?;
        let seal = std::fs::read("../seal.bin")?;
        let encoded_journal = ethers::abi::encode(&[ethers::abi::Token::Bytes(journal.clone())]);

        let journal_hash = Sha256::digest(&encoded_journal);
        let seal_hex = format!("0x{}", hencode(&seal));
        let journal_hash_hex = format!("0x{}", hencode(journal_hash));
        println!("Journal hash: 0x{}", journal_hash_hex);
        println!("Seal hex: {}", seal_hex);

         let output = evm_utils::call_verify_function(
            &private_key,
            VERIFIER_TESTNET_HOLESKY_ADDRESS,
            &journal_hash_hex,
            &seal_hex,
        )?;

        println!("Output: {}", output);
        println!("✅ Tutti i check passati: exploit valido. Il contratto invierebbe ETH.");
       
        Ok(())

    }
}
