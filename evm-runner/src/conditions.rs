
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use alloc::string::String;
use alloc::vec::Vec;
use primitive_types::U256;

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

pub fn hash_conditions(conditions: &[Condition]) -> [u8; 32] {
	// TODO: use SHA3? Or see what is the best hash function for this
    let mut hasher = Sha256::new();

    for cond in conditions {
        let serialized = serialize_condition(cond);
        hasher.update(serialized);
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
