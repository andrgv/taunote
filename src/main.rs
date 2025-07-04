use clap::{error::Result, Parser};
use std::error::Error;

mod whisper;
use whisper::run_whisper;

mod audio;
use audio::preprocess_audio;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long)]
    input_path: std::path::PathBuf,
    #[arg(short, long, default_value = "en")]
    lang: String,
    #[arg(short, long, default_value = "~/transcriptions/")]
    output_path: std::path::PathBuf
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let model = std::path::Path::new("models/ggml-base.bin");
    let tmp_path = std::path::Path::new("tmp/preprocessed.wav");
    preprocess_audio(&args.input_path, tmp_path)?;
    run_whisper(tmp_path, &model, &args.lang, &args.output_path)?;
    Ok(())
}
