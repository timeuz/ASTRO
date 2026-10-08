#!/bin/bash
# Spike to intercept Antigravity (Gemini CLI) hook events

LOG_FILE="$(dirname "$0")/../.spike_hook.log"

echo "=== Hook Execution at $(date) ===" >> "$LOG_FILE"
echo "Arguments: $@" >> "$LOG_FILE"
echo "Environment Variables (Filtered):" >> "$LOG_FILE"
env | grep -iE 'agent|gemini|antigravity' >> "$LOG_FILE" || true
echo "Stdin (if any):" >> "$LOG_FILE"
cat /dev/stdin >> "$LOG_FILE" 2>/dev/null || true
echo "=================================" >> "$LOG_FILE"
