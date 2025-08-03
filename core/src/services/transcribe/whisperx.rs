use std::process::Command;
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result};

pub fn run_whisperx(
    input_path: &Path,
    output_path: &Option<PathBuf>,
    language: &Option<String>,
) -> Result<()> {
    let mut cmd = Command::new("python3");
    cmd.arg("../python_backend/whisperx_runner.py")
       .arg("--input").arg(input_path);

    if let Some(path) = output_path {
        cmd.arg("--output").arg(path);
    }

    if let Some(lang) = language {
        cmd.arg("--lang").arg(lang);
    }

    let status = cmd.status()?;

    if !status.success() {
        return Err(anyhow!("WhisperX subprocess failed {:?}", status.code()));
    }

    Ok(())
}
