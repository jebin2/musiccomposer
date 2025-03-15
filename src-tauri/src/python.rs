use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::thread;
use tauri::AppHandle;
use crate::setup::EnvPaths;
use crate::utils::send_to_frontend;

lazy_static::lazy_static! {
	static ref PYTHON_PROCESS: Arc<Mutex<Option<std::process::Child>>> = Arc::new(Mutex::new(None));
	static ref SENT_RESULTS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

async fn handle_process_output(app: &AppHandle, reader: impl BufRead, event_type: &str) {
	let paths = EnvPaths::new();
	for line in reader.lines().flatten() {
		if line.contains("LogCoQ=1002") {
			send_to_frontend(app, line.to_string(), "initialize_setup_completed");
			send_to_frontend(app, paths.output_file.display().to_string(), "tune_file_created");
		} else if line.contains("LogCoQ=1003") {
			let msg = line.split("LogCoQ=1003").collect::<Vec<&str>>()[1];
			send_to_frontend(app, line.to_string(), "initialize_setup_completed");
			send_to_frontend(app, msg.to_string(), "error");
		} else if line.contains("LogCoQ=1001") {
			let msg = line.split("LogCoQ=1001").collect::<Vec<&str>>()[1];
			send_to_frontend(app, msg.to_string(), event_type);
		}
	}
}

pub async fn start(app: AppHandle, command: String) {
	let paths = EnvPaths::new();
	
	match Command::new(&paths.python)
		.arg("-u")
		.arg(&paths.main_py)
        .arg(command)
		.env("PYTHONUNBUFFERED", "1")
		.current_dir(&paths.temp_dir) // Set the working directory to paths.tempdir
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
	{
		Ok(mut child) => {
			let stdout = child.stdout.take().unwrap();
			let stderr = child.stderr.take().unwrap();
			
			// Store the process (including stdin)
			{
				let mut process_guard = PYTHON_PROCESS.lock().unwrap();
				*process_guard = Some(child); // Store the whole child process, stdin is now part of it.
			}

			// Handle stdout asynchronously
			let app_clone = app.clone();
			thread::spawn(move || {
				let reader = BufReader::new(stdout);
				tauri::async_runtime::block_on(handle_process_output(&app_clone, reader, "initialize_setup_processing"));
			});

			// Handle stderr asynchronously
			let app_clone = app.clone();
			thread::spawn(move || {
				let reader = BufReader::new(stderr);
				tauri::async_runtime::block_on(handle_process_output(&app_clone, reader, "initialize_setup_error"));
			});

			// No need for stdin keep-alive thread anymore. Stdin is accessed directly in send_command_to_python
		}
		Err(e) => {
			println!("Failed to start Python process: {}", e);
			send_to_frontend(&app, format!("Failed to start Python process: {}", e), "initialize_setup_error");
		}
	}
}

// Function to send JSON commands to Python process
pub async fn send_command(app: &AppHandle, command: &str) {
    // Check if process is running and try to send command
    let should_start_new_process = {
        let mut process_guard = PYTHON_PROCESS.lock().unwrap();
        
        if let Some(ref mut child) = *process_guard {
            if let Some(stdin) = child.stdin.as_mut() {
                if let Err(e) = writeln!(stdin, "{}", command) {
                    println!("Failed to write to Python process: {}", e);
                    // send_to_frontend(app, format!("Failed to write to Python process: {}", e), "error");
                    true // Start new process because write failed
                } else {
                    false // Command sent successfully, no need to start new process
                }
            } else {
                true // No stdin available, need to start new process
            }
        } else {
            true // No process, need to start new process
        }
    }; // MutexGuard dropped here before await
    
    // If needed, start a new process
    if should_start_new_process {
        println!("Python process is not running, starting...");
		start(app.clone(), command.to_string()).await; // Safe to await here as guard is dropped
    }
}

// Stop the Python process
pub async fn stop() {
	let mut process_guard = PYTHON_PROCESS.lock().unwrap();
	if let Some(mut child) = process_guard.take() {
		let _ = child.kill();
	}
}