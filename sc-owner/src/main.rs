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
    evm_utils::{
        deploy_contract, deploy_verifier_contract, send_eth, extract_contract_address, get_onchain_links
    },
    input::AccountData,
    utils,
};
use tracing_subscriber::EnvFilter;

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
    let send_eth_value = matches.get_one::<u64>("send-eth").unwrap();
    let verbose = matches.get_flag("verbose");

    let filter = if verbose {
        // debug for everything
        "debug"
    } else {
        // info+ for everything
        "info"
    };

    let env_filter = EnvFilter::new(filter);

    // Initialize the tracing subscriber with the environment filter
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

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
    shared::log_info!(
        "Deploying ImageID contract with bytecode: {}",
        contract_bytecode_deployment_imageid
    );
    let imageid_deploy_output =
        deploy_contract(&private_key, &contract_bytecode_deployment_imageid)?;
    let image_id = extract_contract_address(&imageid_deploy_output)
        .expect("Failed to extract contract address")
        .to_string();
    shared::log_info!("ImageID contract deployed at: {}", get_onchain_links(&image_id));

    // Deploy the Verifier contract
    let contract_bytecode_file_verifier = "./bytecode/VerifierContract.bin";
    let contract_bytecode_deployment_verifier = fs::read_to_string(contract_bytecode_file_verifier)
        .expect("Failed to read contract bytecode file");
    let output = deploy_verifier_contract(
        &private_key,
        &contract_bytecode_deployment_verifier,
        &risc0_verifier_contract_address,
        &program_spec_hash,
        &context_state_hash,
        &image_id,
    )?;

    // Extract the contract address from the deployment output
    let verifier_address = extract_contract_address(&output)
        .expect("Failed to extract contract address")
        .to_string();

    // Send ETH to the contract address
    if *send_eth_value > 0 {
        let send_eth_output =
            send_eth(&private_key, &verifier_address, *send_eth_value).await?;
        shared::log_info!("ETH sent to contract address: {}", send_eth_output);
    } else {
        shared::log_info!("No ETH sent to contract address.");
    }

    shared::log_info!(
        "\nVerifier contract deployed at: {}",
        get_onchain_links(&verifier_address)
    );

    Ok(())
}
