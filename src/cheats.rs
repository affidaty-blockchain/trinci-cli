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

use crate::{client::Client, common, utils};
use glob::glob;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use serde_value::{value, Value};
use trinci_core::{
    base::serialize::{rmp_deserialize, rmp_serialize},
    crypto::{Hash, Hashable, KeyPair},
    Message, Transaction,
};

/// WARNING: Edit this structure could break the node
/// This structure should be the same of the Boostrap
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

const CONTRACT_REGISTER: &str = "register";
const ASSET_TRANSFER: &str = "transfer";
const ASSET_INIT: &str = "asset-init";
const SUBSCRIBE: &str = "subscribe";
const CREATE_BOOTSTRAP: &str = "bootstrap";
const HELP: &str = "help";
const QUIT: &str = "quit";

fn help() {
    println!(" * '{}': print this help", HELP);
    println!(" * '{}': transfer an asset to an account", ASSET_TRANSFER);
    println!(" * '{}': initialize asset account", ASSET_INIT);
    println!(
        " * '{}': register a contract to the service account",
        CONTRACT_REGISTER
    );
    println!(
        " * '{}': subscribe to blockchain events (stop with ctrl+c)",
        SUBSCRIBE
    );
    println!(" * '{}': create a new bootstrap.bin", CREATE_BOOTSTRAP);
    println!(" * '{}': back to main menu", QUIT);
}

#[allow(clippy::too_many_arguments)]
pub fn register_contract_tx(
    caller: &KeyPair,
    network: String,
    service_account: String,
    service_contract: Option<Hash>,
    name: String,
    version: String,
    description: String,
    url: String,
    bin: Vec<u8>,
    fuel_limit: u64,
) -> Transaction {
    let args = value!({
        "name": name,
        "version": version,
        "description": description,
        "url": url,
        "bin": Value::Bytes(bin),
    });
    let args = rmp_serialize(&args).unwrap();
    common::build_transaction(
        caller,
        network,
        service_account,
        service_contract,
        "contract_registration".to_owned(),
        args,
        fuel_limit,
    )
}

pub fn create_service_init_tx(
    caller: &KeyPair,
    network: String,
    service_account: String,
    bin: Vec<u8>,
) -> Transaction {
    let hash = bin.primary_hash();

    common::build_transaction(
        caller,
        network,
        service_account,
        Some(hash),
        "init".to_owned(),
        bin,
        100000,
    )
}

fn register_contract(client: &mut Client) {
    utils::print_unbuf("  Service account: ");
    let service_account = utils::get_input();

    utils::print_unbuf("  Service contract (optional multihash hex string): ");
    let service_contract = utils::read_hash();

    utils::print_unbuf("  Fuel Limit: ");
    let fuel_limit = utils::get_input().parse::<u64>().unwrap();

    utils::print_unbuf("  New contract name: ");
    let name = utils::get_input();

    utils::print_unbuf("  New contract version: ");
    let version = utils::get_input();

    utils::print_unbuf("  New contract description: ");
    let description = utils::get_input();

    utils::print_unbuf("  New contract url: ");
    let url = utils::get_input();

    utils::print_unbuf("  New contract filename: ");
    let filename = utils::get_input();
    let bin = std::fs::read(filename).unwrap();

    let tx = register_contract_tx(
        &client.keypair,
        client.network.clone(),
        service_account,
        service_contract,
        name,
        version,
        description,
        url,
        bin,
        fuel_limit,
    );
    client.put_transaction(tx);
}

fn load_txs_from_directory(txs: &mut Vec<Transaction>, path: &str) {
    for entry in glob(&format!("{}/*.bin", path)).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let filename = path.clone();
                let filename = filename.file_name().unwrap().to_str().unwrap();

                if !filename.starts_with('_') {
                    let mut tx_file = std::fs::File::open(path).unwrap();

                    let mut tx_bin = Vec::new();
                    std::io::Read::read_to_end(&mut tx_file, &mut tx_bin)
                        .unwrap_or_else(|_| panic!("Error reading: {:?}", filename));

                    let msg: Message = rmp_deserialize(&tx_bin).unwrap();

                    let tx = match msg {
                        Message::PutTransactionRequest { confirm: _, tx } => tx,
                        _ => panic!("Expected put transaction request message"),
                    };

                    txs.push(tx);
                    println!("Added tx: {}", filename);
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
}

fn create_bootstrap(client: &mut Client) {
    let network_name;
    let service_account;
    let mut nonce = String::from("bootstrap");

    let mut txs = Vec::<Transaction>::new();

    utils::print_unbuf("  Service wasm contract path: ");
    let service_contract_filename = utils::get_input();

    let bin = std::fs::read(service_contract_filename).unwrap();

    utils::print_unbuf("  Create service init? ");
    let create_init = utils::get_bool();

    if create_init {
        utils::print_unbuf("  Network ID: ");
        network_name = utils::get_input();

        utils::print_unbuf("  Service account: ");
        service_account = utils::get_input();

        txs.push(create_service_init_tx(
            &client.keypair,
            network_name,
            service_account,
            bin.clone(),
        ));
    }

    utils::print_unbuf("  Load txs from directory? ");
    let load_txs = utils::get_bool();
    if load_txs {
        utils::print_unbuf("  txs path: ");
        let txs_path = utils::get_input();
        load_txs_from_directory(&mut txs, &txs_path);
    }
    utils::print_unbuf("  Create unique bootstrap? ");
    let unique = utils::get_bool();
    if unique {
        nonce = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        println!("nonce: {}", nonce);
    }

    let bootstrap = Bootstrap { bin, txs, nonce };

    let bootstrap_buf = trinci_core::base::serialize::rmp_serialize(&bootstrap).unwrap();

    std::fs::write("bootstrap.bin", bootstrap_buf).unwrap();
    println!("Done!\nbootstrap.bin saved.");
}

fn subscribe(client: &mut Client) {
    client.subscribe();
}

pub fn transfer_asset_tx(
    caller: &KeyPair,
    network: String,
    asset_account: String,
    to: String,
    units: u64,
) -> Transaction {
    let args = serde_value::value!({
        "from": caller.public_key().to_account_id(),
        "to": to,
        "units": units,
    });
    let args = rmp_serialize(&args).unwrap();

    common::build_transaction(
        caller,
        network,
        asset_account,
        None,
        "transfer".to_owned(),
        args,
        1_000_000,
    )
}

fn transfer_asset(client: &mut Client) {
    utils::print_unbuf("  Asset account: ");
    let asset_account = utils::get_input();

    utils::print_unbuf("  Destination account: ");
    let destination_account = utils::get_input();

    utils::print_unbuf("  Asset units: ");
    let units = utils::get_input().parse::<u64>().unwrap_or_default();

    let buf = transfer_asset_tx(
        &client.keypair,
        client.network.clone(),
        asset_account,
        destination_account,
        units,
    );
    client.put_transaction(buf);
}

#[allow(clippy::too_many_arguments)]
fn asset_init_tx(
    caller: &KeyPair,
    network: String,
    asset_account: String,
    asset_contract: Option<Hash>,
    name: String,
    desc: String,
    url: String,
    max_units: u64,
) -> Transaction {
    let args = value!({
        "name": name,
        "authorized": Vec::<&str>::new(),
        "description": desc,
        "url": url,
        "max_units": max_units
    });
    let args = rmp_serialize(&args).unwrap();

    common::build_transaction(
        caller,
        network,
        asset_account,
        asset_contract,
        "init".to_owned(),
        args,
        1_000_000,
    )
}

fn asset_init(client: &mut Client) {
    utils::print_unbuf("  Asset account: ");
    let asset_account = utils::get_input();

    utils::print_unbuf("  Asset contract (multihash hex string): ");
    let asset_contract = utils::read_hash();

    utils::print_unbuf("  Asset name: ");
    let name = utils::get_input();

    utils::print_unbuf("  Asset description: ");
    let desc = utils::get_input();

    utils::print_unbuf("  Asset url: ");
    let url = utils::get_input();

    utils::print_unbuf("  Asset max mintable units: ");
    let max_units = utils::get_input().parse::<u64>().unwrap_or_default();

    let buf = asset_init_tx(
        &client.keypair,
        client.network.clone(),
        asset_account,
        asset_contract,
        name,
        desc,
        url,
        max_units,
    );
    client.put_transaction(buf);
}

pub fn run(client: &mut Client, rl: &mut rustyline::Editor<()>) {
    help();
    loop {
        let line = match rl.readline(">>> ") {
            Ok(line) => {
                rl.add_history_entry(&line);
                line
            }
            Err(rustyline::error::ReadlineError::Interrupted)
            | Err(rustyline::error::ReadlineError::Eof) => break,
            Err(_) => continue,
        };

        match line.as_str() {
            ASSET_INIT => asset_init(client),
            ASSET_TRANSFER => transfer_asset(client),
            CONTRACT_REGISTER => register_contract(client),
            SUBSCRIBE => subscribe(client),
            CREATE_BOOTSTRAP => create_bootstrap(client),
            QUIT => break,
            HELP => help(),
            _ => {
                println!("Command not found");
                help();
            }
        }
    }
}
