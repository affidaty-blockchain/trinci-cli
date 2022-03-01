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

pub mod bridge;
pub mod channel;
pub mod file;
pub mod http;
pub mod stdio;

use crate::utils;
use bridge::BridgeChannel;
use channel::ClientChannel;
use file::FileChannel;
use http::HttpChannel;
use serde_value::Value;
use std::sync::Arc;
use stdio::StdioChannel;
use trinci_core::{
    base::serialize::{rmp_deserialize, rmp_serialize},
    blockchain::Event,
    crypto::{Hash, KeyPair},
    Account, Block, Message, Receipt, Transaction,
};

use crate::common::TX_PRINT_ARGS_MAX_SIZE;

#[derive(Clone)]
pub struct Client {
    pub verbose: bool,
    pub keypair: Arc<KeyPair>,
    pub network: String,
    pub chan: Box<dyn ClientChannel>,
}

impl Client {
    pub fn new(
        chan: impl ClientChannel + 'static,
        verbose: bool,
        network: String,
        keypair: KeyPair,
    ) -> Self {
        Client {
            verbose,
            keypair: Arc::new(keypair),
            network,
            chan: Box::new(chan),
        }
    }

    pub fn new_stdio(verbose: bool, network: String, keypair: KeyPair) -> Self {
        let chan = StdioChannel::new();
        Self::new(chan, verbose, network, keypair)
    }

    pub fn new_file(verbose: bool, network: String, keypair: KeyPair) -> Self {
        let chan = FileChannel::new();
        Self::new(chan, verbose, network, keypair)
    }

    pub fn new_http(verbose: bool, network: String, keypair: KeyPair, url: String) -> Self {
        let chan = HttpChannel::new(url);
        Self::new(chan, verbose, network, keypair)
    }

    pub fn new_bridge(
        verbose: bool,
        network: String,
        keypair: KeyPair,
        host: String,
        port: u16,
    ) -> Self {
        let chan = BridgeChannel::new(host, port);
        Self::new(chan, verbose, network, keypair)
    }

    pub fn _put_transaction_verb(&mut self, tx: Transaction) -> Option<Hash> {
        let prev_verbose = self.verbose;
        self.verbose = true;
        let res = self.put_transaction(tx);
        self.verbose = prev_verbose;
        res
    }

    pub fn put_transaction(&mut self, tx: Transaction) -> Option<Hash> {
        if self.verbose {
            let mut printable_tx = tx.clone();
            let mut previous_args_len = 0;
            if printable_tx.get_args().len() > TX_PRINT_ARGS_MAX_SIZE {
                previous_args_len = printable_tx.get_args().len();

                if let Transaction::UnitTransaction(unit_tx) = &mut printable_tx {
                    match &mut unit_tx.data {
                        trinci_core::base::schema::TransactionData::V1(tx_data) => {
                            tx_data.args = tx_data.args[0..TX_PRINT_ARGS_MAX_SIZE].to_vec()
                        }
                        _ => panic!("This should not happen!"),
                    }
                };
            }
            utils::print_serializable(&printable_tx);
            if previous_args_len > 0 {
                println!(
                    "Note: the `args` field is clipped to the first {} bytes of the total {} bytes",
                    TX_PRINT_ARGS_MAX_SIZE, previous_args_len
                );
            }
        }
        match self.put_transaction_err(tx) {
            Ok(hash) => {
                if self.verbose {
                    utils::print_serializable(&hash);
                }
                Some(hash)
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                None
            }
        }
    }

    pub fn get_transaction_verb(&mut self, hash: Hash) -> Option<Transaction> {
        let prev_verbose = self.verbose;
        self.verbose = true;
        let res = self.get_transaction(hash);
        self.verbose = prev_verbose;
        res
    }
    pub fn get_transaction(&mut self, hash: Hash) -> Option<Transaction> {
        match self.get_transaction_err(hash) {
            Ok(mut tx) => {
                if self.verbose {
                    let mut previous_args_len = 0;
                    if tx.get_args().len() > TX_PRINT_ARGS_MAX_SIZE {
                        previous_args_len = tx.get_args().len();

                        if let Transaction::UnitTransaction(tx) = &mut tx {
                            match &mut tx.data {
                                trinci_core::base::schema::TransactionData::V1(tx) => {
                                    tx.args = tx.args[0..TX_PRINT_ARGS_MAX_SIZE].to_vec()
                                }
                                _ => panic!("This should not happen!"),
                            }
                        };
                    }
                    utils::print_serializable(&tx);
                    if previous_args_len > 0 {
                        println!("Note: the `args` field is clipped to the first {} bytes of the total {} bytes",TX_PRINT_ARGS_MAX_SIZE, previous_args_len);
                    }
                }
                Some(tx)
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                None
            }
        }
    }

    pub fn get_receipt_verb(&mut self, hash: Hash) -> Option<Receipt> {
        let prev_verbose = self.verbose;
        self.verbose = true;
        let res = self.get_receipt(hash);
        self.verbose = prev_verbose;
        res
    }

    pub fn get_receipt(&mut self, hash: Hash) -> Option<Receipt> {
        match self.get_receipt_err(hash) {
            Ok(rx) => {
                if self.verbose {
                    utils::print_serializable(&rx);
                }
                Some(rx)
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                None
            }
        }
    }

    pub fn get_block_verb(&mut self, height: u64) -> Option<(Block, Vec<Hash>)> {
        let prev_verbose = self.verbose;
        self.verbose = true;
        let res = self.get_block(height);
        self.verbose = prev_verbose;
        res
    }

    pub fn get_block(&mut self, height: u64) -> Option<(Block, Vec<Hash>)> {
        match self.get_block_err(height) {
            Ok((block, txs)) => {
                if self.verbose {
                    utils::print_serializable(&block);
                    if !txs.is_empty() {
                        utils::print_serializable(&txs);
                    }
                }
                Some((block, txs))
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                None
            }
        }
    }

    pub fn get_account_verb(&mut self, id: String) -> Option<Account> {
        let prev_verbose = self.verbose;
        self.verbose = true;
        let res = self.get_account(id);
        self.verbose = prev_verbose;
        res
    }

    pub fn get_account(&mut self, id: String) -> Option<Account> {
        match self.get_account_err(id) {
            Ok((acc, data)) => {
                if self.verbose {
                    if data.is_empty() {
                        utils::print_serializable(&acc);
                    } else {
                        match data[0].as_ref() {
                            Some(buf) => {
                                let data: Value = rmp_deserialize(buf).unwrap();
                                utils::print_serializable(&data);
                            }
                            None => {
                                println!("NULL");
                            }
                        }
                    }
                }
                Some(acc)
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                None
            }
        }
    }

    pub fn subscribe(&mut self) {
        if let Err(err) = self.subscribe_err() {
            eprintln!("Error: {}", err);
        }
    }

    fn send(&mut self, msg: Message) -> Result<(), Box<dyn std::error::Error>> {
        let buf = rmp_serialize(&msg)?;
        self.chan.send(buf)?;
        Ok(())
    }

    fn recv(&mut self) -> Result<Message, Box<dyn std::error::Error>> {
        let buf = self.chan.recv()?;
        let msg = rmp_deserialize(&buf)?;
        Ok(msg)
    }

    fn send_recv(&mut self, msg: Message) -> Result<Message, Box<dyn std::error::Error>> {
        self.send(msg)?;
        self.recv()
    }

    pub fn put_transaction_err(
        &mut self,
        tx: Transaction,
    ) -> Result<Hash, Box<dyn std::error::Error>> {
        let req = Message::PutTransactionRequest { confirm: true, tx };
        match self.send_recv(req)? {
            Message::PutTransactionResponse { hash } => Ok(hash),
            Message::Exception(err) => {
                let err: Box<dyn std::error::Error> = Box::new(err);
                Err(err)
            }
            res => Err(format!("Unexpected response {:?}", res).into()),
        }
    }

    pub fn get_transaction_err(
        &mut self,
        hash: Hash,
    ) -> Result<Transaction, Box<dyn std::error::Error>> {
        let req = Message::GetTransactionRequest { hash };
        match self.send_recv(req)? {
            Message::GetTransactionResponse { tx } => Ok(tx),
            Message::Exception(err) => {
                let err: Box<dyn std::error::Error> = Box::new(err);
                Err(err)
            }
            res => Err(format!("Unexpected response {:?}", res).into()),
        }
    }

    pub fn get_receipt_err(&mut self, hash: Hash) -> Result<Receipt, Box<dyn std::error::Error>> {
        let req = Message::GetReceiptRequest { hash };
        match self.send_recv(req)? {
            Message::GetReceiptResponse { rx } => Ok(rx),
            Message::Exception(err) => {
                let err: Box<dyn std::error::Error> = Box::new(err);
                Err(err)
            }
            res => Err(format!("Unexpected response {:?}", res).into()),
        }
    }

    pub fn get_block_err(
        &mut self,
        height: u64,
    ) -> Result<(Block, Vec<Hash>), Box<dyn std::error::Error>> {
        let req = Message::GetBlockRequest { height, txs: true };
        let result = match self.send_recv(req)? {
            Message::GetBlockResponse { block, txs } => Ok((block, txs.unwrap_or_default())),
            Message::Exception(err) => {
                let err: Box<dyn std::error::Error> = Box::new(err);
                Err(err)
            }
            res => Err(format!("Unexpected response {:?}", res).into()),
        };
        result
    }
    #[allow(clippy::type_complexity)]
    pub fn get_account_err(
        &mut self,
        id: String,
    ) -> Result<(Account, Vec<Option<Vec<u8>>>), Box<dyn std::error::Error>> {
        let (id, data) = match id.split_once(':') {
            Some((id, key)) => (id.to_owned(), vec![key.to_owned()]),
            None => (id, vec![]),
        };
        let req = Message::GetAccountRequest { id, data };
        match self.send_recv(req)? {
            Message::GetAccountResponse { acc, data } => Ok((acc, data)),
            Message::Exception(err) => {
                let err: Box<dyn std::error::Error> = Box::new(err);
                Err(err)
            }
            res => Err(format!("Unexpected response {:?}", res).into()),
        }
    }

    pub fn subscribe_err(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let events = Event::CONTRACT_EVENTS | Event::BLOCK | Event::TRANSACTION;

        let req = Message::Subscribe {
            id: "cli".to_owned(),
            events,
        };
        self.send(req)?;

        while let Ok(msg) = self.recv() {
            utils::print_serializable(&msg);
        }
        println!("Subscription over");

        Ok(())
    }
}
