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
use std::time::{SystemTime, UNIX_EPOCH};
use trinci_core::{base::serialize::rmp_serialize, Error, ErrorKind, Message};

#[derive(Clone)]
pub struct FileChannel;

impl FileChannel {
    pub fn new() -> Self {
        FileChannel
    }
}

impl ClientChannel for FileChannel {
    fn send(&mut self, buf: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let filename = format!("out-{}.bin", time.as_secs());
        std::fs::write(filename, buf)?;
        Ok(())
    }

    fn recv(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let res = Message::Exception(Error::new_ext(ErrorKind::Other, "write only channel"));
        let buf = rmp_serialize(&res)?;
        Ok(buf)
    }
}
