use crate::models::{
    BlockDocument, IndexerStateDocument, NodeBlock, NodeStats, TransactionDocument,
};
use mongodb::{
    bson::doc,
    options::{FindOneAndUpdateOptions, ReturnDocument},
    Client, Collection,
};
use reqwest::Client as HttpClient;
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio::time::sleep;

pub struct Indexer {
    db: mongodb::Database,
    http: HttpClient,
    rpc_url: String,
}

impl Indexer {
    pub fn new(mongo_client: Client, db_name: &str, rpc_url: String) -> Self {
        Self {
            db: mongo_client.database(db_name),
            http: HttpClient::new(),
            rpc_url: rpc_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn run(&self) {
        println!("Starting Quanta background indexer...");
        let state_col: Collection<IndexerStateDocument> = self.db.collection("indexerstates");

        loop {
            if let Err(e) = self.sync_step(&state_col).await {
                eprintln!("Error during sync step: {}", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }

    async fn sync_step(&self, state_col: &Collection<IndexerStateDocument>) -> Result<(), Box<dyn std::error::Error>> {
        let stats: NodeStats = self
            .http
            .get(&format!("{}/api/stats", self.rpc_url))
            .send()
            .await?
            .json()
            .await?;

        if stats.chain_length == 0 {
            sleep(Duration::from_secs(10)).await;
            return Ok(());
        }

        let chain_height = stats.chain_length - 1;

        let state_doc = state_col
            .find_one_and_update(
                doc! { "key": "last_indexed_block" },
                doc! { "$setOnInsert": { "key": "last_indexed_block", "value": -1_i64 } },
                FindOneAndUpdateOptions::builder()
                    .upsert(true)
                    .return_document(ReturnDocument::After)
                    .build(),
            )
            .await?;

        let last_indexed = state_doc.map(|d| d.value).unwrap_or(-1);

        if last_indexed >= chain_height as i64 {
            // Already synced, wait for new blocks
            sleep(Duration::from_secs(10)).await;
            return Ok(());
        }

        let to_index = chain_height as i64 - last_indexed;
        let batch_size = std::cmp::min(50, to_index);
        let start_height = (last_indexed + 1) as u64;
        let end_height = start_height + batch_size as u64 - 1;

        for height in start_height..=end_height {
            self.process_block(height).await?;

            // Update state
            state_col
                .update_one(
                    doc! { "key": "last_indexed_block" },
                    doc! { "$set": { "value": height as i64 } },
                    None,
                )
                .await?;
            
            if height % 10 == 0 || height == end_height {
                println!("Indexed block {} / {}", height, chain_height);
            }
        }

        Ok(())
    }

    async fn process_block(&self, height: u64) -> Result<(), Box<dyn std::error::Error>> {
        let block: NodeBlock = self
            .http
            .get(&format!("{}/api/block/{}", self.rpc_url, height))
            .send()
            .await?
            .json()
            .await?;

        let block_col: Collection<BlockDocument> = self.db.collection("blocks");
        let tx_col: Collection<TransactionDocument> = self.db.collection("transactions");

        let mut tx_docs = Vec::new();

        for tx_val in &block.transactions {
            if let Some(tx_obj) = tx_val.as_object() {
                let sender = tx_obj
                    .get("sender")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let recipient = tx_obj
                    .get("recipient")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let amount = tx_obj.get("amount").and_then(|v| v.as_u64()).unwrap_or(0);
                let fee = tx_obj.get("fee").and_then(|v| v.as_u64()).unwrap_or(0);
                let signature = tx_obj.get("signature").map(|v| {
                    if let Some(arr) = v.as_array() {
                        let bytes: Vec<u8> = arr.iter().filter_map(|num| num.as_u64().map(|n| n as u8)).collect();
                        hex::encode(bytes)
                    } else {
                        v.as_str().unwrap_or("").to_string()
                    }
                }).unwrap_or_default();
                let pub_key = tx_obj.get("public_key").map(|v| {
                    if let Some(arr) = v.as_array() {
                        let bytes: Vec<u8> = arr.iter().filter_map(|num| num.as_u64().map(|n| n as u8)).collect();
                        hex::encode(bytes)
                    } else {
                        v.as_str().unwrap_or("").to_string()
                    }
                }).unwrap_or_default();

                let is_coinbase = sender == "COINBASE"
                    || sender == "0000000000000000000000000000000000000000000000000000000000000000";

                let tx_hash = tx_obj
                    .get("tx_hash")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| {
                        let mut hasher = Sha256::new();
                        if is_coinbase {
                            hasher.update(
                                format!("coinbase_{}_{}", block.index, recipient).as_bytes(),
                            );
                        } else {
                            hasher.update(signature.as_bytes());
                        }
                        hex::encode(hasher.finalize())
                    });

                tx_docs.push(TransactionDocument {
                    tx_hash,
                    block_height: block.index,
                    block_time: block.timestamp,
                    sender,
                    recipient,
                    amount_microunits: amount,
                    fee_microunits: fee,
                    signature,
                    public_key: pub_key,
                    tx_type: if is_coinbase {
                        "COINBASE".to_string()
                    } else {
                        "TRANSFER".to_string()
                    },
                });
            }
        }

        // V2 BFT: proposer is an explicit field on the block — no coinbase scan needed
        let block_doc = BlockDocument {
            index: block.index,
            hash: block.hash,
            previous_hash: block.previous_hash,
            timestamp: block.timestamp,
            epoch: block.epoch,
            bft_round: block.bft_round,
            proposer: block.proposer,
            sig_count: block.bft_signatures.len(),
            tx_count: block.transactions.len() as u64,
            transactions: block.transactions,
        };

        // Insert block (ignore duplicate on restart)
        let _ = block_col.insert_one(block_doc, None).await;

        // Insert transactions
        if !tx_docs.is_empty() {
            let options = mongodb::options::InsertManyOptions::builder()
                .ordered(false)
                .build();
            let _ = tx_col.insert_many(tx_docs, Some(options)).await;
        }

        Ok(())
    }
}
