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

#### Opção 3: Windows 10/11 (controlar este PC Linux a partir do Windows)
Baixe `DesktopRDP-1.3.1-x64.msi` na aba [Releases](https://github.com/DiLucaYVL/DesktopRDP/releases) e execute (instala em *Arquivos de Programas* e cria atalhos **Área de trabalho remota** no Menu Iniciar e na Área de Trabalho; desinstale por *Configurações → Aplicativos*).

O app do Windows tem o **mesmo visual do app Linux** (Libadwaita, segue o tema claro/escuro do Windows), feito em **Rust nativo** (egui + Direct3D 12) — sem navegador embutido, um único `.exe` de ~8 MB. Ele:
- lista os PCs do seu Tailscale (Linux e Windows) e os PCs salvos;
- abre a sessão no cliente nativo do Windows (`mstsc.exe`), com a senha guardada no **Gerenciador de Credenciais do Windows** (nunca na linha de comando);
- oferece **usar todos os monitores** (como a opção "Usar todos os meus monitores" da Conexão de Área de Trabalho Remota), modo de tela, áudio remoto e área de transferência compartilhada.

No **PC Linux** que será controlado, ative o servidor RDP do GNOME: **Configurações → Sistema → Área de Trabalho Remota → Compartilhamento de Área de Trabalho** (ligue também **Controle Remoto**) e use o usuário/senha RDP mostrados ali.

Para gerar o `.msi` a partir do Linux (Rust + zig + wixl): veja `windows/build.sh`.

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

#### Windows 10/11 (control this Linux PC from Windows)
Download `DesktopRDP-1.3.1-x64.msi` from [Releases](https://github.com/DiLucaYVL/DesktopRDP/releases) and run it (installs to *Program Files* and adds **Remote Desktop** shortcuts to the Start menu and desktop; uninstall from *Settings → Apps*).

The Windows app has the **same look as the Linux app** (Libadwaita, follows the Windows light/dark theme) and is **native Rust** (egui + Direct3D 12) — no embedded browser, a single ~8 MB `.exe`. It:
- lists your Tailscale computers (Linux and Windows) and saved PCs;
- opens the session in the built-in Windows client (`mstsc.exe`), with the password kept in **Windows Credential Manager** (never on a command line);
- offers **use all monitors** (like Remote Desktop Connection's "Use all my monitors"), display mode, remote audio and shared clipboard.

On the **Linux PC** you want to control, enable GNOME's RDP server: **Settings → System → Remote Desktop → Desktop Sharing** (also turn on **Remote Control**) and use the RDP username/password shown there.

To build the `.msi` on Linux (Rust + zig + wixl): see `windows/build.sh`.

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

### Área de transferência com caracteres chineses / Clipboard turning into Chinese characters

O `xfreerdp` 3.x pode colar texto copiado no Windows como caracteres chineses (até a próxima cópia): quando a área de transferência remota muda com um pedido em andamento, a resposta atrasada é lida no formato errado (CF_TEXT como UTF-16). O patch `patches/freerdp/xf_cliprdr-stale-response.patch` corrige; para recompilar e instalar o `xfreerdp` corrigido (sem root):

FreeRDP 3.x `xfreerdp` may paste text copied on Windows as Chinese characters (until the next copy): when the remote clipboard changes while a request is in flight, the late answer is decoded in the wrong format (CF_TEXT as UTF-16). `patches/freerdp/xf_cliprdr-stale-response.patch` fixes it; to rebuild and install the patched `xfreerdp` (no root needed):

```bash
./scripts/build-patched-xfreerdp.sh
```
