/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
use tokio::sync::{mpsc, oneshot};
use tokio::time::{interval, Duration, MissedTickBehavior};

#[derive(Debug)]
pub struct Input {
    recv: mpsc::Receiver<InputMessage>,
    keypad: [bool; 16],
}

#[derive(Debug)]
pub enum InputMessage {
    KeyDown {
        key: u8,
    },
    KeyUp {
        key: u8,
    },
    Status {
        key: u8,
        respond_to: oneshot::Sender<bool>,
    },
}

impl Input {
    fn new(recv: mpsc::Receiver<InputMessage>) -> Self {
        Input {
            recv,
            keypad: [false; 16],
        }
    }

    fn handle_message(&mut self, msg: InputMessage) {
        match msg {
            InputMessage::KeyDown { key } => {
                self.keypad[key as usize] = true;
            }
            InputMessage::KeyUp { key } => {
                self.keypad[key as usize] = false;
            }
            InputMessage::Status { key, respond_to } => {
                let status = self.keypad[key as usize];
                respond_to.send(status).unwrap();
            }
        }
    }
}

pub async fn run_input(mut input: Input) {
    // Count down at 60 Hz
    let mut ival = interval(Duration::from_secs_f64(crate::util::hz_to_secs("60Hz")));
    ival.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        ival.tick().await;
        tokio::select! {
            Some(msg) = input.recv.recv() => { input.handle_message(msg) },
            else => {
                // The input.recv should stay alive as long as the Chip8 is running
                // This branch is activated when the Chip8 stops executing.
                break
            },
        };
    }
}

#[derive(Clone, Debug)]
pub struct InputHandle {
    sender: mpsc::Sender<InputMessage>,
}

impl InputHandle {
    pub fn new() -> Self {
        let (sender, recv) = mpsc::channel(8);
        let actor = Input::new(recv);
        tokio::spawn(run_input(actor));

        Self { sender }
    }

    pub async fn key_down(&self, key: u8) {
        let msg = InputMessage::KeyDown { key };
        let _ = self.sender.send(msg).await;
    }

    pub async fn key_up(&self, key: u8) {
        let msg = InputMessage::KeyUp { key };
        let _ = self.sender.send(msg).await;
    }

    pub async fn pressed(&self, key: u8) -> bool {
        let (send, recv) = oneshot::channel();
        let msg = InputMessage::Status {
            key,
            respond_to: send,
        };
        let _ = self.sender.send(msg).await;
        recv.await.unwrap()
    }
}
