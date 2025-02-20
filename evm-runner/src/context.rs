use std::collections::BTreeMap;
use ethereum_types::{H256, U256};
use crate::input::AccountData;
use sha3::{Digest,Keccak256};

#[derive(Debug)]
pub enum ContextAccountDataType {
    ERC20,
}
const CONTEXT_ERC20_CONTRACT_ADDRESS: &str = "E4C2000000000000000000000000000000000000";
const CONTEXT_ERC20_CONTRACT_BYTECODE: &str = include_str!("../../bytecode/ContextERC20.bin-runtime");

pub fn build_context_account_data(
    context_account_data_type: ContextAccountDataType,
    init_storage: Option<BTreeMap<H256, H256>>
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
        ContextAccountDataType::ERC20 => {
            AccountData {
                address: CONTEXT_ERC20_CONTRACT_ADDRESS.to_string(),
                nonce: U256::zero(),
                balance: U256::zero(),
                storage,
                code: hex::decode(CONTEXT_ERC20_CONTRACT_BYTECODE).unwrap(),
            }
        }
    }
}

pub fn hash_context_data(context_data: &[AccountData]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    
    // Hash each bytecode chunk directly as raw bytes
    for cdata in context_data {
        hasher.update(&cdata.code);  // Use raw bytes directly
    }
    
    hasher.finalize().into()
}