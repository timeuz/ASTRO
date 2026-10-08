# ASTRO Sprint 3 - Release Candidate Decision

## Checklist de Validação Real
- [x] **Instalação e Ambiente Isolado**: Instalado via `astro_config_manager.py` idempotente. Hooks injetados de forma segura em `~/.gemini/hooks.json`. O CLI não travou e o fail-open foi acionado (0:00.00elapsed) quando o daemon não estava presente.
- [x] **Privacidade e Segurança**: As fixtures e logs comprovam que o payload do Gemini CLI (prompt, arquivos) é 100% omitido usando `{"redacted": true}`, sem persistir conteúdo sensível.
- [x] **Fail-Open e Recuperação**: Se o DBus ou daemon fecham, a sessão ativa é corretamente marcada como "Disconnected" no UI.
- [x] **Event Bridge (UI Real-time)**: Os eventos `transcript.jsonl` do Antigravity acionam o script `antigravity_adapter.py` que alimenta a UDS, passando via Rust Daemon para DBus (`EventReceived`), populando o Indicator Panel do GNOME Shell.
- [x] **Remoção de Configuração**: O script previne exclusão caso o desenvolvedor modifique os caminhos do hook (Conflito detectado) e preserva as chaves irmãs intactas.

## Erros e Limitações Conhecidos
1. O Antigravity CLI moderno envia telemetria e fluxos de agente de forma assíncrona para um diretório (`.gemini/antigravity-cli/brain/`), e ignora hooks manuais de `pre_tool_call` para processamento interno de agente autônomo. Desenvolvemos o **Log-Tail Adapter** (`antigravity_adapter.py`) como mitigação, porém ele exige um daemon Python adicional rodando em segundo plano.
2. A extensão do GNOME Shell precisa ser ativada manualmente pelo usuário na primeira execução no Wayland.

## Decisão de Lançamento da Equipe
**DECISÃO: PROSSEGUIR (com ressalvas)**

**Justificativa:** Todos os critérios P0 (ASTRO-12 a ASTRO-16) foram concluídos com sucesso. As regressões críticas (privacidade e fail-open) foram mitigadas. O adaptador Python (`antigravity_adapter.py`) resolve o problema de emissão do Antigravity CLI de forma passiva, cumprindo a meta principal da Sprint. Aprovado pelo *Security Engineer* e *Tester Agent* para o Beta.
