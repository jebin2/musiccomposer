use elevated_command::Command;
use std::process::Command as StdCommand;
use tauri::{AppHandle, Emitter, Manager};
use crate::utils::send_to_frontend;

pub async fn install_fluidsynth(app_handle: AppHandle) -> Result<String, String> {
    // Check if FluidSynth is already installed
    if cfg!(target_os = "windows") || is_fluidsynth_installed() {
        let msg = "FluidSynth is already installed.".to_string();
        send_to_frontend(&app_handle, msg.clone(), "initialize_setup_processing");
        return Ok(msg);
    }

    // Detect WSL
    let is_wsl = std::fs::read_to_string("/proc/version").map_or(false, |v| v.contains("microsoft"));

    // WSL specific handling
    if cfg!(target_os = "linux") && is_wsl {
        send_to_frontend(&app_handle, "Detected WSL, Please enter the password to install dependencies".to_string(), "initialize_setup_processing");
        let mut cmd = StdCommand::new("sudo");
        cmd.args(["apt", "install", "-y", "fluidsynth"]);

        // Inherit stdin/stdout so the password prompt appears in terminal
        cmd.stdin(std::process::Stdio::inherit());
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        send_to_frontend(&app_handle, "Executing sudo apt install -y fluidsynth...".to_string(), "initialize_setup_processing");
        return match cmd.status() {
            Ok(status) if status.success() => {
                send_to_frontend(&app_handle, "WSL installation successful.".to_string(), "initialize_setup_processing");
                Ok("Installation successful.".to_string())
            },
            Ok(status) => {
                send_to_frontend(&app_handle, format!("WSL installation failed with status: {}", status), "initialize_setup_error");
                Err(format!("Installation failed with status: {}", status))
            },
            Err(err) => {
                send_to_frontend(&app_handle, format!("Error executing WSL command: {}", err), "initialize_setup_error");
                Err(format!("Error executing command: {}", err))
            },
        };
    }

    // Platform-specific commands
    send_to_frontend(&app_handle, "Determining installation method for current platform...".to_string(), "initialize_setup_processing");
    let (cmd_path, cmd_args) = if cfg!(target_os = "windows") {
        send_to_frontend(&app_handle, "Windows platform detected. Checking for winget...".to_string(), "initialize_setup_processing");
        // Check if winget is available
        let winget_check = StdCommand::new("winget")
            .arg("--version")
            .output();
            
        match winget_check {
            Ok(output) => {
                if output.status.success() {
                    send_to_frontend(&app_handle, "Winget is available. Will use winget to install FluidSynth.".to_string(), "initialize_setup_processing");
            (
                "winget",
                vec!["install", "-e", "--id", "FluidSynth.FluidSynth"],
            )
        } else {
                    send_to_frontend(&app_handle, format!("Winget check failed: {:?}", output), "initialize_setup_error");
                    return Err("Winget is installed but not working properly. Please install FluidSynth manually.".to_string());
                }
            },
            Err(e) => {
                send_to_frontend(&app_handle, format!("Winget not found: {}", e), "initialize_setup_error");
            return Err("Winget not found. Please install FluidSynth manually.".to_string());
            }
        }
    } else if cfg!(target_os = "macos") {
        send_to_frontend(&app_handle, "macOS platform detected. Checking for Homebrew...".to_string(), "initialize_setup_processing");
        // Check if brew is available
        let brew_check = StdCommand::new("brew")
            .arg("--version")
            .output();
            
        match brew_check {
            Ok(output) => {
                if output.status.success() {
                    send_to_frontend(&app_handle, "Homebrew is available. Will use brew to install FluidSynth.".to_string(), "initialize_setup_processing");
            ("brew", vec!["install", "fluidsynth"])
        } else {
                    send_to_frontend(&app_handle, format!("Homebrew check failed: {:?}", output), "initialize_setup_color");
                    return Err("Homebrew is installed but not working properly. Please install FluidSynth manually.".to_string());
                }
            },
            Err(e) => {
                send_to_frontend(&app_handle, format!("Homebrew not found: {}", e), "initialize_setup_color");
                return Err("Homebrew not found. Please install it first or install FluidSynth manually.".to_string());
            }
        }
    } else if cfg!(target_os = "linux") {
        send_to_frontend(&app_handle, "Linux platform detected. Detecting package manager...".to_string(), "initialize_setup_processing");
        // Try to detect package manager for Linux
        if let Ok(output) = StdCommand::new("apt").arg("-v").output() {
            if output.status.success() {
                send_to_frontend(&app_handle, "apt package manager detected.".to_string(), "initialize_setup_processing");
            ("apt", vec!["install", "-y", "fluidsynth"])
            } else {
                send_to_frontend(&app_handle, "apt check failed, trying dnf...".to_string(), "initialize_setup_processing");
                if let Ok(output) = StdCommand::new("dnf").arg("--version").output() {
                    if output.status.success() {
                        send_to_frontend(&app_handle, "dnf package manager detected.".to_string(), "initialize_setup_processing");
            ("dnf", vec!["install", "-y", "fluidsynth"])
                    } else {
                        send_to_frontend(&app_handle, "dnf check failed, trying pacman...".to_string(), "initialize_setup_processing");
                        if let Ok(output) = StdCommand::new("pacman").arg("-V").output() {
                            if output.status.success() {
                                send_to_frontend(&app_handle, "pacman package manager detected.".to_string(), "initialize_setup_processing");
            ("pacman", vec!["-S", "--noconfirm", "fluidsynth"])
        } else {
                                send_to_frontend(&app_handle, "No supported package manager found.".to_string(), "initialize_setup_color");
                                return Err("Unable to detect package manager. Please install FluidSynth manually.".to_string());
                            }
                        } else {
                            send_to_frontend(&app_handle, "Error checking for pacman.".to_string(), "initialize_setup_color");
                            return Err("Unable to detect package manager. Please install FluidSynth manually.".to_string());
                        }
                    }
                } else {
                    send_to_frontend(&app_handle, "Error checking for dnf.".to_string(), "initialize_setup_color");
                    return Err("Unable to detect package manager. Please install FluidSynth manually.".to_string());
                }
            }
        } else {
            send_to_frontend(&app_handle, "Error checking for apt.".to_string(), "initialize_setup_color");
            return Err("Unable to detect package manager. Please install FluidSynth manually.".to_string());
        }
    } else {
        send_to_frontend(&app_handle, "Unsupported operating system detected.".to_string(), "initialize_setup_color");
        return Err("Unsupported operating system".to_string());
    };

    send_to_frontend(
        &app_handle,
        format!("Installing FluidSynth using {} with args: {:?}", cmd_path, cmd_args),
        "initialize_setup_processing",
    );

    // Check if we're already elevated
    send_to_frontend(&app_handle, "Checking if process is running with elevated privileges...".to_string(), "initialize_setup_processing");
    let is_elevated = Command::is_elevated();
    send_to_frontend(
        &app_handle,
        format!("Process elevation status: {}", if is_elevated { "Elevated" } else { "Not elevated" }),
        "initialize_setup_processing",
    );

    let output = if is_elevated {
        // If already elevated, run the command directly
        send_to_frontend(&app_handle, "Already running with elevated privileges. Executing command directly...".to_string(), "initialize_setup_processing");
        let mut cmd = StdCommand::new(cmd_path);
        cmd.args(&cmd_args);
        send_to_frontend(&app_handle, format!("Executing command: {} {:?}", cmd_path, cmd_args), "initialize_setup_processing");
        cmd.output().map_err(|e| {
            send_to_frontend(&app_handle, format!("Error executing command directly: {}", e), "initialize_setup_color");
            e.to_string()
        })
    } else {
        // If not elevated, use elevated-command to request elevation
        send_to_frontend(&app_handle, "Not running with elevated privileges. Requesting elevation...".to_string(), "initialize_setup_processing");
        let mut cmd = StdCommand::new(cmd_path);
        cmd.args(&cmd_args);
        let elevated_cmd = Command::new(cmd);
        send_to_frontend(&app_handle, "Executing command with elevated privileges...".to_string(), "initialize_setup_processing");
        elevated_cmd.output().map_err(|e| {
            send_to_frontend(&app_handle, format!("Error executing elevated command: {}", e), "initialize_setup_color");
            e.to_string()
        })
    };

    // Process the result
    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            send_to_frontend(
                &app_handle,
                "Installation successful".to_string(),
                "installed_fluidsynth",
            );
            Ok(format!("Installation successful: {}", stdout))
        }
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            send_to_frontend(
                &app_handle,
                format!("Installation failed with status: {}", result.status),
                "initialize_setup_color",
            );
            send_to_frontend(
                &app_handle,
                format!("Installation stdout: {}", stdout),
                "initialize_setup_processing",
            );
            send_to_frontend(
                &app_handle,
                format!("Installation stderr: {}", stderr),
                "initialize_setup_color",
            );
            Err(format!("Installation failed: {}", stderr))
        }
        Err(err) => {
            send_to_frontend(&app_handle, format!("Error executing command: {}", err), "initialize_setup_color");
            Err(format!("Error executing command: {}", err))
        }
    }
}

fn is_fluidsynth_installed() -> bool {
    // Implementation depends on OS
    if cfg!(target_os = "windows") {
        // Check if FluidSynth is in PATH
        match StdCommand::new("fluidsynth").arg("--version").output() {
            Ok(output) => output.status.success(),
            Err(_) => false
        }
    } else if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        // Check using which command
        match StdCommand::new("which").arg("fluidsynth").output() {
            Ok(output) => output.status.success(),
            Err(_) => false
        }
    } else {
        false
    }
}







// use elevated_command::Command;
// use std::process::Command as StdCommand;
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
//         send_to_frontend(&app_handle, "Detected WSL, Please enter the password to install dependencies".to_string(), "initialize_setup_processing");
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