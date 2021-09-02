#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod icongen;

use rodio::{self, Source};
use std::{
  io::{self},
  sync::{Arc, Mutex},
  thread,
};
use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};

#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

trait ToMessage: Send {
  fn value(&self) -> Vec<u8>;
}

const TIME_MULTIPLIER: f32 = 60.0;
const BEEP: &[u8] = include_bytes!("../resources/ring.mp3");

fn main() {
  let pomodoro_time = Arc::new(Mutex::new(99999.0));

  // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
  let quit = CustomMenuItem::new("quit".to_string(), "Quit");
  let p25 = CustomMenuItem::new("p25".to_string(), "25");
  let p15 = CustomMenuItem::new("p15".to_string(), "15");
  let p5 = CustomMenuItem::new("p5".to_string(), "5");
  let tray_menu = SystemTrayMenu::new()
    .add_item(p25)
    .add_item(p15)
    .add_item(p5)
    .add_native_item(SystemTrayMenuItem::Separator)
    .add_item(quit)
    .add_native_item(SystemTrayMenuItem::Separator);

  let system_tray = SystemTray::new().with_menu(tray_menu);
  let (tx, rx) = crossbeam::channel::unbounded();

  let p_time = pomodoro_time.clone();
  thread::spawn(move || {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sound = Sound(Arc::new(BEEP.to_vec()));

    loop {
      let (tx_timer, rx_timer) = crossbeam::channel::unbounded();
      let timer = timer::Timer::new();
      let _guard = timer.schedule_with_delay(chrono::Duration::milliseconds(1000), move || {
        let _ignored = tx_timer.send(());
      });
      rx_timer.recv().unwrap();

      {
        let mut p_time = p_time.lock().unwrap();
        if *p_time >= 0.0 && *p_time != 99999.0 {
          if *p_time % TIME_MULTIPLIER <= 0.0 {
            let minutes = (*p_time / TIME_MULTIPLIER).ceil() as usize;
            tx.send(minutes).unwrap();
          }
          if *p_time == 0.0 {
            stream_handle
              .play_raw(sound.decoder().convert_samples())
              .unwrap();
          }

          *p_time -= 1.0;
        }
      }
    }
  });

  tauri::Builder::default()
    .system_tray(system_tray)
    .on_system_tray_event(move |_app, event| match event {
      SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
        "p25" => {
          let mut p_time = pomodoro_time.lock().unwrap();
          *p_time = 25.0 * TIME_MULTIPLIER;
        }
        "p15" => {
          let mut p_time = pomodoro_time.lock().unwrap();
          *p_time = 15.0 * TIME_MULTIPLIER;
        }
        "p5" => {
          let mut p_time = pomodoro_time.lock().unwrap();
          *p_time = 5.0 * TIME_MULTIPLIER;
        }
        "quit" => {
          std::process::exit(0);
        }
        _ => {}
      },
      _ => {}
    })
    .setup(move |app| {
      let tray_handle = app.tray_handle();

      tray_handle
        .set_icon(tauri::Icon::Raw(icongen::TOMATO_IMAGE.to_vec()))
        .unwrap();

      let icons = icongen::create_all_icons();

      let rx = rx.clone();
      tauri::async_runtime::spawn(async move {
        while let Ok(i) = rx.recv() {
          if i == 99999 {
            tray_handle
              .set_icon(tauri::Icon::Raw(icongen::TOMATO_IMAGE.to_vec()))
              .unwrap();
          } else {
            let selected_icon = &icons[i];
            tray_handle
              .set_icon(tauri::Icon::Raw(selected_icon.clone()))
              .unwrap();
          }
        }
      });

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

pub struct Sound(Arc<Vec<u8>>);

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
