use clap::{error::Result, Parser};
use std::error::Error;

mod services;
// use services::transcribe::whisper::run_whisper;
use services::transcribe::whisperx::run_whisperx;
use services::audio::ffmpeg::preprocess_audio;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long)]
    input_path: std::path::PathBuf,
    #[arg(short, long)]
    lang: Option<String>,
    #[arg(short, long )]
    output_path: Option<std::path::PathBuf>
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    // let model = std::path::Path::new("models/ggml-base.bin");
    let tmp_path = std::path::Path::new("tmp/preprocessed.wav");
    preprocess_audio(&args.input_path, tmp_path)?;
    // run_whisper(tmp_path, &model, &args.lang, &args.output_path)?;
    run_whisperx(tmp_path, &args.output_path, &args.lang)?;
    // TODO: clean tmp/
    Ok(())
}
