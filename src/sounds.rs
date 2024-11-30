use std::collections::HashMap;
use std::fs::read;
use std::io::{BufReader, Cursor};
use std::sync::mpsc::Receiver;
use lazy_static::lazy_static;

use rodio::{Decoder, OutputStream, Sink};
use crate::minifb::ChannelBit;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SoundType {
    Fire,
    InvaderDies,
    PlayerDies,
    Ufo,
    Invader1,
    Invader2,
    Invader3,
    Invader4,
    UfoHit,
}

pub struct Message {
    pub sound_type: SoundType,
    pub on: bool,
}

pub struct Sound {
    receiver: Receiver<Message>,
    sound_files: HashMap<SoundType, Vec<u8>>,
}

pub struct SoundInfo {
    pub sound_type: SoundType,
    pub path: String,
    pub channel_bit: ChannelBit,
}

impl SoundInfo {
    fn new(sound_type: SoundType, path: &str, channel: u8, bit: u8) -> Self {
        Self {
            sound_type, path: path.into(), channel_bit: ChannelBit { channel, bit }
        }
    }
}

lazy_static! {
    pub static ref ALL_SOUNDS: Vec<SoundInfo> = {
        let mut result = Vec::new();
        result.push(SoundInfo::new(SoundType::Ufo, "sounds/ufo_lowpitch.wav", 3, 0));
        result.push(SoundInfo::new(SoundType::Fire, "sounds/shoot.wav", 3, 1));
        result.push(SoundInfo::new(SoundType::PlayerDies, "sounds/explosion.wav", 3, 2));
        result.push(SoundInfo::new(SoundType::InvaderDies, "sounds/invaderkilled.wav", 3, 3));
        result.push(SoundInfo::new(SoundType::Invader1, "sounds/fastinvader1.wav", 5, 0));
        result.push(SoundInfo::new(SoundType::Invader2, "sounds/fastinvader2.wav", 5, 1));
        result.push(SoundInfo::new(SoundType::Invader3, "sounds/fastinvader3.wav", 5, 2));
        result.push(SoundInfo::new(SoundType::Invader4, "sounds/fastinvader4.wav", 5, 3));
        result.push(SoundInfo::new(SoundType::UfoHit, "sounds/explosion.wav", 5, 4));

        result
    };
}

impl Sound {
    pub fn new(receiver: Receiver<Message>) -> Self {
        let mut sound_files = HashMap::new();
        for s in ALL_SOUNDS.iter() {
            sound_files.insert(s.sound_type, read(s.path.clone()).unwrap());
        }
        Self { receiver, sound_files }
    }

    pub fn run(&self) {
        let mut done = false;
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        while ! done {
            match self.receiver.recv() {
                Ok(m) => {
                    let value = self.sound_files.get(&m.sound_type).cloned();
                    if let Some(bytes) = value {
                        if m.on {
                            let source = Decoder::new(BufReader::new(Cursor::new(bytes))).unwrap();
                            sink.append(source);
                            sink.sleep_until_end();
                        }
                    } else {
                        panic!("Unknown sound type: {:#?}", m.sound_type);
                    }
                }
                Err(e) => {
                    println!("Error on channel: {e}");
                    done = true;
                }
            }
        }
    }
}