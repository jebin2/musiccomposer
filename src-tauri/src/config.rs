use std::fs;
use tauri::AppHandle;
use crate::setup::EnvPaths;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[tauri::command]
pub async fn save_config(app: AppHandle, api_key: String, system_prompt: String) -> Result<(), String> {
    let paths = EnvPaths::new();

    // Overwrite env file
    let env_string = format!("CUSTOM_LOGGER_PLAY_ERROR_SOUND=\"False\"\nGEMINI_API_KEYS=\"{api_key}\"\nOUTPUT_WAV=\"output.wav\"");
    fs::write(&paths.env, env_string).map_err(|e| e.to_string())?;

    // Overwrite config file with JSON
    let json_data = json!({ "api_key":api_key, "system_prompt": system_prompt });
    fs::write(&paths.config, json_data.to_string()).map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ConfigData {
    api_key: Option<String>,
}

#[tauri::command]
pub async fn load_config(key: String) -> Result<Option<String>, String> {
    let paths = EnvPaths::new();

    let config_data: ConfigData = if paths.config.exists() {
        let file_content = fs::read_to_string(&paths.config).map_err(|e| e.to_string())?;
        serde_json::from_str(&file_content).map_err(|e| e.to_string())?
    } else {
        return Ok(None);
    };
    println!("{:?}", config_data);
    match key.as_str() {
        "api_key" => Ok(config_data.api_key.clone()),
        _ => Err(format!("Unknown key: {}", key)),
    }
}
