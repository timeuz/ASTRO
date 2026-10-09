# ASTRO Beta: Decisão Formal de UX - Presença e Notificações

## O Problema
Sistemas de IA autônomos geram um alto volume de eventos de transição de estado ("Pensando", "Trabalhando", "Lendo arquivo", etc). Se cada um desses eventos gerasse um alerta ou tomasse o foco da tela do usuário, a ferramenta rapidamente se tornaria invasiva, causando fadiga de notificações e interrompendo o fluxo de trabalho principal.

## A Diretriz Central: "Sem popups soltos, sem interrupções de foco"
Para a versão Beta fechada e a posterior Release Candidate, a equipe do ASTRO decidiu adotar um modelo rigoroso de UI/UX, ancorado nos seguintes princípios:

1. **Estado Contínuo Restrito ao Painel (ASTRO-30/ASTRO-32)**
   As transições de estado normais (como a execução contínua de um subagente) alteram o ícone no Top Bar do GNOME e as informações detalhadas no popover (feed), mas **não disparam alertas**. O usuário pode acompanhar em tempo real, se desejar, apenas abrindo o indicador do ASTRO.

2. **Deduplicação e Cooldown no Daemon (ASTRO-33)**
   O `agent-companiond` atua como um filtro rigoroso:
   - Notificações de desktop (`notify-send` via dbus) ocorrem APENAS para eventos críticos: `Completed`, `Failed`, `WaitingInput` ou `WaitingPermission`.
   - É aplicado um cooldown de **5 minutos** (300 segundos) para o mesmo agente/sessão/estado, evitando rajadas de notificações em caso de loops de erro ou retry.

3. **Onboarding Transparente (ASTRO-34)**
   Adotamos um script de onboarding interativo em terminal (`scripts/onboarding.py`) que deixa explícito desde o primeiro minuto que:
   - O ASTRO é um observador passivo local.
   - Não há leitura de chaves de API.
   - Não há transmissão externa de telemetria.
   - A instalação do hook (`astro-cli hook install`) é o único ponto de interceptação local.

## Conclusão
Com essas implementações, a Sprint 6 garante que o ASTRO se comporte como um painel de instrumentos profissional: ele relata a verdade sobre a operação dos agentes, mas nunca grita com o usuário a menos que a intervenção seja absolutamente necessária para prosseguir. 

Este modelo de UX será a base para a Beta fechada do Ubuntu 26.04.
