#!/usr/bin/env python3
"""Remote Desktop Linux / Área de trabalho remota — GTK4/Libadwaita FreeRDP & Tailscale client."""

from __future__ import annotations

from collections import deque
import json
import locale
import os
import shutil
import socket
import subprocess
import sys
import threading
import uuid
from pathlib import Path
from typing import Any

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
gi.require_version("Secret", "1")

from gi.repository import Adw, Gdk, Gio, GLib, Gtk, Secret

APP_ID = "io.github.dilucayvl.RemoteDesktopLinux"
APP_VERSION = "1.2.0"

CONFIG_DIR = Path.home() / ".config" / "remote-desktop-linux"
PROFILES_FILE = CONFIG_DIR / "profiles.json"
ICONS_DIR = Path.home() / ".local" / "share" / "icons"

SECRET_SCHEMA = Secret.Schema.new(
    APP_ID,
    Secret.SchemaFlags.NONE,
    {"profile_id": Secret.SchemaAttributeType.STRING},
)

KNOWN_ERR_MARKERS = (
    "ERRCONNECT_",
    "Authentication failure",
    "LOGON_FAILURE",
    "CONNECT_FAILED",
    "DNS_NAME_NOT_FOUND",
    "TLS_CONNECT_FAILED",
    "HYBRID_REQUIRED_BY_SERVER",
)

TRANSLATIONS: dict[str, dict[str, str]] = {
    "en": {
        "app_name": "Remote Desktop Linux",
        "keyring_label": "Remote Desktop Linux ({profile_id})",
        "err_host_empty": "Enter the computer IP address or hostname.",
        "err_ipv6_unclosed": "Invalid IPv6 address (missing closing bracket).",
        "err_host_before_port": "Enter the IP address or hostname before the port.",
        "err_port_invalid": "Invalid port: '{port}'. Enter a number between 1 and 65535.",
        "err_port_range": "Port out of allowed range (1-65535): {port}.",
        "err_freerdp_missing": "FreeRDP not found.",
        "err_auth_title": "Username or password rejected",
        "err_auth_body": (
            "The Windows machine ({host}) rejected the provided credentials.\n\n"
            "Tips:\n"
            "• Check your username and password (try .\\username for a local account or your Microsoft account email).\n"
            "• On Windows 11 with PIN-only sign-in, sign in once on the Windows machine using your password to enable NLA remote login."
        ),
        "err_dns_title": "Computer not found",
        "err_dns_body": (
            "Could not resolve address '{host}'.\n\n"
            "Verify the Tailscale IP or local network hostname and try again."
        ),
        "err_connect_title": "Could not connect to {host}:{port}",
        "err_connect_body": (
            "Check that:\n"
            "• The Windows machine is powered on and connected to Tailscale (or the same local network).\n"
            "• Remote Desktop is enabled on Windows (Settings > System > Remote Desktop — requires Windows Pro/Enterprise).\n"
            "• The Windows Firewall is not blocking the RDP port (3389)."
        ),
        "err_technical_details": "Technical details:",
        "mode_windowed": "Resizable window (Recommended)",
        "mode_fullscreen": "Fullscreen (Ctrl+Alt+Enter to exit)",
        "mode_1080": "1920 × 1080 (Full HD)",
        "mode_768": "1366 × 768 (HD)",
        "tooltip_refresh": "Refresh Tailscale PCs",
        "tooltip_clear": "New connection / Clear fields",
        "group_tailscale_title": "Tailscale PCs",
        "group_tailscale_desc": "Click a computer to fill in its IP automatically",
        "group_saved_title": "Saved PCs",
        "group_saved_desc": "Click to edit or press Connect to launch directly",
        "group_creds_title": "Connection Details",
        "entry_host": "IP or Host (Tailscale or Local Network)",
        "entry_user": "Windows Username",
        "entry_pass": "Password",
        "entry_name": "Computer Name (optional)",
        "switch_save_title": "Save this PC and password",
        "switch_save_sub": "Stores the profile above and saves the password in the system keyring",
        "group_opts_title": "Session Options",
        "combo_display": "Display Mode",
        "switch_sound": "Play Windows audio on this PC",
        "switch_downloads": "Share Downloads folder",
        "btn_test": "Test Connection",
        "btn_connect_rdp": "Connect via RDP",
        "btn_connect": "Connect",
        "btn_select": "Select",
        "tooltip_delete": "Remove saved PC",
        "btn_ok": "OK",
        "toast_cleared": "Fields cleared for a new connection.",
        "toast_profile_loaded": "Profile '{name}' loaded.",
        "toast_profile_deleted": "Saved PC removed.",
        "tailscale_loading": "Searching for Tailscale computers...",
        "tailscale_not_found": "Tailscale is not installed on this system.",
        "tailscale_query_err": "Error querying Tailscale: {err}",
        "tailscale_disconnected": "Tailscale is disconnected or unavailable right now.",
        "tailscale_invalid_json": "Invalid response from Tailscale: {err}",
        "tailscale_inactive": "Tailscale is not active ({state}).",
        "tailscale_empty": "No computers found on your Tailscale network.",
        "os_unknown": "unknown",
        "status_online": "Online",
        "status_offline": "Offline",
        "toast_peer_saved_loaded": "Saved credentials for '{name}' loaded.",
        "toast_peer_selected": "{hostname} selected — enter username and password.",
        "toast_testing": "Testing port {port} on {host}...",
        "toast_test_ok": "Connection OK! Port {port} is reachable on {host}.",
        "alert_test_fail_title": "Port {port} unreachable on {host}",
        "alert_test_fail_body": (
            "Could not reach the RDP service.\n\n"
            "Check that Windows is powered on, connected to Tailscale/local network, and has Remote Desktop enabled.\n\n"
            "Error: {err}"
        ),
        "toast_keyring_warn": "PC saved, but the system keyring could not store the password.",
        "toast_need_user_for_pw": "Enter the Windows username before connecting with a password.",
        "alert_freerdp_missing_title": "FreeRDP not installed",
        "alert_freerdp_missing_body": (
            "No FreeRDP client was found on this computer.\n\n"
            "Install one of the options below in a terminal and try again:\n"
            "• Ubuntu/Debian: sudo apt install freerdp-x11\n"
            "• Flatpak: flatpak install --user flathub com.freerdp.FreeRDP"
        ),
        "toast_connecting": "Starting RDP session to {host}...",
        "alert_launch_err": "Failed to start FreeRDP",
        "alert_session_err": "FreeRDP session error",
    },
    "pt_BR": {
        "app_name": "Área de trabalho remota",
        "keyring_label": "Área de trabalho remota ({profile_id})",
        "err_host_empty": "Informe o IP ou nome do computador.",
        "err_ipv6_unclosed": "Endereço IPv6 inválido (colchete não fechado).",
        "err_host_before_port": "Informe o IP ou nome do computador antes da porta.",
        "err_port_invalid": "Porta inválida: '{port}'. Use um número entre 1 e 65535.",
        "err_port_range": "Porta fora do intervalo permitido (1-65535): {port}.",
        "err_freerdp_missing": "FreeRDP não encontrado.",
        "err_auth_title": "Falha de usuário ou senha",
        "err_auth_body": (
            "O computador Windows ({host}) recusou as credenciais informadas.\n\n"
            "Dicas:\n"
            "• Confira usuário e senha (tente .\\usuario para conta local ou seu e-mail da conta Microsoft).\n"
            "• No Windows 11 com login só por PIN, faça um login usando sua senha diretamente no Windows para liberar o acesso remoto (NLA)."
        ),
        "err_dns_title": "Computador não encontrado",
        "err_dns_body": (
            "Não foi possível resolver o endereço '{host}'.\n\n"
            "Verifique o IP do Tailscale ou da rede local e tente novamente."
        ),
        "err_connect_title": "Não foi possível conectar a {host}:{port}",
        "err_connect_body": (
            "Verifique se:\n"
            "• O computador Windows está ligado e conectado ao Tailscale (ou na mesma rede local).\n"
            "• A Área de Trabalho Remota está ativada no Windows (Configurações > Sistema > Área de Trabalho Remota — requer Windows Pro/Enterprise).\n"
            "• O firewall do Windows não está bloqueando a porta RDP (3389)."
        ),
        "err_technical_details": "Detalhes técnicos:",
        "mode_windowed": "Janela redimensionável (Recomendado)",
        "mode_fullscreen": "Tela cheia (Ctrl+Alt+Enter para sair)",
        "mode_1080": "1920 × 1080 (Full HD)",
        "mode_768": "1366 × 768 (HD)",
        "tooltip_refresh": "Atualizar PCs do Tailscale",
        "tooltip_clear": "Nova conexão / Limpar campos",
        "group_tailscale_title": "PCs no Tailscale",
        "group_tailscale_desc": "Clique em um computador para preencher o IP automaticamente",
        "group_saved_title": "Meus PCs Salvos",
        "group_saved_desc": "Clique para editar ou use Conectar para abrir direto",
        "group_creds_title": "Dados de Acesso",
        "entry_host": "IP ou Host (Tailscale ou Rede Local)",
        "entry_user": "Usuário do Windows",
        "entry_pass": "Senha",
        "entry_name": "Nome deste PC (opcional)",
        "switch_save_title": "Salvar este PC e senha",
        "switch_save_sub": "Guarda o acesso na lista acima e a senha no chaveiro do sistema",
        "group_opts_title": "Opções da Sessão",
        "combo_display": "Modo de Tela",
        "switch_sound": "Áudio do Windows neste PC",
        "switch_downloads": "Compartilhar pasta Downloads",
        "btn_test": "Testar Conexão",
        "btn_connect_rdp": "Conectar via RDP",
        "btn_connect": "Conectar",
        "btn_select": "Selecionar",
        "tooltip_delete": "Remover PC salvo",
        "btn_ok": "Entendi",
        "toast_cleared": "Campos limpos para nova conexão.",
        "toast_profile_loaded": "Perfil '{name}' carregado.",
        "toast_profile_deleted": "PC salvo removido.",
        "tailscale_loading": "Buscando computadores no Tailscale...",
        "tailscale_not_found": "Tailscale não encontrado neste sistema.",
        "tailscale_query_err": "Erro ao consultar Tailscale: {err}",
        "tailscale_disconnected": "Tailscale desconectado ou indisponível no momento.",
        "tailscale_invalid_json": "Resposta inválida do Tailscale: {err}",
        "tailscale_inactive": "Tailscale não está ativo ({state}).",
        "tailscale_empty": "Nenhum computador encontrado na sua rede Tailscale.",
        "os_unknown": "desconhecido",
        "status_online": "Online",
        "status_offline": "Offline",
        "toast_peer_saved_loaded": "Credenciais salvas de '{name}' carregadas.",
        "toast_peer_selected": "{hostname} selecionado — informe usuário e senha.",
        "toast_testing": "Testando porta {port} em {host}...",
        "toast_test_ok": "Conexão OK! Porta {port} acessível em {host}.",
        "alert_test_fail_title": "Porta {port} inacessível em {host}",
        "alert_test_fail_body": (
            "Não foi possível alcançar o serviço RDP.\n\n"
            "Verifique se o Windows está ligado, conectado ao Tailscale/rede local e com a Área de Trabalho Remota ativada.\n\n"
            "Erro: {err}"
        ),
        "toast_keyring_warn": "PC salvo, mas o chaveiro do sistema não pôde guardar a senha.",
        "toast_need_user_for_pw": "Informe o usuário do Windows antes de conectar com senha.",
        "alert_freerdp_missing_title": "FreeRDP não instalado",
        "alert_freerdp_missing_body": (
            "Não foi encontrado nenhum cliente FreeRDP neste computador.\n\n"
            "Instale uma das opções abaixo no terminal e tente novamente:\n"
            "• Ubuntu/Debian: sudo apt install freerdp-x11\n"
            "• Flatpak: flatpak install --user flathub com.freerdp.FreeRDP"
        ),
        "toast_connecting": "Iniciando conexão RDP com {host}...",
        "alert_launch_err": "Erro ao abrir FreeRDP",
        "alert_session_err": "Erro na sessão FreeRDP",
    },
}


def detect_language(cli_lang: str | None = None) -> str:
    """Detect user UI language ('pt_BR' or 'en') from CLI flag or system locale."""
    candidates: list[str | None] = [
        cli_lang,
        os.environ.get("LC_ALL"),
        os.environ.get("LC_MESSAGES"),
        next((item for item in os.environ.get("LANGUAGE", "").split(":") if item.strip()), None),
        os.environ.get("LANG"),
        locale.getlocale()[0],
    ]
    for raw in candidates:
        if not raw:
            continue
        cleaned = raw.strip()
        if not cleaned:
            continue
        base = cleaned.split(".")[0].split("@")[0].upper()
        if base in {"C", "POSIX"}:
            continue
        if cleaned.lower().startswith("pt"):
            return "pt_BR"
        return "en"
    return "en"


CURRENT_LANG = detect_language()


def tr(key: str, **kwargs: Any) -> str:
    """Return translated string for CURRENT_LANG with optional format args."""
    table = TRANSLATIONS.get(CURRENT_LANG, TRANSLATIONS["en"])
    template = table.get(key) or TRANSLATIONS["en"].get(key, key)
    return template.format(**kwargs) if kwargs else template


DISPLAY_MODE_KEYS = ["windowed", "fullscreen", "1920x1080", "1366x768"]
DISPLAY_MODE_TR_KEYS = ["mode_windowed", "mode_fullscreen", "mode_1080", "mode_768"]

_FREERDP_CMD_UNSET = object()
_FREERDP_CMD_CACHE: list[str] | None | object = _FREERDP_CMD_UNSET


def find_desktop_file() -> str | None:
    """Locate the installed .desktop file for matching window identity."""
    for candidate in (
        Path.home() / ".local" / "share" / "applications" / f"{APP_ID}.desktop",
        Path("/usr/local/share/applications") / f"{APP_ID}.desktop",
        Path("/usr/share/applications") / f"{APP_ID}.desktop",
    ):
        if candidate.is_file():
            return str(candidate)
    return None


def _is_flatpak_wrapper(binary_path: str) -> bool:
    """Return True if a script wrapper delegates to `flatpak run`."""
    try:
        with open(binary_path, "rb") as fh:
            head = fh.read(512)
        return head.startswith(b"#!") and b"flatpak" in head
    except OSError:
        return False


def parse_host_port(raw_host: str, fallback_port: int = 3389) -> tuple[str, int]:
    """Parse host and optional port from user input, supporting IPv4, hostnames, and IPv6."""
    text = raw_host.strip()
    if not text:
        raise ValueError(tr("err_host_empty"))

    if text.startswith("["):
        if "]:" in text:
            addr, port_str = text[1:].split("]:", 1)
            port = _validate_port(port_str)
            return addr.strip(), port
        if text.endswith("]"):
            return text[1:-1].strip(), fallback_port
        raise ValueError(tr("err_ipv6_unclosed"))

    colon_count = text.count(":")
    if colon_count == 1:
        host_part, port_str = text.split(":", 1)
        host_part = host_part.strip().rstrip(".")
        if not host_part:
            raise ValueError(tr("err_host_before_port"))
        return host_part, _validate_port(port_str)

    if colon_count > 1:
        return text, fallback_port

    return text.rstrip("."), fallback_port


def _validate_port(port_str: str) -> int:
    try:
        port = int(port_str.strip())
    except ValueError as exc:
        raise ValueError(tr("err_port_invalid", port=port_str)) from exc
    if not 1 <= port <= 65535:
        raise ValueError(tr("err_port_range", port=port))
    return port


def resolve_freerdp_command() -> list[str] | None:
    """Prefer native FreeRDP binaries for single-app window grouping, falling back to Flatpak."""
    global _FREERDP_CMD_CACHE
    if _FREERDP_CMD_CACHE is not _FREERDP_CMD_UNSET:
        return list(_FREERDP_CMD_CACHE) if _FREERDP_CMD_CACHE is not None else None

    local_lib_bin = Path.home() / ".local" / "lib" / "remote-desktop-linux" / "usr" / "bin"
    found: list[str] | None = None

    for candidate in (
        "/usr/bin/xfreerdp",
        "/usr/bin/xfreerdp3",
        str(local_lib_bin / "xfreerdp"),
        str(local_lib_bin / "xfreerdp3"),
        "/usr/local/bin/xfreerdp",
        "/usr/local/bin/xfreerdp3",
        "/usr/bin/sdl-freerdp",
        "/usr/bin/sdl-freerdp3",
    ):
        if os.path.isfile(candidate) and os.access(candidate, os.X_OK):
            found = [candidate]
            break

    if found is None:
        for name in ("xfreerdp", "xfreerdp3"):
            which_path = shutil.which(name)
            if which_path and not _is_flatpak_wrapper(which_path):
                found = [which_path]
                break

    if found is None:
        flatpak_bin = shutil.which("flatpak")
        if flatpak_bin:
            try:
                res = subprocess.run(
                    [flatpak_bin, "info", "com.freerdp.FreeRDP"],
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                    timeout=5,
                )
                if res.returncode == 0:
                    found = [flatpak_bin, "run", "com.freerdp.FreeRDP"]
            except (OSError, subprocess.SubprocessError):
                pass

    if found is None:
        for name in ("sdl-freerdp", "sdl-freerdp3"):
            which_path = shutil.which(name)
            if which_path:
                found = [which_path]
                break

    _FREERDP_CMD_CACHE = found
    return list(found) if found is not None else None


def build_freerdp_args(
    host: str,
    port: int,
    username: str,
    password: str,
    display_mode: str,
    sound: bool,
    share_downloads: bool,
    base_cmd: list[str] | None = None,
    session_title: str = "",
) -> tuple[list[str], str | None]:
    """Build FreeRDP CLI argv (never containing password) and optional stdin payload."""
    cmd = list(base_cmd) if base_cmd is not None else (resolve_freerdp_command() or [])
    if not cmd:
        raise FileNotFoundError(tr("err_freerdp_missing"))

    title_str = session_title.strip() or f"{tr('app_name')} — {host}"
    cmd.extend(
        [
            f"/v:{host}",
            f"/port:{port}",
            "/cert:ignore",
            "+clipboard",
            "/network:auto",
            f"/wm-class:{APP_ID}",
            f"/t:{title_str}",
        ]
    )

    user_clean = username.strip()
    if "\\" in user_clean:
        domain, user = user_clean.split("\\", 1)
        if domain:
            cmd.append(f"/d:{domain}")
        if user:
            cmd.append(f"/u:{user}")
    elif user_clean:
        cmd.append(f"/u:{user_clean}")

    if display_mode == "fullscreen":
        cmd.extend(["+f", "+dynamic-resolution"])
    elif display_mode == "1920x1080":
        cmd.extend(["/size:1920x1080", "+dynamic-resolution"])
    elif display_mode == "1366x768":
        cmd.extend(["/size:1366x768", "+dynamic-resolution"])
    else:
        cmd.extend(["+dynamic-resolution", "/size:1440x860"])

    if sound:
        cmd.append("/sound")
    else:
        cmd.append("/audio-mode:2")

    downloads_dir = Path.home() / "Downloads"
    if share_downloads and downloads_dir.is_dir():
        cmd.append(f"/drive:Downloads,{downloads_dir}")

    stdin_payload: str | None = None
    if password and user_clean:
        cmd.append("/from-stdin:force")
        stdin_payload = f"{password}\n"

    return cmd, stdin_payload


def parse_tailscale_peers(raw_json: str) -> tuple[str, list[dict[str, Any]]]:
    """Parse `tailscale status --json` output into (backend_state, sorted computer peers)."""
    data = json.loads(raw_json)
    backend_state = str(data.get("BackendState") or "Unknown")
    raw_peers = data.get("Peer") or {}
    peers: list[dict[str, Any]] = []

    for peer in raw_peers.values():
        os_name = str(peer.get("OS") or "").strip().lower()
        if os_name in {"android", "ios"}:
            continue
        ips = peer.get("TailscaleIPs") or []
        ipv4 = next((ip for ip in ips if ":" not in ip), "")
        if not ipv4:
            continue
        hostname = str(peer.get("HostName") or peer.get("DNSName") or ipv4).rstrip(".")
        online = bool(peer.get("Online"))
        peers.append(
            {
                "hostname": hostname,
                "ip": ipv4,
                "os": os_name or tr("os_unknown"),
                "online": online,
            }
        )

    peers.sort(
        key=lambda p: (
            0 if p["os"] == "windows" else 1,
            0 if p["online"] else 1,
            p["hostname"].lower(),
        )
    )
    return backend_state, peers


def store_password(profile_id: str, password: str) -> bool:
    """Store password in GNOME Keyring, returning False if unavailable."""
    try:
        if password:
            Secret.password_store_sync(
                SECRET_SCHEMA,
                {"profile_id": profile_id},
                Secret.COLLECTION_DEFAULT,
                tr("keyring_label", profile_id=profile_id),
                password,
                None,
            )
        else:
            Secret.password_clear_sync(SECRET_SCHEMA, {"profile_id": profile_id}, None)
        return True
    except GLib.Error:
        return False


def lookup_password(profile_id: str) -> str:
    """Lookup password from GNOME Keyring, returning '' on error or miss."""
    try:
        pw = Secret.password_lookup_sync(SECRET_SCHEMA, {"profile_id": profile_id}, None)
        return pw or ""
    except GLib.Error:
        return ""


def clear_password(profile_id: str) -> None:
    """Delete password from GNOME Keyring if present."""
    try:
        Secret.password_clear_sync(SECRET_SCHEMA, {"profile_id": profile_id}, None)
    except GLib.Error:
        pass


def load_profiles() -> list[dict[str, Any]]:
    """Load saved profiles from disk."""
    if not PROFILES_FILE.exists():
        return []
    try:
        data = json.loads(PROFILES_FILE.read_text(encoding="utf-8"))
        if isinstance(data, list):
            return [p for p in data if isinstance(p, dict) and p.get("host")]
    except (OSError, json.JSONDecodeError):
        pass
    return []


def save_profiles(profiles: list[dict[str, Any]]) -> None:
    """Persist profiles to disk with restricted 0700/0600 permissions."""
    CONFIG_DIR.mkdir(parents=True, exist_ok=True)
    os.chmod(CONFIG_DIR, 0o700)
    tmp_file = PROFILES_FILE.with_suffix(".tmp")
    tmp_file.write_text(json.dumps(profiles, ensure_ascii=False, indent=2), encoding="utf-8")
    os.chmod(tmp_file, 0o600)
    tmp_file.replace(PROFILES_FILE)
    os.chmod(PROFILES_FILE, 0o600)


def format_freerdp_error(host: str, port: int, output: str) -> tuple[str, str]:
    """Generate a localized friendly error title and message from FreeRDP output."""
    lines = [line.strip() for line in output.splitlines() if line.strip()]
    relevant = [
        line
        for line in lines
        if any(tag in line for tag in ("[ERROR]", "[WARN]", "ERRCONNECT_", "Authentication", "failed"))
    ]
    tail_lines = relevant[-8:] if relevant else lines[-8:]
    details = "\n".join(tail_lines)

    upper = output.upper()
    if any(
        tok in upper
        for tok in (
            "LOGON_FAILURE",
            "AUTHENTICATION_FAILED",
            "AUTHENTICATION FAILURE",
            "HYBRID_REQUIRED_BY_SERVER",
        )
    ):
        title = tr("err_auth_title")
        hint = tr("err_auth_body", host=host)
    elif "DNS_NAME_NOT_FOUND" in upper:
        title = tr("err_dns_title")
        hint = tr("err_dns_body", host=host)
    else:
        title = tr("err_connect_title", host=host, port=port)
        hint = tr("err_connect_body")

    if details:
        return title, f"{hint}\n\n{tr('err_technical_details')}\n{details}"
    return title, hint


class RemoteDesktopWindow(Adw.ApplicationWindow):
    """Main window for Remote Desktop Linux / Área de trabalho remota."""

    def __init__(self, app: Adw.Application) -> None:
        super().__init__(application=app, title=tr("app_name"))
        self.set_default_size(560, 720)
        self.set_size_request(380, 500)

        self.profiles: list[dict[str, Any]] = load_profiles()
        self.selected_profile_id: str | None = None
        self._tailscale_rows: list[Gtk.Widget] = []
        self._saved_rows: list[Gtk.Widget] = []
        self._session_passwords: dict[str, str] = {}
        self._active_sessions = 0

        self._build_ui()
        self._rebuild_saved_group()
        self.refresh_tailscale_peers()

    def _build_ui(self) -> None:
        self.toast_overlay = Adw.ToastOverlay()
        self.set_content(self.toast_overlay)

        toolbar_view = Adw.ToolbarView()
        self.toast_overlay.set_child(toolbar_view)

        header = Adw.HeaderBar()

        refresh_btn = Gtk.Button(
            icon_name="view-refresh-symbolic",
            tooltip_text=tr("tooltip_refresh"),
        )
        refresh_btn.connect("clicked", lambda *_: self.refresh_tailscale_peers())
        header.pack_start(refresh_btn)

        clear_btn = Gtk.Button(
            icon_name="edit-clear-all-symbolic",
            tooltip_text=tr("tooltip_clear"),
        )
        clear_btn.connect("clicked", lambda *_: self.clear_form())
        header.pack_end(clear_btn)
        toolbar_view.add_top_bar(header)

        scrolled = Gtk.ScrolledWindow()
        scrolled.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        toolbar_view.set_content(scrolled)

        clamp = Adw.Clamp(maximum_size=520)
        scrolled.set_child(clamp)

        content_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=18)
        content_box.set_margin_top(18)
        content_box.set_margin_bottom(24)
        content_box.set_margin_start(16)
        content_box.set_margin_end(16)
        clamp.set_child(content_box)

        # Group 1: Tailscale peers
        self.tailscale_group = Adw.PreferencesGroup(
            title=tr("group_tailscale_title"),
            description=tr("group_tailscale_desc"),
        )
        content_box.append(self.tailscale_group)

        # Group 2: Saved PCs
        self.saved_group = Adw.PreferencesGroup(
            title=tr("group_saved_title"),
            description=tr("group_saved_desc"),
        )
        self.saved_group.set_visible(False)
        content_box.append(self.saved_group)

        # Group 3: Credentials & Host
        creds_group = Adw.PreferencesGroup(title=tr("group_creds_title"))
        content_box.append(creds_group)

        self.host_entry = Adw.EntryRow(title=tr("entry_host"))
        self.host_entry.set_input_purpose(Gtk.InputPurpose.URL)
        self.host_entry.connect("entry-activated", lambda *_: self.on_connect_clicked())
        creds_group.add(self.host_entry)

        self.user_entry = Adw.EntryRow(title=tr("entry_user"))
        self.user_entry.connect("entry-activated", lambda *_: self.on_connect_clicked())
        creds_group.add(self.user_entry)

        self.pass_entry = Adw.PasswordEntryRow(title=tr("entry_pass"))
        self.pass_entry.connect("entry-activated", lambda *_: self.on_connect_clicked())
        creds_group.add(self.pass_entry)

        self.name_entry = Adw.EntryRow(title=tr("entry_name"))
        self.name_entry.connect("entry-activated", lambda *_: self.on_connect_clicked())
        creds_group.add(self.name_entry)

        self.save_switch = Adw.SwitchRow(
            title=tr("switch_save_title"),
            subtitle=tr("switch_save_sub"),
        )
        self.save_switch.set_active(True)
        creds_group.add(self.save_switch)

        # Group 4: Session options
        opts_group = Adw.PreferencesGroup(title=tr("group_opts_title"))
        content_box.append(opts_group)

        mode_labels = Gtk.StringList.new([tr(k) for k in DISPLAY_MODE_TR_KEYS])
        self.display_combo = Adw.ComboRow(title=tr("combo_display"), model=mode_labels)
        self.display_combo.set_selected(0)
        opts_group.add(self.display_combo)

        self.sound_switch = Adw.SwitchRow(title=tr("switch_sound"))
        self.sound_switch.set_active(True)
        opts_group.add(self.sound_switch)

        self.downloads_switch = Adw.SwitchRow(title=tr("switch_downloads"))
        self.downloads_switch.set_active(True)
        opts_group.add(self.downloads_switch)

        # Action Buttons
        actions_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        actions_box.set_halign(Gtk.Align.CENTER)
        actions_box.set_margin_top(6)
        content_box.append(actions_box)

        self.test_btn = Gtk.Button(label=tr("btn_test"))
        self.test_btn.add_css_class("pill")
        self.test_btn.connect("clicked", lambda *_: self.on_test_connection_clicked())
        actions_box.append(self.test_btn)

        self.connect_btn = Gtk.Button(label=tr("btn_connect_rdp"))
        self.connect_btn.add_css_class("pill")
        self.connect_btn.add_css_class("suggested-action")
        self.connect_btn.connect("clicked", lambda *_: self.on_connect_clicked())
        actions_box.append(self.connect_btn)

    def show_toast(self, message: str) -> None:
        self.toast_overlay.add_toast(Adw.Toast.new(message))

    def show_alert(self, heading: str, body: str) -> None:
        dialog = Adw.AlertDialog.new(heading, body)
        dialog.add_response("ok", tr("btn_ok"))
        dialog.set_default_response("ok")
        dialog.set_close_response("ok")
        dialog.present(self)

    def clear_form(self) -> None:
        self.selected_profile_id = None
        self.host_entry.set_text("")
        self.user_entry.set_text("")
        self.pass_entry.set_text("")
        self.name_entry.set_text("")
        self.save_switch.set_active(True)
        self.display_combo.set_selected(0)
        self.sound_switch.set_active(True)
        self.downloads_switch.set_active(True)
        self.host_entry.grab_focus()
        self.show_toast(tr("toast_cleared"))

    def _clear_group_rows(self, group: Adw.PreferencesGroup, tracked: list[Gtk.Widget]) -> None:
        for row in tracked:
            group.remove(row)
        tracked.clear()

    def _find_profile_by_host(self, host: str) -> dict[str, Any] | None:
        norm = host.strip().lower()
        for prof in self.profiles:
            if str(prof.get("host", "")).strip().lower() == norm:
                return prof
        return None

    def _get_password_for_profile(self, profile_id: str) -> str:
        pw = lookup_password(profile_id)
        if pw:
            return pw
        return self._session_passwords.get(profile_id, "")

    def _load_profile_into_form(self, profile: dict[str, Any]) -> None:
        self.selected_profile_id = profile["id"]
        port = int(profile.get("port", 3389))
        host = str(profile.get("host", ""))
        self.host_entry.set_text(f"{host}:{port}" if port != 3389 else host)
        self.user_entry.set_text(str(profile.get("username", "")))
        self.pass_entry.set_text(self._get_password_for_profile(profile["id"]))
        self.name_entry.set_text(str(profile.get("name", "")))
        self.save_switch.set_active(bool(profile.get("save_password", True)))

        mode = str(profile.get("display_mode", "windowed"))
        idx = DISPLAY_MODE_KEYS.index(mode) if mode in DISPLAY_MODE_KEYS else 0
        self.display_combo.set_selected(idx)
        self.sound_switch.set_active(bool(profile.get("sound", True)))
        self.downloads_switch.set_active(bool(profile.get("share_downloads", True)))

    def _rebuild_saved_group(self) -> None:
        self._clear_group_rows(self.saved_group, self._saved_rows)
        if not self.profiles:
            self.saved_group.set_visible(False)
            return

        self.saved_group.set_visible(True)
        for profile in self.profiles:
            host = profile.get("host", "")
            port = int(profile.get("port", 3389))
            username = str(profile.get("username", "")).strip()
            endpoint = f"{host}:{port}" if port != 3389 else host
            subtitle = f"{username} @ {endpoint}" if username else endpoint

            row = Adw.ActionRow(
                title=GLib.markup_escape_text(str(profile.get("name") or host)),
                subtitle=GLib.markup_escape_text(subtitle),
                activatable=True,
            )
            row.add_prefix(Gtk.Image.new_from_icon_name("computer-symbolic"))
            row.connect("activated", lambda _r, p=profile: self._on_saved_row_activated(p))

            btn_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
            btn_box.set_valign(Gtk.Align.CENTER)

            connect_btn = Gtk.Button(label=tr("btn_connect"))
            connect_btn.add_css_class("suggested-action")
            connect_btn.connect("clicked", lambda _b, p=profile: self._connect_profile(p))
            btn_box.append(connect_btn)

            delete_btn = Gtk.Button(
                icon_name="user-trash-symbolic",
                tooltip_text=tr("tooltip_delete"),
            )
            delete_btn.add_css_class("flat")
            delete_btn.connect("clicked", lambda _b, pid=profile["id"]: self._delete_profile(pid))
            btn_box.append(delete_btn)

            row.add_suffix(btn_box)
            self.saved_group.add(row)
            self._saved_rows.append(row)

    def _on_saved_row_activated(self, profile: dict[str, Any]) -> None:
        self._load_profile_into_form(profile)
        self.show_toast(tr("toast_profile_loaded", name=profile.get("name") or profile.get("host")))

    def _delete_profile(self, profile_id: str) -> None:
        self.profiles = [p for p in self.profiles if p.get("id") != profile_id]
        save_profiles(self.profiles)
        clear_password(profile_id)
        self._session_passwords.pop(profile_id, None)
        if self.selected_profile_id == profile_id:
            self.selected_profile_id = None
        self._rebuild_saved_group()
        self.refresh_tailscale_peers()
        self.show_toast(tr("toast_profile_deleted"))

    def refresh_tailscale_peers(self) -> None:
        self._clear_group_rows(self.tailscale_group, self._tailscale_rows)
        loading_row = Adw.ActionRow(title=tr("tailscale_loading"))
        loading_row.add_prefix(Gtk.Spinner(spinning=True))
        self.tailscale_group.add(loading_row)
        self._tailscale_rows.append(loading_row)

        def worker() -> None:
            ts_bin = shutil.which("tailscale")
            if not ts_bin:
                GLib.idle_add(self._render_tailscale_message, tr("tailscale_not_found"))
                return
            try:
                proc = subprocess.run(
                    [ts_bin, "status", "--json"],
                    capture_output=True,
                    text=True,
                    timeout=5,
                )
            except (OSError, subprocess.SubprocessError) as exc:
                GLib.idle_add(self._render_tailscale_message, tr("tailscale_query_err", err=exc))
                return

            if proc.returncode != 0:
                GLib.idle_add(self._render_tailscale_message, tr("tailscale_disconnected"))
                return

            try:
                backend_state, peers = parse_tailscale_peers(proc.stdout)
            except (json.JSONDecodeError, ValueError) as exc:
                GLib.idle_add(self._render_tailscale_message, tr("tailscale_invalid_json", err=exc))
                return

            GLib.idle_add(self._render_tailscale_peers, backend_state, peers)

        threading.Thread(target=worker, daemon=True).start()

    def _render_tailscale_message(self, message: str) -> bool:
        self._clear_group_rows(self.tailscale_group, self._tailscale_rows)
        row = Adw.ActionRow(title=GLib.markup_escape_text(message))
        row.add_prefix(Gtk.Image.new_from_icon_name("network-offline-symbolic"))
        self.tailscale_group.add(row)
        self._tailscale_rows.append(row)
        return False

    def _render_tailscale_peers(self, backend_state: str, peers: list[dict[str, Any]]) -> bool:
        self._clear_group_rows(self.tailscale_group, self._tailscale_rows)
        if backend_state.lower() != "running":
            return self._render_tailscale_message(tr("tailscale_inactive", state=backend_state))

        if not peers:
            return self._render_tailscale_message(tr("tailscale_empty"))

        for peer in peers:
            ip = peer["ip"]
            os_label = peer["os"].capitalize()
            status_label = tr("status_online") if peer["online"] else tr("status_offline")
            subtitle = f"{ip} • {os_label} • {status_label}"
            icon_name = "network-workgroup-symbolic" if peer["os"] == "windows" else "computer-symbolic"

            row = Adw.ActionRow(
                title=GLib.markup_escape_text(peer["hostname"]),
                subtitle=GLib.markup_escape_text(subtitle),
                activatable=True,
            )
            row.add_prefix(Gtk.Image.new_from_icon_name(icon_name))
            row.connect("activated", lambda _r, p=peer: self._select_tailscale_peer(p))

            matched = self._find_profile_by_host(ip)
            has_saved_creds = bool(matched and matched.get("username"))
            btn = Gtk.Button(label=tr("btn_connect") if has_saved_creds else tr("btn_select"))
            btn.set_valign(Gtk.Align.CENTER)
            if has_saved_creds:
                btn.add_css_class("suggested-action")
                btn.connect("clicked", lambda _b, prof=matched: self._connect_profile(prof))
            else:
                btn.connect("clicked", lambda _b, p=peer: self._select_tailscale_peer(p))
            row.add_suffix(btn)

            self.tailscale_group.add(row)
            self._tailscale_rows.append(row)
        return False

    def _select_tailscale_peer(self, peer: dict[str, Any]) -> None:
        matched = self._find_profile_by_host(peer["ip"])
        if matched:
            self._load_profile_into_form(matched)
            self.show_toast(tr("toast_peer_saved_loaded", name=matched.get("name")))
            return

        self.selected_profile_id = None
        self.host_entry.set_text(peer["ip"])
        self.name_entry.set_text(peer["hostname"])
        self.user_entry.set_text("")
        self.pass_entry.set_text("")
        self.user_entry.grab_focus()
        self.show_toast(tr("toast_peer_selected", hostname=peer["hostname"]))

    def _read_form_values(self) -> tuple[str, int, str, str, str, str, bool, bool, bool]:
        host, port = parse_host_port(self.host_entry.get_text())
        username = self.user_entry.get_text().strip()
        password = self.pass_entry.get_text()
        name = self.name_entry.get_text().strip() or host
        save_flag = self.save_switch.get_active()
        mode_idx = self.display_combo.get_selected()
        display_mode = (
            DISPLAY_MODE_KEYS[mode_idx]
            if 0 <= mode_idx < len(DISPLAY_MODE_KEYS)
            else "windowed"
        )
        sound = self.sound_switch.get_active()
        share_downloads = self.downloads_switch.get_active()
        return host, port, username, password, name, display_mode, sound, share_downloads, save_flag

    def on_test_connection_clicked(self) -> None:
        try:
            host, port = parse_host_port(self.host_entry.get_text())
        except ValueError as exc:
            self.show_toast(str(exc))
            self.host_entry.grab_focus()
            return

        self.test_btn.set_sensitive(False)
        self.show_toast(tr("toast_testing", port=port, host=host))

        def worker() -> None:
            try:
                with socket.create_connection((host, port), timeout=2.5):
                    pass
                GLib.idle_add(self._on_test_done, host, port, True, "")
            except OSError as exc:
                GLib.idle_add(self._on_test_done, host, port, False, str(exc))

        threading.Thread(target=worker, daemon=True).start()

    def _on_test_done(self, host: str, port: int, ok: bool, err: str) -> bool:
        self.test_btn.set_sensitive(True)
        if ok:
            self.show_toast(tr("toast_test_ok", port=port, host=host))
        else:
            self.show_alert(
                tr("alert_test_fail_title", port=port, host=host),
                tr("alert_test_fail_body", err=err),
            )
        return False

    def _upsert_profile(
        self,
        host: str,
        port: int,
        username: str,
        password: str,
        name: str,
        display_mode: str,
        sound: bool,
        share_downloads: bool,
    ) -> dict[str, Any]:
        existing = None
        if self.selected_profile_id:
            existing = next((p for p in self.profiles if p.get("id") == self.selected_profile_id), None)
        if existing is None:
            existing = next(
                (
                    p
                    for p in self.profiles
                    if str(p.get("host", "")).strip().lower() == host.lower()
                    and str(p.get("username", "")).strip().lower() == username.lower()
                ),
                None,
            )

        profile_id = existing["id"] if existing else str(uuid.uuid4())
        record: dict[str, Any] = {
            "id": profile_id,
            "name": name,
            "host": host,
            "port": port,
            "username": username,
            "display_mode": display_mode,
            "sound": sound,
            "share_downloads": share_downloads,
            "save_password": True,
        }

        if existing is not None:
            idx = self.profiles.index(existing)
            self.profiles[idx] = record
        else:
            self.profiles.insert(0, record)

        self.selected_profile_id = profile_id
        save_profiles(self.profiles)
        if not store_password(profile_id, password):
            self._session_passwords[profile_id] = password
            self.show_toast(tr("toast_keyring_warn"))
        self._rebuild_saved_group()
        self.refresh_tailscale_peers()
        return record

    def on_connect_clicked(self) -> None:
        try:
            host, port, username, password, name, display_mode, sound, share_downloads, save_flag = (
                self._read_form_values()
            )
        except ValueError as exc:
            self.show_toast(str(exc))
            self.host_entry.grab_focus()
            return

        if save_flag:
            self._upsert_profile(
                host=host,
                port=port,
                username=username,
                password=password,
                name=name,
                display_mode=display_mode,
                sound=sound,
                share_downloads=share_downloads,
            )

        self._launch_freerdp(
            host=host,
            port=port,
            username=username,
            password=password,
            display_mode=display_mode,
            sound=sound,
            share_downloads=share_downloads,
            label=name,
        )

    def _connect_profile(self, profile: dict[str, Any]) -> None:
        self._load_profile_into_form(profile)
        host = str(profile.get("host", "")).strip()
        port = int(profile.get("port", 3389))
        username = str(profile.get("username", "")).strip()
        password = self._get_password_for_profile(profile["id"])
        display_mode = str(profile.get("display_mode", "windowed"))
        sound = bool(profile.get("sound", True))
        share_downloads = bool(profile.get("share_downloads", True))
        label = str(profile.get("name") or host)
        self._launch_freerdp(
            host=host,
            port=port,
            username=username,
            password=password,
            display_mode=display_mode,
            sound=sound,
            share_downloads=share_downloads,
            label=label,
        )

    def _launch_freerdp(
        self,
        host: str,
        port: int,
        username: str,
        password: str,
        display_mode: str,
        sound: bool,
        share_downloads: bool,
        label: str = "",
    ) -> None:
        if password and not username:
            self.show_toast(tr("toast_need_user_for_pw"))
            self.user_entry.grab_focus()
            return

        base_cmd = resolve_freerdp_command()
        if base_cmd is None:
            self.show_alert(
                tr("alert_freerdp_missing_title"),
                tr("alert_freerdp_missing_body"),
            )
            return

        session_title = f"{tr('app_name')} — {label or host}"
        cmd, stdin_payload = build_freerdp_args(
            host=host,
            port=port,
            username=username,
            password=password,
            display_mode=display_mode,
            sound=sound,
            share_downloads=share_downloads,
            base_cmd=base_cmd,
            session_title=session_title,
        )

        app = self.get_application()
        if app is not None:
            app.hold()
        self._active_sessions += 1
        self.set_visible(False)

        child_env = os.environ.copy()
        desktop_file = find_desktop_file()
        if desktop_file:
            child_env["GIO_LAUNCHED_DESKTOP_FILE"] = desktop_file
        child_env["SDL_VIDEO_WAYLAND_WMCLASS"] = APP_ID
        child_env["SDL_VIDEO_X11_WMCLASS"] = APP_ID
        child_env["SDL_APP_ID"] = APP_ID
        exec_cmd = ["/bin/sh", "-c", 'export GIO_LAUNCHED_DESKTOP_FILE_PID=$$; exec "$@"', "--", *cmd]

        def worker() -> None:
            try:
                proc = subprocess.Popen(
                    exec_cmd,
                    env=child_env,
                    stdin=subprocess.PIPE if stdin_payload is not None else subprocess.DEVNULL,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT,
                    text=True,
                )
            except OSError as exc:
                GLib.idle_add(
                    self._on_session_finished,
                    host,
                    port,
                    -1,
                    "",
                    tr("alert_launch_err"),
                    str(exc),
                )
                return

            lines: deque[str] = deque(maxlen=80)
            try:
                if stdin_payload is not None and proc.stdin is not None:
                    proc.stdin.write(stdin_payload)
                    proc.stdin.flush()
                    proc.stdin.close()
                if proc.stdout is not None:
                    for line in proc.stdout:
                        lines.append(line.rstrip("\n"))
                returncode = proc.wait()
            except OSError as exc:
                GLib.idle_add(
                    self._on_session_finished,
                    host,
                    port,
                    -1,
                    "\n".join(lines),
                    tr("alert_session_err"),
                    str(exc),
                )
                return

            GLib.idle_add(self._on_session_finished, host, port, returncode, "\n".join(lines), None, None)

        threading.Thread(target=worker, daemon=True).start()

    def _on_session_finished(
        self,
        host: str,
        port: int,
        returncode: int,
        output: str,
        direct_err_title: str | None,
        direct_err_body: str | None,
    ) -> bool:
        self._active_sessions = max(0, self._active_sessions - 1)
        if self._active_sessions == 0:
            self.set_visible(True)
            self.present()

        app = self.get_application()
        if app is not None:
            app.release()

        if direct_err_title is not None:
            self.show_alert(direct_err_title, direct_err_body or "")
        elif returncode != 0 and any(marker in output for marker in KNOWN_ERR_MARKERS):
            title, body = format_freerdp_error(host, port, output)
            self.show_alert(title, body)
        return False


class RemoteDesktopApp(Adw.Application):
    """GTK4/Libadwaita Application wrapper."""

    def __init__(self, self_test: bool = False) -> None:
        super().__init__(application_id=APP_ID, flags=Gio.ApplicationFlags.FLAGS_NONE)
        self.self_test = self_test
        self.window: RemoteDesktopWindow | None = None
        self.test_passed = False
        self.test_error = ""

    def do_startup(self) -> None:
        Adw.Application.do_startup(self)
        display = Gdk.Display.get_default()
        if display is not None:
            Gtk.IconTheme.get_for_display(display).add_search_path(str(ICONS_DIR))
        Gtk.Window.set_default_icon_name(APP_ID)

    def do_activate(self) -> None:
        if self.window is None:
            self.window = RemoteDesktopWindow(self)
        self.window.present()
        if self.self_test:
            GLib.timeout_add(900, self._run_self_test)

    def _run_self_test(self) -> bool:
        assert self.window is not None
        existed_before = PROFILES_FILE.exists()
        original_raw = PROFILES_FILE.read_text(encoding="utf-8") if existed_before else None
        temp_id: str | None = None

        try:
            # 1. Verify single-app desktop identity & localized title
            assert GLib.get_prgname() == APP_ID, f"prgname mismatch: {GLib.get_prgname()!r}"
            assert GLib.get_application_name() == tr("app_name"), (
                f"application_name mismatch: {GLib.get_application_name()!r}"
            )
            assert self.window.get_title() == tr("app_name"), (
                f"Expected title {tr('app_name')!r}, got {self.window.get_title()!r}"
            )

            # 2. Verify native FreeRDP resolution (not flatpak on this host)
            resolved = resolve_freerdp_command()
            assert resolved is not None, "resolve_freerdp_command() returned None"
            assert os.path.basename(resolved[0]) in {"xfreerdp", "xfreerdp3", "sdl-freerdp", "sdl-freerdp3"}, (
                f"Expected native FreeRDP binary, got {resolved}"
            )

            # 3. Verify Tailscale peer parsing (portable across machines)
            ts_bin = shutil.which("tailscale")
            if ts_bin:
                res = subprocess.run([ts_bin, "status", "--json"], capture_output=True, text=True, timeout=5)
                if res.returncode == 0:
                    _state, peers = parse_tailscale_peers(res.stdout)
                    assert isinstance(peers, list)

            # 4. Verify command construction (WM_CLASS, title, password never in argv)
            cmd, payload = build_freerdp_args(
                host="100.67.218.113",
                port=3389,
                username="CORP\\diluca",
                password="s3cr3t pass!",
                display_mode="windowed",
                sound=True,
                share_downloads=True,
                base_cmd=resolved,
                session_title=f"{tr('app_name')} — FINTECH_DEV",
            )
            assert all("/p:" not in arg and "s3cr3t" not in arg for arg in cmd), f"Password leaked in argv: {cmd}"
            assert "/from-stdin:force" in cmd and payload == "s3cr3t pass!\n"
            assert "/d:CORP" in cmd and "/u:diluca" in cmd
            assert f"/wm-class:{APP_ID}" in cmd
            assert any(arg.startswith("/t:") for arg in cmd)
            assert "+dynamic-resolution" in cmd and "/v:100.67.218.113" in cmd

            # 5. Verify profile save + Keyring roundtrip + delete cleanup
            prof = self.window._upsert_profile(
                host="127.0.0.254",
                port=3390,
                username="__selftest__",
                password="selftest-pw!@#",
                name="__selftest__",
                display_mode="1920x1080",
                sound=False,
                share_downloads=False,
            )
            temp_id = prof["id"]
            stored_pw = lookup_password(temp_id)
            assert stored_pw == "selftest-pw!@#", f"Keyring mismatch: {stored_pw!r}"
            self.window._delete_profile(temp_id)
            temp_id = None
            assert lookup_password(prof["id"]) == ""

            self.test_passed = True
        except Exception as exc:
            self.test_error = f"{type(exc).__name__}: {exc}"
        finally:
            if temp_id is not None:
                clear_password(temp_id)
            if existed_before and original_raw is not None:
                PROFILES_FILE.write_text(original_raw, encoding="utf-8")
                os.chmod(PROFILES_FILE, 0o600)
            elif PROFILES_FILE.exists():
                PROFILES_FILE.unlink()

        self.quit()
        return False


def _extract_cli_args(argv: list[str]) -> tuple[str | None, bool, bool, list[str]]:
    """Extract --lang, --version, and --self-test flags without passing them to GTK."""
    cli_lang: str | None = None
    show_version = False
    self_test = False
    gtk_args: list[str] = [argv[0]]

    i = 1
    while i < len(argv):
        arg = argv[i]
        if arg == "--self-test":
            self_test = True
        elif arg == "--version":
            show_version = True
        elif arg.startswith("--lang="):
            cli_lang = arg.split("=", 1)[1]
        elif arg == "--lang" and i + 1 < len(argv):
            cli_lang = argv[i + 1]
            i += 1
        else:
            gtk_args.append(arg)
        i += 1

    return cli_lang, show_version, self_test, gtk_args


def main() -> int:
    global CURRENT_LANG
    cli_lang, show_version, self_test, gtk_args = _extract_cli_args(sys.argv)
    CURRENT_LANG = detect_language(cli_lang)

    if show_version:
        print(f"{tr('app_name')} {APP_VERSION}")
        return 0

    GLib.set_prgname(APP_ID)
    GLib.set_application_name(tr("app_name"))

    app = RemoteDesktopApp(self_test=self_test)
    exit_code = app.run(gtk_args)
    if self_test:
        if not app.test_passed:
            print(f"SELF-TEST FAILED: {app.test_error}", file=sys.stderr)
            return 1
        print("SELF-TEST OK")
        return 0
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
