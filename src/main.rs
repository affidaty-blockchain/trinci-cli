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

use std::process::exit;

mod batch;
mod cheats;
mod client;
mod common;
mod config;
mod console;
mod impexp;
mod stress;
mod utils;

use client::Client;
use config::Config;

const STDIO_CHANNEL: &str = "stdio";
const FILE_CHANNEL: &str = "file";
const HTTP_CHANNEL: &str = "http";
const BRIDGE_CHANNEL: &str = "bridge";

fn main() {
    let config = Config::from_args();

    let keypair = utils::load_keypair(config.keyfile);
    let client = match config.channel.as_str() {
        STDIO_CHANNEL => Client::new_stdio(config.verbose, config.network.clone(), keypair),
        FILE_CHANNEL => Client::new_file(config.verbose, config.network.clone(), keypair),
        HTTP_CHANNEL => {
            let url = format!("http://{}:{}/{}", config.host, config.port, config.path);
            Client::new_http(config.verbose, config.network.clone(), keypair, url)
        }
        BRIDGE_CHANNEL => Client::new_bridge(
            config.verbose,
            config.network.clone(),
            keypair,
            config.host,
            config.port,
        ),
        _ => {
            eprintln!("Invalid channel {}", config.channel);
            exit(1);
        }
    };

    if let Some(filename) = config.batch {
        batch::run(client, filename);
    } else if let Some(threads) = config.stress {
        stress::run(client, threads);
    } else {
        console::run(client);
    }
}
