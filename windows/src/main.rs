//! DesktopRDP for Windows: lists Tailscale computers, keeps saved connections and opens RDP
//! sessions (for example to a Linux PC running GNOME Remote Desktop) with mstsc.exe.
//!
//! The UI is drawn natively with egui and reproduces the Linux app's GTK4/Libadwaita look.
#![cfg_attr(windows, windows_subsystem = "windows")]

mod i18n;
mod profiles;
mod rdp;
mod tailscale;
mod ui;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let lang_flag = args
        .iter()
        .position(|a| a == "--lang")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
        .or_else(|| args.iter().find_map(|a| a.strip_prefix("--lang=")));
    i18n::set_lang(i18n::detect(lang_flag));

    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../winres/icon.png")).ok();
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title(i18n::t("app_name"))
        .with_app_id("io.github.dilucayvl.DesktopRDP")
        .with_inner_size([560.0, 720.0])
        .with_min_inner_size([360.0, 420.0]);
    if let Some(icon) = icon {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }
    let options = eframe::NativeOptions { viewport, centered: true, ..Default::default() };
    eframe::run_native(&i18n::t("app_name"), options, Box::new(|cc| Ok(Box::new(ui::App::new(cc)))))
}
