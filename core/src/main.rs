use clap::Parser;
use uuid::Uuid;
// use std::result::Result;
// use std::error::Error;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;

pub mod services;
use services::transcribe::whisperx::run_whisperx;
use services::audio::ffmpeg::preprocess_audio;
use services::llm::llama_queue::init_llama_queue;
use services::llm::prompt_tasks::{summarize, generate_email};
use services::database::schema::init_db;
use services::database::models::AudioProject;
use services::database::queries::{insert_audio_project, insert_project_notes};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long)]
    input_path: PathBuf,
    #[arg(short, long)]
    lang: Option<String>,
    #[arg(short, long)]
    output_path: Option<PathBuf>,
    #[arg(short = 'g', long = "group", default_value = "default")]
    group_name: String,
    #[arg(short = 'n', long = "name")]
    project_name: Option<String>
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    // Finding local directory paths to initialize the database, matches src-tauri identifier
    let proj_dirs = directories_next::ProjectDirs::from("com", "andrea", "taunote")
        .expect("Failed to find platform data directory");
    let base_path = proj_dirs.data_local_dir();
    init_db(&base_path)?;

    init_llama_queue().await;
    
    // Open connection to db
    let db_path = base_path.join("db").join("project.db");
    let conn = rusqlite::Connection::open(db_path)?;

    // TODO (not priority rn): fix the path mess
    let tmp_audio_path = Path::new("../tmp/preprocessed.wav");
    let tmp_transcript_path = Path::new("../tmp/transcript.txt");
    preprocess_audio(&args.input_path, tmp_audio_path)?;
    run_whisperx(tmp_audio_path, &Some(tmp_transcript_path.to_path_buf()), &args.lang)?;
    println!("Generated transcript!");

    // Texts to store
    let transcript = fs::read_to_string(&tmp_transcript_path)?;
    println!("{transcript}");
    let summary = summarize(&tmp_transcript_path.to_path_buf()).await?;
    println!("Generated summary!");
    println!("{}", summary);
    let email = generate_email(&tmp_transcript_path.to_path_buf()).await?;
    println!("Generated email!");
    println!("{}", email);
    
    // Project metadata
    let project_id = Uuid::new_v4().to_string();
    let project_name = args.project_name.clone().unwrap_or_else(|| {
        args.input_path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string()
    });
    let group_name = args.group_name.clone();
    let relative_path = format!("groups/{}/{}", group_name, project_name);
    let date = chrono::Utc::now().to_rfc3339();
    let language = args.lang.clone().unwrap_or_else(|| "Auto".to_string());

    // Create project folder and write texts to .md
    let project_folder = base_path.join(&relative_path);
    fs::create_dir_all(&project_folder)?;
    fs::write(project_folder.join("transcript.md"), &transcript)?;
    fs::write(project_folder.join("summary.md"), &summary)?;
    fs::write(project_folder.join("email.md"), &email)?;
    println!("Finished writing markdown files!");

    // Insert to the database
    let audio_project = AudioProject {
        id: project_id.clone(),
        group_id: group_name.clone(),
        name: project_name.clone(),
        relative_path: relative_path.clone(),
        date,
        project_type: "meeting".to_string(),
        language,
    };
    insert_audio_project(&conn, &audio_project)?;
    insert_project_notes(&conn, &project_id, &transcript, &summary, &email)?;
    // TODO: clean tmp/
    println!("Finished!");
    Ok(())
}
