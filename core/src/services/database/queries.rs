use crate::services::database::models::AudioProject;
use rusqlite::{params, Connection, Result};

pub fn insert_audio_project(conn: &Connection, project: &AudioProject) -> Result<()> {
    conn.execute(
        "INSERT INTO audio_projects (
            id, group_id, name, relative_path, date, type, language
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            project.id,
            project.group_id,
            project.name,
            project.relative_path,
            project.date,
            project.project_type,
            project.language
        ],
    )?;
    Ok(())
}

pub fn insert_project_notes(
    conn: &Connection,
    project_id: &str,
    transcript: &str,
    summary: &str,
    email: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO project_notes (
            project_id, transcript, summary, email
         ) VALUES (?1, ?2, ?3, ?4)",
        params![project_id, transcript, summary, email],
    )?;
    Ok(())
}
