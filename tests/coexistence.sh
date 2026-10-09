#!/bin/bash
set -e

if ! command -v cargo &> /dev/null; then
    source $HOME/.cargo/env
fi

echo "Building project binaries..."
cargo build --manifest-path daemon/agent-companiond/Cargo.toml

DAEMON_BIN="$(pwd)/daemon/agent-companiond/target/debug/agent-companiond"

export XDG_RUNTIME_DIR="/tmp/astro-tests-coexistence-$$"
export ASTRO_LOG_DIR="/tmp/astro-tests-logs-coexistence-$$"
mkdir -p "$XDG_RUNTIME_DIR"
mkdir -p "$ASTRO_LOG_DIR"
SOCKET_PATH="$XDG_RUNTIME_DIR/astro-agent.sock"

echo "Starting daemon for Coexistence test..."
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

echo "Running Coexistence Multi-Agent Event Test..."
# Create a python script to send data concurrently as different agents
cat << 'EOF' > "$XDG_RUNTIME_DIR/send_coexistence.py"
import socket
import json
import sys
import threading
import time

socket_path = sys.argv[1]

def send_payload(agent_name, session_id):
    try:
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        s.connect(socket_path)
        data = {
            "version": "1.0",
            "agent_name": agent_name,
            "session_id": session_id,
            "event_type": "activity", 
            "state": "working", 
            "timestamp": int(time.time()),
            "payload": f"payload_for_{agent_name}"
        }
        payload_str = json.dumps(data) + "\n"
        s.sendall(payload_str.encode('utf-8'))
        s.recv(1024)
        s.close()
    except Exception as e:
        print(f"Thread failed: {e}")

threads = []
for i in range(10):
    t1 = threading.Thread(target=send_payload, args=("Gemini CLI", f"gemini-sess-{i}"))
    t2 = threading.Thread(target=send_payload, args=("Antigravity", f"antigrav-sess-{i}"))
    threads.append(t1)
    threads.append(t2)
    t1.start()
    t2.start()

for t in threads:
    t.join()
EOF

python3 "$XDG_RUNTIME_DIR/send_coexistence.py" "$SOCKET_PATH"

# Give the daemon a little time to process and log
sleep 1
kill $DAEMON_PID || true
wait $DAEMON_PID 2>/dev/null || true

LOG_FILE="$ASTRO_LOG_DIR/astro-agent.log"
if [ ! -f "$LOG_FILE" ]; then
    LOG_FILE=$(ls "$ASTRO_LOG_DIR"/astro-agent.log* | head -n 1)
    if [ -z "$LOG_FILE" ]; then
        echo "Log file not found in $ASTRO_LOG_DIR"
        exit 1
    fi
fi

echo "Checking logs at $LOG_FILE"

# Check if both agents are logged
COUNT_GEMINI=$(grep "Gemini CLI" "$LOG_FILE" | wc -l)
COUNT_ANTIGRAVITY=$(grep "Antigravity" "$LOG_FILE" | wc -l)

echo "Found $COUNT_GEMINI Gemini CLI events"
echo "Found $COUNT_ANTIGRAVITY Antigravity events"

if [ "$COUNT_GEMINI" -lt 10 ]; then
    echo "Coexistence test failed! Processed $COUNT_GEMINI / 10 Gemini CLI events."
    exit 1
fi

if [ "$COUNT_ANTIGRAVITY" -lt 10 ]; then
    echo "Coexistence test failed! Processed $COUNT_ANTIGRAVITY / 10 Antigravity events."
    exit 1
fi

echo "Coexistence tests passed successfully!"
rm -rf "$XDG_RUNTIME_DIR" "$ASTRO_LOG_DIR"
