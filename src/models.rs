use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionDocument {
    #[serde(rename = "txHash")]
    pub tx_hash: String,
    #[serde(rename = "blockHeight")]
    pub block_height: u64,
    #[serde(rename = "blockTime")]
    pub block_time: u64,
    pub sender: String,
    pub recipient: String,
    #[serde(rename = "amountMicrounits")]
    pub amount_microunits: u64,
    #[serde(rename = "feeMicrounits")]
    pub fee_microunits: u64,
    pub signature: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    #[serde(rename = "txType")]
    pub tx_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockDocument {
    pub index: u64,
    pub hash: String,
    #[serde(rename = "previousHash")]
    pub previous_hash: String,
    pub timestamp: u64,
    pub difficulty: u64,
    pub nonce: u64,
    #[serde(rename = "txCount")]
    pub tx_count: u64,
    pub miner: String,
    // We'll store transactions as raw JSON/Bson within the block to match Next.js schema,
    // but we can map it to `serde_json::Value` or our own struct.
    pub transactions: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexerStateDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    pub key: String,
    pub value: i64,
}

// Data structures from Node API
#[derive(Debug, Deserialize)]
pub struct NodeStats {
    pub chain_length: u64,
    pub total_transactions: u64,
    pub current_difficulty: u64,
    pub pending_transactions: u64,
}

#[derive(Debug, Deserialize)]
pub struct NodeTransaction {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub signature: String,
    pub public_key: String,
}

#[derive(Debug, Deserialize)]
pub struct NodeBlock {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<serde_json::Value>,
    pub nonce: u64,
    pub previous_hash: String,
    pub hash: String,
    pub difficulty: u64,
}
