use shared::evm_utils;
use dotenv::dotenv;
use std::env;
use std::fs;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let private_key = env::var("WALLET_PRIV_KEY")?;



    let contract_bytecode_file_imageid = "./bytecode/ImageID.bin";
    let contract_bytecode_deployment_imageid = fs::read_to_string(contract_bytecode_file_imageid).expect("Failed to read contract bytecode file");
    let imageid_deploy_output = evm_utils::deploy_contract(&private_key, &contract_bytecode_deployment_imageid)?;
    let image_id = evm_utils::extract_contract_address(&imageid_deploy_output).expect("Failed to extract contract address").to_string();
    println!("ImageID contract deployed at address: {}", image_id);

    let contract_bytecode_file_verifier = "./bytecode/VerifierContract.bin";
    let contract_bytecode_deployment_verifier = fs::read_to_string(contract_bytecode_file_verifier).expect("Failed to read contract bytecode file");
    let output = evm_utils::deploy_verifier_contract(&private_key, &contract_bytecode_deployment_verifier,"0xf70aBAb028Eb6F4100A24B203E113D94E87DE93C","0x8a2e07835061b39c920fd3356a7722f0ae10a7f4508151831cf4febe733a0279","0x50cbccdabb6afaac348de5304e8ad32dc2a7ea655c031388f989f2b5faaf0552",&image_id)?;
    let td_address = evm_utils::extract_contract_address(&output).expect("Failed to extract contract address").to_string().trim_start_matches("0x").to_string();
    println!("Verifier contract deployed at address: 0x{}", td_address);

    

    Ok(())
}
//context State: 0x50cbccdabb6afaac348de5304e8ad32dc2a7ea655c031388f989f2b5faaf0552
//Prog Spec :0x8a2e07835061b39c920fd3356a7722f0ae10a7f4508151831cf4febe733a0279 