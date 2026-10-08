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

## 🚀 Como Fazer o Assistente Funcionar (Para Usuários Finais)

Siga os passos abaixo para testar, compilar e habilitar o ASTRO no seu ambiente de desenvolvimento.

### 1. Compilar os Binários e Iniciar o Daemon

O "cérebro" do ASTRO roda em background para orquestrar as mensagens de forma segura (garantindo sua privacidade).

```bash
# Entre na pasta do daemon e recompile os binários Rust
cd daemon
cargo build --release

# Inicie o Daemon em background (ele vai ouvir eventos no socket em $XDG_RUNTIME_DIR)
./agent-companiond/target/release/agent-companiond &
```

### 2. Ativar a Extensão do GNOME Shell

A extensão é a sua ponte visual. Para que ela apareça no painel superior:

```bash
# 1. Volte para a raiz do projeto e crie o link simbólico
cd ..
mkdir -p ~/.local/share/gnome-shell/extensions/
ln -s $(pwd)/extension ~/.local/share/gnome-shell/extensions/astro-spike@astro.project.org

# 2. Reinicie sua sessão do GNOME (Faça logoff e login novamente se estiver no Wayland).

# 3. Habilite a extensão
gnome-extensions enable astro-spike@astro.project.org
```
Você deverá ver um novo ícone (indicador de sessão) no seu painel superior!

### 3. Instalar o Hook no Agente de IA

Para que o agente (como o Antigravity) avise o ASTRO sempre que começar a pensar ou usar uma ferramenta, você deve habilitar o hook:

```bash
# Instala de forma segura os hooks no ~/.gemini/hooks.json ou equivalente
./scripts/install-hooks.sh
```
*(Dica: você pode rodar `./scripts/install-hooks.sh --preview` para visualizar as mudanças sem aplicá-las).*

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
