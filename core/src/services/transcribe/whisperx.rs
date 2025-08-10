use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

fn path_to_model() -> Result<PathBuf> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("python_backend")
        .join("whisperx_runner.py");

    if p.exists() {
        return Ok(p.canonicalize()?);
    }

    // fallback for when I have the binary
    let bin_dir = std::env::current_exe()?
        .parent()
        .ok_or_else(|| anyhow!("no parent directory for current_exe"))?
        .to_path_buf();
    for f in [
        bin_dir.join("../python_backend/whisperx_runner.py"),
        bin_dir.join("../../python_backend/whisperx_runner.py")
    ] {
        if f.exists() {
            return Ok(f.canonicalize()?);
        }
    }

    Err(anyhow!("Unbale to find whisperx_runner.py"))
}

pub fn run_whisperx(
    input_path: &Path,
    output_path: &Option<PathBuf>,
    language: &Option<String>,
) -> Result<()> {
    let whisperx_model = path_to_model()?;
    let mut cmd = Command::new("python3");
    cmd.arg(&whisperx_model)
        .arg("--input")
        .arg(input_path);

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
