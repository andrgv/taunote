use std::{path::Path, process::Command, error::Error};

/// run ffmpeg CLI to normalize and trim silence
pub fn preprocess_audio(input: &Path, output: &Path) -> Result<(), Box<dyn Error>> {
    let status = Command::new("ffmpeg")
        .arg("-y") // overwrite
        .arg("-i").arg(input.as_os_str())
        .arg("-af")
        .arg("loudnorm=I=-16:TP=-1.5,silenceremove=start_periods=1:start_threshold=-50dB")
        .arg("-ar").arg("48000")
        .arg("-ac").arg("1")
        .arg("-sample_fmt").arg("s16")
        .arg(output.as_os_str())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err("ffmpeg failed".into())
    }
}
