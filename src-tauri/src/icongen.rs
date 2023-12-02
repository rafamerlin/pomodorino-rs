use image::{ImageFormat, Rgba};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
#[cfg(not(target_os = "linux"))]
use std::io::Cursor;
#[cfg(target_os = "linux")]
use std::path::PathBuf;

const TOMATO_IMAGE: &[u8] = include_bytes!("../icons/tomato.ico");
const YOMATO_IMAGE: &[u8] = include_bytes!("../icons/yomato.ico");
const FONT: &[u8] = include_bytes!("../resources/DejaVuSansMono-Bold.ttf");

#[derive(Debug)]
pub struct PomodoroIcons {
  pub tomato: PomodoroIcon,
  pub yomato: PomodoroIcon,
  pub icons: Vec<PomodoroIcon>,
}

#[derive(Debug)]
pub struct PomodoroIcon {
  #[cfg(target_os = "linux")]
  pub icon: PathBuf,
  #[cfg(not(target_os = "linux"))]
  pub icon: Vec<u8>,
}

#[cfg(target_os = "linux")]
pub fn create_all_icons() -> PomodoroIcons {
  let mut icons = vec![];
  for i in 1..26 {
    icons.push(create_icon(i));
  }

  PomodoroIcons {
    icons,
    tomato: create_base_icons(BaseIcons::Tomato),
    yomato: create_base_icons(BaseIcons::Yomato),
  }
}

#[cfg(not(target_os = "linux"))]
pub fn create_all_icons() -> PomodoroIcons {
  let tomato = PomodoroIcon {
    icon: TOMATO_IMAGE.to_vec(),
  };
  let yomato = PomodoroIcon {
    icon: YOMATO_IMAGE.to_vec(),
  };

  let mut icons = vec![];
  for i in 1..26 {
    icons.push(create_icon(i));
  }

  PomodoroIcons {
    icons,
    tomato,
    yomato,
  }
}

#[cfg(target_os = "linux")]
pub enum BaseIcons {
  Tomato,
  Yomato,
}

#[cfg(target_os = "linux")]
impl std::fmt::Display for BaseIcons {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      BaseIcons::Tomato => write!(f, "tomato"),
      BaseIcons::Yomato => write!(f, "yomato"),
    }
  }
}

#[cfg(target_os = "linux")]
fn create_base_icons(icon: BaseIcons) -> PomodoroIcon {
  let icon_to_create = match icon {
    BaseIcons::Tomato => TOMATO_IMAGE,
    BaseIcons::Yomato => YOMATO_IMAGE,
  };

  let image = image::load_from_memory_with_format(icon_to_create, ImageFormat::Ico)
    .expect("Couldn't load image");

  let path = linux::prepare_tmp_path();
  let path = path.join(format!("{}.ico", &icon));

  if !path.exists() {
    println!("Path to save image {:?}", path);
    image.save(&path).unwrap();
  }

  PomodoroIcon { icon: path }
}

fn create_icon(value: usize) -> PomodoroIcon {
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

  #[cfg(not(target_os = "linux"))]
  {
    let mut i = Cursor::new(Vec::new());
    image.write_to(&mut i, ImageFormat::Ico).unwrap();

    PomodoroIcon {
      icon: i.into_inner(),
    }
  }

  #[cfg(target_os = "linux")]
  {
    let path = linux::prepare_tmp_path();
    let path = path.join(format!("{}.ico", &value));

    if !path.exists() {
      image.save(&path).unwrap();
    }

    PomodoroIcon { icon: path }
  }
}

#[cfg(target_os = "linux")]
mod linux {
  use std::{env, fs, path::PathBuf};

  pub fn prepare_tmp_path() -> PathBuf {
    let mut path = env::temp_dir();
    path.push("pomodorino");

    if !path.exists() {
      fs::create_dir_all(&path).unwrap();
    }

    path
  }
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_icon_generation_in_linux() {
    //To run tests and get the printlns outputed
    // cargo test -- --nocapture
    let icons = create_all_icons();

    println!("Icons {:?}", icons);

    let tomato_path = icons.tomato.icon.to_string_lossy().to_string();
    let extension = tomato_path[tomato_path.len() - 3..].to_string();

    assert_eq!(icons.icons.len(), 25);
    assert_eq!("ico", extension);
  }

  #[test]
  #[cfg(target_os = "linux")]
  fn test_icon_type_enum_display() {
    let to = format!("{}", BaseIcons::Tomato);
    let yo = format!("{}", BaseIcons::Yomato);

    assert_eq!("tomato", to);
    assert_eq!("yomato", yo);
  }
}
