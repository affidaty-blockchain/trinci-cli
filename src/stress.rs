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
fn random_pay_tx_get(network: String) -> Transaction {
    let asset_account = &ACCOUNTS_INFO[0];

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

    cheats::transfer_asset_tx(
        &from.keypair,
        network,
        asset_account.id.clone(),
        to.id.clone(),
        units,
    )
}

fn stress_worker(mut client: Client) {
    loop {
        let tx = random_pay_tx_get(client.network.clone());
        client.put_transaction(tx);
    }
}

pub fn run(client: Client, threads: u8) {
    println!("Performing stress test using {} threads", threads);
    std::thread::sleep(std::time::Duration::from_secs(3));

    let mut thread_handlers = vec![];
    for _ in 0..threads {
        let c = client.clone();
        let h = thread::spawn(move || {
            stress_worker(c);
        });
        thread_handlers.push(h);
    }
    for h in thread_handlers {
        h.join().unwrap();
    }
}
