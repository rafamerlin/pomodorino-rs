#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod icongen;
mod pomodoro;
mod sound;

use crate::pomodoro::{Pomodoro, PomodoroState};
use crate::sound::Beep;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};

#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

trait ToMessage: Send {
  fn value(&self) -> Vec<u8>;
}

fn main() {
  let system_tray = generate_menu();
  let (tx, rx) = crossbeam::channel::unbounded();
  let pomodoro = Arc::new(Mutex::new(Pomodoro::new(tx)));

  let pomo = pomodoro.clone();
  thread::spawn(move || loop {
    let (tx_timer, rx_timer) = crossbeam::channel::unbounded();
    let timer = timer::Timer::new();
    let _guard = timer.schedule_with_delay(chrono::Duration::milliseconds(1000), move || {
      let _ignored = tx_timer.send(());
    });
    rx_timer.recv().unwrap();

    {
      let mut pomo = pomo.lock().unwrap();
      pomo.tick();
    }
  });

  let pomo = pomodoro.clone();

  tauri::Builder::default()
    .system_tray(system_tray)
    .on_system_tray_event(move |_app, event| match event {
      SystemTrayEvent::LeftClick { .. } => {
        let mut pomo = pomo.lock().unwrap();
        pomo.clear();
      }
      SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
        "p25" => {
          let mut pomo = pomo.lock().unwrap();
          pomo.start(25);
        }
        "p15" => {
          let mut pomo = pomo.lock().unwrap();
          pomo.start(15);
        }
        "p5" => {
          let mut pomo = pomo.lock().unwrap();
          pomo.start(5);
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
        let beep = Beep::new();

        while let Ok(pomo) = rx.recv() {
          match pomo {
            PomodoroState::Clear => {
              tray_handle
                .set_icon(tauri::Icon::Raw(icongen::TOMATO_IMAGE.to_vec()))
                .unwrap();
            }
            PomodoroState::Running(_, m) => {
              let selected_icon = &icons[m];
              tray_handle
                .set_icon(tauri::Icon::Raw(selected_icon.clone()))
                .unwrap();
            }
            PomodoroState::Completed => {
              beep.play();
              println!("Beep played");
              tray_handle
                .set_icon(tauri::Icon::Raw(icongen::YOMATO_IMAGE.to_vec()))
                .unwrap();
            }
          }
        }
      });

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

fn generate_menu() -> SystemTray {
  let p25 = CustomMenuItem::new("p25".to_string(), "25");
  let p15 = CustomMenuItem::new("p15".to_string(), "15");
  let p5 = CustomMenuItem::new("p5".to_string(), "5");
  let quit = CustomMenuItem::new("quit".to_string(), "Quit");
  let cancel = CustomMenuItem::new("cancel".to_string(), "Cancel");
  let tray_menu = SystemTrayMenu::new()
    .add_item(p25)
    .add_item(p15)
    .add_item(p5)
    .add_native_item(SystemTrayMenuItem::Separator)
    .add_item(cancel)
    .add_native_item(SystemTrayMenuItem::Separator)
    .add_item(quit);

  SystemTray::new().with_menu(tray_menu)
}
