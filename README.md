# Quanta Indexer

`quanta-indexer` is a high-performance Rust daemon that synchronizes the Quanta post-quantum blockchain node RPC state directly into a MongoDB instance. 

## Features
- **Continuous Syncing:** Fetches blocks from the node RPC and stores them in MongoDB.
- **Idempotent Storage:** Ensures blocks and transactions are not duplicated using unique indexes.
- **Fast Access:** Optimized for high-speed queries from the QuaScan block explorer.

## Prerequisites
- Rust and Cargo
- MongoDB (Local or Atlas)
- Quanta Node RPC endpoint

## Usage
1. Setup a `.env` file with `MONGODB_URI` and `RPC_URL`.
2. Run `cargo run --release`.
