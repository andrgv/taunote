
pub mod commands;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::summarize_project,
            commands::project_email,
            commands::project_lecture_notes,
            commands::transcribe_audio,
            commands::setup_backend,
            commands::get_project_groups,
            commands::insert_project_group_to_db,
            commands::insert_audio_project_to_db,
            commands::insert_project_notes_to_db
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
