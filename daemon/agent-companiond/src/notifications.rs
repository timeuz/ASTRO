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

    pub fn notify_if_needed(&mut self, agent: &str, session_id: &str, state: &SessionState, msg: Option<&str>, category: Option<&str>, payload_hash: u64) {
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

        // Key based on session, state, AND payload hash to differentiate distinct requests
        let key = format!("{}-{:?}-{:016x}", session_id, state, payload_hash);
        let now = Instant::now();

        if let Some(last) = self.last_notified.get(&key) {
            if now.duration_since(*last) < self.cooldown {
                return; // Cooldown active for THIS exact request
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cooldown_same_payload() {
        let mut config = UserConfig::default();
        let mut manager = NotificationManager::new(5, config);
        let state = SessionState::WaitingPermission;
        
        // Mock command execution to not actually call notify-send in tests, 
        // wait, we shouldn't mock it, but Command::new("notify-send") will fail silently in test if not found.
        // For cooldown logic, we just check `last_notified` length or updates.
        
        manager.notify_if_needed("Agent1", "Sess1", &state, None, None, 12345);
        assert_eq!(manager.last_notified.len(), 1);
        
        let first_instant = *manager.last_notified.get("Sess1-WaitingPermission-0000000000003039").unwrap();
        
        // Same payload -> should be skipped (cooldown)
        manager.notify_if_needed("Agent1", "Sess1", &state, None, None, 12345);
        let second_instant = *manager.last_notified.get("Sess1-WaitingPermission-0000000000003039").unwrap();
        assert_eq!(first_instant, second_instant); // Did not update
        
        // Different payload -> should be processed
        manager.notify_if_needed("Agent1", "Sess1", &state, None, None, 67890);
        assert_eq!(manager.last_notified.len(), 2);
    }
}
