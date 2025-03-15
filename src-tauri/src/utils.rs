use std::process::{exit, Child, Command, Stdio};
use tauri::{AppHandle, Emitter, Manager};
use std::io::{BufRead, BufReader};

use crate::setup::EnvPaths;

pub fn send_to_frontend(app_handle: &AppHandle, message: String, event_type: &str) {
    println!("{}", message);
    app_handle.emit(event_type, message).unwrap();
}

pub fn execute_command(app: &AppHandle, command: &mut Command, cmd_type: String) -> std::io::Result<Child> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // Read and send stdout messages in real-time
    for line in stdout_reader.lines() {
        if let Ok(output) = line {
            send_to_frontend(app, format!("{}", output), "status_update");
        }
    }

    // Read and send stderr messages in real-time
    for line in stderr_reader.lines() {
        if let Ok(error) = line {
            send_to_frontend(app, format!("Error: {}", error), "status_update");
        }
    }

    Ok(child)
}

#[tauri::command]
pub async fn reset_all(app: AppHandle) -> Result<bool, String> {
    let paths = EnvPaths::new();
    let mut command = Command::new("rm");
    command.args(&["-rf", paths.temp_dir.to_str().unwrap()]);

    match execute_command(&app, &mut command, "copy_resource".to_string()) {
        Ok(mut child) => match child.wait() {
            Ok(exit_status) if exit_status.success() => {
                send_to_frontend(&app, "Removed All Cached Data".to_string(), "success_reset");
                Ok(true)  // Return `true` on success
            }
            Ok(exit_status) => {
                let error_msg = format!("Failed to remove: {}", exit_status);
                send_to_frontend(&app, error_msg.clone(), "error");
                Err(error_msg) // Return error as `Err`
            }
            Err(e) => {
                let error_msg = format!("Failed to remove: {}", e);
                send_to_frontend(&app, error_msg.clone(), "error");
                Err(error_msg)
            }
        },
        Err(e) => {
            let error_msg = format!("Failed to start copy process: {}", e);
            send_to_frontend(&app, error_msg.clone(), "error");
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub fn relaunch(app: tauri::AppHandle) {
    app.restart();
}