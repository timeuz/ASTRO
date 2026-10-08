# ASTRO Testing Guide

Este documento define a linha de base de testes para os motores do ASTRO (`agent-companiond` e `agent-companion-hook`).

## Testes Unitários

Para rodar a suíte de testes unitários do Daemon:

```bash
cargo test --manifest-path daemon/agent-companiond/Cargo.toml
```

Para rodar os testes do Hook:

```bash
cargo test --manifest-path daemon/agent-companion-hook/Cargo.toml
```

## Checkpoint da Linha de Base

- **Data:** 2026-10-08
- **Módulos Testados:** Tratamento de severidade, fallback de microcopy.
- **Status:** PASS
