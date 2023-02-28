/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
use log::debug;
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
    #[allow(dead_code)]
    fn handle_message(&self, _msg: FuseMessage) {}
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
        debug!("FUSE BLOWN!");
    }

    pub fn alive(&self) -> bool {
        self.send.send(FuseMessage::Alive).is_ok()
    }
}
