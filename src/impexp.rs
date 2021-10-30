use crate::client::Client;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, io::Write};
use trinci_core::{
    base::serialize::{rmp_deserialize, rmp_serialize},
    Transaction,
};

#[derive(Serialize, Deserialize, Debug)]
struct FileBlock {
    height: u64,
    txs: Vec<Transaction>,
}

#[inline]
fn write_len(file: &mut File, len: u64) {
    let len_buf: [u8; 8] = len.to_be_bytes();
    file.write_all(&len_buf).expect("Writing to file");
}

#[inline]
fn read_len(file: &mut File) -> Option<u64> {
    let mut len_buf: [u8; 8] = [0; 8];
    match file.read_exact(&mut len_buf) {
        Ok(_) => Some(u64::from_be_bytes(len_buf)),
        Err(_) => None,
    }
}

impl FileBlock {
    fn from_file(file: &mut File) -> Option<Self> {
        let len = match read_len(file) {
            Some(len) => len,
            None => return None,
        };
        let mut buf: Vec<u8> = vec![0; len as usize];
        file.read_exact(&mut buf).expect("Unexpected file content");
        let blk: FileBlock = rmp_deserialize(&buf).expect("Deserializing block");
        Some(blk)
    }

    fn to_file(&self, file: &mut File) {
        let buf = rmp_serialize(&self).expect("Serializing block");
        write_len(file, buf.len() as u64);
        file.write_all(&buf).expect("Writing to file");
    }
}

pub fn export_txs(file_name: &str, client: &mut Client) {
    let (last_block, _) = client.get_block(u64::MAX).expect("No block");

    let mut file = match File::create(file_name) {
        Err(why) => panic!("couldn't create {}: {}", file_name, why),
        Ok(file) => file,
    };
    let bar = ProgressBar::new(last_block.height);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue}  {msg}")
            .progress_chars("#>-"),
    );

    // write total number of blocks
    write_len(&mut file, last_block.height + 1);

    for i in 0..=last_block.height {
        let (_block, tx_hashes) = client.get_block(i).unwrap();
        //print!("Block: {:?}",block);

        let mut txs = vec![];
        tx_hashes.iter().for_each(|hash| {
            let tx = client.get_transaction(*hash).unwrap();
            //println!("TX: {:?}",buf);
            txs.push(tx);
        });
        let file_blk = FileBlock { height: i, txs };
        file_blk.to_file(&mut file);

        bar.set_message(format!("blocks: {}/{}", i + 1, last_block.height + 1));
        bar.inc(1);
    }
    bar.finish();
    println!("All transaction exported to '{}'", file_name);
    println!("Final state hash: '{}'", hex::encode(last_block.state_hash));
}

pub fn import_txs(file_name: &str, client: &mut Client) {
    let mut file = match File::open(file_name) {
        Err(why) => panic!("couldn't open {}: {}", file_name, why),
        Ok(file) => file,
    };

    let num_blocks = read_len(&mut file).expect("Bad file format");

    let bar = ProgressBar::new(num_blocks);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue}  {msg}")
            .progress_chars("#>-"),
    );

    for i in 0..num_blocks {
        let file_blk = match FileBlock::from_file(&mut file) {
            Some(blk) => blk,
            None => panic!("Unexpected file early termination"),
        };
        let mut hashes = vec![];
        for tx in file_blk.txs {
            let hash = client
                .put_transaction(tx)
                .expect("None hash sending transaction");
            hashes.push(hash);
        }
        // Wait for block execution
        for hash in hashes {
            let mut trials = 10;
            // Use `get_receipt_err` to prevent printing temporary "not found" errors
            // while we're waiting for tx execution.
            while let Err(_) = client.get_receipt_err(hash) {
                if trials == 0 {
                    panic!(
                        "Error waiting for tx execution (hash: {}, block: {})",
                        hex::encode(&hash),
                        file_blk.height
                    );
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
                trials -= 1;
            }
        }

        bar.set_message(format!("blocks: {}/{}", i + 1, num_blocks));
        bar.inc(1);
    }
    bar.finish();

    let (last_block, _) = client.get_block(u64::MAX).expect("No block");
    if last_block.height + 1 != num_blocks {
        println!(
            "Warning: number of blocks differ from the original one (produced: {}, expected {})",
            last_block.height + 1,
            num_blocks
        );
    }
    println!("All transaction imported from '{}'", file_name);
    println!("Final state hash: '{}'", hex::encode(last_block.state_hash));
}
