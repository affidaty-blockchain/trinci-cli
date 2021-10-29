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

use crate::{cheats, client::Client, common::build_transaction, impexp, utils};
use trinci_core::{
    crypto::{Hash, KeyPair},
    Transaction,
};

const HISTORY_FILE: &str = ".history";

const HELP: &str = "help";
const QUIT: &str = "quit";
const PUT_TX: &str = "put-tx";
const GET_TX: &str = "get-tx";
const GET_RX: &str = "get-rx";
const GET_ACCOUNT: &str = "get-acc";
const GET_BLOCK: &str = "get-blk";
const EXPORT_TXS: &str = "export-txs";
const IMPORT_TXS: &str = "import-txs";
const CHEATS: &str = "cheats";

fn help() {
    println!("Available commands:");
    println!(" * '{}': pr-nt this help", HELP);
    println!(" * '{}': submit a transaction and get ticket", PUT_TX);
    println!(" * '{} <tkt>': get transaction by ticket", GET_TX);
    println!(" * '{} <tkt>': get receipt by transaction ticket", GET_RX);
    println!(" * '{} <id>': get account by id", GET_ACCOUNT);
    println!(" * '{} <height>': get block at a given height", GET_BLOCK);
    println!(
        " * '{} <file>': export all transactions into a file",
        EXPORT_TXS
    );
    println!(
        " * '{} <file>': import transactions from a file",
        IMPORT_TXS
    );
    println!(" * '{}': shortcuts for common tasks", CHEATS);
    println!(" * '{}': exit the application", QUIT);
}

fn build_transaction_interactive(caller: &KeyPair) -> Transaction {
    utils::print_unbuf("  Network: ");
    let network = utils::get_input();

    utils::print_unbuf("  Target account: ");
    let target = utils::get_input();

    utils::print_unbuf("  Contract (optional hex string): ");
    let input = utils::get_input();
    let contract = if input.is_empty() {
        None
    } else {
        match Hash::from_hex(&input) {
            Ok(hash) => Some(hash),
            Err(_err) => {
                eprintln!("  Invalid contract format (using null)");
                None
            }
        }
    };

    utils::print_unbuf("  Method: ");
    let method = utils::get_input();

    utils::print_unbuf("  Args (hex string): ");
    let args = utils::get_input();
    let args = match hex::decode(&args) {
        Ok(buf) => buf,
        Err(err) => panic!("Error: {}", err),
    };

    build_transaction(caller, network, target, contract, method, args)
}

pub fn run(mut client: Client) {
    let mut rl = rustyline::Editor::<()>::new();
    rl.load_history(HISTORY_FILE)
        .unwrap_or_else(|_err| println!("No previous history"));

    println!(
        "Keypair account ID: {}\n",
        client.keypair.public_key().to_account_id()
    );
    println!("Enter 'help' to show available commands\n");

    loop {
        let line = match rl.readline("> ") {
            Ok(line) => {
                rl.add_history_entry(&line);
                line
            }
            Err(rustyline::error::ReadlineError::Interrupted)
            | Err(rustyline::error::ReadlineError::Eof) => break,
            Err(_) => continue,
        };

        let splitted: Vec<_> = line.split(' ').collect();
        let command = match splitted.get(0) {
            Some(&value) => value,
            None => {
                println!("Invalid command");
                continue;
            }
        };

        match command {
            PUT_TX => {
                let tx = build_transaction_interactive(&client.keypair);
                client.put_transaction(tx);
            }
            GET_TX => {
                let hash = match splitted.get(1).and_then(|s| Hash::from_hex(s).ok()) {
                    Some(hash) => hash,
                    _ => {
                        println!("Bad input, please provide transaction ticket");
                        continue;
                    }
                };
                client.get_transaction(hash);
            }
            GET_RX => {
                let hash = match splitted.get(1).and_then(|s| Hash::from_hex(s).ok()) {
                    Some(hash) => hash,
                    _ => {
                        println!("Bad input, please provide transaction ticket");
                        continue;
                    }
                };
                client.get_receipt(hash);
            }
            GET_ACCOUNT => {
                let id = match splitted.get(1) {
                    Some(&id) => id,
                    _ => {
                        println!("Bad input, please provide account id");
                        continue;
                    }
                };
                client.get_account(id.to_owned());
            }
            GET_BLOCK => {
                let height = match splitted.get(1).and_then(|s| s.parse::<u64>().ok()) {
                    Some(height) => height,
                    _ => {
                        println!("Bad input, please provide block (numeric) height");
                        continue;
                    }
                };
                client.get_block(height);
            }
            EXPORT_TXS => {
                let _file_name = match splitted.get(1) {
                    Some(file) => {
                        println!("Exporting transaction -> {}", file);
                        impexp::export_txs(file, &mut client);
                    }
                    _ => {
                        println!("Insert a valid file name");
                    }
                };
                continue;
            }
            IMPORT_TXS => {
                let _file_name = match splitted.get(1) {
                    Some(file) => {
                        println!("Importing transaction from {} ", file);
                        impexp::import_txs(file, &mut client);
                    }
                    _ => {
                        println!("Insert a valid file name");
                    }
                };
                continue;
            }
            CHEATS => {
                cheats::run(&mut client, &mut rl);
                help();
            }
            QUIT => break,
            HELP => help(),
            _ => {
                println!("Command not found");
                help();
            }
        }
    }
    rl.save_history(HISTORY_FILE)
        .unwrap_or_else(|err| eprintln!("Error saving history: {}", err));

    println!("Bye!");
}
