# Guia de Instalação e Uso: ASTRO Beta Fechada (RASCUNHO)

**Aviso:** Este guia destina-se apenas a participantes do piloto fechado. Não redistribua.

## 1. Visão Geral
O ASTRO (Agent Session Tracking & Runtime Observer) monitora de forma passiva a execução local de seus agentes (Gemini CLI, Antigravity) e apresenta o estado atual na sua barra de tarefas do GNOME, sem interromper o seu fluxo de trabalho.
**Privacidade:** O ASTRO não envia telemetria e os logs de atividade são mantidos apenas localmente. O daemon não tem acesso às chaves de API dos agentes.

## 2. Instalação do Pacote
1. Baixe o artefato Debian (`astro-agent-companion_1.0.0-beta.1_amd64.deb`).
2. Instale o pacote:
   ```bash
   sudo dpkg -i astro-agent-companion_1.0.0-beta.1_amd64.deb
   sudo apt-get install -f # para dependências (ex: jq)
   ```

## 3. Ativação do Serviço e Extensão
1. **Recarregue os daemons de usuário e habilite o ASTRO:**
   ```bash
   systemctl --user daemon-reload
   systemctl --user enable --now astro-companion.service
   ```
2. **Habilite a Extensão GNOME:**
   ```bash
   gnome-extensions enable astro-spike@astro.project.org
   ```

## 4. Conectando os Agentes (Setup)
Execute a ferramenta de diagnóstico para checar se está tudo operando corretamente:
```bash
python3 /usr/share/astro-agent/scripts/onboarding.py
```
O script mostrará o status do daemon e da extensão, e oferecerá a opção de instalar o Hook do Gemini CLI automaticamente.

## 5. Como usar e interagir
* **Ver Sessões Ativas:** Clique no ícone do ASTRO no canto superior direito. 
* **Notificações:** O ASTRO só emitirá um popup se o agente Falhar, Completar, ou pedir Permissão/Entrada.
* **Modo Neutro:** Você pode esconder textos de microcopy detalhados clicando no ícone e ativando "Modo Neutro". O sistema passará a mostrar alertas padronizados ("Requer sua atenção.").

## 6. Remoção (Uninstall)
```bash
systemctl --user disable --now astro-companion.service
sudo dpkg -r astro-agent-companion
```
