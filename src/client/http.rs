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
use std::io::Read;
use ureq::Error;

#[derive(Clone)]
pub struct HttpChannel {
    url: String,
    res: Option<Result<Vec<u8>, String>>,
}

impl HttpChannel {
    pub fn new(url: String) -> Self {
        HttpChannel { url, res: None }
    }
}

impl ClientChannel for HttpChannel {
    fn send(&mut self, buf: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let url = self.url.to_string() + "/message";
        let mut body = vec![];

        match ureq::post(&url).send_bytes(&buf) {
            Ok(res) => {
                res.into_reader().read_to_end(&mut body)?;
                self.res = Some(Ok(body));
                Ok(())
            }
            Err(Error::Status(_code, res)) => {
                res.into_reader().read_to_end(&mut body)?;
                let msg = String::from_utf8_lossy(&body);
                self.res = Some(Err(msg.to_string()));
                Ok(())
            }
            Err(err) => Err(format!("POST error {}", err.to_string()).into()),
        }
    }

    fn recv(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match self.res.take() {
            Some(Ok(buf)) => Ok(buf),
            Some(Err(s)) => Err(s.into()),
            None => Err("buffer empty".into()),
        }
    }
}
