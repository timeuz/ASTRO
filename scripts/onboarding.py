#!/usr/bin/env python3
import time
import sys

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

def print_slow(text, delay=0.01):
    for char in text:
        sys.stdout.write(char)
        sys.stdout.flush()
        time.sleep(delay)
    print()

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
    
    arch_text = """
### Como Funciona

1. **Daemon Local**: Um processo em segundo plano seguro que recebe sinais dos agentes.
2. **Extensão GNOME**: Uma interface no painel superior que mostra o estado da sessão.
3. **Hooks**: Conectores instalados nos agentes (como o Gemini CLI) que enviam os eventos.
"""
    console.print(Markdown(arch_text))
    time.sleep(1)
    
    privacy_text = """
### Privacidade e Segurança em Primeiro Lugar

* O ASTRO **não lê** o conteúdo dos seus prompts ou as respostas da IA.
* O ASTRO **não envia** nenhum dado para a nuvem. Toda a comunicação é local (Unix Domain Sockets / D-Bus).
* O ASTRO **não rouba** o foco da tela. As atualizações aparecem discretamente no painel superior.
* As chaves de API devem ser gerenciadas individualmente por cada agente. O ASTRO não irá solicitá-las.
"""
    console.print(Panel(Markdown(privacy_text), title="Privacidade Garantida", border_style="green"))
    print()
    time.sleep(1)
    
    if Confirm.ask("Deseja configurar a integração com o Gemini CLI agora?"):
        console.print("\n[bold cyan]Executando configuração do hook...[/bold cyan]")
        time.sleep(1)
        console.print("Para instalar o hook do Gemini CLI no seu ambiente atual, execute o comando:")
        console.print("\n    [bold yellow]python3 scripts/cli.py hook install[/bold yellow]\n")
        console.print("Depois de instalado, as atividades do seu Gemini CLI aparecerão automaticamente no indicador do ASTRO no topo da tela.")
    else:
        console.print("\nVocê pode configurar as integrações mais tarde.")
        console.print("Execute [bold yellow]python3 scripts/cli.py hook install[/bold yellow] quando estiver pronto.")
        
    print()
    console.print(Panel("Configuração inicial concluída. O ASTRO está pronto para observar seus agentes.", border_style="cyan"))

if __name__ == "__main__":
    main()
