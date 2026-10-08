#!/bin/bash
set -euo pipefail
source $HOME/.cargo/env

echo "=== Iniciando Teste E2E (ASTRO-06) ==="

# Compila antes para evitar log sujo
cargo build --manifest-path daemon/agent-companiond/Cargo.toml
cargo build --manifest-path daemon/agent-companion-hook/Cargo.toml

# Inicia o daemon em background
export XDG_RUNTIME_DIR=${XDG_RUNTIME_DIR:-/tmp/xdg_test}
mkdir -p "$XDG_RUNTIME_DIR"
export ASTRO_LOG_DIR="$XDG_RUNTIME_DIR"

daemon/agent-companiond/target/debug/agent-companiond > "$XDG_RUNTIME_DIR/daemon.log" 2>&1 &
DAEMON_PID=$!

# Aguarda UDS iniciar
sleep 2

# Envia evento via hook
echo "Simulando evento de inicio..."
AGENT_SESSION_ID="test-sess-999" AGENT_NAME="TesterAgent" daemon/agent-companion-hook/target/debug/agent-companion-hook pre_tool_call

# Espera processamento
sleep 1

# Encerra daemon
kill $DAEMON_PID || true

# Verifica log
if grep -q "test-sess-999" "$XDG_RUNTIME_DIR/daemon.log"; then
    echo "Sucesso: O daemon recebeu e processou o evento E2E!"
else
    echo "Falha: O evento não foi encontrado no log."
    cat "$XDG_RUNTIME_DIR/daemon.log"
    exit 1
fi

echo "Testes E2E finalizados com sucesso."
exit 0
