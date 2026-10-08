#!/bin/bash
set -euo pipefail

# Idempotent install script for Gemini CLI hooks
# Handles previewing, backup, and rollback on failure.

if [ -d "${HOME}/.agents" ]; then
    HOOKS_DIR="${HOME}/.agents"
else
    HOOKS_DIR="${HOME}/.gemini"
fi

HOOKS_FILE="${HOOKS_DIR}/hooks.json"
BACKUP_DIR="${HOOKS_DIR}/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

if [ "$(id -u)" -eq 0 ]; then
    echo "ERROR: This script must not be run as root to avoid unnecessary privilege escalation." >&2
    exit 1
fi

DRY_RUN=0
if [[ "${1-}" == "--preview" ]] || [[ "${1-}" == "--dry-run" ]]; then
    DRY_RUN=1
    echo "Preview mode enabled. No changes will be made."
fi

mkdir -p "$HOOKS_DIR"

BACKUP_FILE=""
if [ -f "$HOOKS_FILE" ]; then
    BACKUP_FILE="${BACKUP_DIR}/hooks.json.${TIMESTAMP}.bak"
    if [ $DRY_RUN -eq 1 ]; then
        echo "[PREVIEW] Would backup $HOOKS_FILE to $BACKUP_FILE"
    else
        mkdir -p "$BACKUP_DIR"
        cp "$HOOKS_FILE" "$BACKUP_FILE"
        echo "Backed up existing hooks to $BACKUP_FILE"
    fi
fi

HOOK_KEY_PRE="pre-commit"
HOOK_VAL_PRE="agent-companiond-hook pre-commit"
HOOK_KEY_POST="post-commit"
HOOK_VAL_POST="agent-companiond-hook post-commit"

if [ $DRY_RUN -eq 1 ]; then
    echo "[PREVIEW] Would ensure $HOOK_KEY_PRE and $HOOK_KEY_POST hooks are set in $HOOKS_FILE."
    exit 0
fi

trap_rollback() {
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then
        echo "ERROR: Installation failed with exit code $exit_code."
        if [ -n "$BACKUP_FILE" ] && [ -f "$BACKUP_FILE" ]; then
            echo "Rolling back changes from backup: $BACKUP_FILE"
            cp "$BACKUP_FILE" "$HOOKS_FILE"
        elif [ -n "$BACKUP_FILE" ]; then
            echo "Rolling back changes by removing newly created $HOOKS_FILE"
            rm -f "$HOOKS_FILE"
        fi
    fi
    exit $exit_code
}
trap trap_rollback EXIT

if [ ! -f "$HOOKS_FILE" ]; then
    echo "{}" > "$HOOKS_FILE"
fi

if ! command -v jq >/dev/null 2>&1; then
    echo "ERROR: 'jq' is required for safely merging JSON. Please install it."
    exit 1
fi

TEMP_FILE=$(mktemp)
jq --arg kp "$HOOK_KEY_PRE" --arg vp "$HOOK_VAL_PRE" \
   --arg kpo "$HOOK_KEY_POST" --arg vpo "$HOOK_VAL_POST" \
   '.[$kp] = $vp | .[$kpo] = $vpo' "$HOOKS_FILE" > "$TEMP_FILE"

cat "$TEMP_FILE" > "$HOOKS_FILE"
rm -f "$TEMP_FILE"

echo "Hooks installed successfully to $HOOKS_FILE."
trap - EXIT
