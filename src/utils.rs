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

use serde::Serialize;
use std::io::{stdin, stdout, Read, Write};
use trinci_core::crypto::{ecdsa, Hash, KeyPair};

pub fn print_unbuf(string: &str) {
    print!("{}", string);
    stdout().flush().unwrap();
}

pub fn get_input() -> String {
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Cannot read stdin");
    input.trim().to_string()
}

pub fn get_bool() -> bool {
    let mut input = String::new();
    while input != "Y" && input != "N" {
        println!("  >> Please answer with Y or N: ");
        input = String::new();
        stdin().read_line(&mut input).expect("Cannot read stdin");
        input = input.trim().to_ascii_uppercase();
    }
    input == "Y"
}

pub fn read_hash() -> Option<Hash> {
    loop {
        let input = get_input();
        if input.is_empty() {
            return None;
        } else {
            match Hash::from_hex(&input) {
                Ok(hash) => return Some(hash),
                Err(_) => eprintln!("Bad hash format, try again"),
            }
        }
    }
}

fn byte_buffer_try_parse(s: &str) -> Option<String> {
    let mut outstr = String::new();
    let mut numstr = String::new();
    outstr.push_str("[ ");
    for c in s.chars() {
        match c {
            '\r' | '\n' | ',' | ' ' => {
                if !numstr.is_empty() {
                    match numstr.parse::<u8>() {
                        Ok(num) => outstr.push_str(&format!("{:02x}", num)),
                        Err(_) => return None,
                    }
                    numstr.clear();
                }
            }
            _ if c.is_digit(10) => numstr.push(c),
            _ => return None,
        }
    }
    outstr.push_str(" ]]");
    Some(outstr)
}

/// Wrapper to print a json string in a more friendly format.
/// In particular vectors of u8 are printed as hex strings in one line.
fn print_json(s: String) {
    let mut outstr = String::new();
    let mut working_str = s.as_str();

    #[allow(clippy::while_let_loop)]
    loop {
        let start_idx = match working_str.find('[') {
            Some(i) => i + 1,
            _ => break,
        };
        let mut end_idx = match working_str.find(']') {
            Some(i) if i > start_idx => i,
            Some(i) => {
                outstr.push_str(&working_str[..=i]);
                working_str = &working_str[i + 1..];
                continue;
            }
            _ => break,
        };
        let wb = &working_str[start_idx..end_idx];
        if wb.find('[').is_some() {
            outstr.push_str(&working_str[..=start_idx]);
            end_idx = start_idx + 1;
        } else {
            match byte_buffer_try_parse(wb) {
                Some(bytes_str) => {
                    outstr.push_str(&working_str[..start_idx]);
                    outstr.push_str(&bytes_str);
                }
                None => {
                    outstr.push_str(&working_str[..=end_idx]);
                }
            }
            end_idx += 1;
        }
        working_str = &working_str[end_idx..];
    }
    outstr.push_str(working_str);

    println!("{}", outstr);
}

pub fn print_serializable<T: Serialize>(val: &T) {
    let json_str = serde_json::to_string_pretty(val).unwrap();
    print_json(json_str);
}

/// Load node account keypair from PKCS#8 DER file.
pub fn load_ecdsa_secp384_keypair(keyfile: &str) -> KeyPair {
    let mut file = std::fs::File::open(keyfile).unwrap();
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("loading node keypair");
    let ecdsa_keypair =
        ecdsa::KeyPair::from_pkcs8_bytes(ecdsa::CurveId::Secp384R1, &bytes).unwrap();
    KeyPair::Ecdsa(ecdsa_keypair)
}
