use rodio::{OutputStream, OutputStreamHandle, Source};
use std::io;
use std::sync::Arc;

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
  //Must exist even though it's not used as it's what is used by the handle
  //So it needs to be in scope.
  _output_stream: OutputStream,
  output_stream_handle: OutputStreamHandle,
  sound: Sound,
}

impl Beep {
  pub fn new() -> Self {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sound = Sound(Arc::new(BEEP.to_vec()));

    Self {
      _output_stream: _stream,
      output_stream_handle: stream_handle,
      sound,
    }
  }

  pub fn play(&self) {
    self
      .output_stream_handle
      .play_raw(self.sound.decoder().convert_samples())
      .unwrap();
      std::thread::sleep(std::time::Duration::from_secs(1));
  }
}
