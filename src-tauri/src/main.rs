#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod icongen;

use tauri::{
  CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
};

#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

trait ToMessage: Send {
  fn value(&self) -> Vec<u8>;
}

fn main() {
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

  tauri::Builder::default()
    .system_tray(system_tray)
    .on_system_tray_event(move |_app, event| match event {
      SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
        "p25" => {
          tx.send(25).unwrap();
        }
        "p15" => {
          tx.send(15).unwrap();
        }
        "p5" => {
          tx.send(5).unwrap();
        }
        "quit" => {
          std::process::exit(0);
        }
        _ => {}
      },
      _ => {}
    })
    .setup(move |app| {
      let icons = icongen::create_all_icons();
      let tray_handle = app.tray_handle();

      let rx = rx.clone();
      tauri::async_runtime::spawn(async move {
        while let Ok(i) = rx.recv() {
          let selected_icon = &icons[i];
          tray_handle
            .set_icon(tauri::Icon::Raw(selected_icon.clone()))
            .unwrap();
        }
      });

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
