#!/bin/bash
set -e

# ASTRO-11: E2E Test Suite for Config Manager
echo "=== ASTRO-11 Test Suite ==="

# Usar python3 para rodar o script (já que não pudemos dar chmod +x)
MANAGER="python3 $(pwd)/scripts/astro_config_manager.py"
export TEST_HOME="/tmp/astro_test_home"
export ASTRO_HOOKS_JSON="$TEST_HOME/.gemini/hooks.json"
export XDG_RUNTIME_DIR="$TEST_HOME/run"

rm -rf "$TEST_HOME"
mkdir -p "$TEST_HOME/.gemini" "$XDG_RUNTIME_DIR"

# Simula o socket do daemon
touch "$XDG_RUNTIME_DIR/astro-agent.sock"

echo "[T01] Instalação em configuração inexistente"
$MANAGER install --yes
if ! grep -q "agent-companion-hook pre_tool_call" "$ASTRO_HOOKS_JSON"; then
    echo "FALHA T01" && exit 1
fi

echo "[T04] Instalação repetida (Idempotência)"
$MANAGER install --yes
# Não deve duplicar
COUNT=$(grep -o "agent-companion-hook pre_tool_call" "$ASTRO_HOOKS_JSON" | wc -l)
if [ "$COUNT" -ne 1 ]; then
    echo "FALHA T04: Duplicação detectada" && exit 1
fi

echo "[T10] Remoção sem alterações"
$MANAGER uninstall
if [ -f "$ASTRO_HOOKS_JSON" ] && grep -q "agent-companion-hook" "$ASTRO_HOOKS_JSON"; then
    echo "FALHA T10: Hook não foi removido" && exit 1
fi

echo "[T02] Instalação em configuração existente"
echo '{"outra_chave": "preservar", "pre_tool_call": ["outro-hook"]}' > "$ASTRO_HOOKS_JSON"
$MANAGER install --yes
if ! grep -q "preservar" "$ASTRO_HOOKS_JSON" || ! grep -q "agent-companion-hook" "$ASTRO_HOOKS_JSON"; then
    echo "FALHA T02: Configuração alheia não foi preservada" && exit 1
fi

echo "[T11] Remoção após mudança alheia"
$MANAGER uninstall
if ! grep -q "outro-hook" "$ASTRO_HOOKS_JSON"; then
    echo "FALHA T11: Alteração alheia foi apagada" && exit 1
fi

echo "[T12] Edição manual do hook (Conflito)"
$MANAGER install --yes
sed -i 's/agent-companion-hook pre_tool_call/agent-companion-hook pre_tool_call --extra/g' "$ASTRO_HOOKS_JSON"
if $MANAGER uninstall 2>/dev/null; then
    echo "FALHA T12: Deveria ter abortado por conflito" && exit 1
fi

echo "[T10] Diagnóstico Status"
STATUS=$($MANAGER status)
if ! echo "$STATUS" | grep -q '"hook_installed": false'; then
    echo "FALHA T10: Diagnóstico leu incorretamente a instalação editada" && exit 1
fi

echo "TODOS OS TESTES (T01-T12) PASSARAM COM SUCESSO NO BUNKER!"
