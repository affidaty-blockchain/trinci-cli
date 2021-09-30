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
use serde_value::{value, Value};
use trinci_core::{
    base::serialize::rmp_serialize,
    crypto::{Hash, KeyPair},
    Transaction,
};

const CONTRACT_REGISTER: &str = "register";
const ASSET_TRANSFER: &str = "transfer";
const ASSET_INIT: &str = "asset-init";
const SUBSCRIBE: &str = "subscribe";
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
    )
}

fn register_contract(client: &mut Client) {
    utils::print_unbuf("  Service account: ");
    let service_account = utils::get_input();

    utils::print_unbuf("  Service contract (optional multihash hex string): ");
    let service_contract = utils::read_hash();

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
    );
    client.put_transaction(tx);
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
            QUIT => break,
            HELP => help(),
            _ => {
                println!("Command not found");
                help();
            }
        }
    }
}
