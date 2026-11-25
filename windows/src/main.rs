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

const WIDTH: f32 = 560.0;
const HEIGHT: f32 = 720.0;

/// Initial window position and inner size (logical points) centered in the primary monitor's work
/// area (screen minus taskbar), shrunk when the default size does not fit.
#[cfg(windows)]
fn fit_work_area() -> Option<(eframe::egui::Pos2, eframe::egui::Vec2)> {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::HiDpi::GetDpiForSystem;
    use windows_sys::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETWORKAREA};

    let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    if unsafe { SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut r as *mut RECT as *mut _, 0) } == 0 {
        return None;
    }
    let scale = (unsafe { GetDpiForSystem() } as f32 / 96.0).max(1.0);
    let (left, top) = (r.left as f32 / scale, r.top as f32 / scale);
    let (w, h) = ((r.right - r.left) as f32 / scale, (r.bottom - r.top) as f32 / scale);
    if w < 100.0 || h < 100.0 {
        return None;
    }
    // Window frame around the client area: caption (~31) + borders, plus a small gap.
    const FRAME_W: f32 = 16.0;
    const FRAME_H: f32 = 48.0;
    let size = eframe::egui::vec2(WIDTH.min(w - FRAME_W).max(360.0), HEIGHT.min(h - FRAME_H).max(420.0));
    let x = left + ((w - size.x - FRAME_W) / 2.0).max(0.0);
    let y = top + ((h - size.y - FRAME_H) / 2.0).max(0.0);
    Some((eframe::egui::pos2(x, y), size))
}

#[cfg(not(windows))]
fn fit_work_area() -> Option<(eframe::egui::Pos2, eframe::egui::Vec2)> {
    None
}

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
        .with_inner_size([WIDTH, HEIGHT])
        .with_min_inner_size([360.0, 420.0]);
    let mut centered = true;
    // Small screens (e.g. 1366x768 laptops): keep the whole window, buttons included, above the taskbar.
    if let Some((pos, size)) = fit_work_area() {
        viewport = viewport.with_position(pos).with_inner_size(size);
        centered = false;
    }
    if let Some(icon) = icon {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }
    let options = eframe::NativeOptions { viewport, centered, ..Default::default() };
    eframe::run_native(&i18n::t("app_name"), options, Box::new(|cc| Ok(Box::new(ui::App::new(cc)))))
}
