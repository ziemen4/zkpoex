use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use primitive_types::{H256, U256};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountData {
    pub address: String,
    pub nonce: U256,
    pub balance: U256,
    pub storage: BTreeMap<H256, H256>,
    pub code: Vec<u8>,
}
