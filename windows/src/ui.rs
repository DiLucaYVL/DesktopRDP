//! Libadwaita look-alike UI drawn with egui: header bar, boxed lists, action/entry/switch/combo
//! rows, pill buttons, toasts and alert dialogs, with the colors of the Linux app.

use eframe::egui::text::TextWrapping;
use eframe::egui::{
    self, pos2, text::LayoutJob, vec2, Align, Align2, Color32, CornerRadius, FontData, FontDefinitions, FontFamily,
    FontId, Frame, Id, Image, ImageSource, Key, Layout, Margin, Rect, Response, Sense, Shadow, Stroke, StrokeKind,
    Ui, UiBuilder,
};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::i18n::{t, tf};
use crate::profiles::{self, Profile};
use crate::rdp::{self, Options, DISPLAY_MODES};
use crate::tailscale::{self, Peer};

// ---------------------------------------------------------------------------------------------
// Palette (dark values measured on the Linux app; light values from libadwaita)

#[derive(Clone, Copy)]
struct Palette {
    window_bg: Color32,
    header_bg: Color32,
    header_shade: Color32,
    card: Color32,
    card_hover: Color32,
    card_active: Color32,
    separator: Color32,
    fg: Color32,
    dim: Color32,
    btn: Color32,
    btn_hover: Color32,
    btn_active: Color32,
    flat_hover: Color32,
    accent_bg: Color32,
    accent_hover: Color32,
    accent_active: Color32,
    accent_fg: Color32,
    accent_text: Color32,
    switch_off: Color32,
    popover_bg: Color32,
    toast_bg: Color32,
    scrim: Color32,
    focus: Color32,
    destructive: Color32,
    shadow: Color32,
}

const DARK: Palette = Palette {
    window_bg: Color32::from_rgb(0x2c, 0x2c, 0x2c),
    header_bg: Color32::from_rgb(0x2c, 0x2c, 0x2c),
    header_shade: Color32::from_rgb(0x1f, 0x1f, 0x1f),
    card: Color32::from_rgb(0x3d, 0x3d, 0x3d),
    card_hover: Color32::from_rgb(0x42, 0x42, 0x42),
    card_active: Color32::from_rgb(0x4a, 0x4a, 0x4a),
    separator: Color32::from_rgb(0x2d, 0x2d, 0x2d),
    fg: Color32::from_rgb(0xf7, 0xf7, 0xf7),
    dim: Color32::from_rgb(0x9c, 0x9c, 0x9c),
    btn: Color32::from_rgb(0x4f, 0x4f, 0x4f),
    btn_hover: Color32::from_rgb(0x57, 0x57, 0x57),
    btn_active: Color32::from_rgb(0x60, 0x60, 0x60),
    flat_hover: Color32::from_rgb(0x3a, 0x3a, 0x3a),
    accent_bg: Color32::from_rgb(0x71, 0x84, 0x75),
    accent_hover: Color32::from_rgb(0x7b, 0x8e, 0x7f),
    accent_active: Color32::from_rgb(0x66, 0x77, 0x6a),
    accent_fg: Color32::WHITE,
    accent_text: Color32::from_rgb(0xa3, 0xb8, 0xa7),
    switch_off: Color32::from_rgb(0x5a, 0x5a, 0x5a),
    popover_bg: Color32::from_rgb(0x38, 0x38, 0x38),
    toast_bg: Color32::from_rgb(0x48, 0x48, 0x48),
    scrim: Color32::from_rgba_premultiplied(0, 0, 0, 128),
    focus: Color32::from_rgba_premultiplied(0x55, 0x63, 0x58, 0x99),
    destructive: Color32::from_rgb(0xff, 0x7b, 0x63),
    shadow: Color32::from_rgba_premultiplied(0, 0, 0, 40),
};

const LIGHT: Palette = Palette {
    window_bg: Color32::from_rgb(0xfa, 0xfa, 0xfb),
    header_bg: Color32::from_rgb(0xff, 0xff, 0xff),
    header_shade: Color32::from_rgb(0xe0, 0xe0, 0xe2),
    card: Color32::from_rgb(0xff, 0xff, 0xff),
    card_hover: Color32::from_rgb(0xf2, 0xf2, 0xf3),
    card_active: Color32::from_rgb(0xe9, 0xe9, 0xeb),
    separator: Color32::from_rgb(0xe6, 0xe6, 0xe7),
    fg: Color32::from_rgb(0x33, 0x33, 0x37),
    dim: Color32::from_rgb(0x80, 0x80, 0x84),
    btn: Color32::from_rgb(0xeb, 0xeb, 0xec),
    btn_hover: Color32::from_rgb(0xe1, 0xe1, 0xe3),
    btn_active: Color32::from_rgb(0xcf, 0xcf, 0xd2),
    flat_hover: Color32::from_rgb(0xed, 0xed, 0xee),
    accent_bg: Color32::from_rgb(0x71, 0x84, 0x75),
    accent_hover: Color32::from_rgb(0x7d, 0x8f, 0x81),
    accent_active: Color32::from_rgb(0x66, 0x77, 0x6a),
    accent_fg: Color32::WHITE,
    accent_text: Color32::from_rgb(0x56, 0x66, 0x5a),
    switch_off: Color32::from_rgb(0xd9, 0xd9, 0xdb),
    popover_bg: Color32::from_rgb(0xff, 0xff, 0xff),
    toast_bg: Color32::from_rgb(0x3a, 0x3a, 0x3a),
    scrim: Color32::from_rgba_premultiplied(0, 0, 0, 77),
    focus: Color32::from_rgba_premultiplied(0x44, 0x4f, 0x46, 0x99),
    destructive: Color32::from_rgb(0xe6, 0x2d, 0x42),
    shadow: Color32::from_rgba_premultiplied(0, 0, 0, 20),
};

const BODY: f32 = 14.67; // 11pt
const SMALL: f32 = 12.8;
const RADIUS: u8 = 12;

fn regular(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}
fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("bold".into()))
}

// ---------------------------------------------------------------------------------------------
// Icons (Adwaita symbolic, recolored white so tint gives the exact color)

#[derive(Clone, Copy)]
enum Icon {
    Refresh,
    Clear,
    Trash,
    Computer,
    Workgroup,
    Edit,
    Reveal,
    Conceal,
    Down,
    Check,
}

impl Icon {
    fn source(self) -> ImageSource<'static> {
        match self {
            Icon::Refresh => egui::include_image!("../assets/icons/view-refresh-symbolic.svg"),
            Icon::Clear => egui::include_image!("../assets/icons/edit-clear-all-symbolic.svg"),
            Icon::Trash => egui::include_image!("../assets/icons/user-trash-symbolic.svg"),
            Icon::Computer => egui::include_image!("../assets/icons/computer-symbolic.svg"),
            Icon::Workgroup => egui::include_image!("../assets/icons/network-workgroup-symbolic.svg"),
            Icon::Edit => egui::include_image!("../assets/icons/document-edit-symbolic.svg"),
            Icon::Reveal => egui::include_image!("../assets/icons/view-reveal-symbolic.svg"),
            Icon::Conceal => egui::include_image!("../assets/icons/view-conceal-symbolic.svg"),
            Icon::Down => egui::include_image!("../assets/icons/pan-down-symbolic.svg"),
            Icon::Check => egui::include_image!("../assets/icons/object-select-symbolic.svg"),
        }
    }
}

fn paint_icon(ui: &Ui, icon: Icon, center: egui::Pos2, color: Color32) {
    let rect = Rect::from_center_size(center, vec2(16.0, 16.0));
    Image::new(icon.source()).tint(color).paint_at(ui, rect);
}

// ---------------------------------------------------------------------------------------------
// State

enum Peers {
    Loading,
    Message(String),
    List(Vec<Peer>),
}

enum Msg {
    Peers(Result<(String, Vec<Peer>), tailscale::Error>),
    Test { host: String, port: u16, result: Result<(), String> },
    SessionEnded(String),
}

enum DialogKind {
    Info,
    ConfirmDelete(String),
}

struct Dialog {
    title: String,
    body: String,
    kind: DialogKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    Host,
    User,
    Pass,
    Name,
}

/// UI events collected while drawing, applied afterwards.
enum Action {
    RefreshPeers,
    Clear,
    SelectPeer(Peer),
    LoadProfile(Profile, bool),
    ConnectProfile(Profile),
    AskDelete(Profile),
    Delete(String),
    Test,
    Connect,
}

pub struct App {
    profiles: Vec<Profile>,
    peers: Peers,
    host: String,
    user: String,
    pass: String,
    name: String,
    save: bool,
    sound: bool,
    clipboard: bool,
    multimon: bool,
    mode: usize,
    loaded_id: Option<String>,
    pass_saved: bool,
    show_pass: bool,
    focus: Option<Field>,
    toast: Option<(String, Instant)>,
    dialog: Option<Dialog>,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
    ctx: egui::Context,
    #[cfg(windows)]
    caption_dark: Option<bool>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        install_fonts(&cc.egui_ctx);
        cc.egui_ctx.style_mut(|s| {
            s.interaction.selectable_labels = false;
            s.spacing.item_spacing = vec2(0.0, 0.0);
            s.spacing.scroll.floating = true;
        });
        let (tx, rx) = channel();
        let mut app = App {
            profiles: profiles::load(),
            peers: Peers::Loading,
            host: String::new(),
            user: String::new(),
            pass: String::new(),
            name: String::new(),
            save: true,
            sound: true,
            clipboard: true,
            multimon: false,
            mode: 0,
            loaded_id: None,
            pass_saved: false,
            show_pass: false,
            focus: None,
            toast: None,
            dialog: None,
            tx,
            rx,
            ctx: cc.egui_ctx.clone(),
            #[cfg(windows)]
            caption_dark: None,
        };
        app.refresh_peers();
        app
    }

    // ---- actions ----

    fn toast(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), Instant::now()));
    }

    fn info(&mut self, title: impl Into<String>, body: impl Into<String>) {
        self.dialog = Some(Dialog { title: title.into(), body: body.into(), kind: DialogKind::Info });
    }

    fn refresh_peers(&mut self) {
        self.peers = Peers::Loading;
        let tx = self.tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Msg::Peers(tailscale::query()));
            ctx.request_repaint();
        });
    }

    fn update_pass_hint(&mut self) {
        self.pass_saved = rdp::parse_host_port(&self.host).map(|(h, _)| rdp::has_credential(&h)).unwrap_or(false);
    }

    fn load_profile(&mut self, p: &Profile) {
        self.loaded_id = Some(p.id.clone());
        self.host = p.endpoint();
        self.user = p.username.clone();
        self.pass.clear();
        self.name = p.name.clone();
        self.save = true;
        self.mode = DISPLAY_MODES.iter().position(|m| *m == p.display_mode).unwrap_or(0);
        self.sound = p.sound;
        self.clipboard = p.clipboard;
        self.multimon = p.multimon;
        self.update_pass_hint();
    }

    fn select_peer(&mut self, peer: &Peer) {
        if let Some(p) = profiles::find_by_host(&self.profiles, &peer.ip).cloned() {
            self.load_profile(&p);
            self.toast(tf("toast_peer_saved_loaded", &[("name", p.display_name())]));
            return;
        }
        self.loaded_id = None;
        self.host = peer.ip.clone();
        self.name = peer.hostname.clone();
        self.pass.clear();
        self.update_pass_hint();
        self.focus = Some(Field::User);
        self.toast(tf("toast_peer_selected", &[("hostname", &peer.hostname)]));
    }

    fn clear_form(&mut self) {
        self.loaded_id = None;
        self.host.clear();
        self.user.clear();
        self.pass.clear();
        self.name.clear();
        self.save = true;
        self.sound = true;
        self.clipboard = true;
        self.multimon = false;
        self.mode = 0;
        self.pass_saved = false;
        self.focus = Some(Field::Host);
        self.toast(t("toast_cleared"));
    }

    fn delete_profile(&mut self, id: &str) {
        let Some(removed) = self.profiles.iter().find(|p| p.id == id).cloned() else { return };
        let kept: Vec<Profile> = self.profiles.iter().filter(|p| p.id != id).cloned().collect();
        if let Err(e) = profiles::save(&kept) {
            self.info(t("err_title"), tf("err_save", &[("err", &e.to_string())]));
            return;
        }
        self.profiles = kept;
        if profiles::find_by_host(&self.profiles, &removed.host).is_none() {
            rdp::delete_credential(&removed.host);
        }
        if self.loaded_id.as_deref() == Some(id) {
            self.loaded_id = None;
        }
        self.toast(t("toast_profile_deleted"));
    }

    fn test(&mut self) {
        let (host, port) = match rdp::parse_host_port(&self.host) {
            Ok(v) => v,
            Err(e) => return self.info(t("err_title"), e),
        };
        self.toast(tf("toast_testing", &[("host", &host), ("port", &port.to_string())]));
        let tx = self.tx.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            let result = rdp::check_port(&host, port);
            let _ = tx.send(Msg::Test { host, port, result });
            ctx.request_repaint();
        });
    }

    fn connect(&mut self) {
        let (host, port) = match rdp::parse_host_port(&self.host) {
            Ok(v) => v,
            Err(e) => return self.info(t("err_title"), e),
        };
        let user = self.user.trim().to_string();
        if !self.pass.is_empty() && user.is_empty() {
            return self.info(t("err_title"), t("toast_need_user_for_pw"));
        }
        let opts = Options {
            host: host.clone(),
            port,
            username: user.clone(),
            display_mode: DISPLAY_MODES[self.mode].to_string(),
            sound: self.sound,
            clipboard: self.clipboard,
            multimon: self.multimon,
        };
        let mut warning = None;
        let mut temporary_cred = false;
        if self.save {
            match self.save_profile(&opts) {
                Ok(w) => warning = w,
                Err(e) => return self.info(t("err_title"), tf("err_save", &[("err", &e)])),
            }
        } else if !self.pass.is_empty() {
            // Not saved: hand the password to mstsc through Credential Manager and remove it when
            // the session ends, unless a stored password for this host already existed.
            let had = rdp::has_credential(&host);
            temporary_cred = rdp::store_credential(&host, &user, &self.pass).is_ok() && !had;
        }
        self.pass.clear();

        let tx = self.tx.clone();
        let ctx = self.ctx.clone();
        let ended_host = host.clone();
        let result = rdp::start_session(&opts, move || {
            if temporary_cred {
                rdp::delete_credential(&ended_host);
            }
            let _ = tx.send(Msg::SessionEnded(ended_host));
            ctx.request_repaint();
        });
        match result {
            Ok(()) => match warning {
                Some(w) => self.toast(w),
                None => self.toast(tf("toast_connecting", &[("host", &host)])),
            },
            Err(e) => {
                if temporary_cred {
                    rdp::delete_credential(&host);
                }
                self.info(t("alert_launch_err"), e);
            }
        }
        self.update_pass_hint();
    }

    /// Saves the form as a profile (updating the loaded or same-host one). Returns a warning, if any.
    fn save_profile(&mut self, o: &Options) -> Result<Option<String>, String> {
        let found = self.profiles.iter().position(|p| {
            self.loaded_id.as_deref() == Some(p.id.as_str()) || p.host.trim().eq_ignore_ascii_case(&o.host)
        });
        let mut p = match found {
            Some(i) => self.profiles[i].clone(),
            None => Profile {
                id: profiles::new_id(),
                name: String::new(),
                host: String::new(),
                port: profiles::DEFAULT_PORT,
                username: String::new(),
                display_mode: "windowed".into(),
                sound: true,
                clipboard: true,
                multimon: false,
                save_password: true,
            },
        };
        let old_host = p.host.clone();
        p.name = self.name.trim().to_string();
        p.host = o.host.clone();
        p.port = o.port;
        p.username = o.username.clone();
        p.display_mode = o.display_mode.clone();
        p.sound = o.sound;
        p.clipboard = o.clipboard;
        p.multimon = o.multimon;
        p.save_password = true;

        let mut updated = self.profiles.clone();
        match found {
            Some(i) => updated[i] = p.clone(),
            None => updated.push(p.clone()),
        }
        profiles::save(&updated).map_err(|e| e.to_string())?;
        self.profiles = updated;
        self.loaded_id = Some(p.id.clone());

        if !old_host.is_empty()
            && !old_host.eq_ignore_ascii_case(&o.host)
            && profiles::find_by_host(&self.profiles, &old_host).is_none()
        {
            rdp::delete_credential(&old_host);
        }
        if !self.pass.is_empty() && rdp::store_credential(&o.host, &o.username, &self.pass).is_err() {
            return Ok(Some(t("toast_keyring_warn")));
        }
        Ok(None)
    }

    fn apply(&mut self, action: Action) {
        match action {
            Action::RefreshPeers => self.refresh_peers(),
            Action::Clear => self.clear_form(),
            Action::SelectPeer(p) => self.select_peer(&p),
            Action::LoadProfile(p, announce) => {
                self.load_profile(&p);
                if announce {
                    self.toast(tf("toast_profile_loaded", &[("name", p.display_name())]));
                }
            }
            Action::ConnectProfile(p) => {
                self.load_profile(&p);
                self.connect();
            }
            Action::AskDelete(p) => {
                self.dialog = Some(Dialog {
                    title: t("confirm_delete_title"),
                    body: tf("confirm_delete", &[("name", p.display_name())]),
                    kind: DialogKind::ConfirmDelete(p.id.clone()),
                })
            }
            Action::Delete(id) => self.delete_profile(&id),
            Action::Test => self.test(),
            Action::Connect => self.connect(),
        }
    }

    fn poll(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Msg::Peers(res) => {
                    self.peers = match res {
                        Err(tailscale::Error::NotInstalled) => Peers::Message(t("tailscale_not_found")),
                        Err(tailscale::Error::Unavailable) => Peers::Message(t("tailscale_disconnected")),
                        Err(tailscale::Error::Invalid(e)) => {
                            Peers::Message(tf("tailscale_invalid_json", &[("err", &e)]))
                        }
                        Ok((state, _)) if state != "Running" => {
                            Peers::Message(tf("tailscale_inactive", &[("state", &state)]))
                        }
                        Ok((_, list)) if list.is_empty() => Peers::Message(t("tailscale_empty")),
                        Ok((_, list)) => Peers::List(list),
                    }
                }
                Msg::Test { host, port, result } => {
                    let port = port.to_string();
                    match result {
                        Ok(()) => self.toast(tf("toast_test_ok", &[("host", &host), ("port", &port)])),
                        Err(e) => self.info(
                            tf("alert_test_fail_title", &[("host", &host), ("port", &port)]),
                            tf("alert_test_fail_body", &[("port", &port), ("err", &e)]),
                        ),
                    }
                }
                Msg::SessionEnded(host) => self.toast(tf("toast_session_ended", &[("host", &host)])),
            }
        }
    }
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "ubuntu".into(),
        Arc::new(FontData::from_static(include_bytes!("../assets/fonts/UbuntuSans-Regular.ttf"))),
    );
    fonts.font_data.insert(
        "ubuntu-bold".into(),
        Arc::new(FontData::from_static(include_bytes!("../assets/fonts/UbuntuSans-Bold.ttf"))),
    );
    let fallback = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "ubuntu".into());
    let mut bold_family = vec!["ubuntu-bold".to_string()];
    bold_family.extend(fallback);
    fonts.families.insert(FontFamily::Name("bold".into()), bold_family);
    ctx.set_fonts(fonts);
}

// ---------------------------------------------------------------------------------------------
// Widgets

fn galley(ui: &Ui, text: &str, font: FontId, color: Color32, max_width: f32, wrap: bool) -> Arc<egui::Galley> {
    let mut job = LayoutJob::simple(text.to_owned(), font, color, max_width);
    if !wrap {
        job.wrap = TextWrapping::truncate_at_width(max_width);
    }
    ui.fonts(|f| f.layout_job(job))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BtnStyle {
    Normal,
    Suggested,
    Pill,
    PillSuggested,
}

fn button(ui: &mut Ui, pal: &Palette, text: &str, style: BtnStyle) -> Response {
    let pill = matches!(style, BtnStyle::Pill | BtnStyle::PillSuggested);
    let suggested = matches!(style, BtnStyle::Suggested | BtnStyle::PillSuggested);
    let g = galley(ui, text, bold(BODY), Color32::PLACEHOLDER, f32::INFINITY, false);
    let (pad_x, height) = if pill { (32.0, 40.0) } else { (17.0, 34.0) };
    let size = vec2(g.size().x + pad_x * 2.0, height);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let (bg, fg) = match (suggested, resp.is_pointer_button_down_on(), resp.hovered()) {
        (true, true, _) => (pal.accent_active, pal.accent_fg),
        (true, false, true) => (pal.accent_hover, pal.accent_fg),
        (true, false, false) => (pal.accent_bg, pal.accent_fg),
        (false, true, _) => (pal.btn_active, pal.fg),
        (false, false, true) => (pal.btn_hover, pal.fg),
        (false, false, false) => (pal.btn, pal.fg),
    };
    let radius = if pill { CornerRadius::same(255) } else { CornerRadius::same(6) };
    ui.painter().rect_filled(rect, radius, bg);
    ui.painter().galley(rect.center() - g.size() / 2.0, g, fg);
    resp
}

fn icon_button(ui: &mut Ui, pal: &Palette, icon: Icon, tooltip: &str) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::click());
    if resp.is_pointer_button_down_on() {
        ui.painter().rect_filled(rect, CornerRadius::same(6), pal.btn_active);
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(6), pal.flat_hover);
    }
    paint_icon(ui, icon, rect.center(), pal.fg);
    resp.on_hover_text(tooltip)
}

fn switch(ui: &mut Ui, pal: &Palette, on: bool) -> Rect {
    let (rect, _) = ui.allocate_exact_size(vec2(46.0, 26.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(13), if on { pal.accent_bg } else { pal.switch_off });
    let knob_x = if on { rect.right() - 13.0 } else { rect.left() + 13.0 };
    let c = pos2(knob_x, rect.center().y);
    p.circle_filled(c + vec2(0.0, 1.0), 10.5, Color32::from_black_alpha(40));
    p.circle_filled(c, 10.0, Color32::WHITE);
    rect
}

/// Where a row sits in its boxed list, for corner rounding and separators.
#[derive(Clone, Copy)]
struct RowPos {
    first: bool,
    last: bool,
}

fn row_radius(pos: RowPos) -> CornerRadius {
    CornerRadius {
        nw: if pos.first { RADIUS } else { 0 },
        ne: if pos.first { RADIUS } else { 0 },
        sw: if pos.last { RADIUS } else { 0 },
        se: if pos.last { RADIUS } else { 0 },
    }
}

/// Allocates a full-width row, paints hover/press feedback and the separator above it.
fn row_base(ui: &mut Ui, pal: &Palette, height: f32, pos: RowPos, clickable: bool) -> (Rect, Response) {
    let sense = if clickable { Sense::click() } else { Sense::hover() };
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), height), sense);
    if clickable {
        let bg = if resp.is_pointer_button_down_on() {
            Some(pal.card_active)
        } else if resp.hovered() {
            Some(pal.card_hover)
        } else {
            None
        };
        if let Some(bg) = bg {
            ui.painter().rect_filled(rect, row_radius(pos), bg);
        }
    }
    if !pos.first {
        ui.painter().hline(rect.x_range(), rect.top() + 0.5, Stroke::new(1.0_f32, pal.separator));
    }
    (rect, resp)
}

/// Lays out suffix widgets right-aligned inside the row; returns the left edge they reached.
fn suffix(ui: &mut Ui, rect: Rect, add: impl FnOnce(&mut Ui)) -> f32 {
    let inner = rect.shrink2(vec2(12.0, 0.0));
    let mut child = ui.new_child(UiBuilder::new().max_rect(inner).layout(Layout::right_to_left(Align::Center)));
    child.spacing_mut().item_spacing = vec2(6.0, 0.0);
    add(&mut child);
    let used = child.min_rect();
    if used.width() > 0.0 { used.left() } else { inner.right() }
}

/// Title + subtitle texts of an action row, vertically centered.
fn row_texts(ui: &Ui, pal: &Palette, rect: Rect, left: f32, right: f32, title: &str, sub: Option<&str>, wrap: bool) {
    let w = (right - left - 12.0).max(10.0);
    let tg = galley(ui, title, regular(BODY), pal.fg, w, wrap);
    let sg = sub.map(|s| galley(ui, s, regular(SMALL), pal.dim, w, wrap));
    let total = tg.size().y + sg.as_ref().map(|g| g.size().y + 2.0).unwrap_or(0.0);
    let mut y = rect.center().y - total / 2.0;
    let th = tg.size().y;
    ui.painter().galley(pos2(left, y), tg, pal.fg);
    y += th + 2.0;
    if let Some(sg) = sg {
        ui.painter().galley(pos2(left, y), sg, pal.dim);
    }
}

fn text_height(ui: &Ui, title: &str, sub: Option<&str>, width: f32) -> f32 {
    let th = galley(ui, title, regular(BODY), Color32::WHITE, width, true).size().y;
    let sh = sub.map(|s| galley(ui, s, regular(SMALL), Color32::WHITE, width, true).size().y + 2.0).unwrap_or(0.0);
    th + sh
}

fn boxed_list(ui: &mut Ui, pal: &Palette, add: impl FnOnce(&mut Ui)) {
    Frame::new()
        .fill(pal.card)
        .corner_radius(CornerRadius::same(RADIUS))
        .shadow(Shadow { offset: [0, 1], blur: 4, spread: 0, color: pal.shadow })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        });
}

fn group_header(ui: &mut Ui, pal: &Palette, title: &str, desc: Option<&str>) {
    let w = ui.available_width() - 12.0;
    let tg = galley(ui, title, bold(BODY), pal.fg, w, true);
    let dg = desc.map(|d| galley(ui, d, regular(BODY), pal.dim, w, true));
    let h = tg.size().y + dg.as_ref().map(|g| g.size().y + 6.0).unwrap_or(0.0) + 12.0;
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::hover());
    let th = tg.size().y;
    ui.painter().galley(pos2(rect.left() + 6.0, rect.top()), tg, pal.fg);
    if let Some(dg) = dg {
        ui.painter().galley(pos2(rect.left() + 6.0, rect.top() + th + 6.0), dg, pal.dim);
    }
}

/// Adw.Clamp width: grows from the tightening threshold up to the maximum with an ease-out curve.
fn clamp_width(available: f32) -> f32 {
    let (lower, max) = (400.0_f32, 520.0_f32);
    let amplitude = max - lower;
    let upper = 4.0 * amplitude + lower;
    if available <= lower {
        return available;
    }
    if available >= upper {
        return max;
    }
    let p = (available - lower) / (upper - lower);
    let eased = 1.0 - (1.0 - p).powi(3);
    lower + amplitude * eased
}

// ---------------------------------------------------------------------------------------------
// Frame

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll();
        let dark = ctx.theme() == egui::Theme::Dark;
        let pal = if dark { DARK } else { LIGHT };
        #[cfg(windows)]
        if self.caption_dark != Some(dark) {
            self.caption_dark = Some(dark);
            set_caption_color(_frame, &pal, dark);
        }
        ctx.style_mut(|s| {
            s.visuals.text_cursor.stroke = Stroke::new(2.0_f32, pal.accent_bg);
            s.visuals.selection.bg_fill = pal.accent_bg.gamma_multiply(0.5);
            s.visuals.selection.stroke = Stroke::new(1.0_f32, pal.fg);
            s.visuals.panel_fill = pal.window_bg;
            s.visuals.window_fill = pal.popover_bg;
            s.visuals.override_text_color = Some(pal.fg);
        });

        let mut actions: Vec<Action> = Vec::new();

        // Header bar
        egui::TopBottomPanel::top("header")
            .exact_height(47.0)
            .show_separator_line(false)
            .frame(Frame::new().fill(pal.header_bg).inner_margin(Margin::symmetric(6, 0)))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                ui.painter().hline(rect.x_range(), rect.bottom() - 0.5, Stroke::new(1.0_f32, pal.header_shade));
                ui.painter().text(rect.center(), Align2::CENTER_CENTER, t("app_name"), bold(BODY), pal.fg);
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    if icon_button(ui, &pal, Icon::Refresh, &t("tooltip_refresh")).clicked() {
                        actions.push(Action::RefreshPeers);
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if icon_button(ui, &pal, Icon::Clear, &t("tooltip_clear")).clicked() {
                            actions.push(Action::Clear);
                        }
                    });
                });
            });

        egui::CentralPanel::default().frame(Frame::new().fill(pal.window_bg)).show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                let full = ui.available_width();
                let width = clamp_width(full - 32.0);
                let side = ((full - width) / 2.0).max(0.0);
                ui.horizontal_top(|ui| {
                    ui.add_space(side);
                    ui.vertical(|ui| {
                        ui.set_width(width);
                        ui.add_space(18.0);
                        self.draw_content(ui, &pal, &mut actions);
                        ui.add_space(24.0);
                    });
                });
            });
        });

        self.draw_dialog(ctx, &pal, &mut actions);
        self.draw_toast(ctx, &pal);

        for a in actions {
            self.apply(a);
        }
    }
}

impl App {
    fn draw_content(&mut self, ui: &mut Ui, pal: &Palette, actions: &mut Vec<Action>) {
        // PCs no Tailscale
        group_header(ui, pal, &t("group_tailscale_title"), Some(&t("group_tailscale_desc")));
        boxed_list(ui, pal, |ui| match &self.peers {
            Peers::Loading => self.placeholder_row(ui, pal, &t("tailscale_loading")),
            Peers::Message(m) => {
                let m = m.clone();
                self.placeholder_row(ui, pal, &m)
            }
            Peers::List(list) => {
                let n = list.len();
                for (i, peer) in list.iter().enumerate() {
                    let pos = RowPos { first: i == 0, last: i + 1 == n };
                    let (rect, resp) = row_base(ui, pal, 55.0, pos, true);
                    let matched = profiles::find_by_host(&self.profiles, &peer.ip).filter(|p| !p.username.is_empty());
                    let mut clicked_btn = false;
                    let left_edge = suffix(ui, rect, |ui| {
                        let (label, style) = if matched.is_some() {
                            (t("btn_connect"), BtnStyle::Suggested)
                        } else {
                            (t("btn_select"), BtnStyle::Normal)
                        };
                        clicked_btn = button(ui, pal, &label, style).clicked();
                    });
                    let icon = if peer.os == "windows" { Icon::Workgroup } else { Icon::Computer };
                    paint_icon(ui, icon, pos2(rect.left() + 22.0, rect.center().y), pal.fg);
                    let os = if peer.os.is_empty() {
                        t("os_unknown")
                    } else {
                        let mut c = peer.os.chars();
                        c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
                    };
                    let status = if peer.online { t("status_online") } else { t("status_offline") };
                    let sub = format!("{} • {} • {}", peer.ip, os, status);
                    row_texts(ui, pal, rect, rect.left() + 42.0, left_edge, &peer.hostname, Some(&sub), false);
                    if clicked_btn {
                        match matched {
                            Some(p) => actions.push(Action::ConnectProfile(p.clone())),
                            None => actions.push(Action::SelectPeer(peer.clone())),
                        }
                    } else if resp.clicked() {
                        actions.push(Action::SelectPeer(peer.clone()));
                    }
                }
            }
        });
        ui.add_space(18.0);

        // Meus PCs Salvos
        if !self.profiles.is_empty() {
            group_header(ui, pal, &t("group_saved_title"), Some(&t("group_saved_desc")));
            boxed_list(ui, pal, |ui| {
                let n = self.profiles.len();
                for (i, p) in self.profiles.iter().enumerate() {
                    let pos = RowPos { first: i == 0, last: i + 1 == n };
                    let (rect, resp) = row_base(ui, pal, 55.0, pos, true);
                    let (mut connect, mut delete) = (false, false);
                    let left_edge = suffix(ui, rect, |ui| {
                        delete = icon_button(ui, pal, Icon::Trash, &t("tooltip_delete")).clicked();
                        connect = button(ui, pal, &t("btn_connect"), BtnStyle::Suggested).clicked();
                    });
                    paint_icon(ui, Icon::Computer, pos2(rect.left() + 22.0, rect.center().y), pal.fg);
                    let ep = p.endpoint();
                    let sub = if p.username.is_empty() { ep } else { format!("{} @ {}", p.username, ep) };
                    row_texts(ui, pal, rect, rect.left() + 42.0, left_edge, p.display_name(), Some(&sub), false);
                    if connect {
                        actions.push(Action::ConnectProfile(p.clone()));
                    } else if delete {
                        actions.push(Action::AskDelete(p.clone()));
                    } else if resp.clicked() {
                        actions.push(Action::LoadProfile(p.clone(), true));
                    }
                }
            });
            ui.add_space(18.0);
        }

        // Dados de Acesso
        group_header(ui, pal, &t("group_creds_title"), None);
        let mut enter = false;
        boxed_list(ui, pal, |ui| {
            enter |= self.entry_row(ui, pal, Field::Host, RowPos { first: true, last: false });
            enter |= self.entry_row(ui, pal, Field::User, RowPos { first: false, last: false });
            enter |= self.entry_row(ui, pal, Field::Pass, RowPos { first: false, last: false });
            enter |= self.entry_row(ui, pal, Field::Name, RowPos { first: false, last: false });
            let title = t("switch_save_title");
            let sub = t("switch_save_sub");
            let tw = ui.available_width() - 12.0 - 12.0 - 46.0 - 12.0;
            let h = (text_height(ui, &title, Some(&sub), tw) + 16.0).max(50.0);
            let (rect, resp) = row_base(ui, pal, h, RowPos { first: false, last: true }, true);
            let left_edge = suffix(ui, rect, |ui| {
                switch(ui, pal, self.save);
            });
            row_texts(ui, pal, rect, rect.left() + 12.0, left_edge, &title, Some(&sub), true);
            if resp.clicked() {
                self.save = !self.save;
            }
        });
        if enter {
            actions.push(Action::Connect);
        }
        ui.add_space(18.0);

        // Opções da Sessão
        group_header(ui, pal, &t("group_opts_title"), None);
        boxed_list(ui, pal, |ui| {
            self.combo_row(ui, pal, RowPos { first: true, last: false });
            let title = t("switch_multimon");
            let sub = t("switch_multimon_sub");
            let tw = ui.available_width() - 12.0 - 12.0 - 46.0 - 12.0;
            let h = (text_height(ui, &title, Some(&sub), tw) + 16.0).max(50.0);
            let (rect, resp) = row_base(ui, pal, h, RowPos { first: false, last: false }, true);
            let left_edge = suffix(ui, rect, |ui| {
                switch(ui, pal, self.multimon);
            });
            row_texts(ui, pal, rect, rect.left() + 12.0, left_edge, &title, Some(&sub), true);
            if resp.clicked() {
                self.multimon = !self.multimon;
            }
            let title = t("switch_sound");
            let (rect, resp) = row_base(ui, pal, 50.0, RowPos { first: false, last: false }, true);
            let left_edge = suffix(ui, rect, |ui| {
                switch(ui, pal, self.sound);
            });
            row_texts(ui, pal, rect, rect.left() + 12.0, left_edge, &title, None, true);
            if resp.clicked() {
                self.sound = !self.sound;
            }
            let title = t("switch_clipboard");
            let sub = t("switch_clipboard_sub");
            let tw = ui.available_width() - 12.0 - 12.0 - 46.0 - 12.0;
            let h = (text_height(ui, &title, Some(&sub), tw) + 16.0).max(50.0);
            let (rect, resp) = row_base(ui, pal, h, RowPos { first: false, last: true }, true);
            let left_edge = suffix(ui, rect, |ui| {
                switch(ui, pal, self.clipboard);
            });
            row_texts(ui, pal, rect, rect.left() + 12.0, left_edge, &title, Some(&sub), true);
            if resp.clicked() {
                self.clipboard = !self.clipboard;
            }
        });
        ui.add_space(24.0);

        // Botões
        let test_label = t("btn_test");
        let connect_label = t("btn_connect_rdp");
        let w = |s: &str, ui: &Ui| galley(ui, s, bold(BODY), Color32::WHITE, f32::INFINITY, false).size().x + 64.0;
        let total = w(&test_label, ui) + 12.0 + w(&connect_label, ui);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(12.0, 0.0);
            ui.add_space(((ui.available_width() - total) / 2.0).max(0.0));
            if button(ui, pal, &test_label, BtnStyle::Pill).clicked() {
                actions.push(Action::Test);
            }
            if button(ui, pal, &connect_label, BtnStyle::PillSuggested).clicked() {
                actions.push(Action::Connect);
            }
        });
    }

    fn placeholder_row(&self, ui: &mut Ui, pal: &Palette, text: &str) {
        let w = ui.available_width() - 24.0;
        let h = (text_height(ui, text, None, w) + 16.0).max(50.0);
        let (rect, _) = row_base(ui, pal, h, RowPos { first: true, last: true }, false);
        row_texts(ui, pal, rect, rect.left() + 12.0, rect.right() - 12.0, text, None, true);
    }

    /// Adw.EntryRow: title inside the row that floats up when focused or filled. Returns true on Enter.
    fn entry_row(&mut self, ui: &mut Ui, pal: &Palette, field: Field, pos: RowPos) -> bool {
        let id = Id::new(("entry", field as u8));
        let (rect, resp) = row_base(ui, pal, 54.0, pos, true);
        let has_focus = ui.ctx().memory(|m| m.has_focus(id));
        let is_pass = field == Field::Pass;

        // Suffix icons: edit pencil (hidden while editing) and the password peek toggle.
        let mut toggle_peek = false;
        let show_pass = self.show_pass;
        let left_edge = suffix(ui, rect, |ui| {
            if is_pass {
                toggle_peek = icon_button(ui, pal, if show_pass { Icon::Conceal } else { Icon::Reveal }, "").clicked();
                ui.add_space(-6.0);
            }
            if !has_focus {
                let (r, _) = ui.allocate_exact_size(vec2(28.0, 34.0), Sense::hover());
                paint_icon(ui, Icon::Edit, r.center(), pal.fg);
            }
        });
        if toggle_peek {
            self.show_pass = !self.show_pass;
        }

        let label = match field {
            Field::Host => t("entry_host"),
            Field::User => t("entry_user"),
            Field::Pass if self.pass_saved => format!("{} {}", t("entry_pass"), t("entry_pass_saved")),
            Field::Pass => t("entry_pass"),
            Field::Name => t("entry_name"),
        };
        let value = match field {
            Field::Host => &mut self.host,
            Field::User => &mut self.user,
            Field::Pass => &mut self.pass,
            Field::Name => &mut self.name,
        };
        let floating = has_focus || !value.is_empty();
        let left = rect.left() + 12.0;
        let text_right = left_edge - 6.0;
        if floating {
            let color = if has_focus { pal.accent_text } else { pal.dim };
            let g = galley(ui, &label, regular(SMALL), color, text_right - left, false);
            ui.painter().galley(pos2(left, rect.top() + 7.0), g, color);
        } else {
            let g = galley(ui, &label, regular(BODY), pal.dim, text_right - left, false);
            let y = rect.center().y - g.size().y / 2.0;
            ui.painter().galley(pos2(left, y), g, pal.dim);
        }

        let edit_rect = Rect::from_min_max(pos2(left, rect.top() + 25.0), pos2(text_right, rect.bottom() - 7.0));
        let mut child = ui.new_child(UiBuilder::new().max_rect(edit_rect).layout(Layout::left_to_right(Align::Center)));
        let edit = egui::TextEdit::singleline(value)
            .id(id)
            .frame(false)
            .margin(Margin::ZERO)
            .font(regular(BODY))
            .text_color(pal.fg)
            .desired_width(edit_rect.width())
            .password(is_pass && !self.show_pass);
        let out = child.add(edit);

        if resp.clicked() || self.focus == Some(field) {
            out.request_focus();
            if self.focus == Some(field) {
                self.focus = None;
            }
        }
        if field == Field::Host && out.lost_focus() {
            self.update_pass_hint();
        }
        if has_focus {
            ui.painter().rect_stroke(rect.shrink(1.0), row_radius(pos), Stroke::new(2.0_f32, pal.focus), StrokeKind::Inside);
        }
        out.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter))
    }

    /// Adw.ComboRow with a popover list.
    fn combo_row(&mut self, ui: &mut Ui, pal: &Palette, pos: RowPos) {
        let modes = [t("mode_windowed"), t("mode_fullscreen"), t("mode_1080"), t("mode_768")];
        let (rect, resp) = row_base(ui, pal, 50.0, pos, true);
        let popup_id = Id::new("mode-popover");
        let value = &modes[self.mode];
        let arrow_x = rect.right() - 22.0;
        paint_icon(ui, Icon::Down, pos2(arrow_x, rect.center().y), pal.fg);
        let vg = galley(ui, value, regular(BODY), pal.dim, rect.width() * 0.6, false);
        let value_left = arrow_x - 14.0 - vg.size().x;
        let vh = vg.size().y;
        ui.painter().galley(pos2(value_left, rect.center().y - vh / 2.0), vg, pal.dim);
        row_texts(ui, pal, rect, rect.left() + 12.0, value_left - 6.0, &t("combo_display"), None, false);

        let anchor = Rect::from_min_max(pos2(value_left - 12.0, rect.top()), pos2(rect.right() - 6.0, rect.bottom() - 4.0));
        let anchor_resp = ui.interact(anchor, Id::new("mode-anchor"), Sense::hover());
        if resp.clicked() {
            ui.memory_mut(|m| m.toggle_popup(popup_id));
        }
        let mut chosen = None;
        let current = self.mode;
        let pal_c = *pal;
        egui::popup_below_widget(ui, popup_id, &anchor_resp, egui::PopupCloseBehavior::CloseOnClick, |ui| {
            ui.set_min_width(anchor.width().max(240.0));
            for (i, label) in modes.iter().enumerate() {
                let (r, item) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::click());
                if item.hovered() {
                    ui.painter().rect_filled(r, CornerRadius::same(6), pal_c.flat_hover);
                }
                ui.painter().text(pos2(r.left() + 12.0, r.center().y), Align2::LEFT_CENTER, label, regular(BODY), pal_c.fg);
                if i == current {
                    paint_icon(ui, Icon::Check, pos2(r.right() - 20.0, r.center().y), pal_c.fg);
                }
                if item.clicked() {
                    chosen = Some(i);
                }
            }
        });
        if let Some(i) = chosen {
            self.mode = i;
        }
    }

    fn draw_toast(&mut self, ctx: &egui::Context, pal: &Palette) {
        let Some((msg, at)) = &self.toast else { return };
        let age = at.elapsed().as_secs_f32();
        let total = 3.2_f32;
        if age > total {
            self.toast = None;
            return;
        }
        let alpha = (age / 0.2).min(1.0).min((total - age) / 0.25).clamp(0.0, 1.0);
        let screen = ctx.screen_rect();
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, Id::new("toast")));
        let g = painter.layout(msg.clone(), regular(BODY), Color32::WHITE, (screen.width() - 96.0).max(100.0));
        let size = g.size() + vec2(44.0, 20.0);
        let center = pos2(screen.center().x, screen.bottom() - 24.0 - size.y / 2.0 + (1.0 - alpha) * 20.0);
        let rect = Rect::from_center_size(center, size);
        let shadow = Shadow { offset: [0, 2], blur: 8, spread: 0, color: Color32::from_black_alpha((90.0 * alpha) as u8) };
        painter.add(shadow.as_shape(rect, CornerRadius::same(255)));
        painter.rect_filled(rect, CornerRadius::same(255), pal.toast_bg.gamma_multiply(alpha));
        painter.galley(rect.center() - g.size() / 2.0, g, Color32::WHITE.gamma_multiply(alpha));
        ctx.request_repaint_after(Duration::from_millis(16));
    }

    fn draw_dialog(&mut self, ctx: &egui::Context, pal: &Palette, actions: &mut Vec<Action>) {
        let Some(dialog) = &self.dialog else { return };
        let mut result: Option<bool> = None; // Some(true) = confirm, Some(false) = cancel
        let frame = Frame::new()
            .fill(pal.popover_bg)
            .corner_radius(CornerRadius::same(18))
            .inner_margin(Margin { left: 24, right: 24, top: 32, bottom: 24 })
            .shadow(Shadow { offset: [0, 8], blur: 32, spread: 0, color: Color32::from_black_alpha(90) });
        let modal = egui::Modal::new(Id::new("alert")).frame(frame).backdrop_color(pal.scrim).show(ctx, |ui| {
            let width = (ctx.screen_rect().width() - 96.0).clamp(220.0, 324.0);
            ui.set_width(width);
            let tg = galley(ui, &dialog.title, bold(18.7), pal.fg, width, true);
            let (r, _) = ui.allocate_exact_size(vec2(width, tg.size().y), Sense::hover());
            let tw = tg.size().x;
            ui.painter().galley(pos2(r.center().x - tw / 2.0, r.top()), tg, pal.fg);
            if !dialog.body.is_empty() {
                ui.add_space(12.0);
                let max_h = (ctx.screen_rect().height() * 0.55).max(120.0);
                egui::ScrollArea::vertical().max_height(max_h).auto_shrink([false, true]).show(ui, |ui| {
                    let bg = galley(ui, &dialog.body, regular(BODY), pal.fg, width, true);
                    let (r, _) = ui.allocate_exact_size(bg.size(), Sense::hover());
                    ui.painter().galley(r.min, bg, pal.fg);
                });
            }
            ui.add_space(24.0);
            let buttons: Vec<(String, bool, bool)> = match dialog.kind {
                DialogKind::Info => vec![(t("btn_ok"), true, false)],
                DialogKind::ConfirmDelete(_) => vec![(t("btn_cancel"), false, false), (t("btn_delete"), true, true)],
            };
            let n = buttons.len() as f32;
            let bw = (width - 12.0 * (n - 1.0)) / n;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(12.0, 0.0);
                for (label, confirm, destructive) in buttons {
                    let (r, resp) = ui.allocate_exact_size(vec2(bw, 40.0), Sense::click());
                    let bg = if resp.is_pointer_button_down_on() {
                        pal.btn_active
                    } else if resp.hovered() {
                        pal.btn_hover
                    } else {
                        pal.btn
                    };
                    ui.painter().rect_filled(r, CornerRadius::same(255), bg);
                    let color = if destructive { pal.destructive } else { pal.fg };
                    ui.painter().text(r.center(), Align2::CENTER_CENTER, label, bold(BODY), color);
                    if resp.clicked() {
                        result = Some(confirm);
                    }
                }
            });
            if ui.input(|i| i.key_pressed(Key::Enter)) {
                result = Some(true);
            }
        });
        if result.is_none() && modal.should_close() {
            result = Some(false);
        }
        if let Some(confirm) = result {
            if let (true, DialogKind::ConfirmDelete(id)) = (confirm, &dialog.kind) {
                actions.push(Action::Delete(id.clone()));
            }
            self.dialog = None;
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Windows 11 title bar in the header bar color

#[cfg(windows)]
fn set_caption_color(frame: &eframe::Frame, pal: &Palette, dark: bool) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::Graphics::Dwm::DwmSetWindowAttribute;
    let Ok(handle) = frame.window_handle() else { return };
    let RawWindowHandle::Win32(h) = handle.as_raw() else { return };
    let hwnd = h.hwnd.get() as *mut core::ffi::c_void;
    let set = |attr: u32, value: u32| unsafe {
        DwmSetWindowAttribute(hwnd, attr, &value as *const u32 as *const _, 4);
    };
    let c = pal.header_bg;
    let colorref = u32::from(c.r()) | u32::from(c.g()) << 8 | u32::from(c.b()) << 16;
    set(20, u32::from(dark)); // DWMWA_USE_IMMERSIVE_DARK_MODE
    set(35, colorref); // DWMWA_CAPTION_COLOR
    set(34, colorref); // DWMWA_BORDER_COLOR
}
