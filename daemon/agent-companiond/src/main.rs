use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CanonicalEvent {
    pub version: String,
    pub agent_name: String,
    pub session_id: String,
    pub event_type: String, // e.g. "tool_call", "message", "error"
    pub state: Option<SessionState>,
    pub timestamp: u64,
    pub payload: serde_json::Value,
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Security,
    DataLoss,
    Permission,
}

impl Severity {
    pub fn forces_neutral(&self) -> bool {
        matches!(
            self,
            Severity::Error | Severity::Security | Severity::DataLoss | Severity::Permission
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CategoryConfig {
    pub neutral: Vec<String>,
    #[serde(default)]
    pub jokes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MicrocopyConfig {
    pub categories: HashMap<String, CategoryConfig>,
}

pub struct MicrocopyEngine {
    pub config: MicrocopyConfig,
    pub neutral_mode: bool,
    pub joke_cooldown: Duration,
    pub last_joke_time: Option<Instant>,
    pub used_jokes: HashSet<String>,
}

impl MicrocopyEngine {
    pub fn new(config: MicrocopyConfig) -> Self {
        Self {
            config,
            neutral_mode: false,
            joke_cooldown: Duration::from_secs(60),
            last_joke_time: None,
            used_jokes: HashSet::new(),
        }
    }

    pub fn load(path: &PathBuf) -> Result<Self> {
        let data = fs::read_to_string(path).context("Failed to read microcopy config")?;
        let config: MicrocopyConfig = serde_json::from_str(&data).context("Failed to parse microcopy config")?;
        Ok(Self::new(config))
    }

    pub fn set_neutral_mode(&mut self, enabled: bool) {
        self.neutral_mode = enabled;
    }

    pub fn get_message(&mut self, category: &str, severity: &Severity) -> Option<String> {
        let cat_config = self.config.categories.get(category)?;

        let force_neutral = self.neutral_mode || severity.forces_neutral();
        
        let now = Instant::now();
        let in_cooldown = self
            .last_joke_time
            .map(|t| now.duration_since(t) < self.joke_cooldown)
            .unwrap_or(false);

        if force_neutral || in_cooldown || cat_config.jokes.is_empty() {
            return cat_config.neutral.first().cloned();
        }

        for joke in &cat_config.jokes {
            if !self.used_jokes.contains(joke) {
                self.used_jokes.insert(joke.clone());
                self.last_joke_time = Some(now);
                return Some(joke.clone());
            }
        }

        // Fallback to neutral if all jokes used
        cat_config.neutral.first().cloned()
    }
}

// D-Bus interface
struct AgentCompanion {
    engine: Arc<Mutex<MicrocopyEngine>>,
}

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
        
        // Example usage of engine
        let mut msg = String::new();
        if let Ok(mut engine) = self.engine.lock() {
            if let Some(m) = engine.get_message(&event_type, &Severity::Info) {
                msg = m;
            }
        }
        format!("Event received. Msg: {}", msg)
    }

    async fn set_neutral_mode(&self, enabled: bool) -> String {
        info!("Received D-Bus set_neutral_mode: {}", enabled);
        if let Ok(mut engine) = self.engine.lock() {
            engine.set_neutral_mode(enabled);
            format!("Neutral mode set to {}", enabled)
        } else {
            "Failed to lock engine".to_string()
        }
    }
}

async fn run_dbus(engine: Arc<Mutex<MicrocopyEngine>>) -> Result<()> {
    let _conn = connection::Builder::session()?
        .name("org.astro.AgentCompanion")?
        .serve_at("/org/astro/AgentCompanion", AgentCompanion { engine })?
        .build()
        .await?;

    info!("D-Bus server listening at org.astro.AgentCompanion");

    // Keep it alive
    std::future::pending::<()>().await;
    Ok(())
}

async fn run_uds(engine: Arc<Mutex<MicrocopyEngine>>) -> Result<()> {
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

                let engine_clone = engine.clone();
                tokio::spawn(async move {
                    use tokio::io::{AsyncBufReadExt, BufReader};
                    let mut reader = BufReader::new(socket);
                    let mut line = String::new();
                    // We will not store deduplication history here because this is a new spawned task per connection.
                    // If we want global dedup, it should be in an Arc<Mutex<HashSet>> passed to UDS.
                    // For now, let's just parse the line securely.
                    loop {
                        line.clear();
                        match reader.read_line(&mut line).await {
                            Ok(0) => break, // Connection closed
                            Ok(_) => {
                                match serde_json::from_str::<CanonicalEvent>(&line) {
                                    Ok(mut event) => {
                                        // Sanitize payload
                                        event.payload = serde_json::json!({ "redacted": true, "msg": "Payload sanitizado para evitar vazamento" });
                                        info!("Received UDS Event: type={} agent={} session={} timestamp={}", event.event_type, event.agent_name, event.session_id, event.timestamp);
                                        

                                        if let Ok(mut eng) = engine_clone.lock() {
                                            if let Some(msg) = eng.get_message(&event.event_type, &Severity::Info) {
                                                info!("Microcopy message: {}", msg);
                                            }
                                        }

                                        let _ = std::process::Command::new("dbus-send")
                                            .args(["--session", "--type=signal", "/org/astro/Service", "org.astro.Service.EventReceived", &format!("string:{}", line)])
                                            .spawn();


                                        let response = b"OK\n";
                                        if let Err(e) = reader.get_mut().write_all(response).await {
                                            error!("Failed to write response: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to parse JSON schema: {}", e);
                                        let response = b"ACK\n";
                                        let _ = reader.get_mut().write_all(response).await;
                                    }
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

    let possible_paths = vec![
        PathBuf::from("microcopy.json"), // CWD
        PathBuf::from("../microcopy.json"), // Cargo run from daemon/
        PathBuf::from("../../microcopy.json"), // Cargo run from daemon/agent-companiond/
        env::current_exe().unwrap_or_default().parent().unwrap_or_else(|| std::path::Path::new("")).join("microcopy.json"),
        PathBuf::from(env::var("HOME").unwrap_or_else(|_| "".to_string())).join(".config/astro-agent-companion/microcopy.json"),
        PathBuf::from("/usr/share/astro-agent-companion/microcopy.json"),
    ];

    let mut microcopy_path = PathBuf::from("microcopy.json");
    for path in possible_paths {
        if path.exists() {
            microcopy_path = path;
            break;
        }
    }

    let engine = match MicrocopyEngine::load(&microcopy_path) {
        Ok(engine) => {
            info!("Loaded microcopy config from {}", microcopy_path.display());
            engine
        }
        Err(e) => {
            error!("Failed to load microcopy config: {}. Proceeding with empty config.", e);
            MicrocopyEngine::new(MicrocopyConfig { categories: HashMap::new() })
        }
    };
    
    let engine = Arc::new(Mutex::new(engine));

    let engine_dbus = engine.clone();
    let _dbus_task = tokio::spawn(async move {
        if let Err(e) = run_dbus(engine_dbus).await {
            error!("D-Bus server failed: {}", e);
        }
    });

    let engine_uds = engine.clone();
    let _uds_task = tokio::spawn(async move {
        if let Err(e) = run_uds(engine_uds).await {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_forces_neutral() {
        assert!(Severity::Error.forces_neutral());
        assert!(Severity::Security.forces_neutral());
        assert!(Severity::DataLoss.forces_neutral());
        assert!(Severity::Permission.forces_neutral());
        assert!(!Severity::Info.forces_neutral());
        assert!(!Severity::Warning.forces_neutral());
    }

    #[test]
    fn test_microcopy_engine_neutral_fallback() {
        let config = MicrocopyConfig {
            categories: {
                let mut map = HashMap::new();
                map.insert("error".to_string(), CategoryConfig {
                    neutral: vec!["Fallback neutral".to_string()],
                    jokes: vec![],
                });
                map
            }
        };
        let mut engine = MicrocopyEngine::new(config);
        assert_eq!(engine.get_message("error", &Severity::Error), Some("Fallback neutral".to_string()));
    }
}
