#!/bin/bash
set -e

if ! command -v cargo &> /dev/null; then
    source $HOME/.cargo/env
fi

echo "Building project binaries..."
cargo build --manifest-path daemon/agent-companiond/Cargo.toml
cargo build --manifest-path daemon/agent-companion-hook/Cargo.toml

DAEMON_BIN="$(pwd)/daemon/agent-companiond/target/debug/agent-companiond"
HOOK_BIN="$(pwd)/daemon/agent-companion-hook/target/debug/agent-companion-hook"

export XDG_RUNTIME_DIR="/tmp/astro-tests-$$"
export ASTRO_LOG_DIR="/tmp/astro-tests-logs-$$"
mkdir -p "$XDG_RUNTIME_DIR"
mkdir -p "$ASTRO_LOG_DIR"
SOCKET_PATH="$XDG_RUNTIME_DIR/astro-agent.sock"

# ---------------------------------------------------------
# 1. Teste Fail-Open
# ---------------------------------------------------------
echo "Running Teste Fail-Open..."
# Ensure socket doesn't exist
rm -f "$SOCKET_PATH"

START=$(date +%s%3N)
if $HOOK_BIN; then
    echo "Hook exited gracefully without daemon."
else
    echo "Hook failed with non-zero exit code!"
    exit 1
fi
END=$(date +%s%3N)
DUR=$((END-START))
if [ $DUR -gt 600 ]; then
    echo "Hook blocked for too long ($DUR ms)! Should fail open quickly."
    exit 1
fi
echo "Teste Fail-Open passed!"

# ---------------------------------------------------------
# 2. Teste Multi-Sessão & 3. Teste de Privacidade
# ---------------------------------------------------------
echo "Starting daemon for Multi-Sessão and Privacidade tests..."
$DAEMON_BIN &
DAEMON_PID=$!

# Wait for socket
for i in {1..50}; do
    if [ -e "$SOCKET_PATH" ]; then
        break
    fi
    sleep 0.1
done

if [ ! -e "$SOCKET_PATH" ]; then
    echo "Daemon failed to create socket!"
    kill -9 $DAEMON_PID
    exit 1
fi

echo "Running Teste Multi-Sessão..."
# Create a python script to send data concurrently
cat << 'EOF' > "$XDG_RUNTIME_DIR/send.py"
import socket
import json
import sys
import threading

socket_path = sys.argv[1]

def send_payload(i):
    try:
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        s.connect(socket_path)
        data = {"event_type": "test_concurrent", "state": "working", "payload": f"secret_multi_session_{i}"}
        s.sendall(json.dumps(data).encode('utf-8'))
        s.recv(1024)
        s.close()
    except Exception as e:
        print(f"Thread {i} failed: {e}")

threads = []
for i in range(10):
    t = threading.Thread(target=send_payload, args=(i,))
    threads.append(t)
    t.start()

for t in threads:
    t.join()
EOF

python3 "$XDG_RUNTIME_DIR/send.py" "$SOCKET_PATH"

# Give the daemon a little time to process and log
sleep 1
kill $DAEMON_PID || true
wait $DAEMON_PID 2>/dev/null || true

echo "Running Teste de Privacidade..."
LOG_FILE="$ASTRO_LOG_DIR/astro-agent.log"
if [ ! -f "$LOG_FILE" ]; then
    # find out the log name, maybe rolling date
    LOG_FILE=$(ls "$ASTRO_LOG_DIR"/astro-agent.log*)
    if [ -z "$LOG_FILE" ]; then
        echo "Log file not found in $ASTRO_LOG_DIR"
        exit 1
    fi
fi

echo "Checking logs at $LOG_FILE"

# Check for plain text payload
if grep -q "secret_multi_session_" "$LOG_FILE"; then
    echo "Privacy Test Failed! Found plain text payload in logs."
    exit 1
fi

# Check for redacted payload
if ! grep -q ""redacted": true" "$LOG_FILE"; then
    echo "Privacy Test Failed! Did not find REDACTED payloads in logs."
    exit 1
fi

# Check multi-session count
COUNT=$(grep "test_concurrent" "$LOG_FILE" | wc -l)
if [ "$COUNT" -lt 10 ]; then
    echo "Multi-session test failed! Processed $COUNT / 10 events."
    exit 1
fi

echo "All tests passed successfully!"
rm -rf "$XDG_RUNTIME_DIR" "$ASTRO_LOG_DIR"
