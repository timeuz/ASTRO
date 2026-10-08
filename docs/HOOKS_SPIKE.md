# Spike: Hooks do Antigravity / Gemini CLI

O CLI do Antigravity permite configurar hooks em `~/.gemini/hooks.json` (ou `.agents/hooks.json` no workspace).
Estes hooks são acionados durante o ciclo de vida do agente.

## Contrato Descoberto

- `pre_tool_call`: Disparado antes do agente chamar uma ferramenta (tool). Recebe dados sobre a ferramenta que será chamada (no stdin ou como argumento) e pode vetar a execução retornando exit code != 0.
- `post_tool_call`: Disparado após a ferramenta ser executada.

### Comportamento Observado
- Os argumentos fornecidos aos hooks geralmente incluem os nomes das tools e o contexto da sessão (conversation_id).
- O payload de entrada vem pelo `stdin` (formato JSON) descrevendo o tool call e seus parâmetros.

### Sanitização (Privacidade e Segurança)
- O relay deve sempre truncar ou ocultar os valores literais do payload da ferramenta, enviando apenas o nome do agente, ID da sessão e o tipo de evento (ex: `working`, `waiting_permission`), descartando completamente tokens sensíveis e conteúdo do código fonte lido/escrito pelo agente.

## Testando o Spike Local
1. Adicione o seguinte ao `~/.gemini/hooks.json`:
```json
{
  "pre_tool_call": ["/caminho/absoluto/para/ASTRO/scripts/spike_hook.sh"]
}
```
2. Invoque o agente no terminal.
3. Leia o arquivo `.spike_hook.log` gerado na raiz do projeto.
