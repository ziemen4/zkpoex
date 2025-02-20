
use ethereum_types::{H160, H256};
use serde::{Serialize, Deserialize};
use sha2::Digest;
use alloc::string::String;
use alloc::vec::Vec;
use primitive_types::U256;
use sha3::Keccak256;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Operator {
    Eq,  // Equal
    Neq, // Not equal
    Gt,  // Greater than
    Ge,  // Greater than or equal
    Lt,  // Less than
    Le,  // Less than or equal
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FixedCondition {
    pub k_s: String,         // State key
    pub op: Operator,        // Operation
    pub v: U256,              // Expected value
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RelativeCondition {
    pub k_s: String,         // State key
    pub op: Operator,        // Operation
    pub k_s_prime: String,   // End state key
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Condition {
    Fixed(FixedCondition),
    Relative(RelativeCondition),
}

pub fn hash_program_spec(program_spec: &[(Condition, String)]) -> [u8; 32] {
    let mut hasher = Keccak256::new();

    for (cond, method) in program_spec {
        let serialized_condition = serialize_condition(cond);
        let serialized_method = method.as_bytes();
        // Concat the serialized condition and method
        let concat_condition_method = [serialized_condition, serialized_method.to_vec()].concat();
        hasher.update(concat_condition_method);
    }

    hasher.finalize().into()
}

fn serialize_fixed_condition(cond: &FixedCondition) -> Vec<u8> {
    let mut serialized = Vec::new();

    serialized.extend(cond.k_s.as_bytes());
    serialized.push(0); // Null terminator

    serialized.push(match cond.op {
        Operator::Eq => 0x00,
        Operator::Neq => 0x01,
        Operator::Gt => 0x02,
        Operator::Ge => 0x03,
        Operator::Lt => 0x04,
        Operator::Le => 0x05,
    });

    let mut v_bytes = [0u8; 32]; // U256 is 32 bytes
    cond.v.to_little_endian(&mut v_bytes);
    serialized.extend(&v_bytes); // 8-byte little-endian
    serialized
}

fn serialize_relative_condition(cond: &RelativeCondition) -> Vec<u8> {
    let mut serialized = Vec::new();

    serialized.extend(cond.k_s.as_bytes());
    serialized.push(0); // Null terminator

    serialized.push(match cond.op {
        Operator::Eq => 0x00,
        Operator::Neq => 0x01,
        Operator::Gt => 0x02,
        Operator::Ge => 0x03,
        Operator::Lt => 0x04,
        Operator::Le => 0x05,
    });

    serialized.extend(cond.k_s_prime.as_bytes());
    serialized.push(0); // Null terminator
    serialized
}

fn serialize_condition(cond: &Condition) -> Vec<u8> {
    match cond {
        Condition::Fixed(fixed) => serialize_fixed_condition(fixed),
        Condition::Relative(relative) => serialize_relative_condition(relative),
    }
}

pub fn compute_mapping_storage_key(key: H160, base_slot: U256) -> H256 {
    // Convert the address to a 32-byte representation (left-padded with zeros)
    let mut padded_key = [0u8; 32];
    padded_key[12..].copy_from_slice(&key.as_bytes());

    // Create a buffer to hold the bytes
    let mut base_bytes = [0u8; 32];

    // Convert the base_slot (U256) to 32 bytes (big-endian)
    base_slot.to_big_endian(&mut base_bytes);

    // Concatenate padded_key and base_bytes
    let mut hasher = Keccak256::new();
    hasher.update(&padded_key);
    hasher.update(&base_bytes);
    let result = hasher.finalize();
    H256::from_slice(&result)
}
