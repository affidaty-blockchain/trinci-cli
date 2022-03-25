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

use crate::{cheats, client::Client, common::*};
use rand::Rng;
use std::thread;
use trinci_core::Transaction;

/// Create a random payment transaction between two accounts in the
/// `ACCOUNTS_INFO` list.
fn random_pay_tx_get(network: String, asset_account: String) -> Transaction {
    let mut rng = rand::thread_rng();
    let from = {
        let idx = rng.gen::<usize>() % ACCOUNTS_INFO.len();
        &ACCOUNTS_INFO[idx]
    };
    let to = {
        let idx = rng.gen::<usize>() % ACCOUNTS_INFO.len();
        &ACCOUNTS_INFO[idx]
    };
    let units = rng.gen::<u64>() % 20;

    cheats::transfer_asset_tx(&from.keypair, network, asset_account, to.id.clone(), units)
}

fn stress_worker(mut client: Client, asset_account: String) {
    loop {
        let tx = random_pay_tx_get(client.network.clone(), asset_account.clone());
        client.put_transaction(tx);
    }
}

pub fn run(mut client: Client, threads: u8) {
    println!("Performing stress test using {} threads", threads);

    // Register advanced Asset contract
    crate::utils::print_unbuf("Register asset contract [Y/n]? ");
    let input = crate::utils::get_input().to_lowercase();

    if input.is_empty() || input == "y" || input == "yes" {
        println!("Register Advanced Asset contract");
        crate::cheats::register_contract(&mut client);
    }
    // Init advanced asset contract on an #Account
    crate::utils::print_unbuf("Initialize asset contract on an account [Y/n]? ");
    let input = crate::utils::get_input().to_lowercase();

    let mut asset_account = None;

    if input.is_empty() || input == "y" || input == "yes" {
        println!("Init Advanced Asset contract");
        asset_account = crate::cheats_advanced_asset::adv_asset_init(&mut client);
    } else {
        let mut input = String::new();
        while input.is_empty() {
            crate::utils::print_unbuf("Specify the asset account: ");
            input = crate::utils::get_input();
            asset_account = Some(input.clone());
        }
    }
    let asset_account = if let Some(acc) = asset_account {
        acc
    } else {
        String::new()
    };

    // Mint 10000 units in each of the 3 accounts
    crate::utils::print_unbuf(&format!("Mint {} on the accounts [Y/n]? ", asset_account));
    let input = crate::utils::get_input().to_lowercase();
    if input.is_empty() || input == "y" || input == "yes" {
        for account_number in 0..=2 {
            println!(
                "Mint 10000 {} on account {}",
                &asset_account, &ACCOUNTS_INFO[account_number].id,
            );
            crate::cheats_advanced_asset::adv_asset_mint(
                &mut client,
                Some(asset_account.clone()),
                None,
                Some((ACCOUNTS_INFO[account_number].id).to_string()),
                Some(10_000),
            );
        }
    }
    println!("Waiting 30 secs...");
    std::thread::sleep(std::time::Duration::from_secs(30));

    let mut thread_handlers = vec![];
    for _ in 0..threads {
        let c = client.clone();
        let asset_acc = asset_account.clone();
        let h = thread::spawn(move || {
            stress_worker(c, asset_acc);
        });
        thread_handlers.push(h);
    }
    for h in thread_handlers {
        h.join().unwrap();
    }
}
