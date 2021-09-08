use std::sync::Arc;
use std::io;
use rodio::{OutputStream, OutputStreamHandle};

pub struct Sound(pub Arc<Vec<u8>>);

impl AsRef<[u8]> for Sound {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Sound {
    pub fn cursor(&self) -> io::Cursor<Sound> {
        io::Cursor::new(Sound(self.0.clone()))
    }
    pub fn decoder(&self) -> rodio::Decoder<io::Cursor<Sound>> {
        rodio::Decoder::new(self.cursor()).unwrap()
    }
}

const BEEP: &[u8] = include_bytes!("../resources/ring.mp3");

pub struct Beep {
    output_stream: OutputStream,
    output_stream_handle: OutputStreamHandle,
    sound: Sound,
}

impl Beep
{
    pub fn new() -> Self {
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sound = Sound(Arc::new(BEEP.to_vec()));

        Self {
            output_stream: stream,
            output_stream_handle: stream_handle,
            sound,
        }
    }

    pub fn play(&self) {
        self.output_stream_handle
            .play_raw(sound.decoder().convert_samples())
            .unwrap();
    }
}