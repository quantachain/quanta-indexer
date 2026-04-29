mod indexer;
mod models;

use dotenv::dotenv;
use mongodb::{options::ClientOptions, Client};
use std::env;
use indexer::Indexer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let mongodb_uri = env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://127.0.0.1:27017/quanta".to_string());
    
    let rpc_url = env::var("RPC_URL")
        .unwrap_or_else(|_| "https://rpc.quantachain.org".to_string());

    let mut client_options = ClientOptions::parse(&mongodb_uri).await?;
    client_options.app_name = Some("QuantaIndexer".to_string());

    let client = Client::with_options(client_options)?;

    // Ensure indexes
    let db = client.database("quanta");
    setup_indexes(&db).await?;

    let indexer = Indexer::new(client, "quanta", rpc_url);
    indexer.run().await;

    Ok(())
}

async fn setup_indexes(db: &mongodb::Database) -> Result<(), Box<dyn std::error::Error>> {
    use mongodb::{bson::doc, IndexModel};

    // Blocks
    let blocks = db.collection::<mongodb::bson::Document>("blocks");
    let index_model = IndexModel::builder()
        .keys(doc! { "index": -1 })
        .options(mongodb::options::IndexOptions::builder().unique(true).build())
        .build();
    blocks.create_index(index_model, None).await?;

    let index_model2 = IndexModel::builder().keys(doc! { "hash": 1 }).build();
    blocks.create_index(index_model2, None).await?;

    // Transactions
    let txs = db.collection::<mongodb::bson::Document>("transactions");
    let tx_index1 = IndexModel::builder()
        .keys(doc! { "txHash": 1 })
        .options(mongodb::options::IndexOptions::builder().unique(true).build())
        .build();
    txs.create_index(tx_index1, None).await?;

    let tx_index2 = IndexModel::builder().keys(doc! { "sender": 1 }).build();
    txs.create_index(tx_index2, None).await?;

    let tx_index3 = IndexModel::builder().keys(doc! { "recipient": 1 }).build();
    txs.create_index(tx_index3, None).await?;

    let tx_index4 = IndexModel::builder().keys(doc! { "blockHeight": -1 }).build();
    txs.create_index(tx_index4, None).await?;

    println!("MongoDB indexes configured.");
    Ok(())
}
