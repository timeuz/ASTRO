use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::process::Command;
use tracing::{info, error};

use crate::SessionState;
use crate::config::UserConfig;

pub struct NotificationManager {
    last_notified: HashMap<String, Instant>,
    cooldown: Duration,
    pub user_config: UserConfig,
}

impl NotificationManager {
    pub fn new(cooldown_secs: u64, user_config: UserConfig) -> Self {
        Self {
            last_notified: HashMap::new(),
            cooldown: Duration::from_secs(cooldown_secs),
            user_config,
        }
    }

    pub fn notify_if_needed(&mut self, agent: &str, session_id: &str, state: &SessionState, msg: Option<&str>, category: Option<&str>) {
        if let Some(cat) = category {
            if self.user_config.muted_categories.contains(&cat.to_string()) {
                return;
            }
        }

        let title = match state {
            SessionState::Failed => format!("Erro no Agente: {}", agent),
            SessionState::WaitingPermission | SessionState::WaitingInput => format!("Atenção: {}", agent),
            SessionState::Completed => {
                if !self.user_config.notify_on_completion {
                    return;
                }
                format!("Concluído: {}", agent)
            },
            _ => return, // Don't notify for other states
        };

        let key = format!("{}-{:?}", session_id, state);
        let now = Instant::now();

        if let Some(last) = self.last_notified.get(&key) {
            if now.duration_since(*last) < self.cooldown {
                return; // Cooldown active
            }
        }

        self.last_notified.insert(key, now);

        let mut final_msg = msg.unwrap_or("Verifique o painel do ASTRO para mais detalhes.").to_string();
        if self.user_config.neutral_mode {
            final_msg = "Requer sua atenção.".to_string();
        }

        info!("Sending desktop notification: {} - {}", title, final_msg);

        if let Err(e) = Command::new("notify-send")
            .args(&["-a", "ASTRO Agent Companion", "-i", "dialog-information", &title, &final_msg])
            .spawn()
        {
            error!("Failed to send desktop notification: {}", e);
        }
    }
}
