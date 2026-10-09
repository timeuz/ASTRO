# A.S.T.R.O. - Agent Session Tracking & Runtime Observer

**A.S.T.R.O.** é um Companion Desktop Linux-First focado em monitorar agentes de Inteligência Artificial em tempo real de forma nativa e integrada ao Ubuntu (GNOME/Wayland). Ele fornece uma interface discreta, segura e observável para acompanhar o que os agentes autônomos (como o Antigravity / Gemini CLI) estão fazendo no seu sistema.

---

## 🎯 Arquitetura e Componentes

O projeto segue a governança rigorosa do Gitflow e foi construído com a seguinte stack:

- **Backend Daemon (`agent-companiond`)**: Construído em Rust + Tokio, servindo em um Unix Domain Socket isolado (D-Bus e `$XDG_RUNTIME_DIR`) com fail-open e persistência rotativa.
- **Relay/Hook (`agent-companion-hook`)**: Binário ultraleve em Rust que envia os eventos do agente para o daemon em <500ms, impossibilitando que a execução original trave (fail-open).
- **Frontend GNOME Extension**: Interface em GJS para o painel superior do GNOME Shell, com suporte a feed cronológico truncado, sem dependência de janelas flutuantes que quebram no Wayland.
- **Settings App**: Aplicação moderna GTK4 + libadwaita (`app/main.py`) para gerenciar as preferências do usuário.
- **Motor de Personalidade**: Um sistema de *microcopy* (`microcopy.json`) determinístico com tom de voz adaptável e travas estritas de neutralidade para cenários de risco.

---

## 🚀 Como Instalar e Testar o Assistente

Criamos um script automatizado que faz todo o trabalho duro de compilação e configuração do ambiente para você de forma segura.

### 1. Instalação Automática

Para compilar os binários em Rust, configurar a infraestrutura de backend e plugar os interceptadores do Antigravity CLI, basta rodar na raiz do projeto:

```bash
# Você pode visualizar o que o script fará usando:
./install.sh --preview

# Para efetivar a instalação no seu usuário:
./install.sh
```

Isso fará o seguinte:
- Checa dependências (Rust, Python, Git).
- Compila o `agent-companiond` e o `agent-companion-hook` em release.
- Move o Daemon para `~/.local/bin/agent-companiond`.
- Copia a personalidade (`microcopy.json`) para a sua pasta de configuração `~/.config/astro-agent-companion/`.
- Instala os hooks integrados de maneira idempotente com backup automático.

### 2. Iniciar o Daemon

Com a instalação finalizada, você pode subir o motor do ASTRO:

```bash
# Inicie o Daemon em background (ele vai ouvir eventos no socket isolado em $XDG_RUNTIME_DIR)
agent-companiond &
```

### 3. Ativar a Extensão do GNOME Shell

A extensão é a sua ponte visual. Para ativá-la:

```bash
# 1. Crie o link simbólico
mkdir -p ~/.local/share/gnome-shell/extensions/
ln -s $(pwd)/extension ~/.local/share/gnome-shell/extensions/astro-spike@astro.project.org

# 2. Reinicie sua sessão do GNOME (Faça logoff e login novamente se estiver no Wayland).

# 3. Habilite a extensão
gnome-extensions enable astro-spike@astro.project.org
```
Você deverá ver o indicador de sessão no seu painel superior!

### 4. Desinstalação

Caso precise remover a infraestrutura limpa, rode:

```bash
./install.sh --uninstall
```

### 4. Personalizar e Configurar (GTK4 App)

Quer mudar a quantidade de notificações ou ajustar o nível de humor do assistente para o "Modo Discreto"?

```bash
# Execute o aplicativo GTK4
python3 app/main.py
```

---

## 🔐 Privacidade Local-First

A arquitetura do ASTRO implementa os seguintes princípios inegociáveis:
- Validação UID: Somente o seu próprio usuário (`libc::geteuid()`) consegue mandar eventos para o Socket.
- Permissões Estritas: O Socket no XDG_RUNTIME_DIR roda travado com permissões `0600`.
- Redação Automática: Payloads contendo prompts ou código que passam pelos canais de log IPC são substituídos imediatamente por `[REDACTED]`.

---

## 🛠️ Contribuindo (Governance)

Leia as regras estritas em nosso [`AGENTS.md`](./AGENTS.md). 
- Use a branch `develop` para testar implementações antes da `main`.
- Todas as implementações críticas (segurança, kernel, IPC) exigem *Cross-Code-Review*.
- Rode `./tests/integration.sh` antes de efetuar commits.
