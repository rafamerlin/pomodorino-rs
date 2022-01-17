#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod icongen;
mod pomodoro;
mod sound;

use crate::pomodoro::{Pomodoro, PomodoroState};
use crate::sound::Beep;
use icongen::PomodoroIcon;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};

const INFO: &'static str = "info";

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

  // TODO: Use this to create Linux's icons.
  // let path = std::env::current_dir().unwrap();
  // println!("The current directory is {}", path.display());

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
        "cancel" => {
          let mut pomo = pomo.lock().unwrap();
          pomo.cancel();
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
      let icons = icongen::create_all_icons();
      set_tray_icon(&tray_handle, &icons.tomato);

      let rx = rx.clone();
      tauri::async_runtime::spawn(async move {
        while let Ok(pomo) = rx.recv() {
          match pomo {
            PomodoroState::Clear => {
              set_tray_icon(&tray_handle, &icons.tomato);
            }
            PomodoroState::Running(_, m, info) => {
              let selected_icon = &icons.icons[m-1];
              set_tray_icon(&tray_handle, selected_icon);
              if let Some(info) = info {
                let item_handle = tray_handle.get_item(INFO);
                item_handle.set_title(info).unwrap();
              }
            }
            PomodoroState::Completed(info) => {
              set_tray_icon(&tray_handle, &icons.yomato);
              if let Some(info) = info {
                let item_handle = tray_handle.get_item(INFO);
                item_handle.set_title(info).unwrap();
              }
              Beep::new().play();
            }
          }
        }
      });

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[cfg(not(target_os = "linux"))]
fn set_tray_icon(tray_handle: &tauri::SystemTrayHandle, pomo_icon: &PomodoroIcon) {
  tray_handle
    .set_icon(tauri::Icon::Raw(pomo_icon.icon))
    .unwrap();
}

#[cfg(target_os = "linux")]
fn set_tray_icon(tray_handle: &tauri::SystemTrayHandle, pomo_icon: &PomodoroIcon) {
  tray_handle
    .set_icon(tauri::Icon::File(pomo_icon.icon.clone()))
    .unwrap();
}

fn generate_menu() -> SystemTray {
  let info = CustomMenuItem::new(INFO.to_string(), "Pomodorino").disabled();
  let p25 = CustomMenuItem::new("p25".to_string(), "25");
  let p15 = CustomMenuItem::new("p15".to_string(), "15");
  let p5 = CustomMenuItem::new("p5".to_string(), "5");
  let quit = CustomMenuItem::new("quit".to_string(), "Quit");
  let cancel = CustomMenuItem::new("cancel".to_string(), "Cancel");
  let tray_menu = SystemTrayMenu::new()
    .add_item(info)
    .add_native_item(SystemTrayMenuItem::Separator)
    .add_item(p25)
    .add_item(p15)
    .add_item(p5)
    .add_native_item(SystemTrayMenuItem::Separator)
    .add_item(cancel)
    .add_native_item(SystemTrayMenuItem::Separator)
    .add_item(quit);

  SystemTray::new().with_menu(tray_menu)
}
