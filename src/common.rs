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

use lazy_static::lazy_static;
use std::sync::Mutex;
use trinci_core::{
    self,
    base::{
        schema::{SignedTransaction, TransactionData},
        serialize::MessagePack,
    },
    crypto::Hash,
    crypto::{ecdsa, KeyPair},
    Transaction, TransactionDataV1,
};

lazy_static! {
    pub static ref CONTRACT_ID: Mutex<Option<Hash>> = Mutex::new(None);
    pub static ref METHOD_ID: Mutex<String> = Mutex::new("transfer".to_string());
}

pub const INITIAL_NETWORK_NAME: &str = "bootstrap";
pub struct AccountInfo {
    pub id: String,
    pub keypair: KeyPair,
}

fn create_keypair(pub_key: &str, pvt_key: &str) -> KeyPair {
    let public_bytes = hex::decode(pub_key).unwrap();
    let private_bytes = hex::decode(pvt_key).unwrap();
    let ecdsa_keypair =
        ecdsa::KeyPair::new(ecdsa::CurveId::Secp384R1, &private_bytes, &public_bytes).unwrap();
    KeyPair::Ecdsa(ecdsa_keypair)
}

impl AccountInfo {
    pub fn new(pub_key: &str, pvt_key: &str) -> Self {
        let keypair = create_keypair(pub_key, pvt_key);
        let id = keypair.public_key().to_account_id();
        AccountInfo { id, keypair }
    }
}

const PUB_KEY1: &str = "04afb0e912e48257da3ce9847d518dc84174403ca4c903ada40f313e2bfabee754d2976d13bb2ad63ea87df60fec758f04c9c1b147273615bbeccac7dd489beb203910171274e1b15d274d6cc0f4629166036992f2a21844c7c4c0fddd1b0e9870";
const PVT_KEY1: &str = "50148455edac9c3a408dc350fa0c263c4e9b5c164e206e3d1eff97db22e875c65543d3b7cecb053ef686b00cb3711330";

const PUB_KEY2: &str = "04755974cec8051cd19adb9f6a5daea99c768418a84f6a8e1a3c17e20b863b5e3372af75fdb1164288bcc6a85f54a781f0ad533dd722cf28437dfe763cf4d5e9ff2a862518609a0b41ba46dd6f3b9f03e4815047b5ffe2a03d1f4e6f42b2dbcca1";
const PVT_KEY2: &str = "4007db25c582d39d9912ef6095d9064bcb6b84211cf570b5dd95a10545dde27707ff4042708eae0f357b5c8bcbbfbddb";

const PUB_KEY3: &str = "0415cf93b220a8baca938323e0977db3b5a3ccfc1e02d41d92a00394776cfb03409946b22b29b1103bfe82ff9bd946f16d422045bf8ee6a3fc03e80deb10b8b163b13c521aebd943c799b67f26974932f8c3c9f836e069d354642ed9216beff000";
const PVT_KEY3: &str = "e5e86a167ddad2d28baa5b1792b3bb83ff366f57dada85d7f268f750a70bbb20d0c463ee7c71e669250efb44375735d1";

lazy_static! {
    pub static ref ACCOUNTS_INFO: Vec<AccountInfo> = vec![
        AccountInfo::new(PUB_KEY1, PVT_KEY1),
        AccountInfo::new(PUB_KEY2, PVT_KEY2),
        AccountInfo::new(PUB_KEY3, PVT_KEY3),
    ];
}

pub fn default_keypair() -> KeyPair {
    create_keypair(PUB_KEY1, PVT_KEY1)
}

pub fn build_transaction(
    caller: &KeyPair,
    network: String,
    account: String,
    contract: Option<Hash>,
    method: String,
    args: Vec<u8>,
    fuel_limit: u64,
) -> Transaction {
    let nonce = rand::random::<u64>().to_be_bytes().to_vec();
    let data = TransactionDataV1 {
        network,
        account,
        fuel_limit,
        nonce,
        contract,
        method,
        caller: caller.public_key(),
        args,
    };
    let data = TransactionData::V1(data);
    let bytes = data.serialize();
    let signature = caller.sign(&bytes).unwrap();
    Transaction::UnitTransaction(SignedTransaction { data, signature })
}
