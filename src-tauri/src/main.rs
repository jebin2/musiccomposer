// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod setup;
mod python;
mod utils;
mod fluidsynth_config;
mod tune_processor;
mod config;
mod audio_player;
use audio_player::initialize_audio;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .manage(initialize_audio())
        .invoke_handler(tauri::generate_handler![
            setup::initialize_setup,
            tune_processor::generate_tunes,
            config::save_config,
            config::load_config,
            audio_player::play_audio,
            audio_player::pause_audio,
            audio_player::stop_audio,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
