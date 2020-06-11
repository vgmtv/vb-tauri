#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;

use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};
use tauri::{Handle};

fn main() {// Register a listener to the "js-event" event
  tauri::AppBuilder::new()
    .setup(|_webview, _| {
        let handle_clone = _webview.handle().clone();
        std::thread::spawn(move || {
            spawn_vietbible_flutter(&handle_clone);
        });
    })
    .invoke_handler(|_webview, arg| {
      use cmd::Cmd::*;
      match serde_json::from_str(arg) {
        Err(e) => {
          Err(e.to_string())
        }
        Ok(command) => {
          match command {
            // definitions for your custom commands from Cmd here
            PerformRequest {
              endpoint,
              body,
              callback,
              error,
            } => {
              // tauri::execute_promise is a helper for APIs that uses the tauri.promisified JS function
              // so you can easily communicate between JS and Rust with promises
              tauri::execute_promise(
                _webview,
                move || {
                  println!("{} {:?}", endpoint, body);
                  // perform an async operation here
                  // if the returned value is Ok, the promise will be resolved with its value
                  // if the returned value is Err, the promise will be rejected with its value
                  // the value is a string that will be eval'd
                  Ok("{ message: 'Hello World from Rust!' }".to_string())
                },
                callback,
                error,
              )
            }
          }
          Ok(())
        }
      }
    })
    .build()
    .run();
}

fn get_bin_command(name: &str) -> String {
    tauri::api::command::relative_command(
        tauri::api::command::binary_command(name.to_string()).unwrap(),
    )
    .unwrap()
}

fn spawn_vietbible_flutter<T: 'static>(handle: &Handle<T>) {
    // Get path to orchestrator and main binary
    let ipfs_binary = get_bin_command("ipfs");
    let vietbible_binary = get_bin_command("Viet.Bible.app");
    println!("IPFS Binary: {}", ipfs_binary);
    let stdout = Command::new(ipfs_binary)
        .args(vec!["deamon", "--init"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("EXIT STATUS")
        .stdout
        .expect("Failed to get ipfs server stdout");

    let reader = BufReader::new(stdout);
    let mut vietbible_started = false;
    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| {
            if line.starts_with("Daemon is ready") {
                let url = "http://127.0.0.1:5001/webui";
                if !vietbible_started {
                    vietbible_started = true;
                    handle
                        .dispatch(move |webview| webview.eval(&format!("window.location.replace('{}')", url)))
                        .expect("failed to init app");
                }
            }
        })
}
