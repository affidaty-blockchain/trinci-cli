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

use crate::{
    cheats,
    client::Client,
    common::{
        self, build_bulk_transaction, build_unit_transaction, get_args_from_user,
        get_contract_from_user, FUEL_LIMIT,
    },
    impexp, utils,
};
use trinci_core::{
    base::schema::{SignedTransaction, UnsignedTransaction},
    crypto::{Hash, Hashable, KeyPair},
    Transaction,
};

const HISTORY_FILE: &str = ".history";

const HELP: &str = "help";
const QUIT: &str = "quit";
const PUT_TX: &str = "put-tx";
const PUT_BULK_TX: &str = "put-bulk-tx";
const GET_TX: &str = "get-tx";
const GET_RX: &str = "get-rx";
const GET_ACCOUNT: &str = "get-acc";
const GET_BLOCK: &str = "get-blk";
const GET_TXS: &str = "get-txs";
const EXPORT_TXS: &str = "export-txs";
const IMPORT_TXS: &str = "import-txs";
const CHEATS: &str = "cheats";

fn help() {
    println!("Available commands:");
    println!(" * '{}': print this help", HELP);
    println!(" * '{}': submit a transaction and get ticket", PUT_TX);
    println!(
        " * '{}': submit a bulk transaction and get ticket",
        PUT_BULK_TX
    );
    println!(" * '{} <tkt>': get transaction by ticket", GET_TX);
    println!(" * '{} <tkt>': get receipt by transaction ticket", GET_RX);
    println!(" * '{} <id>': get account by id", GET_ACCOUNT);
    println!(
        " * '{} <height>': get block at a given height (use `max` for the last)",
        GET_BLOCK
    );
    println!(" * '{}': get the number of transactions executed", GET_TXS);
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

fn build_transaction_interactive(caller: &KeyPair, config_network: &str) -> Transaction {
    utils::print_unbuf(&format!("  Network [{}]: ", config_network));
    let mut network = utils::get_input();
    if network.is_empty() {
        network = config_network.to_string();
    }

    utils::print_unbuf("  Target account: ");
    let target = utils::get_input();

    let contract = get_contract_from_user();

    utils::print_unbuf(&format!("  Fuel Limit [{}]: ", FUEL_LIMIT));
    let fuel_limit = utils::get_input().parse::<u64>().unwrap_or(FUEL_LIMIT);

    utils::print_unbuf("  Method: ");
    let method = utils::get_input();

    let args = get_args_from_user();

    build_unit_transaction(caller, network, target, contract, method, args, fuel_limit)
}

fn build_bulk_root_transaction_interactive(
    empty_root: bool,
    caller: &KeyPair,
    network: String,
) -> UnsignedTransaction {
    println!("  >> Prepare Root Transaction");

    utils::print_unbuf(&format!("  Fuel Limit [{}]: ", FUEL_LIMIT));
    let fuel_limit = utils::get_input().parse::<u64>().unwrap_or(FUEL_LIMIT);

    let (target_account, contract, method, args) = if empty_root {
        (None, None, None, None)
    } else {
        utils::print_unbuf("  Target account: ");
        let target_account = utils::get_input();

        let contract = get_contract_from_user();

        utils::print_unbuf("  Method: ");
        let method = utils::get_input();

        let args = get_args_from_user();

        (Some(target_account), contract, Some(method), Some(args))
    };

    common::build_bulk_root_transaction(
        empty_root,
        caller,
        network,
        fuel_limit,
        target_account,
        contract,
        method,
        args,
    )
}

fn build_bulk_node_transaction_interactive(
    index: u64,
    network: String,
    depends_on: Hash,
) -> SignedTransaction {
    println!("  >> Prepare Node Transaction {}", index + 1);

    utils::print_unbuf("  Caller Keypair : ");
    let caller_keypair_filename = utils::get_input();
    let caller = utils::load_keypair(Some(caller_keypair_filename)).unwrap();

    utils::print_unbuf(&format!("  Fuel Limit [{}]: ", FUEL_LIMIT));
    let fuel_limit = utils::get_input().parse::<u64>().unwrap_or(FUEL_LIMIT);

    utils::print_unbuf("  Target account: ");
    let target_account = utils::get_input();

    let contract = get_contract_from_user();

    utils::print_unbuf("  Method: ");
    let method = utils::get_input();

    let args = get_args_from_user();

    common::build_bulk_node_transaction(
        &caller,
        network,
        target_account,
        contract,
        method,
        args,
        fuel_limit,
        depends_on,
    )
}

fn build_bulk_transaction_interactive(caller: &KeyPair, config_network: &str) -> Transaction {
    utils::print_unbuf(&format!("  Network [{}]: ", config_network));
    let mut network = utils::get_input();
    if network.is_empty() {
        network = config_network.to_string();
    }

    utils::print_unbuf("  Do you want send an empty root tx? [y/N]: ");
    let input = utils::get_input().to_lowercase();
    let empty_root = !(input.is_empty() || input == "n" || input == "no");

    utils::print_unbuf("  How many node transaction do you want to add? [1]: ");
    let input = utils::get_input();
    let node_tx_no = input.parse::<u64>().unwrap_or(1);

    let root_tx = build_bulk_root_transaction_interactive(empty_root, caller, network.clone());

    let depends_on = root_tx.data.primary_hash();

    let nodes = if node_tx_no == 0 {
        None
    } else {
        Some(
            (0..node_tx_no)
                .into_iter()
                .map(|index| {
                    build_bulk_node_transaction_interactive(index, network.clone(), depends_on)
                })
                .collect::<Vec<SignedTransaction>>(),
        )
    };

    build_bulk_transaction(caller, root_tx, nodes)
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
                let tx = build_transaction_interactive(&client.keypair, &client.network);
                if let Some(hash) = client.put_transaction(tx) {
                    utils::print_serializable(&hash);
                }
            }
            PUT_BULK_TX => {
                let tx = build_bulk_transaction_interactive(&client.keypair, &client.network);
                if let Some(hash) = client.put_transaction(tx) {
                    utils::print_serializable(&hash);
                }
            }
            GET_TX => {
                let hash = match splitted.get(1).and_then(|s| Hash::from_hex(s).ok()) {
                    Some(hash) => hash,
                    _ => {
                        println!("Bad input, please provide transaction ticket");
                        continue;
                    }
                };
                client.get_transaction_verb(hash);
            }
            GET_RX => {
                let hash = match splitted.get(1).and_then(|s| Hash::from_hex(s).ok()) {
                    Some(hash) => hash,
                    _ => {
                        println!("Bad input, please provide transaction ticket");
                        continue;
                    }
                };
                client.get_receipt_verb(hash);
            }
            GET_ACCOUNT => {
                let id = match splitted.get(1) {
                    Some(&id) => id,
                    _ => {
                        println!("Bad input, please provide account id");
                        continue;
                    }
                };
                client.get_account_verb(id.to_owned());
            }
            GET_BLOCK => {
                let height = match splitted.get(1).and_then(|s| {
                    if s.to_uppercase() == *"MAX" {
                        Some(u64::MAX)
                    } else {
                        s.parse::<u64>().ok()
                    }
                }) {
                    Some(height) => height,
                    _ => {
                        println!("Bad input, please provide block (numeric) height");
                        continue;
                    }
                };
                client.get_block_verb(height);
            }
            GET_TXS => {
                let (height, mut counter) = match client.get_block(u64::MAX) {
                    Some((blk, txs)) => (blk.data.height, txs.len()),
                    None => {
                        println!("There are no transactions");
                        continue;
                    }
                };
                for i in 0..height {
                    if let Some((_, txs)) = client.get_block(i) {
                        counter += txs.len();
                    };
                }
                println!("Executed Transactions: {}", counter);
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
                cheats::commons::run(&mut client, &mut rl);
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
