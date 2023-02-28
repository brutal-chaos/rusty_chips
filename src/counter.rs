/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
use tokio::sync::{mpsc, oneshot};
use tokio::time::{interval, Duration, MissedTickBehavior};

#[derive(Debug)]
pub enum CounterMessage {
    GetCount { respond_to: oneshot::Sender<u8> },
    SetCount { new_value: u8 },
}

#[derive(Debug)]
pub struct Counter {
    recv: mpsc::Receiver<CounterMessage>,
    value: u8,
}

impl Counter {
    fn new(recv: mpsc::Receiver<CounterMessage>) -> Self {
        Counter { recv, value: 0 }
    }

    fn handle_message(&mut self, msg: CounterMessage) {
        match msg {
            CounterMessage::GetCount { respond_to } => {
                let _ = respond_to.send(self.value);
            }
            CounterMessage::SetCount { new_value } => {
                self.value = new_value;
            }
        }
    }
}

pub async fn run_counter(mut counter: Counter) {
    // Count down at 60 Hz
    let mut ival = interval(Duration::from_secs_f64(crate::util::hz_to_secs("60Hz")));
    ival.set_missed_tick_behavior(MissedTickBehavior::Burst);
    loop {
        ival.tick().await;
        tokio::select! {
            Some(msg) = counter.recv.recv() => { counter.handle_message(msg) },
            else => {
                // The counter.recv should stay alive as long as the Chip8 is running
                // This branch is activated when the Chip8 stops executing.
                break
            },
        };
        if counter.value > 0 {
            counter.value -= 1;
        }
    }
}

#[derive(Clone, Debug)]
pub struct CounterHandle {
    sender: mpsc::Sender<CounterMessage>,
}

impl CounterHandle {
    pub fn new() -> Self {
        let (sender, recv) = mpsc::channel(10);
        let actor = Counter::new(recv);
        tokio::spawn(run_counter(actor));

        Self { sender }
    }

    pub async fn get(&self) -> u8 {
        let (send, recv) = oneshot::channel();
        let msg = CounterMessage::GetCount { respond_to: send };
        let _ = self.sender.send(msg).await;
        recv.await.unwrap()
    }

    pub async fn set(&self, value: u8) {
        let msg = CounterMessage::SetCount { new_value: value };
        let _ = self.sender.send(msg).await;
    }
}
