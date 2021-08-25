#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::api::process::{Command, CommandEvent};
use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use tauri::{Manager, Window};
use crossbeam::channel;

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
  let hide = CustomMenuItem::new("hide".to_string(), "Hide");
  let p25 = CustomMenuItem::new("p25".to_string(), "25");
  let p15 = CustomMenuItem::new("p15".to_string(), "15");
  let p5 = CustomMenuItem::new("p5".to_string(), "5");
  let tray_menu = SystemTrayMenu::new()
    .add_item(p25)
    .add_item(p15)
    .add_item(p5)
    .add_native_item(SystemTrayMenuItem::Separator)
    .add_item(quit)
    .add_native_item(SystemTrayMenuItem::Separator)
    .add_item(hide);

  let system_tray = SystemTray::new().with_menu(tray_menu);

  let (tx, rx) = crossbeam::channel::unbounded();

  tauri::Builder::default()
    .system_tray(system_tray)
    .on_system_tray_event(move |app, event| match event {
      SystemTrayEvent::LeftClick {
        position: _,
        size: _,
        ..
      } => {
        println!("system tray received a left click");
        app
          .tray_handle()
          .set_icon(tauri::Icon::Raw(
            include_bytes!("../icons/tomato.ico").to_vec(),
          ))
          .unwrap();

        app
          .emit_all(
            "event-name",
            Payload {
              message: "Tauri is awesome!".into(),
            },
          )
          .unwrap();
        println!("Emited Tauri event");
      }
      // SystemTrayEvent::RightClick {
      //   position: _,
      //   size: _,
      //   ..
      // } => {
      //   println!("system tray received a right click");
      // }
      // SystemTrayEvent::DoubleClick {
      //   position: _,
      //   size: _,
      //   ..
      // } => {
      //   println!("system tray received a double click");
      // }
      SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
        "p25" => {
          tx.send(25);
        }
        "p15" => {
          tx.send(15);
        }
        "p5" => {
          tx.send(5);
        }
        "quit" => {
          std::process::exit(0);
        }
        "hide" => {
          let window = app.get_window("main").unwrap();
          window.hide().unwrap();
        }
        _ => {}
      },
      _ => {}
    })
    .setup(move |app| {
      app.get_window("main").unwrap().hide().unwrap();

      let rx = rx.clone();
      tauri::async_runtime::spawn(async move {
        while let Ok(i) = rx.recv() {
          println!("got = {}", i);
        }

        // let (mut rx, _child) = Command::new("node")
        //   .args(&[script_path])
        //   .spawn()
        //   .expect("Failed to spawn node");

        // #[allow(clippy::collapsible_match)]
        // while let Some(event) = rx.recv().await {
        //   if let CommandEvent::Stdout(line) = event {
        //     window
        //       .emit("message", Some(format!("'{}'", line)))
        //       .expect("failed to emit event");
        //   }
        // }
      });

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
