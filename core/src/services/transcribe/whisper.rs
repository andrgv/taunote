use shellexpand;
use std::path::Path;

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
    let output_filename_string = output_filename_path.to_str().ok_or("Invalid output path")?;

    match std::process::Command::new("./whisper-main")
        .arg("-m")
        .arg(model)
        .arg("-f")
        .arg(input_path)
        .arg("-l")
        .arg(lang)
        .arg("-otxt")
        .arg("-of")
        .arg(output_filename_string)
        .status()
    {
        Ok(status) if status.success() => {}
        Ok(_) => return Err("whisper-cli exited with error".into()),
        Err(e) => return Err(format!("Failed running whisper-cli: {e}").into()),
    }

    std::fs::read_to_string(output_filename_path.with_extension("txt"))?;

    println!("Transcript written to: {output_filename_string}.txt");
    Ok(())
}
