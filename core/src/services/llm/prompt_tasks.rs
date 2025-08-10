use crate::services::llm::llama_queue::enqueue_completion;
use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;

fn truncate(text: &str, max_len: usize) -> &str {
    &text[..std::cmp::min(max_len, text.len())]
}

pub async fn summarize(path: &PathBuf) -> Result<String> {
    let text = fs::read_to_string(path).await?;
    let prompt = format!(
        "Summarize the following transcript:\n{}",
        truncate(&text, 3000)
    );
    enqueue_completion(prompt, 512).await
}

pub async fn generate_email(path: &PathBuf) -> Result<String> {
    let text = fs::read_to_string(path).await?;
    let prompt = format!(
        "Write a professional follow-up email based on this meeting:\n{}",
        truncate(&text, 3000)
    );
    enqueue_completion(prompt, 512).await
}

pub async fn generate_lecture_notes(path: &PathBuf) -> Result<String> {
    let text = fs::read_to_string(path).await?;
    let prompt = format!(
        "Write clear and concise lecture notes with bullet points and sections from this transcript:\n{}",
        truncate(&text, 3000)
    );
    enqueue_completion(prompt, 600).await
}
