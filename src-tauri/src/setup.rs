use std::env;
use std::fs;
use std::process::Command;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::utils::{send_to_frontend, execute_command};
use crate::fluidsynth_config::install_fluidsynth;

const APP_TEMP_DIR: &str = "musiccomposer";
const VENV_DIR: &str = "venv";
const CONFIG_FILE: &str = "config.json";
const MAIN_PY: &str = "main.py";
const ENV: &str = ".env";
// const SOUNDFONT: &str = "FluidR3_GM.sf2";
const OUTPUT_FILE: &str = "output.wav";

pub struct EnvPaths {
    pub python: PathBuf,
    pub config: PathBuf,
    pub temp_dir: PathBuf,
    pub main_py: PathBuf,
    pub env: PathBuf,
    // pub soundfont: PathBuf,
    pub output_file: PathBuf,
}

impl EnvPaths {
    pub fn new() -> Self {
        let temp_dir = env::temp_dir().join(APP_TEMP_DIR);
        fs::create_dir_all(&temp_dir).expect("Failed to create application directory");

        let python = if cfg!(target_os = "windows") {
            temp_dir.join(VENV_DIR).join("Scripts").join("python.exe")
        } else {
            temp_dir.join(VENV_DIR).join("bin").join("python")
        };

        let config = temp_dir.join(CONFIG_FILE);
        let main_py = temp_dir.join(MAIN_PY);
        let env = temp_dir.join(ENV);
        // let soundfont = temp_dir.join(SOUNDFONT);
        let output_file = temp_dir.join(OUTPUT_FILE);

        Self {
            python,
            config,
            temp_dir,
            main_py,
            env,
            // soundfont,
            output_file
        }
    }
}

fn get_resource_path(app: &AppHandle, resource_type: &str) -> PathBuf {
    let base = "bin/dependency/";
    let resource_path = match resource_type {
        "python" => format!("{base}{VENV_DIR}"),
        "config" => format!("{base}{CONFIG_FILE}"),
        "main_py" => format!("{base}{MAIN_PY}"),
        "env" => format!("{base}{ENV}"),
        "fluidsynth" => format!("{base}fluidsynth.exe"),
        _ => {
            send_to_frontend(app, format!("Unsupported resource type: {}", resource_type), "error");
            return PathBuf::new();
        }
    };

    match app.path().resolve(resource_path, tauri::path::BaseDirectory::Resource) {
        Ok(path) => {
            if !path.exists() {
                send_to_frontend(app, format!("Resource not found at {:?}", path), "error");
                PathBuf::new()
            } else {
                path
            }
        }
        Err(e) => {
            send_to_frontend(app, format!("Failed to resolve path: {}", e), "error");
            PathBuf::new()
        }
    }
}

async fn setup_python(app: &AppHandle, paths: &EnvPaths) -> Result<String, String> {
    if paths.python.exists() {
        send_to_frontend(app, format!("Virtual environment already exists at {:?}", paths.python), "initialize_setup_processing");
        return Ok("already installed.".to_string());
    }

    copy_resource(&app, &get_resource_path(app, "python"), &paths.temp_dir).await
}

async fn install_pip_package(app: &AppHandle, paths: &EnvPaths, package_name: &str, git_path: &str) -> Result<(), String> {
    let check_installed = Command::new(&paths.python)
        .args(&["-m", "pip", "show", package_name])
        .output();

    if let Ok(output) = check_installed {
        if output.status.success() {
            send_to_frontend(app, "Music Composer is already installed".to_string(), "initialize_setup_processing");
            return Ok(()); // Return early if it's installed
        }
    }

    let mut command = Command::new(&paths.python);
    command.args(&["-m", "pip", "install", git_path]);

    match execute_command(app, &mut command, format!("install_pip_package: {}",package_name)) {
        Ok(mut child) => match child.wait() {
            Ok(exit_status) if exit_status.success() => {
                send_to_frontend(app, "Pip Package installed successfully".to_string(), "initialize_setup_processing");
                Ok(())
            }
            Ok(exit_status) => {
                let error_msg = format!("Installation failed with status: {}", exit_status);
                send_to_frontend(app, error_msg.clone(), "initialize_setup_error");
                Err(error_msg)
            }
            Err(e) => {
                let error_msg = format!("Failed to wait for installation process: {}", e);
                send_to_frontend(app, error_msg.clone(), "initialize_setup_error");
                Err(error_msg)
            }
        },
        Err(e) => {
            let error_msg = format!("Failed to start installation process: {}", e);
            send_to_frontend(app, error_msg.clone(), "initialize_setup_error");
            Err(error_msg)
        }
    }
}

async fn copy_resource(app: &AppHandle, source: &PathBuf, destination: &PathBuf) -> Result<String, String> {
    let mut command = if cfg!(target_os = "windows") {
        let mut cmd = Command::new("xcopy");
        cmd.args(&[source.to_str().unwrap(), destination.to_str().unwrap(), "/E", "/I"]);
        cmd
    } else {
        let mut cmd = Command::new("cp");
        cmd.args(&["-r", source.to_str().unwrap(), destination.to_str().unwrap()]);
        cmd
    };

    match execute_command(app, &mut command, "copy_resource".to_string()) {
        Ok(mut child) => {
            match child.wait() {
                Ok(exit_status) if exit_status.success() => {
                    send_to_frontend(app, "Copy completed successfully.".to_string(), "initialize_setup_processing");
                    Ok("Done.".to_string())
                }
                Ok(exit_status) => {
                    let error_msg = format!("Copy failed with status: {} {}", exit_status, source.display());
                    send_to_frontend(app, error_msg.clone(), "initialize_setup_error");
                    Err(error_msg)
                }
                Err(e) => {
                    let error_msg = format!("Failed to wait for copy process: {}", e);
                    send_to_frontend(app, error_msg.clone(), "initialize_setup_error");
                    Err(error_msg)
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to start copy process: {}", e);
            send_to_frontend(app, error_msg.clone(), "error");
            Err(error_msg)
        }
    }
}

async fn setup_config(app: &AppHandle, paths: &EnvPaths) -> Result<String, String> {
    if let Err(e) = copy_resource(app, &get_resource_path(app, "config"), &paths.config).await {
        return Err(e);
    }

    if let Err(e) = copy_resource(app, &get_resource_path(app, "main_py"), &paths.main_py).await {
        return Err(e);
    }

    copy_resource(app, &get_resource_path(app, "env"), &paths.env).await
}

#[tauri::command]
pub async fn initialize_setup(app: AppHandle) {
    let paths = EnvPaths::new();

    if let Ok(message) = setup_python(&app, &paths).await {
        if message == "already installed.".to_string() {
            send_to_frontend(&app, "Python already installed".to_string(), "initialize_setup_completed");
            return;
        }
    } else if let Err(e) = setup_python(&app, &paths).await {
        send_to_frontend(&app, format!("Failed to setup python: {}", e), "initialize_setup_error");
        return;
    }
    
    if let Err(e) = install_pip_package(&app, &paths, "music_composer", "git+https://github.com/jebin2/music_composer.git").await {
        send_to_frontend(&app, format!("Failed to install dependencies: {}", e), "initialize_setup_error");
        return;
    }
    
    if let Err(e) = setup_config(&app, &paths).await {
        send_to_frontend(&app, format!("Failed to setup config: {}", e), "initialize_setup_error");
        return;
    }

    if let Err(e) = install_fluidsynth(app.clone()).await {
        send_to_frontend(&app, format!("Failed to install FluidSynth: {}", e), "initialize_setup_error");
        return;
    }

    // Only send this if all previous steps succeeded
    send_to_frontend(&app, "All Setup Initialized".to_string(), "initialize_setup_completed");
}