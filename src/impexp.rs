use crate::client::Client;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fs::File, io::Read, io::Write};
use trinci_core::{
    base::serialize::{rmp_deserialize, rmp_serialize},
    Transaction,
};

const MAGIC: u64 = 0xe7aa30ddc4db5bdd;

#[derive(Serialize, Deserialize, Debug)]
struct FileHeader {
    magic: u64,
    num_blocks: u64,
}

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

fn write_to_file<T: Serialize>(file: &mut File, value: T) {
    let buf = rmp_serialize(&value).expect("serializing");
    write_len(file, buf.len() as u64);
    file.write_all(&buf).expect("Writing to file");
}

fn read_from_file<T: DeserializeOwned>(file: &mut File) -> Option<T> {
    let len = match read_len(file) {
        Some(len) => len,
        None => return None,
    };
    let mut buf: Vec<u8> = vec![0; len as usize];
    file.read_exact(&mut buf).expect("Unexpected file content");
    let value: T = rmp_deserialize(&buf).expect("Deserializing block");
    Some(value)
}

pub fn export_txs(file_name: &str, client: &mut Client) {
    let (last_block, _) = client.get_block(u64::MAX).expect("No block");

    let mut file = match File::create(file_name) {
        Err(why) => panic!("couldn't create {}: {}", file_name, why),
        Ok(file) => file,
    };
    let bar = ProgressBar::new(last_block.data.height);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue}  {msg}")
            .progress_chars("#>-"),
    );

    let head = FileHeader {
        magic: MAGIC,
        num_blocks: last_block.data.height + 1,
    };
    write_to_file(&mut file, head);

    for i in 0..=last_block.data.height {
        let (_block, tx_hashes) = client.get_block(i).unwrap();
        //print!("Block: {:?}",block);

        let mut txs = vec![];
        tx_hashes.iter().for_each(|hash| {
            let tx = client.get_transaction(*hash).unwrap();
            //println!("TX: {:?}",buf);
            txs.push(tx);
        });
        let file_blk = FileBlock { height: i, txs };
        write_to_file(&mut file, file_blk);

        bar.set_message(format!("blocks: {}/{}", i + 1, last_block.data.height + 1));
        bar.inc(1);
    }
    bar.finish();
    println!("All transaction exported to '{}'", file_name);
    println!(
        "Final state hash: '{}'",
        hex::encode(last_block.data.state_hash)
    );
}

pub fn import_txs(file_name: &str, client: &mut Client) {
    let mut file = match File::open(file_name) {
        Err(why) => panic!("couldn't open {}: {}", file_name, why),
        Ok(file) => file,
    };

    let head: FileHeader = read_from_file(&mut file).expect("Bad file format");
    if head.magic != MAGIC {
        panic!("Bad magic number not found");
    }

    let bar = ProgressBar::new(head.num_blocks);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue}  {msg}")
            .progress_chars("#>-"),
    );

    for i in 0..head.num_blocks {
        let file_blk = match read_from_file::<FileBlock>(&mut file) {
            Some(blk) => blk,
            None => panic!("Unexpected file early termination"),
        };
        let mut hashes = vec![];
        for tx in file_blk.txs {
            if let Some(hash) = client.put_transaction(tx) {
                hashes.push(hash)
            }
        }
        // Wait for block execution
        for hash in hashes {
            let mut trials = 10;
            // Use `get_receipt_err` to prevent printing temporary "not found" errors
            // while we're waiting for tx execution.
            while client.get_receipt_err(hash).is_err() {
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

        bar.set_message(format!("blocks: {}/{}", i + 1, head.num_blocks));
        bar.inc(1);
    }
    bar.finish();

    let (last_block, _) = client.get_block(u64::MAX).expect("No block");
    if last_block.data.height + 1 != head.num_blocks {
        println!(
            "Warning: number of blocks differ from the original one (produced: {}, expected {})",
            last_block.data.height + 1,
            head.num_blocks
        );
    }
    println!("All transaction imported from '{}'", file_name);
    println!(
        "Final state hash: '{}'",
        hex::encode(last_block.data.state_hash)
    );
}
