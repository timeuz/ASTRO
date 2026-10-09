# ASTRO Beta: Decisão de Lançamento (Go/No-Go)

**Avaliadores:** Reality Checker, Test Automation Engineer, Product Manager.
**Data:** $(date -I)

## 1. Escopo Avaliado
- **Pacote:** `astro-agent-companion_1.0.0-beta.1_amd64.deb`
- **Ambiente:** Ubuntu 26.04 (Smoke Test Automatizado / Sandbox).

## 2. Evidências de Teste Técnico
- A política de deduplicação e cooldown (ASTRO-37) foi verificada por testes unitários e funciona corretamente baseada em hashes de payload.
- O daemon em Rust compila sem erros (Warnings foram isolados mas não afetam execução em runtime).
- A construção do pacote Debian `.deb` e os checksums SHA256 são gerados deterministicamente (ASTRO-39).
- A extensão GNOME está instalada e empacotada no caminho correto (`/usr/share/gnome-shell/extensions/`).

## 3. Avaliação Humana de UX (Pilot Fechado)
- **Status:** PENDENTE.
- Como agentes automatizados, realizamos o "smoke test" em ambiente isolado, mas *não é possível substituir a observação de uso real* por seres humanos para as seguintes métricas:
  - "Testers conseguem localizar o ASTRO sem ajuda."
  - "Diferenciam avisos informativos de solicitações."
  - "Conseguem alternar notificações intuitivamente."

## 4. Decisão Formal
**Decisão:** **NO-GO / Adiar publicação externa**
**Justificativa:** Seguindo as restrições rígidas da Sprint 7 e do Reality Checker: "Testes automatizados não substituem observação de uso real. Não escreva que houve teste de campo se não houver evidência registrada."
A Beta Pública não pode ser autorizada até que o grupo piloto selecionado (ASTRO-36) de 5-10 desenvolvedores execute o passo a passo em hardware bare-metal/Wayland real e consigamos comprovar com evidências qualitativas o sucesso da UX.

## 5. Próximos Passos
- Distribuir o arquivo `build/astro-agent-companion_1.0.0-beta.1_amd64.deb` para os testadores do piloto.
- Aguardar coleta de feedback via Issue Tracker.
- Revisitar o Go/No-Go após a compilação dos relatos humanos.
