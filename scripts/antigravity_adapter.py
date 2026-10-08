import json
import os
import time
import socket
import sys
from pathlib import Path

def get_latest_transcript():
    brain_dir = Path.home() / ".gemini" / "antigravity-cli" / "brain"
    if not brain_dir.exists():
        return None
    dirs = [d for d in brain_dir.iterdir() if d.is_dir()]
    if not dirs:
        return None
    latest = max(dirs, key=lambda x: x.stat().st_mtime)
    tfile = latest / ".system_generated" / "logs" / "transcript.jsonl"
    return tfile if tfile.exists() else None

def send_uds_event(socket_path, event_type, payload_data):
    try:
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        s.connect(str(socket_path))
        data = {
            "version": "1.0",
            "agent_name": "Antigravity",
            "session_id": "current-session",
            "event_type": event_type,
            "state": "working",
            "timestamp": int(time.time()),
            "payload": payload_data
        }
        s.sendall(json.dumps(data).encode('utf-8') + b'\n')
        s.recv(1024)
        s.close()
    except Exception as e:
        pass

def tail_file(file_path, socket_path):
    with open(file_path, 'r') as f:
        f.seek(0, 2)
        while True:
            line = f.readline()
            if not line:
                time.sleep(0.5)
                continue
            try:
                event = json.loads(line)
                if event.get("type") == "USER_INPUT":
                    # Sanitize prompt
                    send_uds_event(socket_path, "session_start", {"redacted": True})
                elif "tool_calls" in event:
                    send_uds_event(socket_path, "working", {"redacted": True})
            except:
                pass

if __name__ == "__main__":
    xdg_runtime = os.environ.get("XDG_RUNTIME_DIR", f"/run/user/{os.getuid()}")
    sock_path = Path(xdg_runtime) / "astro-agent.sock"
    
    while True:
        tfile = get_latest_transcript()
        if tfile:
            print(f"Tailing {tfile}")
            send_uds_event(sock_path, "session_start", {"redacted": True})
            tail_file(tfile, sock_path)
        time.sleep(2)
