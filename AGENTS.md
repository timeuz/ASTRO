# ASTRO - Agent Session Tracking & Runtime Observer

## Gitflow Governance Flow

1. **Branching Strategy**: 
   - `main`: Production-ready code.
   - `develop`: Integration branch.
   - `feature/*`: New features (branched from `develop`).
   - `bugfix/*`: Bug fixes.
   - `security/*`: Security patches and enhancements.

2. **Cross-Code-Review**: 
   - All PRs (especially between feature and security branches) require mandatory cross-code-review.
   - Security Engineer and Code Reviewer must approve security-critical PRs.

3. **Mandatory Testing**: 
   - Unit tests, IPC integration tests, and fail-open validation must pass before merge.
   - The Tester Agent handles QA validation.

## Architecture

- **Backend Daemon** (`agent-companiond`): Rust + Tokio.
- **Relay** (`agent-companion-hook`): Rust, handles IPC via Unix Socket/D-Bus.
- **Frontend Extension**: GNOME Shell Extension in GJS.
- **Settings App**: GTK4 + libadwaita.
- **Packaging**: Debian `.deb` with `systemd --user`.
