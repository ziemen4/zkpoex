use shared::log_info;
use anyhow::Context;
use std::fs;

// Shared project modules
use shared::{
    evm_utils::{encode_seal},
    utils::{parse_cli_args_onchain_verifier},
};

use alloy::{
    network::{EthereumWallet,TransactionBuilder},
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
        function verify(bytes calldata seal, bytes calldata journal) public payable;
    }
}

fn load_receipt_binary(path: &str) -> risc0_zkvm::Receipt {
    let data = fs::read(path).expect("Failed to read receipt file");
    bincode::deserialize(&data).expect("Failed to deserialize receipt")
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();

    let private_key_str = std::env::var("WALLET_PRIV_KEY")?;
    let eth_rpc_url_str = std::env::var("ETH_RPC_URL")?;
    let matches = parse_cli_args_onchain_verifier();
    let smart_contract = matches.get_one::<String>("smart-contract").unwrap();

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

    shared::log_info!("Ethereum RPC URL: {}", eth_rpc_url_str);
    shared::log_info!("Private key: {}", private_key_str);
    let private_key = private_key_str.parse::<PrivateKeySigner>()?;
    let wallet = EthereumWallet::from(private_key);
    let eth_rpc_url: Url = eth_rpc_url_str.parse()?;
    let provider = ProviderBuilder::new().wallet(wallet).on_http(eth_rpc_url);

    // build calldata
    let calldata = VerifierContract::verifyCall {
        seal: onchain_seal.into(),
        journal: onchain_journal.into(),
    };

    shared::log_info!("Smart-Contract Address: {}", smart_contract);
    let address_contract = smart_contract.parse::<Address>()?;

    let tx = TransactionRequest::default()
        .with_to(address_contract)
        .with_call(&calldata);

    let transaction_result = provider
        .send_transaction(tx)
        .await
        .context("Failed to send transaction")?;
    let tx_hash = transaction_result.tx_hash();
    println!("🌐 Transaction sent with hash: {:?}", tx_hash);
    Ok(())
}
