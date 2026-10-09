# Especificação de UX e UI do ASTRO

## 1. Posicionamento e Presença
O ASTRO atua exclusivamente através do **Painel Superior do GNOME** (Status Area).
Ele não abre janelas flutuantes sem solicitação, não emite popups na tela de trabalho e não rouba o foco do teclado de outras aplicações. 

## 2. Estados Visuais do Ícone
O ícone do painel se altera para refletir instantaneamente a saúde do daemon e o status das sessões:
- **Inativo (Ocioso):** Ícone neutro (`system-run-symbolic` com opacidade 70%). Significa que o daemon está rodando, mas nenhum agente está em atividade.
- **Trabalhando:** Ícone destacado com badge numérico. A badge indica quantas sessões estão ativas simultaneamente.
- **Atenção Necessária:** Badge ganha cor de alerta (ex: Laranja) e pulsa suavemente (quando animações habilitadas). Usado quando o Gemini CLI exige aprovação (Aguardando Input) ou ocorreu falha.
- **Desconectado/Erro:** Ícone com símbolo de falha ou "X", opacidade reduzida, sem badge numérico. (Ex: Daemon crashou ou D-Bus inacessível).

## 3. Comportamento do Popover
- Clicar no ícone abre o Popover de Sessões.
- O popover agrupa sessões em "Gemini CLI" e "Antigravity".
- Cada item na lista exibe um resumo. Clicar em um item deve **expandir os detalhes** da sessão.
- Fechar e reabrir o popover não descarta o estado expandido/selecionado, desde que a sessão permaneça ativa.
- Estado Vazio: Se a `sessionCount` for 0, o popover exibe o Empty State: *"Nenhum agente em atividade. Quando o Gemini CLI ou Antigravity processarem comandos, eles aparecerão aqui."*

## 4. Políticas de Notificação Nativas
O ASTRO respeita a concentração do usuário. Notificações do GNOME são enviadas apenas em:
1. Conclusão de uma tarefa importante (Opcional).
2. Erro fatal em tempo de execução.
3. Pedido de permissão do usuário.
*Restrição:* Transições triviais de estado ("Lendo arquivo...", "Buscando web...") não geram notificações.
