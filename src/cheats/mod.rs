use serde::{Deserialize, Serialize};
use trinci_core::Transaction;

pub mod advanced_asset;
pub mod commons;

/// WARNING: Edit this structure could break the node
/// This structure should be the same of the Bootstrap
/// struct on the Node
#[derive(Serialize, Deserialize)]
struct Bootstrap {
    // Binary bootstrap.wasm
    #[serde(with = "serde_bytes")]
    bin: Vec<u8>,
    // Vec of transaction for the genesis block
    txs: Vec<Transaction>,
    // Random string to generate unique file
    nonce: String,
}
