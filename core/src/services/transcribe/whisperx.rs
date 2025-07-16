use std::process::Command;
use std::path::{Path, PathBuf};
use std::error::Error;

pub fn run_whisperx(
    input_path: &Path,
    output_path: &Option<PathBuf>,
    language: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::new("python3");
    cmd.arg("python_backend/whisperx_runner.py")
       .arg("--input").arg(input_path);

    if let Some(path) = output_path {
        cmd.arg("--output").arg(path);
    }

    if let Some(lang) = language {
        cmd.arg("--lang").arg(lang);
    }

    if !cmd.status()?.success() {
        return Err("WhisperX subprocess failed".into());
    }

    Ok(())
}
