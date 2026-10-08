use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::timeout;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Starting,
    Idle,
    Thinking,
    Working,
    WaitingInput,
    WaitingPermission,
    Completed,
    Failed,
    Disconnected,
}

#[derive(Serialize, Deserialize, Debug)]
struct DummyEvent {
    event_type: String,
    state: Option<SessionState>,
    payload: String,
}

async fn run_hook() -> Result<()> {
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").context("XDG_RUNTIME_DIR must be set")?;
    let socket_path = PathBuf::from(xdg_runtime_dir).join("astro-agent.sock");

    let mut stream = UnixStream::connect(&socket_path).await?;

    let event = DummyEvent {
        event_type: "state_update".to_string(),
        state: Some(SessionState::Starting),
        payload: "Hello from agent-companion-hook!".to_string(),
    };

    let serialized = serde_json::to_vec(&event)?;
    stream.write_all(&serialized).await?;
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

#[tokio::main]
async fn main() -> Result<()> {
    // Fail-open: wrap the entire network interaction in a timeout
    // If it fails or times out, we catch it and exit cleanly with Ok(())
    match timeout(Duration::from_millis(450), run_hook()).await {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => {
            eprintln!("Hook encountered an error, failing open. Error: {}", e);
        }
        Err(_) => {
            eprintln!("Hook timed out, failing open.");
        }
    }
    Ok(())
}
