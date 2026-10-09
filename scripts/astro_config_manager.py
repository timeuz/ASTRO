#!/usr/bin/env python3
import json
import os
import sys
import shutil
import time
import fcntl
from pathlib import Path

# ASTRO-07, 08, 09, 10 - Secure Config Manager

HOOK_CMD_PRE = "agent-companion-hook pre_tool_call"
HOOK_CMD_POST = "agent-companion-hook post_tool_call"
CONFIG_FILE_DEFAULT = Path.home() / ".gemini" / "hooks.json"
BACKUP_DIR = Path.home() / ".gemini" / "backups"

def check_daemon_alive():
    xdg_runtime = os.environ.get("XDG_RUNTIME_DIR", f"/run/user/{os.getuid()}")
    sock_path = Path(xdg_runtime) / "astro-agent.sock"
    return sock_path.exists()

def atomic_write_json(target_path: Path, data: dict):
    # T08: Preserve symlinks
    actual_path = Path(os.path.realpath(target_path))
    actual_path.parent.mkdir(parents=True, exist_ok=True)
    temp_path = actual_path.with_name(actual_path.name + ".tmp")
    
    with open(temp_path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)
    os.chmod(temp_path, 0o600)
    # T09: Atomic replace
    os.replace(temp_path, actual_path)

def backup_config(target_path: Path) -> Path:
    if not target_path.exists():
        return None
    BACKUP_DIR.mkdir(parents=True, exist_ok=True)
    timestamp = int(time.time())
    backup_path = BACKUP_DIR / f"hooks.json.{timestamp}.bak"
    shutil.copy2(target_path, backup_path)
    os.chmod(backup_path, 0o600)
    return backup_path

def lock_and_run(func, *args):
    # T16: Lock to prevent simultaneous installations
    lock_file = Path.home() / ".gemini" / ".astro_config.lock"
    lock_file.parent.mkdir(parents=True, exist_ok=True)
    with open(lock_file, "w") as f:
        try:
            fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
            return func(*args)
        except BlockingIOError:
            print("Erro: Outro processo está alterando a configuração no momento.")
            sys.exit(1)

def do_install(config_path: Path, auto_confirm: bool):
    print("Iniciando instalação...")
    actual_path = Path(os.path.realpath(config_path))
    
    # T05: Se nem a pasta existe, o CLI pode não estar instalado
    if not actual_path.parent.exists() and not auto_confirm:
        print("Aviso: Diretório do Gemini CLI não detectado.")
        ans = input("Deseja criar a configuração mesmo assim? (s/N): ")
        if ans.lower() != "s":
            sys.exit(0)
            
    data = {}
    if actual_path.exists():
        try:
            with open(actual_path, "r", encoding="utf-8") as f:
                data = json.load(f)
        except Exception as e:
            # T06: Malformado
            print(f"Erro: Arquivo original inválido ou ilegível ({e}). Abortando.")
            sys.exit(1)

    new_data = dict(data)
    
    # Adicionar hooks preserving others
    for key, cmd in [("pre_tool_call", HOOK_CMD_PRE), ("post_tool_call", HOOK_CMD_POST)]:
        existing = new_data.get(key, [])
        if not isinstance(existing, list):
            existing = [existing] if existing else []
        if cmd not in existing:
            existing.append(cmd)
        new_data[key] = existing

    # T04: Idempotente
    if data == new_data:
        print("Hooks já estão instalados corretamente. Nenhuma alteração necessária.")
        return

    # Diff preview
    print("\n[Preview das Alterações]")
    print("Antes:")
    print(json.dumps(data, indent=2))
    print("\nDepois:")
    print(json.dumps(new_data, indent=2))
    print("-" * 30)
    
    if not auto_confirm:
        ans = input("Confirmar alteração? (s/N): ")
        if ans.lower() != 's':
            print("Operação cancelada. Nenhuma alteração realizada.")
            sys.exit(0)
    
    backup_config(actual_path)
    atomic_write_json(actual_path, new_data)
    print("Instalação concluída com sucesso.")

def do_uninstall(config_path: Path):
    print("Iniciando remoção...")
    actual_path = Path(os.path.realpath(config_path))
    if not actual_path.exists():
        print("Nenhuma configuração encontrada. Nada a remover.")
        return

    try:
        with open(actual_path, "r", encoding="utf-8") as f:
            data = json.load(f)
    except Exception as e:
        print(f"Erro: Arquivo original inválido ou ilegível ({e}). Abortando.")
        sys.exit(1)

    new_data = dict(data)
    changed = False

    for key, cmd in [("pre_tool_call", HOOK_CMD_PRE), ("post_tool_call", HOOK_CMD_POST)]:
        existing = new_data.get(key, [])
        if not isinstance(existing, list):
            existing = [existing] if existing else []
        
        # Check if user modified the command (conflict)
        for h in existing:
            if "agent-companion-hook" in h and h != cmd:
                # T12: Conflito detectado
                print(f"Conflito detectado em '{key}': Hook parece ter sido editado ('{h}').")
                print("Por segurança, o ASTRO não fará a remoção automática dessa chave.")
                print("Por favor, edite manualmente ou restaure o backup.")
                sys.exit(1)
        
        if cmd in existing:
            existing.remove(cmd)
            new_data[key] = existing
            changed = True
            # T11: Se ficar vazio, limpa
            if not existing:
                del new_data[key]

    if not changed:
        print("Os hooks do ASTRO não foram encontrados na configuração.")
        return

    backup_config(actual_path)
    atomic_write_json(actual_path, new_data)
    print("Remoção seletiva concluída com sucesso.")

def do_status(config_path: Path):
    actual_path = Path(os.path.realpath(config_path))
    cli_detected = actual_path.parent.exists()
    hook_installed = False
    
    if actual_path.exists():
        try:
            with open(actual_path, "r", encoding="utf-8") as f:
                data = json.load(f)
                pre = data.get("pre_tool_call", [])
                if HOOK_CMD_PRE in (pre if isinstance(pre, list) else [pre]):
                    hook_installed = True
        except:
            pass

    daemon_alive = check_daemon_alive()
    
    print(json.dumps({
        "cli_detected": cli_detected,
        "hook_installed": hook_installed,
        "daemon_alive": daemon_alive
    }, indent=2))

if __name__ == "__main__":
    check_antigravity()
    if len(sys.argv) < 2:
        print("Uso: astro_config_manager.py [install|uninstall|status] [--yes]")
        sys.exit(1)
        
    cmd = sys.argv[1]
    auto_confirm = "--yes" in sys.argv
    
    # Permite override via env para testes E2E (T01-T16)
    target = Path(os.environ.get("ASTRO_HOOKS_JSON", CONFIG_FILE_DEFAULT))
    
    if cmd == "install":
        lock_and_run(do_install, target, auto_confirm)
    elif cmd == "uninstall":
        lock_and_run(do_uninstall, target)
    elif cmd == "status":
        do_status(target)
    else:
        print("Comando inválido.")
        sys.exit(1)


def check_antigravity():
    brain_dir = Path.home() / ".gemini" / "antigravity-cli" / "brain"
    if brain_dir.exists():
        print("\n[ASTRO] Antigravity CLI v1.3+ detectado.")
        print("[ASTRO] Nenhuma injeção de hook necessária (File-Tail Adapter).")
