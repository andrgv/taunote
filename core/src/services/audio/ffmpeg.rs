use anyhow::{anyhow, Result};
use std::{path::Path, process::Command, fs::{OpenOptions, create_dir_all}};

// run ffmpeg CLI to normalize and trim silence
pub fn preprocess_audio(input: &Path, output: &Path) -> Result<()> {
    // make sure the output file atually exists
    if let Some(parent) = output.parent() {
        create_dir_all(parent)?;
    }
    let output_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(output)?;

    let status = Command::new("ffmpeg")
        .arg("-y") // overwrite
        .arg("-i")
        .arg(input.as_os_str())
        .arg("-af")
        .arg("loudnorm=I=-16:TP=-1.5,silenceremove=start_periods=1:start_threshold=-50dB")
        .arg("-ar")
        .arg("16000")
        .arg("-ac")
        .arg("1")
        .arg("-sample_fmt")
        .arg("s16")
        .arg(output.as_os_str())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "ffmpeg failed with exit status: {:?}",
            status.code()
        ))
    }
}
