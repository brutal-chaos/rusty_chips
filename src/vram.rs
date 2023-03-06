/// Copyright 2015-2023, Justin Noah <justinnoah at gmail.com>, All Rights Reserved
use std::ops::{Index, IndexMut};

use tokio::sync::mpsc;

// TODO: Remove this allowance when SuperChip8 is ready
#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum ScreenSize {
    L,
    S,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub enum Memory {
    l(Box<[[bool; 128]; 64]>),
    s(Box<[[bool; 64]; 32]>),
}

#[allow(non_snake_case)]
impl Memory {
    fn L() -> Self {
        Memory::l(Box::new([[false; 128]; 64]))
    }

    fn S() -> Self {
        Memory::s(Box::new([[false; 64]; 32]))
    }
}

impl Index<(usize, usize)> for Memory {
    type Output = bool;

    fn index(&self, pos: (usize, usize)) -> &Self::Output {
        match self {
            Memory::l(scrn) => &scrn[pos.1][pos.0],
            Memory::s(scrn) => &scrn[pos.1][pos.0],
        }
    }
}

impl IndexMut<(usize, usize)> for Memory {
    fn index_mut(&mut self, pos: (usize, usize)) -> &mut Self::Output {
        match self {
            Memory::l(scrn) => &mut scrn[pos.1][pos.0],
            Memory::s(scrn) => &mut scrn[pos.1][pos.0],
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub struct VRAM {
    width: usize,
    height: usize,
    mem: Memory,
    receiver: mpsc::Receiver<VRAMMessage>,
}

impl VRAM {
    fn new_large(receiver: mpsc::Receiver<VRAMMessage>) -> Self {
        VRAM {
            width: 128,
            height: 64,
            mem: Memory::L(),
            receiver,
        }
    }

    fn new_small(receiver: mpsc::Receiver<VRAMMessage>) -> Self {
        VRAM {
            width: 64,
            height: 32,
            mem: Memory::S(),
            receiver,
        }
    }

    async fn handle_message(&mut self, msg: VRAMMessage) {
        match msg {
            VRAMMessage::Get { respond_to } => respond_to.send(self.mem.clone()).await.unwrap(),
            VRAMMessage::GetPixel { x, y, respond_to } => {
                respond_to.send(self[(x, y)]).await.unwrap()
            }
            VRAMMessage::SetPixel { x, y, value } => self[(x, y)] = value,
            VRAMMessage::Clear => {
                for y in 0..self.height {
                    for x in 0..self.width {
                        self[(x, y)] = false
                    }
                }
            }
        }
    }
}

impl Index<(usize, usize)> for VRAM {
    type Output = bool;

    fn index(&self, pos: (usize, usize)) -> &Self::Output {
        &self.mem[pos]
    }
}

impl IndexMut<(usize, usize)> for VRAM {
    fn index_mut(&mut self, pos: (usize, usize)) -> &mut Self::Output {
        &mut self.mem[pos]
    }
}

pub async fn vram_runner(mut video: VRAM) {
    while let Some(msg) = video.receiver.recv().await {
        video.handle_message(msg).await
    }
}

#[derive(Debug)]
pub enum VRAMMessage {
    Get {
        respond_to: mpsc::Sender<Memory>,
    },
    GetPixel {
        x: usize,
        y: usize,
        respond_to: mpsc::Sender<bool>,
    },
    SetPixel {
        x: usize,
        y: usize,
        value: bool,
    },
    Clear,
}

#[derive(Clone, Debug)]
pub struct VRAMHandle {
    sender: mpsc::Sender<VRAMMessage>,
    screen_size: ScreenSize,
}

impl VRAMHandle {
    pub fn new(screen_size: ScreenSize) -> Self {
        let (sender, receiver) = mpsc::channel(10);
        let vram = match screen_size {
            ScreenSize::L => VRAM::new_large(receiver),
            ScreenSize::S => VRAM::new_small(receiver),
        };
        tokio::spawn(vram_runner(vram));

        Self {
            sender,
            screen_size,
        }
    }

    pub fn get_screen_size(&self) -> (usize, usize) {
        match self.screen_size {
            ScreenSize::L => (128, 64),
            ScreenSize::S => (64, 32),
        }
    }

    pub async fn get(&self) -> Memory {
        let (send, mut recv) = mpsc::channel(10);
        let msg = VRAMMessage::Get { respond_to: send };
        let _ = self.sender.send(msg).await;
        loop {
            let m = recv.recv().await;
            return match m {
                Some(mem) => mem,
                _ => continue,
            };
        }
    }

    pub async fn get_pixel(&self, x: usize, y: usize) -> bool {
        let (send, mut recv) = mpsc::channel(1);
        let msg = VRAMMessage::GetPixel {
            x,
            y,
            respond_to: send,
        };
        let _ = self.sender.send(msg).await;
        recv.recv().await.unwrap()
    }

    pub async fn set_pixel(&self, x: usize, y: usize, value: bool) {
        let msg = VRAMMessage::SetPixel { x, y, value };
        let _ = self.sender.send(msg).await;
    }

    pub async fn clear_screen(&self) {
        let msg = VRAMMessage::Clear;
        let _ = self.sender.send(msg).await;
    }
}
