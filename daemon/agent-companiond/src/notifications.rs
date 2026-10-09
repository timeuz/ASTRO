use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use std::process::Command;
use tracing::{info, error};

use crate::SessionState;

pub struct NotificationManager {
    last_notified: HashMap<String, Instant>,
    cooldown: Duration,
}

impl NotificationManager {
    pub fn new(cooldown_secs: u64) -> Self {
        Self {
            last_notified: HashMap::new(),
            cooldown: Duration::from_secs(cooldown_secs),
        }
    }

    pub fn notify_if_needed(&mut self, agent: &str, session_id: &str, state: &SessionState, msg: Option<&str>) {
        let title = match state {
            SessionState::Failed => format!("Erro no Agente: {}", agent),
            SessionState::WaitingPermission | SessionState::WaitingInput => format!("Atenção: {}", agent),
            SessionState::Completed => format!("Concluído: {}", agent),
            _ => return, // Don't notify for other states
        };

        let key = format!("{}-{:?}", session_id, state);
        let now = Instant::now();

        if let Some(last) = self.last_notified.get(&key) {
            if now.duration_since(*last) < self.cooldown {
                // Skip due to cooldown
                return;
            }
        }

        self.last_notified.insert(key, now);

        let body = msg.unwrap_or("Verifique o painel do ASTRO para mais detalhes.");
        info!("Sending desktop notification: {} - {}", title, body);

        if let Err(e) = Command::new("notify-send")
            .args(&["-a", "ASTRO Agent Companion", "-i", "dialog-information", &title, body])
            .spawn()
        {
            error!("Failed to send desktop notification: {}", e);
        }
    }
}
