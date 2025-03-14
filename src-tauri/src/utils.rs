use elevated_command::Command;
use std::process::Command as StdCommand;
use anyhow::{Result, Error};
use tauri::{AppHandle, Emitter};

pub fn send_to_frontend(app_handle: &AppHandle, message: String, event_type: &str) {
    println!("{}", message);
    app_handle.emit(event_type, message).unwrap();
}

#[tauri::command]
pub async fn install_fluidsynth(app_handle: AppHandle) -> Result<String, String> {
    // Check if FluidSynth is already installed
    if is_fluidsynth_installed() {
        return Ok("FluidSynth is already installed.".to_string());
    }

    // Detect WSL
    let is_wsl = std::fs::read_to_string("/proc/version").map_or(false, |v| v.contains("microsoft"));

    // WSL specific handling
    if cfg!(target_os = "linux") && is_wsl {
        send_to_frontend(&app_handle, "Detected WSL, Please enter the password to install dependencies".to_string(), "info");
        let mut cmd = StdCommand::new("sudo");
        cmd.args(["apt", "install", "-y", "fluidsynth"]);

        // Inherit stdin/stdout so the password prompt appears in terminal
        cmd.stdin(std::process::Stdio::inherit());
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        return match cmd.status() {
            Ok(status) if status.success() => Ok("Installation successful.".to_string()),
            Ok(status) => Err(format!("Installation failed with status: {}", status)),
            Err(err) => Err(format!("Error executing command: {}", err)),
        };
    }

    // Platform-specific commands
    let (cmd_path, cmd_args) = if cfg!(target_os = "windows") {
        // Check if winget is available
        if StdCommand::new("winget")
            .arg("--version")
            .output()
            .map_or(false, |o| o.status.success())
        {
            (
                "winget",
                vec!["install", "-e", "--id", "FluidSynth.FluidSynth"],
            )
        } else {
            return Err("Winget not found. Please install FluidSynth manually.".to_string());
        }
    } else if cfg!(target_os = "macos") {
        // Check if brew is available
        if StdCommand::new("brew")
            .arg("--version")
            .output()
            .map_or(false, |o| o.status.success())
        {
            ("brew", vec!["install", "fluidsynth"])
        } else {
            return Err(
                "Homebrew not found. Please install it first or install FluidSynth manually."
                    .to_string(),
            );
        }
    } else if cfg!(target_os = "linux") {
        // Try to detect package manager for Linux
        if StdCommand::new("apt")
            .arg("-v")
            .output()
            .map_or(false, |o| o.status.success())
        {
            ("apt", vec!["install", "-y", "fluidsynth"])
        } else if StdCommand::new("dnf")
            .arg("--version")
            .output()
            .map_or(false, |o| o.status.success())
        {
            ("dnf", vec!["install", "-y", "fluidsynth"])
        } else if StdCommand::new("pacman")
            .arg("-V")
            .output()
            .map_or(false, |o| o.status.success())
        {
            ("pacman", vec!["-S", "--noconfirm", "fluidsynth"])
        } else {
            return Err(
                "Unable to detect package manager. Please install FluidSynth manually.".to_string(),
            );
        }
    } else {
        return Err("Unsupported operating system".to_string());
    };

    send_to_frontend(
        &app_handle,
        format!("Installing FluidSynth using {}...", cmd_path),
        "info",
    );

    // Check if we're already elevated
    let is_elevated = Command::is_elevated();

    let output = if is_elevated {
        // If already elevated, run the command directly
        let mut cmd = StdCommand::new(cmd_path);
        cmd.args(&cmd_args);
        cmd.output().map_err(|e| e.to_string())
    } else {
        // If not elevated, use elevated-command to request elevation
        let mut cmd = StdCommand::new(cmd_path);
        cmd.args(&cmd_args);
        let elevated_cmd = Command::new(cmd);
        elevated_cmd.output().map_err(|e| e.to_string())
    };

    // Process the result
    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            send_to_frontend(
                &app_handle,
                "Installation successful".to_string(),
                "success",
            );
            Ok(format!("Installation successful: {}", stdout))
        }
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            send_to_frontend(
                &app_handle,
                format!("Installation failed: {}", stderr),
                "error",
            );
            Err(format!("Installation failed: {}", stderr))
        }
        Err(err) => {
            send_to_frontend(&app_handle, format!("Error: {}", err), "error");
            Err(format!("Error executing command: {}", err))
        }
    }
}

fn is_fluidsynth_installed() -> bool {
    // Implementation depends on OS
    false // Placeholder
}










// use elevated_command::Command;
// use std::process::Command as StdCommand;
// use anyhow::{Result, Error};
// use tauri::{AppHandle, Emitter};

// pub fn send_to_frontend(app_handle: &AppHandle, message: String, event_type: &str) {
//     println!("{}", message);
//     app_handle.emit(event_type, message).unwrap();
// }

// #[tauri::command]
// pub async fn install_fluidsynth(app_handle: AppHandle) -> Result<String, String> {
//     // 🔹 Step 1: Detect if running inside WSL
//     let is_wsl = std::fs::read_to_string("/proc/version").map_or(false, |v| v.contains("microsoft"));

//     // 🔹 Step 2: Linux & WSL Handling
//     if cfg!(target_os = "linux") && is_wsl {
//         send_to_frontend(&app_handle, "Detected WSL, Please enter the password to install dependencies".to_string(), "info");
//         let mut cmd = StdCommand::new("sudo");
//         cmd.args(["apt", "install", "-y", "fluidsynth"]);

//         // Inherit stdin/stdout so the password prompt appears in terminal
//         cmd.stdin(std::process::Stdio::inherit());
//         cmd.stdout(std::process::Stdio::inherit());
//         cmd.stderr(std::process::Stdio::inherit());

//         return match cmd.status() {
//             Ok(status) if status.success() => Ok("Installation successful.".to_string()),
//             Ok(status) => Err(format!("Installation failed with status: {}", status)),
//             Err(err) => Err(format!("Error executing command: {}", err)),
//         };
//     }

//     // 🔹 Step 3: Windows/macOS Handling
//     let is_elevated = Command::is_elevated();

//     let cmd_args = if cfg!(target_os = "windows") {
//         vec!["cmd", "/C", "winget install -e --id FluidSynth.FluidSynth"]
//     } else if cfg!(target_os = "macos") {
//         vec!["brew", "install", "fluidsynth"]
//     } else {
//         vec!["apt", "install", "fluidsynth"]
//     };

//     let output: Result<std::process::Output> = if is_elevated {
//         let mut cmd = StdCommand::new(cmd_args[0]);
//         cmd.args(&cmd_args[1..]);
//         cmd.output().map_err(anyhow::Error::from)
//     } else {
//         let mut c = StdCommand::new(cmd_args[0]);
//         c.args(&cmd_args[1..]);
//         let elevated_cmd = Command::new(c);
//         elevated_cmd.output().map_err(anyhow::Error::from)
//     };

//     match output {
//         Ok(result) => Ok(format!("Installation successful: {}", String::from_utf8_lossy(&result.stdout))),
//         Err(err) => Err(format!("Error executing command: {}", err)),
//     }
// }