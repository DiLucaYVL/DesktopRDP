# DesktopRDP — Remote Desktop Linux (Área de trabalho remota)

Cliente gráfico simples e moderno em **GTK4 / Libadwaita** para acessar máquinas **Windows** no Linux via **FreeRDP**, com descoberta automática de computadores no **Tailscale** ou conexão direta por **IP, usuário e senha**.

## Recursos

- **Descoberta automática via Tailscale:** lista seus computadores da rede Tailscale (`tailscale status --json`) e permite preencher o IP ou conectar com 1 clique.
- **Conexão direta por IP / Host:** suporte a IP da rede local, IP Tailscale, `IP:PORTA` (padrão `3389`), `DOMINIO\usuario`, `.\usuario` ou conta Microsoft.
- **Perfis salvos e senhas seguras:** salva os PCs em `~/.config/remote-desktop-linux/profiles.json` (`0600`) e armazena as senhas no **GNOME Keyring** (`libsecret`). A senha nunca é exposta nos argumentos de linha de comando (`argv`), sendo enviada com `/from-stdin:force`.
- **Opções práticas da sessão:**
  - Janela redimensionável (`+dynamic-resolution`), Tela cheia (`+f`, saia com `Ctrl+Alt+Enter`), `1920 × 1080` ou `1366 × 768`.
  - Redirecionamento de áudio e compartilhamento da pasta `~/Downloads`.
  - Botão **Testar Conexão** para validar se a porta RDP (`3389`) está acessível antes de abrir a sessão.
  - Mensagens de erro amigáveis em português quando houver falha de rede ou autenticação.

---

## Instalação

### Opção 1: Pacote `.deb` (Ubuntu / Debian e derivados)

Baixe o arquivo `remote-desktop-linux_1.0.0_all.deb` na aba [Releases](https://github.com/DiLucaYVL/DesktopRDP/releases) e instale com:

```bash
sudo apt install ./remote-desktop-linux_1.0.0_all.deb
```

### Opção 2: Instalação local no usuário (qualquer distribuição Linux)

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

## Pré-requisitos (FreeRDP e bibliotecas GTK4)

A aplicação detecta automaticamente o **FreeRDP** instalado via pacote da distro (`sdl-freerdp`, `wlfreerdp` ou `xfreerdp`) ou via **Flatpak** (`com.freerdp.FreeRDP`).

Se você ainda não tiver o FreeRDP instalado, use uma das opções abaixo:

- **Via Flatpak (sem `sudo`, recomendado):**
  ```bash
  flatpak install --user flathub com.freerdp.FreeRDP
  ```
- **Via APT (Ubuntu / Debian):**
  ```bash
  sudo apt install freerdp-sdl python3-gi gir1.2-gtk-4.0 gir1.2-adw-1 gir1.2-secret-1
  ```

---

## Dicas para conectar ao Windows

1. **Ative a Área de Trabalho Remota no Windows:** vá em *Configurações > Sistema > Área de Trabalho Remota* e ative a opção (requer Windows 10/11 Pro, Enterprise ou Education).
2. **Atalho de tela cheia:** dentro de uma sessão FreeRDP em tela cheia, pressione `Ctrl + Alt + Enter` para alternar entre tela cheia e janela.
3. **Login com PIN no Windows 11:** se o Windows pedir senha no RDP e recusar seu PIN, faça um login no Windows usando a senha da conta uma vez para liberar o cache NLA.
