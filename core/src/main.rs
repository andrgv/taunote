use clap::Parser;
use std::result::Result;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::fs;

mod services;
use services::transcribe::whisperx::run_whisperx;
use services::audio::ffmpeg::preprocess_audio;
use services::summarizer::llama_client::summarize;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long)]
    input_path: PathBuf,
    #[arg(short, long)]
    lang: Option<String>,
    #[arg(short, long)]
    output_path: Option<PathBuf>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    // let model = std::path::Path::new("models/ggml-base.bin");
    // TODO: fix the path mess
    let tmp_audio_path = Path::new("tmp/preprocessed.wav");
    let tmp_transcript_path = Path::new("tmp/transcript.txt");
    preprocess_audio(&args.input_path, tmp_audio_path)?;
    // run_whisper(tmp_path, &model, &args.lang, &args.output_path)?;
    run_whisperx(tmp_audio_path, &Some(tmp_transcript_path.to_path_buf()), &args.lang)?;

    let text = summarize(&tmp_transcript_path.to_path_buf()).await?;
    println!("Summary:\n{}", text);
    println!("Finished with pipeline!");
    // TODO: clean tmp/
    Ok(())
}
