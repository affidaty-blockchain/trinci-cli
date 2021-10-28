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

    pub fn put_transaction(&mut self, tx: Transaction) -> Option<Hash> {
        match self.put_transaction_err(tx) {
            Ok(hash) => Some(hash),
            Err(err) => {
                eprintln!("Error: {}", err.to_string());
                None
            }
        }
    }

    pub fn get_transaction(&mut self, hash: Hash) -> Option<Transaction> {
        match self.get_transaction_err(hash) {
            Ok(tx) => Some(tx),
            Err(err) => {
                eprintln!("Error: {}", err.to_string());
                None
            }
        }
    }

    pub fn get_receipt(&mut self, hash: Hash) -> Option<Receipt> {
        match self.get_receipt_err(hash) {
            Ok(rx) => Some(rx),
            Err(err) => {
                eprintln!("Error: {}", err.to_string());
                None
            }
        }
    }

    pub fn get_block(&mut self, height: u64) -> Option<(Block,Vec<Hash>)> {
        match self.get_block_err(height) {
            Ok(tblock) => Some(tblock),
            Err(err) => {
                eprintln!("Error: {}", err.to_string());
                None
            }
        }
    }

    pub fn get_account(&mut self, id: String) -> Option<Account> {
        match self.get_account_err(id) {
            Ok(acc) => Some(acc),
            Err(err) => {
                eprintln!("Error: {}", err.to_string());
                None
            }
        }
    }

    pub fn subscribe(&mut self) {
        if let Err(err) = self.subscribe_err() {
            eprintln!("Error: {}", err.to_string());
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
        if self.verbose {
            utils::print_serializable(&tx);
        }

        let req = Message::PutTransactionRequest { confirm: true, tx };
        match self.send_recv(req)? {
            Message::PutTransactionResponse { hash } => {
                if self.verbose {
                    utils::print_serializable(&hash);
                }
                Ok(hash)
            }
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
            Message::GetTransactionResponse { tx } => {
                if self.verbose {
                    utils::print_serializable(&tx);
                }
                Ok(tx)
            }
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
            Message::GetReceiptResponse { rx } => {
                if self.verbose {
                    utils::print_serializable(&rx);
                }
                Ok(rx)
            }
            Message::Exception(err) => {
                let err: Box<dyn std::error::Error> = Box::new(err);
                Err(err)
            }
            res => Err(format!("Unexpected response {:?}", res).into()),
        }
    }

    pub fn get_block_err(&mut self, height: u64) -> Result<(Block,Vec<Hash>), Box<dyn std::error::Error>> {
        let req = Message::GetBlockRequest { height, txs: true };
        let result = match self.send_recv(req)? {
            Message::GetBlockResponse { block, txs } => {
                if self.verbose {
                    utils::print_serializable(&block);

                    if let Some(ref txs) = txs {
                        utils::print_serializable(&txs);
                    }
                }
                Ok((block,txs.unwrap_or_default()))
            }
            Message::Exception(err) => {
                let err: Box<dyn std::error::Error> = Box::new(err);
                Err(err)
            }
            res => Err(format!("Unexpected response {:?}", res).into()),
        };
        result
    }

    pub fn get_account_err(&mut self, id: String) -> Result<Account, Box<dyn std::error::Error>> {
        let (id, data) = match id.split_once(':') {
            Some((id, key)) => (id.to_owned(), vec![key.to_owned()]),
            None => (id, vec![]),
        };
        let req = Message::GetAccountRequest { id, data };
        match self.send_recv(req)? {
            Message::GetAccountResponse { acc, data } => {
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
                Ok(acc)
            }
            Message::Exception(err) => {
                let err: Box<dyn std::error::Error> = Box::new(err);
                Err(err)
            }
            res => Err(format!("Unexpected response {:?}", res).into()),
        }
    }

    pub fn subscribe_err(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let events = Event::BLOCK | Event::TRANSACTION;
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
