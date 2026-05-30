use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionDocument {
    #[serde(rename = "txHash")]
    pub tx_hash: String,
    #[serde(rename = "blockHeight")]
    pub block_height: u64,
    #[serde(rename = "blockTime")]
    pub block_time: i64,
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

/// V2 BFT block document stored in MongoDB.
/// Replaces PoW fields (difficulty, nonce, miner) with BFT fields.
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockDocument {
    pub index: u64,
    pub hash: String,
    #[serde(rename = "previousHash")]
    pub previous_hash: String,
    pub timestamp: i64,
    /// BFT epoch this block belongs to.
    pub epoch: u64,
    /// Tendermint voting round in which 2/3+ agreement was reached.
    #[serde(rename = "bftRound")]
    pub bft_round: u32,
    /// Address of the validator that proposed this block.
    pub proposer: String,
    /// Number of BFT validators that signed this block.
    #[serde(rename = "sigCount")]
    pub sig_count: usize,
    #[serde(rename = "txCount")]
    pub tx_count: u64,
    pub transactions: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexerStateDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    pub key: String,
    pub value: i64,
}

// ── Data structures from the Node API ─────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct NodeStats {
    pub chain_length: u64,
    pub total_transactions: u64,
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

/// V2 BFT block as returned by GET /api/block/:height
#[derive(Debug, Deserialize)]
pub struct NodeBlock {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<serde_json::Value>,
    pub previous_hash: String,
    pub hash: String,
    // BFT fields
    pub epoch: u64,
    pub bft_round: u32,
    pub proposer: String,
    pub bft_signatures: Vec<serde_json::Value>,
}
