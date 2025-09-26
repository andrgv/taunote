use crate::services::llm::llama_client::LlamaClient;
use anyhow::{anyhow, Error as AnyhowError, Result};
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use tokio::{
    sync::{
        mpsc::{self, Sender},
        oneshot, Mutex,
    },
    task,
};

pub struct CompletionJob {
    pub prompt: String,
    pub n_predict: u32,
    pub responder: oneshot::Sender<Result<String, AnyhowError>>,
}

static LLAMA_QUEUE: OnceCell<Mutex<Sender<CompletionJob>>> = OnceCell::new();

pub async fn init_llama_queue() {
    // create new channel with buffer size 16.
    let (tx, mut rx) = mpsc::channel::<CompletionJob>(16);

    // spawn a background task to handle LLM completion requests
    task::spawn(async move {
        let client = match LlamaClient::try_new(
            PathBuf::from("/home/andrea/llama.cpp/build/bin/llama-server"),
            PathBuf::from(
                "/home/andrea/taunote/models/llama/Meta-Llama-3.1-8B-Instruct-Q6_K_L.gguf",
            ),
            "127.0.0.1".to_string(),
            8081,
        )
        .await
        {
            Ok(c) => c,
            Err(err) => {
                eprintln!("Failed to start LlamaClient: {err}");
                return;
            }
        };

        // process jobs from the queue as they arrive
        while let Some(CompletionJob {
            prompt,
            n_predict,
            responder,
        }) = rx.recv().await
        {
            let raw = client.complete(prompt, n_predict).await;
            let anyhow_result = raw.map_err(|e| anyhow!(e.to_string()));
            let _ = responder.send(anyhow_result);
        }
    });

    // store the sender in static OnceCell, will panic if called more than once.
    LLAMA_QUEUE
        .set(Mutex::new(tx))
        .expect("LLAMA_QUEUE was already initialized");
}

pub async fn enqueue_completion(prompt: String, n_predict: u32) -> Result<String> {
    // create oneshot channel to receive the result from the processing task.
    let (tx, rx) = oneshot::channel();
    let job = CompletionJob {
        prompt,
        n_predict,
        responder: tx,
    };

    // get a lock on the global queue sender.
    let sender = LLAMA_QUEUE
        .get()
        .ok_or_else(|| anyhow!("LLAMA_QUEUE not initialized"))?
        .lock()
        .await;

    // send the job to queue and wait for response
    println!("Sending job to llm");
    sender.send(job).await?;
    let response: Result<String, AnyhowError> = rx.await?;
    let output = response?;

    Ok(output)
}
