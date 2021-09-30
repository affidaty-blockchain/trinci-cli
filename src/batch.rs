// This file is part of TRINCI.
//
// Copyright (C) 2021 Affidaty Spa.
//
// TRINCI is free software: you can redistribute it and/or modify it under
// the terms of the GNU Affero General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at your
// option) any later version.
//
// TRINCI is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License
// for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with TRINCI. If not, see <https://www.gnu.org/licenses/>.

use crate::{client::Client, utils};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
use trinci_core::{base::serialize::rmp_deserialize, Message};

#[derive(Debug, Serialize, Deserialize)]
struct TableEntry<'a> {
    desc: &'a str,
    file: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
struct BatchConfig<'a> {
    #[serde(default)]
    interactive: bool,
    #[serde(borrow)]
    batch: BTreeMap<String, Vec<TableEntry<'a>>>,
}

pub fn run(mut client: Client, filename: String) {
    let filepath = Path::new(filename.as_str());
    let dir = filepath
        .parent()
        .unwrap()
        .as_os_str()
        .to_string_lossy()
        .to_string();

    let content = match std::fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file: {}", err);
            return;
        }
    };
    let config: BatchConfig = match toml::from_str(&content) {
        Ok(map) => map,
        Err(err) => {
            eprintln!("Bad batch file format: {}", err);
            return;
        }
    };

    for (block, txs) in config.batch.iter() {
        println!("\nSubmitting transactions batch: {}", block);
        let mut tickets = vec![];
        for tx in txs {
            println!(" * {}", tx.desc);
            if config.interactive {
                utils::print_unbuf("Execute [Y/n]? ");
                let input = utils::get_input().to_lowercase();
                if !input.is_empty() && input != "y" && input != "yes" {
                    continue;
                }
            }
            let file = dir.clone() + "/" + tx.file;
            let msg_bin = std::fs::read(file).unwrap();
            let msg: Message = rmp_deserialize(&msg_bin).unwrap();
            let tx = match msg {
                Message::PutTransactionRequest { confirm: _, tx } => tx,
                _ => panic!("Expected put transaction request message"),
            };
            let ticket = match client.put_transaction(tx) {
                Some(ticket) => ticket,
                None => {
                    eprint!("Something bad happened. Stop batch execution [Y/n]? ");
                    let input = utils::get_input().to_lowercase();
                    if !input.is_empty() && input != "y" && input != "yes" {
                        continue;
                    }
                    return;
                }
            };
            tickets.push(ticket);
        }

        // wait for confirmations of the batch...
        std::thread::sleep(std::time::Duration::from_secs(3));
        for hash in tickets {
            println!("Waiting for: {}", hex::encode(hash));
            let mut trials = 3;
            loop {
                match client.get_receipt(hash) {
                    Some(_) => break,
                    None if trials > 0 => {
                        trials -= 1;
                        println!("... not ready, retry in 3 seconds");
                        std::thread::sleep(std::time::Duration::from_secs(3));
                        continue;
                    }
                    None => {
                        eprintln!("Timeout waiting for receipt");
                        break;
                    }
                }
            }
        }
    }
}
