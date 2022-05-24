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

use clap::{Arg, Command};

const DEFAULT_NETWORK: &str = "skynet";
const DEFAULT_HOST: &str = "localhost";
const DEFAULT_PORT: u16 = 8000;
const DEFAULT_PATH: &str = "api/v1";
const DEFAULT_CHANNEL: &str = "http";

#[derive(Clone)]
pub struct Config {
    pub verbose: bool,
    pub network: String,
    pub host: String,
    pub channel: String,
    pub port: u16,
    pub path: String,
    pub keyfile: Option<String>,
    pub stress: Option<u8>,
    pub batch: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            verbose: false,
            network: DEFAULT_NETWORK.to_owned(),
            channel: DEFAULT_CHANNEL.to_owned(),
            host: DEFAULT_HOST.to_owned(),
            port: DEFAULT_PORT,
            path: DEFAULT_PATH.to_owned(),
            keyfile: None,
            stress: None,
            batch: None,
        }
    }
}

impl Config {
    pub fn from_args() -> Config {
        let matches = Command::new("T2 CLI")
            .version(clap::crate_version!())
            .author(clap::crate_authors!())
            .about(clap::crate_description!())
            .arg(
                Arg::new("verbose")
                    .long("verbose")
                    .short('v')
                    .help("Print sent and received structures")
                    .required(false),
            )
            .arg(
                Arg::new("batch")
                    .long("batch")
                    .help("Submit the ordered set of transactions specified in the file")
                    .value_name("PATH")
                    .required(false),
            )
            .arg(
                Arg::new("stress")
                    .long("stress")
                    .help(
                        "Run stress test using the specified number of threads (0 -> light mode) (stop with CTRL^C)",
                    )
                    .value_name("threads")
                    .required(false),
            )
            .arg(
                Arg::new("channel")
                    .short('c')
                    .long("channel")
                    .help("IO channel (default stdio)")
                    .value_name("CHANNEL")
                    .required(false)
                    .possible_values(&["stdio", "file", "http", "bridge"]),
            )
            .arg(
                Arg::new("network")
                    .short('n')
                    .long("network")
                    .help("Blockchain network identifier (default skynet)")
                    .value_name("NETWORK")
                    .required(false),
            )
            .arg(
                Arg::new("host")
                    .short('h')
                    .long("host")
                    .help("Target hostname (default localhost)")
                    .value_name("HOST")
                    .required(false),
            )
            .arg(
                Arg::new("port")
                    .short('p')
                    .long("port")
                    .help("Target port (default 8000)")
                    .value_name("PORT")
                    .required(false),
            )
            .arg(
                Arg::new("path")
                    .long("path")
                    .help("API path (default '/api/v1')")
                    .value_name("PATH")
                    .required(false),
            )
            .arg(
                Arg::new("keyfile")
                    .long("keyfile")
                    .help("Keypair file (PKCS#8 DER format)")
                    .value_name("PATH")
                    .required(false),
            )
            .get_matches();

        let mut config = Config {
            verbose: matches.is_present("verbose"),
            ..Default::default()
        };

        if let Some(value) = matches.value_of("batch") {
            config.batch = Some(value.to_owned());
        }
        if let Some(value) = matches.value_of("stress") {
            config.stress = Some(value.parse::<u8>().unwrap_or(1));
        }
        if let Some(value) = matches.value_of("network") {
            config.network = value.to_owned();
        }
        if let Some(value) = matches.value_of("channel") {
            config.channel = value.to_owned();
        }
        if let Some(value) = matches.value_of("host") {
            config.host = value.to_owned();
        }
        if let Some(value) = matches.value_of("port") {
            config.port = value.parse::<u16>().unwrap_or(DEFAULT_PORT);
        }
        if let Some(value) = matches.value_of("path") {
            config.path = value.to_owned();
        }
        if let Some(value) = matches.value_of("keyfile") {
            config.keyfile = Some(value.to_owned());
        }

        config
    }
}
