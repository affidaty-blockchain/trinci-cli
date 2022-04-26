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
    client::Client,
    common::{self},
    utils,
};
use serde_value::value;
use trinci_core::{
    base::serialize::rmp_serialize,
    crypto::{Hash, KeyPair},
    Transaction,
};

const ADV_ASSET_TRANSFER: &str = "transfer";
const ADV_ASSET_INIT: &str = "init";
const ADV_ASSET_MINT: &str = "mint";
const ADV_ASSET_BALANCE: &str = "balance";
const HELP: &str = "help";
const QUIT: &str = "quit";

fn help() {
    println!(" @ '{}': print this help", HELP);
    println!(
        " @ '{}': initialize the advanced asset in an account",
        ADV_ASSET_INIT
    );
    println!(" @ '{}': initialize asset account", ADV_ASSET_MINT);
    println!(
        " @ '{}': transfer an asset to an account",
        ADV_ASSET_BALANCE
    );
    println!(
        " @ '{}': transfer an asset to an account",
        ADV_ASSET_TRANSFER
    );
    println!(" @ '{}': back to cheats menu", QUIT);
}

fn adv_asset_transfer_tx(
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

#[allow(clippy::too_many_arguments)]
fn adv_asset_init_tx(
    caller: &KeyPair,
    network: String,
    asset_account: String,
    asset_contract: Option<Hash>,
    name: String,
    desc: String,
    url: String,
    max_units: u64,
    (fee_part, to_part): (u64, u64),
    fixed_fees: u64,
    fees_account: &str,
) -> Transaction {
    let args = value!({
        "name": name,
        "description": desc,
        "url": url,
        "max_units": max_units,
        "fees": [fee_part, to_part],
        "fixed_fees": fixed_fees,
        "fees_account": fees_account
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

pub fn adv_asset_mint_tx(
    caller: &KeyPair,
    network: String,
    asset_account: String,
    asset_contract: Option<Hash>,
    destination_account: &str,
    units: u64,
) -> Transaction {
    let args = value!({
        "to": destination_account,
        "units": units,
    });
    let args = rmp_serialize(&args).unwrap();

    common::build_transaction(
        caller,
        network,
        asset_account,
        asset_contract,
        "mint".to_owned(),
        args,
        1_000_000,
    )
}

fn adv_asset_balance_tx(
    caller: &KeyPair,
    network: String,
    asset_account: String,
    asset_contract: Option<Hash>,
    balance_account: Option<String>,
) -> Transaction {
    let args = match balance_account {
        Some(account) => rmp_serialize(&value!({ "account": account })).unwrap(),
        None => vec![],
    };

    common::build_transaction(
        caller,
        network,
        asset_account,
        asset_contract,
        "balance".to_owned(),
        args,
        1_000_000,
    )
}

pub fn adv_asset_init(client: &mut Client) -> Option<String> {
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

    utils::print_unbuf("  Asset max mintable units [ <integer> | max ]: ");
    let max_units = utils::get_input();
    let max_units = if max_units.to_lowercase() == "max" {
        u64::MAX
    } else {
        max_units.parse::<u64>().unwrap_or_default()
    };

    let tx = adv_asset_init_tx(
        &client.keypair,
        client.network.clone(),
        asset_account.clone(),
        asset_contract,
        name,
        desc,
        url,
        max_units,
        (0, 100),
        0,
        "",
    );
    if let Some(hash) = client.put_transaction(tx) {
        utils::print_serializable(&hash);
        return Some(asset_account);
    }
    None
}

pub fn adv_asset_mint(
    client: &mut Client,
    asset_account: Option<String>,
    asset_contract: Option<Hash>,
    dest_account: Option<String>,
    units: Option<u64>,
) {
    let asset_account = if let Some(asset_account) = asset_account {
        asset_account
    } else {
        utils::print_unbuf("  Asset account: ");
        utils::get_input()
    };

    let asset_contract = if let Some(asset_contract) = asset_contract {
        if asset_contract == Hash::default() {
            utils::print_unbuf("  Asset contract (multihash hex string): ");
            utils::read_hash()
        } else {
            Some(asset_contract)
        }
    } else {
        None
    };

    let dest_account = if let Some(dest_account) = dest_account {
        dest_account
    } else {
        utils::print_unbuf("  Destination account: ");
        utils::get_input()
    };

    let units = if let Some(units) = units {
        units
    } else {
        utils::print_unbuf("  Asset units: ");
        utils::get_input().parse::<u64>().unwrap_or_default()
    };

    let tx = adv_asset_mint_tx(
        &client.keypair,
        client.network.clone(),
        asset_account,
        asset_contract,
        &dest_account,
        units,
    );
    if let Some(hash) = client.put_transaction(tx) {
        utils::print_serializable(&hash);
    }
}

fn adv_asset_balance(client: &mut Client) {
    utils::print_unbuf("  Asset account: ");
    let asset_account = utils::get_input();

    utils::print_unbuf("  Asset contract (multihash hex string): ");
    let asset_contract = utils::read_hash();

    utils::print_unbuf("  Account to check [if empty is the caller account]: ");
    let balance_account = utils::get_input();
    let balance_account = if balance_account.is_empty() {
        None
    } else {
        Some(balance_account)
    };

    let tx = adv_asset_balance_tx(
        &client.keypair,
        client.network.clone(),
        asset_account,
        asset_contract,
        balance_account,
    );
    if let Some(hash) = client.put_transaction(tx) {
        utils::print_serializable(&hash);
    }
}

fn adv_transfer_asset(client: &mut Client) {
    utils::print_unbuf("  Asset account: ");
    let asset_account = utils::get_input();

    utils::print_unbuf("  Destination account: ");
    let destination_account = utils::get_input();

    utils::print_unbuf("  Asset units: ");
    let units = utils::get_input().parse::<u64>().unwrap_or_default();

    let tx = adv_asset_transfer_tx(
        &client.keypair,
        client.network.clone(),
        asset_account,
        destination_account,
        units,
    );
    if let Some(hash) = client.put_transaction(tx) {
        utils::print_serializable(&hash);
    }
}

pub fn run(client: &mut Client, rl: &mut rustyline::Editor<()>) {
    help();
    loop {
        let line = match rl.readline(">>>> ") {
            Ok(line) => {
                rl.add_history_entry(&line);
                line
            }
            Err(rustyline::error::ReadlineError::Interrupted)
            | Err(rustyline::error::ReadlineError::Eof) => break,
            Err(_) => continue,
        };

        match line.as_str() {
            ADV_ASSET_INIT => {
                let _ = adv_asset_init(client);
            }
            ADV_ASSET_MINT => adv_asset_mint(client, None, Some(Hash::default()), None, None),
            ADV_ASSET_BALANCE => adv_asset_balance(client),
            ADV_ASSET_TRANSFER => adv_transfer_asset(client),
            QUIT => break,
            HELP => help(),
            _ => {
                println!("Command not found");
                help();
            }
        };
    }
}
