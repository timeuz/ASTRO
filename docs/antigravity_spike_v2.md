# ASTRO-18: Antigravity Events Spike

## Environment
- **OS**: Ubuntu 26.04.1 LTS
- **Target Version**: Antigravity CLI v1.3.2
- **Integration Method**: Log-Tail / File System Observability via `~/.gemini/antigravity-cli/brain/<conversation_id>/.system_generated/logs/transcript.jsonl`.
- **Hooks status**: The classic `~/.gemini/hooks.json` is not consistently triggered for all background tools in this version.

## Observable Events
The `transcript.jsonl` provides a very rich JSON stream.
- **Session ID**: `<conversation_id>` (folder name)
- **Start**: Detected by `type: "USER_INPUT"`
- **Activity (Tool Call)**: Detected by `type: "PLANNER_RESPONSE"` containing `"tool_calls"`.
- **Conclusion/Idle**: Detected by `type: "PLANNER_RESPONSE"` without `"tool_calls"`.
- **Error**: Detected by `type: "GENERIC"` with `"status": "ERROR"`.

## Limitations
- **Questions & Permissions**: Antigravity uses tools like `ask_question` and requires CLI interaction. Capturing this interactively without blocking is currently unfeasible solely via log-tail. The Log-Tail method is read-only. We cannot safely "answer" questions via ASTRO yet. We will mark permissions/questions as **Not Supported** for Antigravity in the current UI iteration.
- **Risks**: File system tailing is generally safe (read-only), meaning it natively supports Fail-Open without any risk of blocking the CLI.

## Security Risk Assessment (ASTRO-20)
Dado que a integração depende de observar `transcript.jsonl` (um arquivo de log gerado pelo agente), o risco é passivo (ASTRO não bloqueia o Antigravity). Contudo, há um leve risco de *path traversal* ou parse de dados incorretos se o arquivo não for JSON válido. O adaptador no ASTRO deve tratar parses de JSON rigorosamente com blocos `try/catch`. Não modificaremos arquivos de configuração do Antigravity.
