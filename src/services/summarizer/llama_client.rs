use std::path::{Path, PathBuf};
use std::error::Error;
use std::{fs, thread};
use std::process::{Command, Child};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Serialize)]
struct CompletionRequest {
    prompt: String,
    stream: bool,
    temperature: f32,
    n_predict: u32,
}

#[derive(Deserialize)]
struct CompletionResponse {
    content: String
}

fn wait_for_server(host: &str, port: u16, retries: u32, delay_ms: u64) -> bool {
    for _ in 0..retries {
        if TcpStream::connect((host, port)).is_ok() {
            return true;
        }
        thread::sleep(Duration::from_millis(delay_ms));
    }
    false
}

fn wait_for_model_ready(host: &str, port: u16, retries: u32, delay_ms: u64) -> bool {
    let client = reqwest::blocking::Client::new();
    let url = format!("http://{}:{}/completion", host, port);

    let dummy = serde_json::json!({
        "prompt": "ping",
        "stream": false,
        "temperature": 0.0,
        "n_predict": 1
    });

    for _ in 0..retries {
        if let Ok(resp) = client.post(&url).json(&dummy).send() {
            if resp.status().is_success() {
                return true;
            }
        }
        thread::sleep(Duration::from_millis(delay_ms));
    }
    false
}

fn spawn_llama_server(
    server_path: &PathBuf,
    model_path: &PathBuf
) -> Result<Child, Box<dyn Error>> {
    // cmd for new child
    // TODO: is it going to be fine with(out) nvidia
    let child = Command::new(server_path)
        .args(["--model", model_path.to_str().unwrap(),
               "--n-gpu-layers", "35",
               "--ctx-size", "4096"])
        .spawn()?;
    Ok(child)
}

pub async fn summarize(
    transcript_path: &PathBuf
) -> Result<String, Box<dyn Error>> {
    // Hardcoding paths for now, TODO extract later
    let server_path = PathBuf::from("/home/andrea/llama.cpp/build/bin/llama-server");
    let model_path = PathBuf::from("/home/andrea/taunote/models/llama/Meta-Llama-3.1-8B-Instruct-Q6_K_L.gguf");
    let server_addr = "http://127.0.0.1:8080/completion";
    
    // Extract transcript
    let transcript = fs::read_to_string(transcript_path)?;
    // TODO: figure out tokens and length limits
    let truncated_transcript = &transcript[..std::cmp::min(3000, transcript.len())];
    // Start server
    let mut server = spawn_llama_server(&server_path, &model_path)?;
    if !wait_for_server("127.0.0.1", 8080, 10, 500) {
        server.kill().ok();
        return Err("LLaMA server failed to start on port 8080".into());
    }
    if !wait_for_model_ready("127.0.0.1", 8080, 60, 500) {
        server.kill().ok();
        return Err("LLaMA model not ready after timeout".into());
    }

    let client = Client::new();
    let request_body = CompletionRequest {
        prompt: format!("Summarize the following transcript:\n{}", truncated_transcript),
        stream: false,
        temperature: 0.7,
        n_predict: 512
    };
    // Post request
    let response = client
        .post(server_addr)
        .json(&request_body)
        .send()
        .await?;

    // TODO: might change to a struct for structured metadata
    let raw = response.text().await?;
    let json: serde_json::Value = serde_json::from_str(&raw)?;

    if json.get("error").is_some() {
        server.kill().ok();
        return Err(format!("LLaMA server responded with error: {}", json).into());
    }

    let summary = json.get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("No summary found")
        .to_string();

    server.kill()?;
    Ok(summary)
}