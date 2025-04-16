// Environment variable loader (.env support)
use dotenv::dotenv;

use hex;
use serde_json;
use std::fs;
use tokio;

// Shared project modules
use shared::{
    conditions::{hash_program_spec, MethodSpec},
    context::hash_context_state,
    evm_utils,
    input::AccountData,
    utils,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Parse CLI arguments
    let matches = utils::parse_cli_args_sc_owner();
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

    // Read the context state file
    let context_state_json = fs::read_to_string(context_state_file).expect("Failed to read file");
    let context_state: Vec<AccountData> = serde_json::from_str(&context_state_json)?;
    let context_state_hash: String = hex::encode(hash_context_state(&context_state));

    // Read the program spec file
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

    // Extract the contract address from the deployment output
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
    pub const VERIFIER_TESTNET_HOLESKY_ADDRESS: &str = "0xb94aA3E7a1CEFd86B5F439d0Ca34aA9D2c612bd9";

    // Dependancies for call_verify_function()
    use alloy_provider::ProviderBuilder;
    use alloy_signer_local::PrivateKeySigner;
    use alloy_sol_types::{sol, SolType};
    use anyhow::Context;
    use hex::encode;
    use std::env;
    use std::fs;
    use url::Url;

    sol! {
        struct PublicInput {
            bool exploitFound;
            bytes32 programSpecHash;
            bytes32 contextStateHash;
            address proverAddress;
        }
    }

    /// -------------------------------------------
    /// Calls the `verify()` function of the deployed VerifierContract
    /// -------------------------------------------
    pub async fn call_verify_function(
        private_key: &str,
        verifier_contract_address: &str,
        public_input: Vec<u8>,
        seal: Vec<u8>,
    ) -> anyhow::Result<()> {
        let rpc_url = env::var("ETH_RPC_URL").context("Impossible to find env var: RPC_URL")?;
        println!("RPC URL: {}", rpc_url);
        let url = Url::parse(&rpc_url)?;
        let pk: PrivateKeySigner = private_key.parse()?;
        let provider = ProviderBuilder::new().wallet(pk).on_http(url);

        sol!(
            #[sol(rpc)]
            VerifierContract,
            "../contracts/out/VerifierContract.sol/VerifierContract.json"
        );

        let verifier = VerifierContract::new(verifier_contract_address.parse()?, provider);
        let verify = verifier.verify(public_input.into(), seal.into());
        let calldata_hex = format!("0x{}", encode(&verify.calldata()));
        fs::write("calldata.txt", &calldata_hex)?;
        verify.call().await?;
        println!("Verify function called successfully");

        Ok(())
    }

    #[tokio::test]
    async fn test_onchain_verify_basic_vuln() -> Result<(), Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();
        let private_key =
            std::env::var("WALLET_PRIV_KEY").expect("PRIVATE_KEY must be set in the .env file");

        let journal = std::fs::read("src/test/journal.bin")?;
        let seal = std::fs::read("src/test/seal.bin")?;

        let input = <PublicInput as SolType>::abi_decode(&journal)
            .expect("Impossible to decode journal.bin");

        println!("\n=====================");
        println!("PUBLIC INPUT DECODED");
        println!("=====================\n");

        println!("Exploit found: {}", input.exploitFound);
        println!(
            "Program spec hash: 0x{}",
            hex::encode(input.programSpecHash)
        );
        println!(
            "Context state hash: 0x{}",
            hex::encode(input.contextStateHash)
        );
        println!("Prover address: 0x{}", hex::encode(input.proverAddress));

        let _output = call_verify_function(
            &private_key,
            VERIFIER_TESTNET_HOLESKY_ADDRESS,
            journal,
            seal,
        )
        .await?;

        println!("✅ All tests passed! Valid proof verified on-chain.");

        Ok(())
    }
}
