# ADR 001: Superfície de UI para a Beta (GNOME vs TUI)

## Contexto
O ASTRO (Agent Session Tracking & Runtime Observer) atua como daemon local processando eventos enviados por agentes de IA. Atualmente, possuímos duas superfícies de interface desenvolvidas:
1. Uma **Extensão GNOME Shell**, que exibe indicadores de estado nativos no painel superior e um popover com lista de sessões.
2. Um script de **Onboarding em Terminal** (`scripts/onboarding.py`) usando `rich` para diagnóstico e guias de instalação.

Existe o questionamento (ASTRO-38) se deveríamos construir uma Terminal User Interface (TUI) completa (usando Textual ou similar) como entrada primária, seja para simplificar a beta ou como alternativa.

## Avaliação

### 1. Extensão GNOME
- **Prós:** Integração nativa no desktop. O usuário não precisa manter um terminal aberto para acompanhar o agente em background. Usa `notify-send` de forma integrada. Já está 100% implementada e testada no Wayland.
- **Contras:** Restrito a usuários de GNOME. Requer processo de ativação via `gnome-extensions enable`.

### 2. TUI (Terminal User Interface)
- **Prós:** Universal no Linux (independe de Window Manager/Desktop Environment). Altamente acessível para desenvolvedores de backend.
- **Contras:** TUI exigiria duplicar a lógica de feed, botões de ação e empty state já construídos no GJS (Javascript da extensão). Obriga o usuário a ter uma janela de terminal permanentemente aberta para não perder a notificação de status.

## Decisão (UX Architect & Desktop App Engineer)
Para a **Release Candidate e Piloto Fechado da Beta**, a decisão é:
- **Manter o GNOME Shell como cliente principal de visualização.** 
- **O script de terminal atual (`scripts/onboarding.py`)** atuará estritamente como "Diagnóstico e Onboarding" em modo CLI (CLI-wizard), sendo o responsável exclusivo pelo setup de hooks e verificação do daemon. Não vamos evoluí-lo para uma TUI de tempo real no momento.

## Consequências
1. Não duplicaremos a máquina de estados. O foco permanece em refinar o pacote Debian e o manifesto do GNOME.
2. A documentação (ASTRO-40) deve tratar a chamada no terminal apenas como setup inicial.
