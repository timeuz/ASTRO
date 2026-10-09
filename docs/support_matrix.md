# ASTRO Support Matrix

This document outlines the current capabilities and feature support across different agent integrations within the ASTRO framework.

## Agent Integrations

| Feature | Gemini CLI | Antigravity |
|---------|------------|-------------|
| **Detection & Observability** | Yes | Yes |
| **Session Tracking (Lifecycle)** | Yes | Yes |
| **State Tracking (Running/Idle)** | Yes | Yes |
| **Active Hook Injection Required** | Yes | No (Passive Log-Tail) |
| **Fail-Open Guarantees** | Yes (via Hook Timeout) | Yes (Out-of-band observer) |
| **Interactive Prompts / Questions** | Yes | No (Unsupported via Log-Tail) |
| **Interactive Permissions** | Yes | No (Unsupported via Log-Tail) |
| **Visual Separation in UI** | Yes | Yes |

### Implementation Details

#### Gemini CLI
- **Method:** Active hook injection. ASTRO dynamically injects a Rust wrapper (`agent-companion-hook`) as the entrypoint for Gemini CLI operations.
- **Capabilities:** Fully synchronous capabilities including the ability to interrupt, pause, or query the user during an active session via IPC channels.
- **Security:** Requires careful validation of idempotency and rollback to ensure original configuration files aren't corrupted.

#### Antigravity
- **Method:** Passive observability via Log-Tailing. ASTRO utilizes a Python adapter (`scripts/antigravity_adapter.py`) to stream the `transcript.jsonl` files natively produced by Antigravity.
- **Capabilities:** Excellent for post-facto state observation and tracking, but inherently limited when requiring active interruptions (e.g., stopping the agent) or interactive queries (e.g., approving sandboxed tool calls) because the log-tail is unidirectional.
- **Security:** Highly robust since it doesn't modify any executable hooks or configuration wrappers of the Antigravity system.
