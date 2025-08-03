use std::{
    path::PathBuf,
    process::{Child, Command},
    ffi::OsStr,
};
use tokio::{
    time::{sleep, Duration},
    net::TcpStream,
};
use reqwest::Client;
use sysinfo::System;
use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct LlamaClient {
    server: Child,
    client: Client,
    host: String,
    port: u16,
}

impl LlamaClient {
    pub async fn try_new(
        server_path: PathBuf, 
        model_path: PathBuf, 
        host: String, 
        port: u16,
    ) -> Result<Self> {
        let server = spawn_llama_server(&server_path, &model_path, port)?;
        wait_for_server(&host, port, None, None).await?;
        let client = Client::new();
        wait_for_model_ready(&host, port, &client, None, None).await?;
        
        Ok(Self { server, client, host, port })
    }

    pub async fn complete(
        &self, 
        prompt: String, 
        n_predict: u32,
    ) -> Result<String> {
        let url = format!("http://{}:{}/completions", self.host, self.port);
        let body = serde_json::json!({
            "prompt": prompt,
            "stream": false,
            "temperature": 0.7,
            "n_predict": n_predict,
        });

        let resp = self.client.post(&url).json(&body).send().await?;
        let raw = resp.text().await?;

        let json: serde_json::Value = serde_json::from_str(&raw)?;

        if let Some(err) = json.get("error") {
            Err(anyhow!("LLM error: {}", err))
        } else {
            Ok(json["content"].as_str().unwrap_or("").to_string())
        }
    }
}

impl Drop for LlamaClient {
    fn drop(&mut self) {
        let _ = self.server.kill();
    }
}

// Helper for default configuration for the LlamaClient
pub fn default_llama(port: u16) -> (PathBuf, PathBuf, String, u16) {
    (
        PathBuf::from("/home/andrea/llama.cpp/build/bin/llama-server"),
        PathBuf::from("/home/andrea/taunote/models/llama/Meta-Llama-3.1-8B-Instruct-Q6_K_L.gguf"),
        "127.0.0.1".to_string(),
        port,
    )
}

// Kills all currently running llama server processes
fn kill_all_llama_servers() {
    let sys = System::new_all();
    for proc in sys.processes_by_name(OsStr::new("llama-server")) {
        let _ = proc.kill();
    }
}

// Spawns the llama server subprocess
fn spawn_llama_server(
    server_path: &PathBuf, 
    model_path: &PathBuf, 
    port: u16,
) -> Result<Child> {
    kill_all_llama_servers();
    let child = Command::new(server_path)
        .args([
            "--model",        model_path.to_str().unwrap(),
            "--port",         &port.to_string(),
            "--n-gpu-layers", "35",
            "--ctx-size",     "4096",
            "--no-warmup",
        ])
        .spawn()?;
    Ok(child)
}

// waits for TCP server to be available
async fn wait_for_server(
    host: &str,
    port: u16, 
    retries: Option<u32>, 
    delay_ms: Option<u64>,
) -> Result<()> {
    let addr = format!("{}:{}", host, port);
    for _ in 0..retries.unwrap_or(10) {
        if TcpStream::connect(&addr).await.is_ok() {
            return Ok(());
        }
        sleep(Duration::from_millis(delay_ms.unwrap_or(500))).await;
    }
    Err(anyhow!("TCP Timeout"))
}

// Waits for model to be ready by sending a dummy completion request
async fn wait_for_model_ready(
    host: &str,
    port: u16,
    client: &Client,
    retries: Option<u32>,
    delay_ms: Option<u64>,
) -> Result<()> {
    let url = format!("http://{}:{}/completions", host, port);
    let dummy = serde_json::json!({
        "prompt": "ping",
        "stream": false,
        "temperature": 0.0,
        "n_predict": 1
    });

    for _ in 0..retries.unwrap_or(60) {
        if let Ok(resp) = client.post(&url).json(&dummy).send().await {
            if resp.status().is_success() {
                return Ok(());
            }
        }
        sleep(Duration::from_millis(delay_ms.unwrap_or(500))).await;
    }
    Err(anyhow!("HTTP Timeout"))
}
