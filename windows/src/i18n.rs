//! UI strings (pt-BR and English), matching the Linux app's wording.

use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Pt,
    En,
}

static LANG: OnceLock<Lang> = OnceLock::new();

pub fn set_lang(lang: Lang) {
    let _ = LANG.set(lang);
}

pub fn lang() -> Lang {
    *LANG.get().unwrap_or(&Lang::En)
}

/// Picks pt-BR or English from a `--lang` value or the system UI language.
pub fn detect(flag: Option<&str>) -> Lang {
    if let Some(f) = flag {
        return if f.to_lowercase().starts_with("pt") { Lang::Pt } else { Lang::En };
    }
    #[cfg(windows)]
    {
        // Primary language 0x16 = Portuguese.
        let id = unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() };
        return if id & 0x3ff == 0x16 { Lang::Pt } else { Lang::En };
    }
    #[cfg(not(windows))]
    {
        let loc = ["LC_ALL", "LC_MESSAGES", "LANGUAGE", "LANG"]
            .iter()
            .filter_map(|k| std::env::var(k).ok())
            .find(|v| !v.is_empty())
            .unwrap_or_default();
        if loc.to_lowercase().starts_with("pt") { Lang::Pt } else { Lang::En }
    }
}

/// Returns the message for `key`.
pub fn t(key: &str) -> String {
    let table: &[(&str, &str)] = if lang() == Lang::Pt { PT } else { EN };
    table
        .iter()
        .find(|(k, _)| *k == key)
        .or_else(|| EN.iter().find(|(k, _)| *k == key))
        .map(|(_, v)| v.to_string())
        .unwrap_or_else(|| key.to_string())
}

/// Returns the message for `key` with `{name}` placeholders filled from `vars`.
pub fn tf(key: &str, vars: &[(&str, &str)]) -> String {
    let mut s = t(key);
    for (name, value) in vars {
        s = s.replace(&format!("{{{name}}}"), value);
    }
    s
}

const EN: &[(&str, &str)] = &[
    ("app_name", "Remote Desktop"),
    ("btn_ok", "OK"),
    ("btn_cancel", "Cancel"),
    ("btn_delete", "Remove"),
    ("tooltip_refresh", "Refresh Tailscale PCs"),
    ("tooltip_clear", "New connection / Clear fields"),
    ("tooltip_delete", "Remove saved PC"),
    ("group_tailscale_title", "Tailscale PCs"),
    ("group_tailscale_desc", "Click a computer to fill in its IP automatically"),
    ("group_saved_title", "Saved PCs"),
    ("group_saved_desc", "Click to edit or press Connect to launch directly"),
    ("group_creds_title", "Connection Details"),
    ("entry_host", "IP or Host (Tailscale or Local Network)"),
    ("entry_user", "Username"),
    ("entry_pass", "Password"),
    ("entry_pass_saved", "(saved)"),
    ("entry_name", "Computer Name (optional)"),
    ("switch_save_title", "Save this PC and password"),
    ("switch_save_sub", "Stores the profile above and saves the password in Windows Credential Manager"),
    ("group_opts_title", "Session Options"),
    ("combo_display", "Display Mode"),
    ("mode_windowed", "Resizable window (Recommended)"),
    ("mode_fullscreen", "Fullscreen (Ctrl+Alt+Break to exit)"),
    ("mode_1080", "1920 × 1080 (Full HD)"),
    ("mode_768", "1366 × 768 (HD)"),
    ("switch_sound", "Play remote audio on this PC"),
    ("switch_multimon", "Use all my monitors for the remote session"),
    ("switch_multimon_sub", "Opens the session in full screen across every monitor on this PC (Ctrl+Alt+Break to leave full screen)"),
    ("switch_clipboard", "Shared clipboard"),
    ("switch_clipboard_sub", "Copy and paste text and files between this PC and the remote one"),
    ("btn_test", "Test Connection"),
    ("btn_connect_rdp", "Connect via RDP"),
    ("btn_connect", "Connect"),
    ("btn_select", "Select"),
    ("toast_cleared", "Fields cleared for a new connection."),
    ("toast_profile_loaded", "Profile '{name}' loaded."),
    ("toast_profile_deleted", "Saved PC removed."),
    ("tailscale_loading", "Searching for Tailscale computers..."),
    ("tailscale_not_found", "Tailscale is not installed on this PC."),
    ("tailscale_disconnected", "Tailscale is disconnected or unavailable right now."),
    ("tailscale_invalid_json", "Invalid response from Tailscale: {err}"),
    ("tailscale_inactive", "Tailscale is not active ({state})."),
    ("tailscale_empty", "No computers found on your Tailscale network."),
    ("os_unknown", "unknown"),
    ("status_online", "Online"),
    ("status_offline", "Offline"),
    ("toast_peer_saved_loaded", "Saved credentials for '{name}' loaded."),
    ("toast_peer_selected", "{hostname} selected — enter username and password."),
    ("toast_testing", "Testing port {port} on {host}..."),
    ("toast_test_ok", "Connection OK! Port {port} is reachable on {host}."),
    ("alert_test_fail_title", "Port {port} unreachable on {host}"),
    ("alert_test_fail_body", "Could not reach the RDP service.\n\nCheck that:\n• The computer is on and connected to Tailscale (or the same local network).\n• Remote Desktop is enabled on it — Linux (GNOME): Settings > System > Remote Desktop > Desktop Sharing, with Remote Control on; Windows: Settings > System > Remote Desktop.\n• No firewall blocks port {port}.\n\nError: {err}"),
    ("toast_keyring_warn", "PC saved, but Windows Credential Manager could not store the password."),
    ("toast_need_user_for_pw", "Enter the username before connecting with a password."),
    ("toast_connecting", "Starting RDP session to {host}..."),
    ("toast_session_ended", "Session with {host} ended."),
    ("err_title", "Remote Desktop"),
    ("err_host_empty", "Enter the computer IP address or hostname."),
    ("err_ipv6_unclosed", "Invalid IPv6 address (missing closing bracket)."),
    ("err_host_before_port", "Enter the IP address or hostname before the port."),
    ("err_port_invalid", "Invalid port: '{port}'. Enter a number between 1 and 65535."),
    ("alert_launch_err", "Failed to start the Windows Remote Desktop client"),
    ("err_save", "Could not save the PC: {err}"),
    ("confirm_delete_title", "Remove saved PC?"),
    ("confirm_delete", "'{name}' and its saved password will be removed from this PC."),
];

const PT: &[(&str, &str)] = &[
    ("app_name", "Área de trabalho remota"),
    ("btn_ok", "Entendi"),
    ("btn_cancel", "Cancelar"),
    ("btn_delete", "Remover"),
    ("tooltip_refresh", "Atualizar PCs do Tailscale"),
    ("tooltip_clear", "Nova conexão / Limpar campos"),
    ("tooltip_delete", "Remover PC salvo"),
    ("group_tailscale_title", "PCs no Tailscale"),
    ("group_tailscale_desc", "Clique em um computador para preencher o IP automaticamente"),
    ("group_saved_title", "Meus PCs Salvos"),
    ("group_saved_desc", "Clique para editar ou use Conectar para abrir direto"),
    ("group_creds_title", "Dados de Acesso"),
    ("entry_host", "IP ou Host (Tailscale ou Rede Local)"),
    ("entry_user", "Usuário"),
    ("entry_pass", "Senha"),
    ("entry_pass_saved", "(salva)"),
    ("entry_name", "Nome deste PC (opcional)"),
    ("switch_save_title", "Salvar este PC e senha"),
    ("switch_save_sub", "Guarda o acesso na lista acima e a senha no Gerenciador de Credenciais do Windows"),
    ("group_opts_title", "Opções da Sessão"),
    ("combo_display", "Modo de Tela"),
    ("mode_windowed", "Janela redimensionável (Recomendado)"),
    ("mode_fullscreen", "Tela cheia (Ctrl+Alt+Break para sair)"),
    ("mode_1080", "1920 × 1080 (Full HD)"),
    ("mode_768", "1366 × 768 (HD)"),
    ("switch_sound", "Áudio do PC remoto neste PC"),
    ("switch_multimon", "Usar todos os meus monitores na sessão remota"),
    ("switch_multimon_sub", "Abre a sessão em tela cheia ocupando todos os monitores deste PC (Ctrl+Alt+Break para sair da tela cheia)"),
    ("switch_clipboard", "Área de transferência compartilhada"),
    ("switch_clipboard_sub", "Copie e cole textos e arquivos entre este PC e o remoto"),
    ("btn_test", "Testar Conexão"),
    ("btn_connect_rdp", "Conectar via RDP"),
    ("btn_connect", "Conectar"),
    ("btn_select", "Selecionar"),
    ("toast_cleared", "Campos limpos para nova conexão."),
    ("toast_profile_loaded", "Perfil '{name}' carregado."),
    ("toast_profile_deleted", "PC salvo removido."),
    ("tailscale_loading", "Buscando computadores no Tailscale..."),
    ("tailscale_not_found", "Tailscale não encontrado neste PC."),
    ("tailscale_disconnected", "Tailscale desconectado ou indisponível no momento."),
    ("tailscale_invalid_json", "Resposta inválida do Tailscale: {err}"),
    ("tailscale_inactive", "Tailscale não está ativo ({state})."),
    ("tailscale_empty", "Nenhum computador encontrado na sua rede Tailscale."),
    ("os_unknown", "desconhecido"),
    ("status_online", "Online"),
    ("status_offline", "Offline"),
    ("toast_peer_saved_loaded", "Credenciais salvas de '{name}' carregadas."),
    ("toast_peer_selected", "{hostname} selecionado — informe usuário e senha."),
    ("toast_testing", "Testando porta {port} em {host}..."),
    ("toast_test_ok", "Conexão OK! Porta {port} acessível em {host}."),
    ("alert_test_fail_title", "Porta {port} inacessível em {host}"),
    ("alert_test_fail_body", "Não foi possível alcançar o serviço RDP.\n\nVerifique se:\n• O computador está ligado e conectado ao Tailscale (ou na mesma rede local).\n• A Área de Trabalho Remota está ativada nele — Linux (GNOME): Configurações > Sistema > Área de Trabalho Remota > Compartilhamento de Área de Trabalho, com Controle Remoto ligado; Windows: Configurações > Sistema > Área de Trabalho Remota.\n• Nenhum firewall bloqueia a porta {port}.\n\nErro: {err}"),
    ("toast_keyring_warn", "PC salvo, mas o Gerenciador de Credenciais do Windows não guardou a senha."),
    ("toast_need_user_for_pw", "Informe o usuário antes de conectar com senha."),
    ("toast_connecting", "Iniciando conexão RDP com {host}..."),
    ("toast_session_ended", "Sessão com {host} encerrada."),
    ("err_title", "Área de trabalho remota"),
    ("err_host_empty", "Informe o IP ou nome do computador."),
    ("err_ipv6_unclosed", "Endereço IPv6 inválido (colchete não fechado)."),
    ("err_host_before_port", "Informe o IP ou nome do computador antes da porta."),
    ("err_port_invalid", "Porta inválida: '{port}'. Use um número entre 1 e 65535."),
    ("alert_launch_err", "Erro ao abrir o cliente de Área de Trabalho Remota do Windows"),
    ("err_save", "Não foi possível salvar o PC: {err}"),
    ("confirm_delete_title", "Remover PC salvo?"),
    ("confirm_delete", "'{name}' e a senha salva serão removidos deste PC."),
];
