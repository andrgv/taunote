use clap::{error::Result, Parser};
use std::path::Path;
use shellexpand;

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

pub fn run_whisper(
    input_path: &Path,
    model: &Path,
    lang: &str,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = shellexpand::tilde(output_path.to_str().unwrap()).to_string();
    let output_path = Path::new(&output_path);
    std::fs::create_dir_all(output_path)?;

    let output_filename_path = output_path.join("transcript");
    let output_filename_string = output_filename_path
        .to_str()
        .ok_or("Invalid output path")?;

    match std::process::Command::new("./whisper-main")
        .arg("-m").arg(model)
        .arg("-f").arg(input_path)
        .arg("-l").arg(lang)
        .arg("-otxt")
        .arg("-of").arg(&output_filename_string)
        .status()
    {
        Ok(status) if status.success() => {}
        Ok(_) => return Err("whisper-cli exited with error".into()),
        Err(e) => return Err(format!("Failed running whisper-cli: {e}").into()),
    }

    std::fs::read_to_string(output_filename_path.with_extension("txt"))?;

    println!("Transcript written to: {}.txt", output_filename_string);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let model = std::path::Path::new("models/ggml-base.bin");
    run_whisper(&args.input_path, &model, &args.lang, &args.output_path);
    Ok(())
}
