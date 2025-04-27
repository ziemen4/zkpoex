// SPDX-License-Identifier: MIT

use anyhow::Context;
use std::fs;

// Shared project modules
use shared::{evm_utils::{encode_seal, get_onchain_links}, utils::parse_cli_args_onchain_verifier};

use alloy::{
    network::{EthereumWallet, TransactionBuilder},
    primitives::Address,
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol,
};

use tracing_subscriber::EnvFilter;
use url::Url;

sol! {
    interface VerifierContract {
        function verify(address beneficiary, bytes calldata seal, bytes calldata journal) public payable;
    }
}

#[allow(dead_code)]
pub fn load_receipt_binary(path: &str) -> risc0_zkvm::Receipt {
    let data = fs::read(path).expect("Failed to read receipt file");
    bincode::deserialize(&data).expect("Failed to deserialize receipt")
}

pub async fn verify_onchain(
    private_key: &str,
    eth_rpc_url: &str,
    contract_address: &str,
    onchain_seal: Vec<u8>,
    onchain_journal: Vec<u8>,
) -> Result<(), anyhow::Error> {
    // Build PrivateKeySigner with private_key
    let private_key_signer = private_key.parse::<PrivateKeySigner>().unwrap();
    let wallet = EthereumWallet::from(private_key_signer.clone());
    let rpc_url: Url = eth_rpc_url.parse().unwrap();
    let provider = ProviderBuilder::new().wallet(wallet).on_http(rpc_url);

    // Get the beneficiary address from the private key
    let beneficiary_address = private_key_signer.address();

    // build calldata
    let calldata = VerifierContract::verifyCall {
        beneficiary: beneficiary_address,
        seal: onchain_seal.into(),
        journal: onchain_journal.into(),
    };

    shared::log_info!("Smart-Contract Address: {}", contract_address);
    let address_contract = contract_address
        .trim()
        .trim_start_matches("0x")
        .parse::<Address>()?;

    let tx = TransactionRequest::default()
        .with_to(address_contract)
        .with_call(&calldata);

    let transaction_result = provider
        .send_transaction(tx)
        .await
        .context("Failed to send transaction")?;
    let tx_hash = transaction_result.tx_hash().to_string();
    println!("\n🌐 Transaction sent with hash: {} \n", get_onchain_links(&tx_hash));

    // Return
    Ok(())
}

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();

    let private_key_str =
        std::env::var("WALLET_PRIV_KEY").expect("PRIVATE_KEY must be set in the .env file");
    let eth_rpc_url_str =
        std::env::var("ETH_RPC_URL").expect("ETHEREUM_RPC_URL must be set in the .env file");

    shared::log_info!("Ethereum RPC URL: {}", eth_rpc_url_str);
    shared::log_info!("Private key: {}", private_key_str);

    let matches = parse_cli_args_onchain_verifier();
    let contract_address = matches.get_one::<String>("contract-address").unwrap();
    let verbose = matches.get_flag("verbose");

    println!("Verbose: {}", verbose);
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

    shared::log_debug!("Starting on-chain verifier...");
    let receipt = load_receipt_binary("receipt.bin");

    // Extract the journal from the receipt.
    let onchain_journal = receipt.journal.bytes.clone();

    // Encode the seal with the selector.
    let onchain_seal = encode_seal(&receipt)?;

    // Call the on-chain verification function
    verify_onchain(
        &private_key_str,
        &eth_rpc_url_str,
        contract_address,
        onchain_seal,
        onchain_journal,
    )
    .await
    .context("Failed to verify on-chain")?;

    Ok(())
}
