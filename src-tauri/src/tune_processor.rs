use tauri::{AppHandle, Manager};
use crate::python::send_command;
use serde_json::json;
use crate::setup::EnvPaths;

#[tauri::command]
pub async fn generate_tunes(app: AppHandle, text: String) {
	let paths = EnvPaths::new();
	let json_data = json!({ "text":text, "soundfont": "" });
	send_command(&app, &json_data.to_string()).await;
}