use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::Mutex;
use crate::setup::EnvPaths;

pub struct AudioState {
    pub sink: Mutex<Option<Sink>>,
    pub stream_handle: OutputStreamHandle, // No longer wrapped in Mutex<Option>
}

pub fn initialize_audio() -> AudioState {
    // Create and keep the stream on the main thread
    let (stream, stream_handle) = OutputStream::try_default()
        .expect("Failed to create audio output stream");
    
    // Leak the stream to keep it alive indefinitely
    Box::leak(Box::new(stream));
    
    AudioState {
        sink: Mutex::new(None),
        stream_handle, // Directly store the handle
    }
}

#[tauri::command]
pub fn play_audio(state: tauri::State<AudioState>) -> Result<(), String> {
    let paths = EnvPaths::new();
    let file_path = paths.output_file;
    println!("Playing audio: {}", file_path.display());
    
    // Access the stream handle with explicit error checking
    let sink = Sink::try_new(&state.stream_handle)
        .map_err(|e| format!("Error creating sink: {}", e))?;
    
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
    
    // Keep sink alive by storing it in state
    println!("Storing sink in state...");
    match state.sink.lock() {
        Ok(mut sink_lock) => {
            *sink_lock = Some(sink);
            println!("Sink stored in state");
        },
        Err(e) => return Err(format!("Failed to lock sink state: {}", e)),
    }
    
    // Prevent immediate return to keep context alive
    std::thread::sleep(std::time::Duration::from_millis(100));
    println!("Play command completed");
    
    Ok(())
}

#[tauri::command]
pub fn pause_audio(state: tauri::State<AudioState>) -> Result<(), String> {
    let sink_lock = state.sink.lock().unwrap();
    
    if let Some(sink) = sink_lock.as_ref() {
        sink.pause();
        Ok(())
    } else {
        Err("No audio is currently playing".to_string())
    }
}

#[tauri::command]
pub fn stop_audio(state: tauri::State<AudioState>) -> Result<(), String> {
    let mut sink_lock = state.sink.lock().unwrap();
    
    if sink_lock.is_some() {
        // Clear the sink - this will stop playback
        *sink_lock = None;
        Ok(())
    } else {
        Err("No audio is currently playing".to_string())
    }
}