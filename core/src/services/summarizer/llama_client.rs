use std::path::PathBuf;
use std::error::Error;
use std::{fs, process::Child, process::Command};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};
use tokio::net::TcpStream;
use reqwest::Client;
use sysinfo::System;
use std::ffi::OsStr;

#[derive(Serialize)]
struct CompletionRequest {
    prompt: String,
    stream: bool,
    temperature: f32,
    n_predict: u32,
}

#[derive(Deserialize)]
struct CompletionResponse {
    content: String,
}

fn kill_all_llama_servers() {
    let sys = System::new_all();
    for proc in sys.processes_by_name(OsStr::new("llama-server")) {
        let _ = proc.kill();
    }
}

// async wait for TCP port to accept connections
async fn wait_for_server(host: &str, port: u16, retries: u32, delay_ms: u64) -> bool {
    let addr = format!("{}:{}", host, port);
    for _ in 0..retries {
        if TcpStream::connect(&addr).await.is_ok() {
            return true;
        }
        sleep(Duration::from_millis(delay_ms)).await;
    }
    false
}

// async polling /completions endpoint until 200 resp
async fn wait_for_model_ready(
    host: &str,
    port: u16,
    retries: u32,
    delay_ms: u64,
    client: &Client,
) -> bool {
    let url = format!("http://{}:{}/completions", host, port);
    let dummy = serde_json::json!({
        "prompt": "ping",
        "stream": false,
        "temperature": 0.0,
        "n_predict": 1
    });

    for _ in 0..retries {
        if let Ok(resp) = client.post(&url).json(&dummy).send().await {
            if resp.status().is_success() {
                return true;
            }
        }
        sleep(Duration::from_millis(delay_ms)).await;
    }
    false
}

fn spawn_llama_server(
    server_path: &PathBuf,
    model_path: &PathBuf,
) -> Result<Child, Box<dyn Error>> {
    kill_all_llama_servers();
    let child = Command::new(server_path)
        .args([
            "--model",        model_path.to_str().unwrap(),
            "--port",         "8081",
            "--n-gpu-layers", "35",
            "--ctx-size",     "4096",
            "--no-warmup",
        ])
        .spawn()?;
    Ok(child)
}

pub async fn summarize(
    transcript_path: &PathBuf,
) -> Result<String, Box<dyn Error>> {
    // hardcode dedicated port
    // TODO: might need to change in future
    let port: u16 = 8081;
    let server_path = PathBuf::from("/home/andrea/llama.cpp/build/bin/llama-server");
    let model_path  = PathBuf::from("/home/andrea/taunote/models/llama/Meta-Llama-3.1-8B-Instruct-Q6_K_L.gguf");
    let server_addr = format!("http://127.0.0.1:{}/completions", port);

    // read+truncate transcript
    let transcript = fs::read_to_string(transcript_path)?;
    let truncated = &transcript[..std::cmp::min(3000, transcript.len())];

    // spawn server and wait
    let mut server = spawn_llama_server(&server_path, &model_path)?;
    if !wait_for_server("127.0.0.1", port, 10, 500).await {
        server.kill().ok();
        return Err("LLaMA server failed to start on port 8081".into());
    }

    // async client for ready+summarization
    let client = Client::new();

    if !wait_for_model_ready("127.0.0.1", port, 60, 500, &client).await {
        server.kill().ok();
        return Err("Model not ready after timeout".into());
    }

    // summary request
    let request_body = CompletionRequest {
        prompt: format!("Summarize the following transcript:\n{}", truncated),
        stream: false,
        temperature: 0.7,
        n_predict: 512,
    };

    let response = client
        .post(&server_addr)
        .json(&request_body)
        .send()
        .await?;

    let raw  = response.text().await?;
    let json: serde_json::Value = serde_json::from_str(&raw)?;
    if json.get("error").is_some() {
        server.kill().ok();
        return Err(format!("LLaMA server error: {}", json).into());
    }

    // extract summary and kill server
    let summary = json["content"]
        .as_str()
        .unwrap_or("No summary found")
        .to_string();

    server.kill()?;
    Ok(summary)
}
