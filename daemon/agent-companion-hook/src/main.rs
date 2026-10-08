use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[derive(Serialize, Deserialize, Debug)]
struct DummyEvent {
    event_type: String,
    payload: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let socket_path = PathBuf::from(xdg_runtime_dir).join("astro-agent.sock");

    let mut stream = UnixStream::connect(&socket_path)
        .await
        .with_context(|| format!("Failed to connect to socket at {:?}", socket_path))?;

    let event = DummyEvent {
        event_type: "dummy_event".to_string(),
        payload: "Hello from agent-companion-hook!".to_string(),
    };

    let serialized = serde_json::to_vec(&event).context("Failed to serialize event")?;
    
    stream.write_all(&serialized).await.context("Failed to write to socket")?;
    println!("Sent event: {:?}", event);

    let mut response = vec![0; 1024];
    match stream.read(&mut response).await {
        Ok(n) => {
            let resp_str = String::from_utf8_lossy(&response[..n]);
            println!("Received response: {}", resp_str.trim());
        }
        Err(e) => {
            println!("Failed to read response: {}", e);
        }
    }

    Ok(())
}
