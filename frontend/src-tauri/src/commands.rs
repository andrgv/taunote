use directories_next::ProjectDirs;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use taunote_core::services::{
    audio::ffmpeg::preprocess_audio,
    database::{
        models::AudioProject,
        queries::{insert_audio_project, insert_project_notes},
        schema::init_db,
    },
    llm::{
        llama_queue::init_llama_queue,
        prompt_tasks::{generate_email, generate_lecture_notes, summarize},
    },
    transcribe::whisperx::run_whisperx,
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectGroup {
    pub id: String,
    pub name: String,
    pub audioProjects: Vec<AudioProject>,
}

fn connect_to_db() -> Result<Connection, String> {
    let proj_dirs = ProjectDirs::from("com", "andrea", "taunote")
        .expect("Failed to find platform data directory");
    let base_path = proj_dirs.data_local_dir();
    let db_path = base_path.join("db").join("project.db");
    Connection::open(db_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_project_groups() -> std::result::Result<Vec<ProjectGroup>, String> {
    // Find db
    let conn = connect_to_db()?;

    // Load project groups
    let mut grp_stmt = conn
        .prepare("SELECT id, name FROM project_groups")
        .map_err(|e| e.to_string())?;
    let group_iter = grp_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    // Collect into vector group with empty audioProjects
    let mut groups = group_iter
        .map(|res| {
            res.map(|(id, name)| ProjectGroup {
                id,
                name,
                audioProjects: Vec::new(),
            })
        })
        .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    // for each group, load its audioProjects
    for group in &mut groups {
        let mut ap_stmt = conn
            .prepare(
                "SELECT id, group_id, name, relative_path, date, type, language
             FROM audio_projects WHERE group_id = ?1",
            )
            .map_err(|e| e.to_string())?;

        let audio_iter = ap_stmt
            .query_map(params![&group.id], |r| {
                Ok(AudioProject {
                    id: r.get(0)?,
                    group_id: r.get(1)?,
                    name: r.get(2)?,
                    relative_path: r.get(3)?,
                    date: r.get(4)?,
                    project_type: r.get(5)?,
                    language: r.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;

        group.audioProjects = audio_iter
            .map(|res| res.map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
    }

    Ok(groups)
}

#[tauri::command]
pub async fn summarize_project(transcript_path: String) -> Result<String, String> {
    let path = PathBuf::from(transcript_path);
    summarize(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn project_email(transcript_path: String) -> Result<String, String> {
    let path = PathBuf::from(transcript_path);
    generate_email(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn project_lecture_notes(transcript_path: String) -> Result<String, String> {
    let path = PathBuf::from(transcript_path);
    generate_lecture_notes(&path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn transcribe_audio(audio_path: String, lang: String) -> Result<String, String> {
    let path = PathBuf::from(audio_path);
    let tmp_preprocessed_audio_path_name = "../tmp/preprocessed.wav";
    let tmp_preprocessed_audio_path = Path::new(tmp_preprocessed_audio_path_name);
    let tmp_transcript_path_name = "../tmp/transcript.txt";
    let tmp_transcript_path = Path::new(tmp_transcript_path_name);

    preprocess_audio(&path, tmp_preprocessed_audio_path).map_err(|e| e.to_string())?;

    let lang_input = if (lang == "auto") { None } else { Some(lang) };

    run_whisperx(
        tmp_preprocessed_audio_path,
        &Some(tmp_transcript_path.to_path_buf()),
        &lang_input,
    )
    .map_err(|e| e.to_string())?;

    Ok(tmp_transcript_path_name.to_string())
}

#[tauri::command]
pub async fn setup_backend() -> Result<(), String> {
    // start + open db
    let proj_dirs = directories_next::ProjectDirs::from("com", "andrea", "taunote")
        .expect("Failed to find platform data directory");
    let base_path = proj_dirs.data_local_dir();
    init_db(base_path).map_err(|e| e.to_string())?;

    // start llama queue
    // TODO: might want to actually make it return errors @sp
    init_llama_queue().await;

    Ok(())
}

#[tauri::command]
pub fn insert_project_group_to_db(id: String, name: String) -> Result<(), String> {
    let conn = connect_to_db()?;
    conn.execute(
        "INSERT INTO project_groups (id, name) VALUES (?1, ?2)",
        params![id, name],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn insert_audio_project_to_db(audio_project: AudioProject) -> Result<(), String> {
    let conn = connect_to_db()?;
    insert_audio_project(&conn, &audio_project).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn insert_project_notes_to_db(
    project_id: String,
    transcript: String,
    summary: String,
    email: String,
) -> Result<(), String> {
    let conn = connect_to_db()?;
    insert_project_notes(&conn, &project_id, &transcript, &summary, &email)
        .map_err(|e| e.to_string())?;

    Ok(())
}
