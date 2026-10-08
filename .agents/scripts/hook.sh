#!/bin/bash
# Read stdin
INPUT=$(cat)
# Send to daemon (for now, we'll just log it to a file to verify the spike)
echo "$INPUT" >> /home/timeu/Documentos/AI/ASTRO/hook-spike.log
# Also try to call the agent-companion-hook if the daemon is running
/home/timeu/Documentos/AI/ASTRO/daemon/agent-companion-hook/target/debug/agent-companion-hook > /dev/null 2>&1 || true

# Must output valid JSON for decision
echo '{"decision": "allow"}'
