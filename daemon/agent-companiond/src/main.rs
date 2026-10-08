use anyhow::{Context, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::signal;
use zbus::{connection, interface};

#[derive(Serialize, Deserialize, Debug)]
struct DummyEvent {
    event_type: String,
    payload: String,
}

// D-Bus interface
struct AgentCompanion;

#[interface(name = "org.astro.AgentCompanion")]
impl AgentCompanion {
    async fn ping(&self) -> String {
        info!("Received D-Bus Ping");
        "Pong from AgentCompanion".to_string()
    }

    async fn send_event(&self, event_type: String, payload: String) -> String {
        info!("Received D-Bus event: type={} payload={}", event_type, payload);
        "Event received".to_string()
    }
}

async fn run_dbus() -> Result<()> {
    let _conn = connection::Builder::session()?
        .name("org.astro.AgentCompanion")?
        .serve_at("/org/astro/AgentCompanion", AgentCompanion)?
        .build()
        .await?;

    info!("D-Bus server listening at org.astro.AgentCompanion");

    // Keep it alive
    std::future::pending::<()>().await;
    Ok(())
}

async fn run_uds() -> Result<()> {
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let socket_path = PathBuf::from(xdg_runtime_dir).join("astro-agent.sock");

    // Remove existing socket if any
    if socket_path.exists() {
        fs::remove_file(&socket_path).context("Failed to remove existing socket")?;
    }

    let listener = UnixListener::bind(&socket_path).context("Failed to bind UDS")?;
    info!("UDS server listening at {}", socket_path.display());

    loop {
        match listener.accept().await {
            Ok((mut socket, _addr)) => {
                tokio::spawn(async move {
                    let mut buf = vec![0; 1024];
                    loop {
                        match socket.read(&mut buf).await {
                            Ok(0) => break, // Connection closed
                            Ok(n) => {
                                let received = &buf[..n];
                                if let Ok(event) = serde_json::from_slice::<DummyEvent>(received) {
                                    info!("Received UDS Event: {:?}", event);
                                    let response = b"OK\n";
                                    if let Err(e) = socket.write_all(response).await {
                                        error!("Failed to write response: {}", e);
                                    }
                                } else {
                                    let text = String::from_utf8_lossy(received);
                                    info!("Received Raw UDS: {}", text.trim());
                                    let response = b"ACK\n";
                                    let _ = socket.write_all(response).await;
                                }
                            }
                            Err(e) => {
                                error!("Failed to read from socket: {}", e);
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept UDS connection: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting ASTRO Agent Companion Daemon...");

    let _dbus_task = tokio::spawn(async {
        if let Err(e) = run_dbus().await {
            error!("D-Bus server failed: {}", e);
        }
    });

    let _uds_task = tokio::spawn(async {
        if let Err(e) = run_uds().await {
            error!("UDS server failed: {}", e);
        }
    });

    // Wait for SIGINT or SIGTERM
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Received Ctrl-C, shutting down.");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }

    // Cleanup socket on exit
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let socket_path = PathBuf::from(xdg_runtime_dir).join("astro-agent.sock");
    if socket_path.exists() {
        let _ = fs::remove_file(socket_path);
    }

    Ok(())
}
