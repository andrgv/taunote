use directories_next::ProjectDirs;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

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

fn write_to_file(text: &str, dir_path: &str, filename: &str) -> Result<(), String> {
    let project_folder = PathBuf::from(dir_path);
    fs::create_dir_all(&project_folder)
        .map_err(|e| format!("Could not create directory {}: {}", dir_path, e))?;
    fs::write(project_folder.join(filename), text)
        .map_err(|e| format!("Could not write to file {}: {}", filename, e))?;
    Ok(())
}

// TODO: rethink what each of these 3 functions is going to do differently
// If not, condense with hof or match case
#[tauri::command]
pub async fn summarize_transcript(transcript_path: String) -> Result<(String, String), String> {
    let path = PathBuf::from(transcript_path);
    let summary = summarize(&path).await
        .map_err(|e| e.to_string())?;
    let filename = "transcript.md";
    let dir_path = path
        .parent().ok_or("How tf is there no parent dir?")?
        .to_str().ok_or("Could not convert path to string")?;
    write_to_file(summary.as_str(), dir_path, &filename)?;
    Ok((filename.to_string(), summary))
}

#[tauri::command]
pub async fn write_email(transcript_path: String) -> Result<(String, String), String> {
    let path = PathBuf::from(transcript_path);
    let text = generate_email(&path).await
        .map_err(|e| e.to_string())?;
    let filename = "email.md";
    let dir_path = path
        .parent().ok_or("How tf is there no parent dir?")?
        .to_str().ok_or("Could not convert path to string")?;
    write_to_file(text.as_str(), dir_path, &filename)?;
    Ok((filename.to_string(), text))
}

#[tauri::command]
pub async fn write_lecture_notes(transcript_path: String) -> Result<(String, String), String> {
    let path = PathBuf::from(transcript_path);
    let text = generate_lecture_notes(&path)
        .await.map_err(|e| e.to_string())?;
    let filename = "lecture_notes.md";
    let dir_path = path
        .parent().ok_or("How tf is there no parent dir?")?
        .to_str().ok_or("Could not convert path to string")?;
    write_to_file(text.as_str(), dir_path, &filename)?;
    Ok((filename.to_string(), text))
}

#[tauri::command]
pub async fn transcribe_audio(
    audio_path: String,
    lang: String,
    group_name: String,
    project_name: String
) -> Result<(String, String), String> {
    let path = PathBuf::from(audio_path);
    let tmp_preprocessed_audio_path_name = "../tmp/preprocessed.wav";
    let tmp_preprocessed_audio_path = Path::new(tmp_preprocessed_audio_path_name);

    preprocess_audio(&path, tmp_preprocessed_audio_path).map_err(|e| e.to_string())?;

    let proj_dirs = directories_next::ProjectDirs::from("com", "andrea", "taunote")
        .expect("Failed to find platform data directory");
    let base_path = proj_dirs.data_local_dir();
    let relative_path = format!("groups/{group_name}/{project_name}");
    let project_folder = base_path.join(&relative_path);
    fs::create_dir_all(&project_folder)
        .map_err(|e| format!("Unable to create directory: {}", e))?;

    let filename = project_folder.join("transcript.md");

    let lang_input = if lang.eq_ignore_ascii_case("auto") { None } else { Some(lang) };

    run_whisperx(
        tmp_preprocessed_audio_path,
        &Some(filename.clone()),
        &lang_input,
    )
    .map_err(|e| e.to_string())?;

    let transcript = String::from_utf8(
        fs::read(&filename).map_err(|e| format!("Could not read from file: {}", e))?
    ).map_err(|e| format!("Could not convert to utf8 string: {}", e))?;

    let filename_string = filename.to_string_lossy().into_owned();

    Ok((filename_string, transcript))
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
pub fn insert_audio_project_to_db(
    // TODO: this is absolutely awful and it makes my eyes bleed
    // but yeah... quick fix?
    audio_project: Option<AudioProject>,
    audioProject: Option<AudioProject>,
) -> Result<(), String> {
    let conn = connect_to_db()?;

    let ap = audio_project
        .or(audioProject)
        .ok_or_else(|| "missing arg: audio_project / audioProject".to_string())?;

    insert_audio_project(&conn, &ap).map_err(|e| e.to_string())?;
    Ok(())
}


#[tauri::command]
pub fn insert_project_notes_to_db(
    project_id: String,
    transcript: String,
    summary: String,
    email: String,
    lecture_notes: String
) -> Result<(), String> {
    let conn = connect_to_db()?;
    insert_project_notes(&conn, &project_id, &transcript, &summary, &email, &lecture_notes)
        .map_err(|e| e.to_string())?;

    Ok(())
}