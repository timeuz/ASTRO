# Antigravity Adapter Spike Report

## Introdução
Este relatório investiga a viabilidade de integração do **Antigravity CLI (agy)** com o ecossistema ASTRO, baseado na experimentação em ambiente isolado no Linux.

## Análise de Eventos no Antigravity CLI
Diferente das versões clássicas do Gemini CLI que dependem de hooks definidos em `~/.gemini/hooks.json` (que acionam subprocessos síncronos), o Antigravity opera como um agente assíncrono complexo. 
Durante os testes na branch `feature/e2e-validation`, constatamos que o Antigravity CLI:
1. **Ignora hooks clássicos**: Tool calls gerados autonomamente pelos agentes (ex: `run_command`, `view_file`) **não acionam** o `pre_tool_call` e `post_tool_call` do arquivo `hooks.json` tradicional.
2. **Registro rico em tempo real (Transcripts)**: O Antigravity CLI mantém um registro detalhado e estruturado (JSONL) de todas as etapas em `~/.gemini/antigravity-cli/brain/<conversation_id>/.system_generated/logs/transcript.jsonl`.
   - Início de sessão/User Input (`source: USER_EXPLICIT`)
   - Atividades e Tool Calls (`source: MODEL`, `type: PLANNER_RESPONSE`, contendo a chave `tool_calls`)
   - Respostas de Ferramentas (`source: SYSTEM`, `type: TOOL_RESPONSE`)

## Viabilidade para o Próximo Adaptador

Com base nessas descobertas, recomendamos as seguintes abordagens arquitetônicas para o Adaptador Antigravity:

### Opção A: Log-Tail Adapter (Recomendada - Baixa latência e não-intrusiva)
O adaptador ASTRO não dependerá de injeção de hooks. Em vez disso, usaremos mecanismos de *file-watching* inotify/eBPF no diretório de `brain/`.
- **Início de Sessão**: Criação de um novo diretório `<conversation_id>`.
- **Atividades**: O daemon do ASTRO faz um "tail" (leitura contínua) do arquivo `transcript.jsonl`. Ao detectar a chave `tool_calls`, o adaptador dispara um evento DBus para a extensão GNOME renderizar a atividade na UI.
- **Vantagem**: Totalmente passivo, fail-open por natureza (não tem como bloquear o CLI).

### Opção B: Custom MCP Server (Intrusivo, mas interativo)
Criar um servidor MCP (Model Context Protocol) chamado `astro-observer`. O agente Antigravity faria chamadas explícitas para este servidor (`call_tool: astro_notify_activity`).
- **Desvantagem**: Requer que o agente ativamente decida relatar suas ações, o que pode consumir tokens e diminuir a performance.

## Conclusão
O modelo de hooks síncronos usado no passado não é adequado nem totalmente suportado pelo fluxo assíncrono do Antigravity CLI. O próximo adaptador deve focar em **File System Observability** dos transcripts (Opção A) para alimentar o daemon Rust via UDS, mantendo o design **Fail-Open** validado durante nossa execução end-to-end.
