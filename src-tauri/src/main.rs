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

fn main() {
  tauri::AppBuilder::new()
    .setup(|_webview, _| {
        let handle_clone = _webview.handle().clone();
        std::thread::spawn(move || {
            spawn_vietbible_flutter(&handle_clone);
        });
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
    let stdout = Command::new(ipfs_binary)
        .args(vec!["daemon", "--init"])
        .env("IPFS_PATH", "~/.ipfs")
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
            print!("{}", line);
            if line.starts_with("Daemon is ready") {
                let url = "http://127.0.0.1:5001/webui";
                if !vietbible_started {
                    vietbible_started = true;
                    handle
                        .dispatch(move |webview| webview.eval(&format!("window.location.replace('{}')", url)))
                        .expect("failed to init app");
                    // let vietbible_binary = get_bin_command("Viet.Bible.app");
                    // print!("VB Bin: {}", vietbible_binary);
                    // let stdout_flutter = Command::new(vietbible_binary)
                    //     .stdout(Stdio::piped())
                    //     .spawn()
                    //     .expect("")
                    //     .stdout.expect("Error");
                    // let reader_vb = BufReader::new(stdout_flutter);
                    // reader_vb
                    //     .lines()
                    //     .filter_map(|line| line.ok())
                    //     .for_each(|line| {
                    //         println!("Message from flutter: {}", line);
                    //     });
                }
            }
        })
}
