# DesktopRDP

**Remote Desktop Linux** (English) / **Área de trabalho remota** (Português) — A simple native **GTK4 / Libadwaita** GUI client for **FreeRDP** with automatic **Tailscale** discovery and direct IP/username/password login.

O aplicativo adapta automaticamente o nome no menu do sistema, o título da janela e toda a interface de acordo com o idioma do usuário (`pt_BR` → **Área de trabalho remota**, demais idiomas → **Remote Desktop Linux**). Você também pode forçar o idioma pela linha de comando usando `--lang pt_BR` ou `--lang en`.

---

## Português (pt-BR)

### Recursos
- **Aplicação unificada (janela única no Dock/Alt-Tab):** ao conectar, a janela inicial se oculta e a sessão RDP nativa (`xfreerdp`) abre agrupada sob o mesmo ícone e nome da aplicação (**Área de trabalho remota**), voltando à tela principal ao encerrar a sessão.
- **Descoberta automática via Tailscale:** lista seus computadores da rede Tailscale (`tailscale status --json`) e permite preencher o IP ou conectar com 1 clique.
- **Conexão direta por IP / Host:** suporte a IP da rede local, IP Tailscale, `IP:PORTA` (padrão `3389`), `DOMINIO\usuario`, `.\usuario` ou conta Microsoft.
- **Perfis salvos e senhas seguras:** salva os PCs em `~/.config/remote-desktop-linux/profiles.json` (`0600`) e armazena as senhas no **GNOME Keyring** (`libsecret`). A senha nunca é exposta na linha de comando (`argv`), sendo enviada com `/from-stdin:force`.
- **Opções práticas da sessão:** janela redimensionável (`+dynamic-resolution`), tela cheia (`+f`, saia com `Ctrl+Alt+Enter`), `1920 × 1080` ou `1366 × 768`, áudio e compartilhamento da pasta `~/Downloads`, além do botão **Testar Conexão**.

### Instalação

#### Opção 1: Pacote `.deb` (Ubuntu / Debian e derivados)
Baixe o arquivo `remote-desktop-linux_1.2.1_all.deb` na aba [Releases](https://github.com/DiLucaYVL/DesktopRDP/releases) e instale com:

```bash
sudo apt install ./remote-desktop-linux_1.2.1_all.deb
```

#### Opção 2: Instalação local no usuário (qualquer Linux)
```bash
git clone https://github.com/DiLucaYVL/DesktopRDP.git
cd DesktopRDP
./install.sh
```

Para desinstalar na instalação local:
```bash
./uninstall.sh
```

---

## English

### Features
- **Single-App Unified Lifecycle:** hides the launcher while the native `xfreerdp` session is open and binds both windows to the same desktop app ID (`WM_CLASS` + `GIO_LAUNCHED_DESKTOP_FILE`), keeping a single icon in the Dock and Alt-Tab.
- **Tailscale Auto-Discovery:** detects Windows/desktop peers from `tailscale status --json` so you can fill the IP or launch in one click.
- **Direct IP / Host Access:** supports LAN IP, Tailscale IP, `HOST:PORT` (default `3389`), `DOMAIN\user`, `.\user`, or Microsoft accounts.
- **Saved Profiles & GNOME Keyring:** saves profiles in `~/.config/remote-desktop-linux/profiles.json` (`0600`) and stores passwords in **GNOME Keyring** (`libsecret`), passing credentials securely over stdin (`/from-stdin:force`).
- **Automatic Language Support:** switches automatically between **Remote Desktop Linux** (`en`) and **Área de trabalho remota** (`pt_BR`) based on system locale (`LC_ALL`, `LC_MESSAGES`, `LANGUAGE`, `LANG`), or manually via `--lang en` / `--lang pt_BR`.

### Installation

#### Debian / Ubuntu (`.deb`)
Download `remote-desktop-linux_1.2.1_all.deb` from [Releases](https://github.com/DiLucaYVL/DesktopRDP/releases):
```bash
sudo apt install ./remote-desktop-linux_1.2.1_all.deb
```

#### User-level Install (Any Linux Distro)
```bash
git clone https://github.com/DiLucaYVL/DesktopRDP.git
cd DesktopRDP
./install.sh
```

---

## Prerequisites / Pré-requisitos (FreeRDP)

A aplicação prioriza o **FreeRDP nativo (`xfreerdp`)** para manter a integração em aplicativo único no GNOME/Dock:

- **Via APT (Ubuntu / Debian, recomendado):**
  ```bash
  sudo apt install freerdp-x11 python3-gi gir1.2-gtk-4.0 gir1.2-adw-1 gir1.2-secret-1
  ```
- **Via Flatpak (alternativa):**
  ```bash
  flatpak install --user flathub com.freerdp.FreeRDP
  ```
