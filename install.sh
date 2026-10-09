#!/bin/bash
set -euo pipefail

echo "=== ASTRO Environment Installer ==="

DRY_RUN=0
UNINSTALL=0

for arg in "$@"; do
    case $arg in
        --preview) DRY_RUN=1 ;;
        --uninstall) UNINSTALL=1 ;;
    esac
done

check_deps() {
    echo "Verificando dependências..."
    for cmd in python3 rustc cargo git; do
        if ! command -v $cmd &> /dev/null; then
            echo "Erro: Dependência '$cmd' não encontrada."
            exit 1
        fi
    done
    
    # Opcional: Checar Gemini CLI / Antigravity
    # if ! command -v gemini &> /dev/null; then
    #    echo "Aviso: gemini CLI não encontrado no PATH."
    # fi
    echo "Todas as dependências encontradas."
}

do_install() {
    check_deps
    
    local DEST_DIR="$HOME/.local/bin"
    local SHARE_DIR="/usr/share/astro-agent-companion"
    local HOOKS_DIR="$HOME/.gemini"
    local TIMESTAMP=$(date +%Y%m%d_%H%M%S)

    if [ $DRY_RUN -eq 1 ]; then
        echo "[PREVIEW] O que será feito:"
        echo "1. Build dos binários em daemon/"
        echo "2. Copiar agent-companiond para $DEST_DIR"
        echo "3. Instalar hooks usando scripts/install-hooks.sh"
        echo "4. (Requer sudo) Copiar microcopy.json para $SHARE_DIR"
        exit 0
    fi

    echo "Compilando binários..."
    source $HOME/.cargo/env || true
    cargo build --release --manifest-path daemon/agent-companiond/Cargo.toml
    cargo build --release --manifest-path daemon/agent-companion-hook/Cargo.toml

    echo "Instalando daemon..."
    mkdir -p "$DEST_DIR"
    cp daemon/agent-companiond/target/release/agent-companiond "$DEST_DIR/"
    
    echo "Instalando hooks no Gemini CLI..."
    python3 scripts/astro_config_manager.py install --yes
    
    # Note: Copying to /usr/share usually requires sudo, 
    # For user installation, we can copy microcopy.json to ~/.config/astro-agent-companion/
    local USER_SHARE_DIR="$HOME/.config/astro-agent-companion"
    mkdir -p "$USER_SHARE_DIR"
    cp microcopy.json "$USER_SHARE_DIR/"
    echo "Configuração salva em $USER_SHARE_DIR/microcopy.json"

    echo "Instalação concluída com sucesso."
    echo "Para rodar o daemon: $DEST_DIR/agent-companiond"
}

do_uninstall() {
    local DEST_DIR="$HOME/.local/bin"
    local USER_SHARE_DIR="$HOME/.config/astro-agent-companion"
    
    if [ $DRY_RUN -eq 1 ]; then
        echo "[PREVIEW] O que será removido:"
        echo "- Binário $DEST_DIR/agent-companiond"
        echo "- Configurações em $USER_SHARE_DIR"
        echo "- Hooks via astro_config_manager.py"
        exit 0
    fi
    
    echo "Desinstalando ASTRO..."
    rm -f "$DEST_DIR/agent-companiond"
    rm -rf "$USER_SHARE_DIR"
    
    if [ -f "./scripts/astro_config_manager.py" ]; then
        python3 scripts/astro_config_manager.py uninstall
    fi
    
    echo "ASTRO removido completamente."
}

if [ $UNINSTALL -eq 1 ]; then
    do_uninstall
else
    do_install
fi
