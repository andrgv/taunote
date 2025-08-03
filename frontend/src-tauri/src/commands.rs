use std::path::{Path, PathBuf};
use directories_next;
use rusqlite;

use taunote_core::services::{
    audio::ffmpeg::preprocess_audio,
    database::{
        queries::{
            insert_audio_project,
            insert_project_notes
        },
        models::AudioProject,
        schema::init_db
    },
    llm::{
        llama_queue::{init_llama_queue},
        prompt_tasks::{
            summarize, 
            generate_email, 
            generate_lecture_notes
        }
    },
    transcribe::whisperx::run_whisperx
};

#[tauri::command]
pub async fn summarize_project(
    transcript_path: String
) -> Result<String, String> {
    let path = PathBuf::from(transcript_path);
    summarize(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn project_email(
    transcript_path: String
) -> Result<String, String> {
    let path = PathBuf::from(transcript_path);
    generate_email(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn project_lecture_notes(
    transcript_path: String
) -> Result<String, String> {
    let path = PathBuf::from(transcript_path);
    generate_lecture_notes(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn transcribe_audio(
    audio_path: String,
    lang: String
) -> Result<String, String> {
    let path = PathBuf::from(audio_path);
    let tmp_preprocessed_audio_path_name = "../tmp/preprocessed.wav";
    let tmp_preprocessed_audio_path = Path::new(tmp_preprocessed_audio_path_name);
    let tmp_transcript_path_name = "../tmp/transcript.txt";
    let tmp_transcript_path = Path::new(tmp_transcript_path_name);
    
    preprocess_audio(&path, 
        tmp_preprocessed_audio_path
    ).map_err(|e| e.to_string())?;

    run_whisperx(tmp_preprocessed_audio_path, 
        &Some(tmp_transcript_path.to_path_buf()), 
        &Some(lang)
    ).map_err(|e| e.to_string())?;
    
    Ok(tmp_transcript_path_name.to_string())
}

#[tauri::command]
pub async fn setup_backend() -> Result<(), String> {
    // start + open db
    let proj_dirs = directories_next::ProjectDirs::from("com", "andrea", "taunote")
        .expect("Failed to find platform data directory");
    let base_path = proj_dirs.data_local_dir();
    init_db(&base_path).map_err(|e| e.to_string())?;
    
    // start llama queue
    // TODO: might want to actually make it return errors @sp
    init_llama_queue();

    Ok(())
}

#[tauri::command]
pub fn insert_audio_project_to_db(
    audio_project: AudioProject
) -> Result<(), String> {
    let proj_dirs = directories_next::ProjectDirs::from("com", "andrea", "taunote")
        .expect("Failed to find platform data directory");
    let base_path = proj_dirs.data_local_dir();
    let db_path = base_path.join("db").join("project.db");
    let conn = rusqlite::Connection::open(db_path)?;
    insert_audio_project(&conn, &audio_project)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn insert_project_notes_to_db(
    project_id: String,
    transcript: String,
    summary: String,
    email: String
) -> Result<(), String> {
    let proj_dirs = directories_next::ProjectDirs::from("com", "andrea", "taunote")
        .expect("Failed to find platform data directory");
    let base_path = proj_dirs.data_local_dir();
    let db_path = base_path.join("db").join("project.db");
    let conn = rusqlite::Connection::open(db_path)?;
    insert_project_notes(&conn, &project_id, &transcript, &summary, &email)
        .map_err(|e| e.to_string())?;

    Ok(())
}