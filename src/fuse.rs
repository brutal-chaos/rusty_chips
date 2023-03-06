/// fuse.rs: an actor that tracks the powered state of the chip8
/// Copyright (C) 2023 Justin Noah <justinnoah+rusty_chips@gmail.com>

/// This program is free software: you can redistribute it and/or modify
/// it under the terms of the GNU Affero General Public License as published
/// by the Free Software Foundation, either version 3 of the License, or
/// (at your option) any later version.

/// This program is distributed in the hope that it will be useful,
/// but WITHOUT ANY WARRANTY; without even the implied warranty of
/// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
/// GNU Affero General Public License for more details.

/// You should have received a copy of the GNU Affero General Public License
/// along with this program.  If not, see <https://www.gnu.org/licenses/>.
use log::trace;
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
enum FuseMessage {
    Alive,
    Blow,
}

#[derive(Debug)]
struct Fuse {
    recv: broadcast::Receiver<FuseMessage>,
}

impl Fuse {
    fn _handle_message(&self, _msg: FuseMessage) {}
}

async fn run_fuse(mut fuse: Fuse) {
    loop {
        let _msg = fuse.recv.recv().await;
        match _msg {
            Ok(FuseMessage::Blow) => break,
            _ => continue,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FuseHandle {
    send: broadcast::Sender<FuseMessage>,
}

impl FuseHandle {
    pub fn new() -> Self {
        let (send, recv) = broadcast::channel(1);
        let fuse = Fuse { recv };
        tokio::spawn(run_fuse(fuse));

        Self { send }
    }

    pub fn blow(&self) {
        let _ = self.send.send(FuseMessage::Blow);
        trace!("FUSE BLOWN!");
    }

    pub fn alive(&self) -> bool {
        self.send.send(FuseMessage::Alive).is_ok()
    }
}
