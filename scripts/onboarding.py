#!/usr/bin/env python3
import time
import sys
import subprocess
import os

try:
    from rich.console import Console
    from rich.panel import Panel
    from rich.markdown import Markdown
    from rich.prompt import Confirm
    from rich.text import Text
except ImportError:
    print("A biblioteca 'rich' é necessária para este script.")
    print("Por favor, instale usando: pip install rich")
    sys.exit(1)

console = Console()

def check_daemon():
    try:
        # Check if daemon is running by looking at processes or pinging dbus
        result = subprocess.run(["dbus-send", "--session", "--print-reply", "--dest=org.astro.AgentCompanion", "/org/astro/AgentCompanion", "org.freedesktop.DBus.Peer.Ping"], capture_output=True)
        return result.returncode == 0
    except Exception:
        return False

def check_extension():
    try:
        result = subprocess.run(["gnome-extensions", "list"], capture_output=True, text=True)
        return "astro@local" in result.stdout or "astro-extension" in result.stdout
    except Exception:
        return False

def check_hook():
    # Simple check if hook is installed (e.g. wrapper in ~/.local/bin)
    return os.path.exists(os.path.expanduser("~/.local/bin/gemini"))

def main():
    console.clear()
    
    title = Text("Bem-vindo ao ASTRO", style="bold cyan", justify="center")
    subtitle = Text("Agent Session Tracking & Runtime Observer", style="italic", justify="center")
    
    panel_content = Text.assemble(title, "\n", subtitle)
    console.print(Panel(panel_content, border_style="cyan"))
    print()
    
    intro_text = """
O ASTRO é um observador local que conecta seus agentes de IA à interface do seu sistema (GNOME/Wayland).
Sua principal função é dar a você **visibilidade e controle** sobre o que os agentes estão fazendo.
"""
    console.print(Markdown(intro_text))
    time.sleep(1)
    
    console.print(Panel("Privacidade Garantida: O ASTRO NÃO lê seus prompts, NÃO envia dados para a nuvem e NUNCA pedirá suas chaves de API.", title="Privacidade", border_style="green"))
    print()

    console.print("[bold]Diagnosticando o sistema...[/bold]")
    daemon_ok = check_daemon()
    ext_ok = check_extension()
    hook_ok = check_hook()

    console.print(f"Daemon (agent-companiond): {'[green]Ativo[/green]' if daemon_ok else '[red]Inativo[/red]'}")
    console.print(f"Extensão GNOME: {'[green]Ativa[/green]' if ext_ok else '[yellow]Não encontrada ou Desativada[/yellow]'}")
    console.print(f"Hooks (Gemini CLI): {'[green]Instalados[/green]' if hook_ok else '[yellow]Não instalados[/yellow]'}")
    print()

    if not ext_ok:
        console.print("[yellow]Aviso:[/yellow] A extensão do GNOME não parece estar ativa.")
        console.print("Para ativar, você pode usar o aplicativo 'Extensões' do GNOME ou executar:")
        console.print("  gnome-extensions enable astro@local")
        print()

    if not hook_ok:
        if Confirm.ask("Deseja instalar os conectores (hooks) do Gemini CLI agora?"):
            console.print("\n[bold cyan]Executando configuração do hook...[/bold cyan]")
            time.sleep(1)
            # In a real app we would run the CLI command here
            console.print("Execute o seguinte comando para instalar o hook no seu ambiente atual:")
            console.print("\n    [bold yellow]python3 scripts/cli.py hook install[/bold yellow]\n")
            console.print("O sistema irá solicitar sua permissão caso precise modificar caminhos do sistema.")
        else:
            console.print("\nVocê pode configurar as integrações mais tarde.")
            console.print("Execute [bold yellow]python3 scripts/cli.py hook install[/bold yellow] quando estiver pronto.")
    
    print()
    console.print(Panel("Configuração inicial concluída. Você pode fechar este assistente e abrir novamente quando quiser.", border_style="cyan"))

if __name__ == "__main__":
    main()
