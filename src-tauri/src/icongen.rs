use image::{ImageFormat, Rgba};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};

pub const TOMATO_IMAGE: &[u8] = include_bytes!("../icons/tomato.ico");
pub const YOMATO_IMAGE: &[u8] = include_bytes!("../icons/yomato.ico");
const FONT: &[u8] = include_bytes!("../resources/DejaVuSansMono-Bold.ttf");

pub fn create_all_icons() -> Vec<Vec<u8>> {
  let mut icons = vec![YOMATO_IMAGE.to_vec()];
  for i in 1..26 {
    icons.push(create_icon(i));
  }

  icons
}

fn create_icon(value: usize) -> Vec<u8> {
  let font = Font::try_from_bytes(FONT).expect("Couldn't load font");
  let mut image = image::load_from_memory_with_format(TOMATO_IMAGE, ImageFormat::Ico)
    .expect("Couldn't load image");

  let height = 35.0;
  let scale = Scale {
    x: height * 0.8,
    y: height,
  };

  let x = if value >= 10 { 1 } else { 9 };

  draw_text_mut(
    &mut image,
    Rgba([0u8, 0u8, 0u8, 255u8]),
    x,
    0,
    scale,
    &font,
    &value.to_string(),
  );

  let mut i = Vec::new();
  image.write_to(&mut i, ImageFormat::Ico).unwrap();

  i
}
