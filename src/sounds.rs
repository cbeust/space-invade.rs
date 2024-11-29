use std::collections::HashMap;
use std::fs::read;
use std::io::{BufReader, Cursor};
use std::sync::mpsc::Receiver;

use rodio::{Decoder, OutputStream, Sink};

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

impl Sound {
    pub fn new(receiver: Receiver<Message>) -> Self {
        let mut sound_files = HashMap::new();
        for (sound_type, path) in &[
            (SoundType::Fire, "sounds/shoot.wav"),
            (SoundType::Fire, "sounds/shoot.wav"),
            (SoundType::PlayerDies, "sounds/explosion.wav"),
            (SoundType::InvaderDies, "sounds/invaderkilled.wav"),
            (SoundType::Ufo, "sounds/ufo_lowpitch.wav"),
            (SoundType::Invader1, "sounds/fastinvader1.wav"),
            (SoundType::Invader2, "sounds/fastinvader2.wav"),
            (SoundType::Invader3, "sounds/fastinvader3.wav"),
            (SoundType::Invader4, "sounds/fastinvader4.wav"),
            (SoundType::UfoHit, "sounds/explosion.wav"),
        ] {
            sound_files.insert(sound_type.clone(), read(path).unwrap());
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