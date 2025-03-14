use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use crate::setup::EnvPaths;
use crate::utils::send_to_frontend;
use tauri::AppHandle;

pub struct AudioState {
    pub sink: Arc<Mutex<Option<Arc<Sink>>>>,
    pub stream_handle: OutputStreamHandle,
}

pub fn initialize_audio() -> AudioState {
    // Create and keep the stream on the main thread
    let (stream, stream_handle) = OutputStream::try_default()
        .expect("Failed to create audio output stream");
    
    // Leak the stream to keep it alive indefinitely
    Box::leak(Box::new(stream));
    
    AudioState {
        sink: Arc::new(Mutex::new(None)),
        stream_handle,
    }
}

#[tauri::command]
pub fn play_audio(app: AppHandle, state: tauri::State<AudioState>) -> Result<(), String> {
    let paths = EnvPaths::new();
    let file_path = paths.output_file;
    println!("Playing audio: {}", file_path.display());
    
    // Open and decode file with detailed logging
    let file = match File::open(&file_path) {
        Ok(f) => {
            println!("File opened successfully: {}", file_path.display());
            f
        },
        Err(e) => return Err(format!("Error opening file: {}", e)),
    };
    
    let source = match Decoder::new(BufReader::new(file)) {
        Ok(s) => {
            println!("Audio decoded successfully");
            s
        },
        Err(e) => return Err(format!("Error decoding audio: {}", e)),
    };
    
    // Create sink with explicit error handling
    println!("Creating sink...");
    let sink = match Sink::try_new(&state.stream_handle) {
        Ok(s) => {
            println!("Sink created successfully");
            s
        },
        Err(e) => return Err(format!("Error creating sink: {}", e)),
    };
    
    println!("Appending source to sink...");
    sink.append(source);
    println!("Source appended");
    
    println!("Starting playback...");
    sink.play();
    println!("Playback initiated");
    
    // Clone the shared state for the thread
    let sink_state = Arc::clone(&state.sink);
    
    // Store the sink in state before creating the monitoring thread
    println!("Storing sink in state...");
    match sink_state.lock() {
        Ok(mut sink_lock) => {
            *sink_lock = Some(Arc::new(sink));
            println!("Sink stored in state");
        },
        Err(e) => return Err(format!("Failed to lock sink state: {}", e)),
    }
    
    // Clone the app handle and state reference for the monitoring thread
    let app_handle = app.clone();
    let thread_sink_state = Arc::clone(&sink_state);
    
    // Create a thread to monitor when playback is finished
    std::thread::spawn(move || {
        // Get access to the sink for monitoring
        let maybe_sink: Option<Arc<Sink>> = {
            match thread_sink_state.lock() {
                Ok(sink_lock) => sink_lock.as_ref().map(|s| Arc::clone(s)),
                Err(_) => None,
            }
        };
        
        if let Some(monitor_sink) = maybe_sink {
            // Wait for the sink to finish
            monitor_sink.sleep_until_end();
            println!("Audio playback completed");
            
            // Send a message to the frontend
            send_to_frontend(&app_handle, "audio-playback-finished".to_string(), "play_finished");
        }
    });
    
    println!("Play command completed");
    Ok(())
}

#[tauri::command]
pub fn pause_audio(state: tauri::State<AudioState>) -> Result<(), String> {
    let sink_lock = state.sink.lock().map_err(|e| format!("Failed to lock sink: {}", e))?;
    
    if let Some(sink) = sink_lock.as_ref() {
        sink.pause();
        Ok(())
    } else {
        Err("No audio is currently playing".to_string())
    }
}

#[tauri::command]
pub fn stop_audio(state: tauri::State<AudioState>) -> Result<(), String> {
    let mut sink_lock = state.sink.lock().map_err(|e| format!("Failed to lock sink: {}", e))?;
    
    if sink_lock.is_some() {
        // Clear the sink - this will stop playback
        *sink_lock = None;
        Ok(())
    } else {
        Err("No audio is currently playing".to_string())
    }
}