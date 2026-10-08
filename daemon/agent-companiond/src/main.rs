use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::signal;
use zbus::{connection, interface};
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::rolling;
use tracing_appender::non_blocking;

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

// D-Bus interface
struct AgentCompanion;

#[interface(name = "org.astro.AgentCompanion")]
impl AgentCompanion {
    async fn ping(&self) -> String {
        info!("Received D-Bus Ping");
        "Pong from AgentCompanion".to_string()
    }

    async fn send_event(&self, event_type: String, payload: String) -> String {
        let len = payload.len();
        let sanitized = format!("[REDACTED, len={}]", len);
        info!("Received D-Bus event: type={} payload={}", event_type, sanitized);
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
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").context("XDG_RUNTIME_DIR must be set for security")?;
    let socket_path = PathBuf::from(xdg_runtime_dir).join("astro-agent.sock");

    // Remove existing socket if any
    if socket_path.exists() {
        fs::remove_file(&socket_path).context("Failed to remove existing socket")?;
    }

    let listener = UnixListener::bind(&socket_path).context("Failed to bind UDS")?;
    
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600)).context("Failed to set 0600 on socket")?;
    
    info!("UDS server listening at {}", socket_path.display());

    loop {
        match listener.accept().await {
            Ok((mut socket, _addr)) => {
                // UID validation
                if let Ok(cred) = socket.peer_cred() {
                    let daemon_uid = unsafe { libc::geteuid() };
                    if cred.uid() != daemon_uid {
                        error!("Rejected connection from UID {} (expected {})", cred.uid(), daemon_uid);
                        continue;
                    }
                } else {
                    error!("Failed to get peer cred");
                    continue;
                }

                tokio::spawn(async move {
                    let mut buf = vec![0; 1024];
                    loop {
                        match socket.read(&mut buf).await {
                            Ok(0) => break, // Connection closed
                            Ok(n) => {
                                let received = &buf[..n];
                                if let Ok(mut event) = serde_json::from_slice::<DummyEvent>(received) {
                                    let len = event.payload.len();
                                    event.payload = format!("[REDACTED, len={}]", len);
                                    info!("Received UDS Event: {:?}", event);
                                    let response = b"OK\n";
                                    if let Err(e) = socket.write_all(response).await {
                                        error!("Failed to write response: {}", e);
                                    }
                                } else {
                                    info!("Received Raw UDS data");
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

fn setup_logging() -> Option<non_blocking::WorkerGuard> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let stdout_layer = fmt::layer().with_writer(std::io::stdout);

    // Optional rotating persistence
    let log_dir = if let Ok(dir) = env::var("ASTRO_LOG_DIR") {
        PathBuf::from(dir)
    } else {
        let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let state_home = env::var("XDG_STATE_HOME").unwrap_or_else(|_| format!("{}/.local/state", home));
        PathBuf::from(state_home).join("astro-agent").join("logs")
    };

    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("Failed to create log directory at {}: {}", log_dir.display(), e);
        tracing_subscriber::registry()
            .with(env_filter)
            .with(stdout_layer)
            .init();
        None
    } else {
        let file_appender = rolling::daily(&log_dir, "astro-agent.log");
        let (non_blocking_appender, guard) = non_blocking(file_appender);
        let file_layer = fmt::layer().with_writer(non_blocking_appender);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(stdout_layer)
            .with(file_layer)
            .init();
        
        info!("Logging initialized. Logs are stored in {}", log_dir.display());
        Some(guard)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _log_guard = setup_logging();
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
    if let Ok(xdg_runtime_dir) = env::var("XDG_RUNTIME_DIR") {
        let socket_path = PathBuf::from(xdg_runtime_dir).join("astro-agent.sock");
        if socket_path.exists() {
            let _ = fs::remove_file(socket_path);
        }
    }

    Ok(())
}
