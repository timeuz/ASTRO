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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CanonicalEvent {
    pub version: String,
    pub agent_name: String,
    pub session_id: String,
    pub event_type: String,
    pub state: Option<SessionState>,
    pub timestamp: u64,
    pub payload: serde_json::Value,
}

async fn run_hook() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let hook_type = args.get(1).map(|s| s.as_str()).unwrap_or("unknown");
    
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").context("XDG_RUNTIME_DIR must be set")?;
    let socket_path = PathBuf::from(xdg_runtime_dir).join("astro-agent.sock");

    let mut stream = UnixStream::connect(&socket_path).await?;

    let event_type = match hook_type {
        "pre-tool-call" | "pre_tool_call" => "working",
        "post-tool-call" | "post_tool_call" => "idle",
        "error" => "error",
        _ => hook_type,
    };
    
    let state = match event_type {
        "working" => Some(SessionState::Working),
        "error" => Some(SessionState::Failed),
        "idle" => Some(SessionState::Idle),
        _ => Some(SessionState::Idle),
    };

    let session_id = env::var("AGENT_SESSION_ID").unwrap_or_else(|_| "unknown-session".to_string());
    let agent_name = env::var("AGENT_NAME").unwrap_or_else(|_| "Antigravity".to_string());

    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

    let event = CanonicalEvent {
        version: "1.0".to_string(),
        agent_name,
        session_id,
        event_type: event_type.to_string(),
        state,
        timestamp,
        payload: serde_json::json!({"hook": hook_type}),
    };

    let mut serialized = serde_json::to_string(&event)?;
    serialized.push('\n'); // important for read_line in UDS!
    
    stream.write_all(serialized.as_bytes()).await?;

    let mut response = vec![0; 1024];
    match stream.read(&mut response).await {
        Ok(_) => {}
        Err(_) => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Fail-open timeout of 450ms
    let _ = timeout(Duration::from_millis(450), run_hook()).await;
    Ok(())
}
