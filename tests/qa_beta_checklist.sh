#!/bin/bash
set -e

echo "=== ASTRO Beta QA Checklist Execution ==="
DEB_FILE=$(ls build/astro-agent_*.deb | head -n 1)

echo "1. Artefato e Ambiente"
echo "DEB: $DEB_FILE"
sha256sum "$DEB_FILE"

echo "2. Instalação (Simulating clean user)"
sudo dpkg -i "$DEB_FILE"

echo "Checking daemon placement..."
test -f /usr/bin/agent-companiond
test -f /usr/lib/systemd/user/astro-agent.service
echo "Checking extension placement..."
test -d /usr/share/gnome-shell/extensions/astro-spike@astro.project.org

echo "3. Instalação do Hook (Simulating Gemini CLI integration)"
export ASTRO_HOOKS_JSON=~/.gemini/hooks_test.json
echo "{}" > "$ASTRO_HOOKS_JSON"

python3 /usr/share/astro-agent/scripts/astro_config_manager.py install --force

echo "Checking if hook was installed without breaking..."
grep "agent-companion-hook" "$ASTRO_HOOKS_JSON"

echo "Checking privacy (markers in UDS)..."
export XDG_RUNTIME_DIR=/tmp/astro-qa-run
mkdir -p "$XDG_RUNTIME_DIR"
/usr/bin/agent-companiond &
DAEMON_PID=$!

sleep 1
# Send test payload
python3 -c "
import socket, json
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect('$XDG_RUNTIME_DIR/astro-agent.sock')
s.sendall((json.dumps({'version':'1.0','agent_name':'Gemini CLI','session_id':'qa-1','event_type':'activity','state':'working','timestamp':100,'payload':{'text':'MY_SECRET_PROMPT_123'}})+'\n').encode())
s.recv(1024)
"

sleep 1
kill $DAEMON_PID || true

echo "Verifying log sanitization..."
LOG_FILE=$(ls ~/.local/state/astro-agent/logs/astro-agent.log* | head -n 1)
if grep -q "MY_SECRET_PROMPT_123" "$LOG_FILE"; then
    echo "FAIL: Secret prompt found in logs!"
    exit 1
else
    echo "PASS: Secret prompt not in logs."
    grep "REDACTED" "$LOG_FILE"
fi

echo "5. Remoção e Purge"
sudo dpkg -r astro-agent

# State files should remain after remove, but let's check purge
sudo dpkg -P astro-agent
test -d ~/.local/state/astro-agent/logs
echo "PASS: User state logs remain after purge (safe policy)."

echo "QA Checklist technical steps passed."
