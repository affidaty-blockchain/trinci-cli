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

pub trait ClientChannel: ClientChannelClone + Send {
    fn send(&mut self, buf: Vec<u8>) -> Result<(), Box<dyn std::error::Error>>;

    fn recv(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

pub trait ClientChannelClone {
    fn clone_box(&self) -> Box<dyn ClientChannel>;
}

impl<T> ClientChannelClone for T
where
    T: 'static + ClientChannel + Clone,
{
    fn clone_box(&self) -> Box<dyn ClientChannel> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn ClientChannel> {
    fn clone(&self) -> Box<dyn ClientChannel> {
        self.clone_box()
    }
}
