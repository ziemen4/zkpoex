
use serde::{Serialize, Deserialize};
use alloc::vec::Vec;
use primitive_types::{U256,H256};
use alloc::collections::BTreeMap;
use alloc::string::String;

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountData {
    pub address: String,
    pub nonce: U256,
    pub balance: U256,
    pub storage: BTreeMap<H256, H256>,
    pub code: Vec<u8>,
}