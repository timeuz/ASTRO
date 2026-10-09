# ASTRO Beta Pilot Plan (Sprint 7)

## 1. Escopo e Ambiente
- **OS:** Ubuntu 26.04 LTS
- **Arquitetura:** amd64 (x86_64)
- **Interface Gráfica:** GNOME Shell (Wayland nativo)
- **Superfície a testar (Decisão ASTRO-38 pendente):** Extensão GNOME (prioridade atual) vs TUI (a avaliar).
- **Agentes Suportados Oficialmente:** Gemini CLI, Antigravity (adaptador).

## 2. Participantes e Responsabilidades
- **Público do Piloto:** 5 a 10 desenvolvedores de software com experiência em CLI e agentes locais. (Nenhum teste com usuário real ocorreu ainda. Resultados não serão inventados).
- **Prazo do Piloto:** 14 dias a partir da aprovação da Release Candidate.
- **Responsável por Triagem:** Product Manager (coordenado pelo UX Researcher).
- **Tarefas do Participante:**
  1. Instalar o pacote `.deb`.
  2. Executar o onboarding.
  3. Instalar o hook do Gemini CLI ou Antigravity.
  4. Executar tarefas cotidianas com a CLI.
  5. Relatar se o comportamento visual (GNOME/TUI) e as notificações são úteis e não-intrusivas.

## 3. Matriz de Suporte e Estados
| Estado | Gemini CLI | Antigravity |
| --- | --- | --- |
| Detectado (daemon rodando) | ✅ Sim | ✅ Sim |
| Hook/Adapter Instalado | ✅ Sim (via `astro-cli hook install`) | ✅ Sim (log tailing/IPC) |
| Evento Recebido (UDS/DBus) | ✅ Sim | ✅ Sim |
| Sessão Exibida na UI | ✅ Sim | ✅ Sim |
| Ação Interativa (Mute, etc) | ✅ Sim | ⚠️ Parcial (Mute sem suporte no log-tail) |

## 4. Limitações Conhecidas e Ambiente Mínimo
- **Ambiente Mínimo:** Ubuntu 26.04 com `systemd --user` habilitado e dbus de sessão ativo.
- **Privacidade:** O ASTRO não lê prompts ou respostas completas. Dados de log localizados em `~/.local/state/astro-agent`.
- **Limitação:** Agentes que não expõem logs estruturados (Antigravity no modo legado) possuem integração limitada de interatividade.
