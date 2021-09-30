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

use crate::client::channel::ClientChannel;
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct BridgeChannel {
    ip: String,
    port: u16,
    stream: TcpStream,
}

impl Clone for BridgeChannel {
    fn clone(&self) -> Self {
        let stream = TcpStream::connect((self.ip.as_str(), self.port)).unwrap();
        BridgeChannel {
            ip: self.ip.clone(),
            port: self.port,
            stream,
        }
    }
}

impl BridgeChannel {
    pub fn new(ip: String, port: u16) -> Self {
        let stream = TcpStream::connect((ip.as_str(), port)).unwrap();
        BridgeChannel { ip, port, stream }
    }
}

impl ClientChannel for BridgeChannel {
    fn send(&mut self, buf: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let len = buf.len() as u32;
        let head: [u8; 4] = len.to_be_bytes();
        self.stream.write_all(&head)?;
        self.stream.write_all(&buf)?;
        Ok(())
    }

    fn recv(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut head: [u8; 4] = [0u8; 4];
        self.stream.read_exact(&mut head)?;
        let len = u32::from_be_bytes(head);
        let mut buf = vec![0u8; len as usize];
        self.stream.read_exact(&mut buf)?;
        Ok(buf)
    }
}
