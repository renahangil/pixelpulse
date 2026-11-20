//! Geometria nativa da ilha: não sobra uma área transparente interceptando cliques.

use crate::{
    cache::{AppState, Mode},
    settings::Settings,
};
use std::io;
use tauri::{
    AppHandle, LogicalSize, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder, WindowEvent,
};

const TOP_MARGIN: f64 = 8.0;
pub const EXPANDED_HEIGHT: u32 = 230;
pub const EXTRA_ROW_HEIGHT: u32 = 36;

#[derive(Clone, Copy, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GlassMode {
    Acrylic,
    Opaque,
    Fallback,
}

fn configure_glass(window: &WebviewWindow) -> GlassMode {
    let prefer_acrylic = match std::env::var("PIXELPULSE_GLASS").as_deref() {
        Ok("acrylic") => true,
        Ok("fallback") => false,
        _ => return GlassMode::Opaque,
    };
    if prefer_acrylic {
        // No Windows 11 recente, o crate usa Desktop Acrylic e ignora esta tinta.
        // A tupla só colore o caminho legado; a superfície CSS escurece ambos.
        match window_vibrancy::apply_acrylic(window, Some((20, 20, 26, 72))) {
            Ok(()) => {
                eprintln!("[glass] acrylic");
                return GlassMode::Acrylic;
            }
            Err(error) => {
                eprintln!("[glass] acrylic indisponível, usando RGBA: {error}");
                let _ = window_vibrancy::clear_acrylic(window);
            }
        }
    }
    eprintln!("[glass] fallback RGBA sem desfoque do sistema");
    GlassMode::Fallback
}

fn geometry(
    mode: Mode,
    origin: PhysicalPosition<i32>,
    monitor: PhysicalSize<u32>,
    scale: f64,
) -> (PhysicalSize<u32>, PhysicalPosition<i32>) {
    let logical = match mode {
        Mode::Collapsed => LogicalSize::new(180.0, 36.0),
        Mode::Expanded => LogicalSize::new(340.0, f64::from(EXPANDED_HEIGHT)),
    };
    // Monitor usa pixels físicos; converter a janela evita misturar unidades no DPI.
    let size: PhysicalSize<u32> = logical.to_physical(scale);
    let x = origin.x + ((monitor.width as i64 - size.width as i64) / 2) as i32;
    let y = origin.y + (TOP_MARGIN * scale).round() as i32;
    (size, PhysicalPosition::new(x, y))
}

pub fn apply_mode(window: &WebviewWindow, mode: Mode) -> Result<(), Box<dyn std::error::Error>> {
    let monitor = window.primary_monitor()?.ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "monitor primário não encontrado")
    })?;
    let (mut size, position) = geometry(
        mode,
        *monitor.position(),
        *monitor.size(),
        monitor.scale_factor(),
    );
    if mode == Mode::Expanded {
        let rows = window
            .state::<Settings>()
            .visible_volumes()
            .min(extra_limit(window));
        let height = EXPANDED_HEIGHT + rows as u32 * EXTRA_ROW_HEIGHT;
        size.height = (f64::from(height) * monitor.scale_factor()).round() as u32;
    }
    // Posicionar primeiro permite ao Windows ajustar o monitor/DPI antes do tamanho.
    if window.outer_position()? != position {
        window.set_position(position)?;
    }
    if window.inner_size()? != size {
        window.set_size(size)?;
    }
    #[cfg(windows)]
    if window.is_visible()? {
        super::z_order::raise(window.hwnd()?)?;
    }
    Ok(())
}

pub fn extra_limit(window: &WebviewWindow) -> usize {
    let Ok(Some(monitor)) = window.primary_monitor() else {
        return 0;
    };
    let area = monitor.work_area();
    let bottom = i64::from(area.position.y) + i64::from(area.size.height);
    let top =
        i64::from(monitor.position().y) + (TOP_MARGIN * monitor.scale_factor()).round() as i64;
    let available = ((bottom - top).max(0) as f64 / monitor.scale_factor()).floor() as u32;
    available
        .saturating_sub(EXPANDED_HEIGHT)
        .saturating_sub(8)
        .checked_div(EXTRA_ROW_HEIGHT)
        .unwrap_or_default() as usize
}

pub fn resize_expanded(window: &WebviewWindow, height: u32) -> tauri::Result<()> {
    let Some(monitor) = window.primary_monitor()? else {
        return Ok(());
    };
    let max = EXPANDED_HEIGHT + extra_limit(window) as u32 * EXTRA_ROW_HEIGHT;
    let height = height.clamp(EXPANDED_HEIGHT, max);
    window.set_size(
        LogicalSize::new(340.0, f64::from(height)).to_physical::<u32>(monitor.scale_factor()),
    )
}

pub fn open_settings(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show()?;
        return window.set_focus();
    }
    let window =
        WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
            .title("PixelPulse")
            .inner_size(710.0, 520.0)
            .min_inner_size(480.0, 340.0)
            .decorations(false)
            .transparent(true)
            .visible(false)
            .center()
            .build()?;
    if let Some(monitor) = window.primary_monitor()? {
        let area = monitor.work_area();
        let size = window.outer_size()?;
        let width = size.width.min(area.size.width);
        let height = size.height.min(area.size.height);
        if width != size.width || height != size.height {
            window.set_size(PhysicalSize::new(width, height))?;
        }
        window.set_position(PhysicalPosition::new(
            area.position.x + ((area.size.width - width) / 2) as i32,
            area.position.y + ((area.size.height - height) / 2) as i32,
        ))?;
    }
    #[cfg(windows)]
    let _ = request_small_corners(&window);
    window.show()?;
    window.set_focus()
}

pub fn initialize(
    app: &AppHandle,
    state: AppState,
) -> Result<GlassMode, Box<dyn std::error::Error>> {
    let window = app.get_webview_window("main").ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "janela principal não encontrada")
    })?;
    apply_mode(&window, Mode::Collapsed)?;
    let dpi_window = window.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::ScaleFactorChanged { .. }) {
            // Sem polling: mudanças de escala também refazem a centralização.
            if let Err(error) = apply_mode(&dpi_window, state.mode()) {
                eprintln!("[janela] falha ao ajustar DPI: {error}");
            }
        }
    });
    // focus=false mantém a exibição sem ativação no Windows; não chamamos set_focus.
    // Mostrar só após posicionar evita um flash da janela em uma posição provisória.
    let glass = configure_glass(&window);
    if let Err(error) = request_small_corners(&window) {
        eprintln!("[glass] cantos nativos indisponíveis: {error}");
    }
    let report = glass_diagnostics(&window);
    eprintln!(
        "[glass] DWM backdrop={:?}, cantos={:?}, erros={:?}",
        report.backdrop_type, report.corner_preference, report.errors
    );
    window.show()?;
    #[cfg(windows)]
    {
        let hwnd = window.hwnd()?;
        super::z_order::raise(hwnd)?;
        if let Err(error) = super::z_order::watch(hwnd) {
            eprintln!("[janela] monitoramento de sobreposição indisponível: {error}");
        }
    }
    Ok(glass)
}

/// Consulta apenas a nossa janela; atributo aceito não comprova ausência de artefatos.
#[derive(Debug, serde::Serialize)]
pub struct GlassDiagnostics {
    pub backdrop_type: Option<i32>,
    pub corner_preference: Option<i32>,
    pub errors: Vec<String>,
}

#[cfg(target_os = "windows")]
fn request_small_corners(window: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    use windows::Win32::{
        Foundation::HWND,
        Graphics::Dwm::{DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUNDSMALL, DwmSetWindowAttribute},
    };
    let hwnd = HWND(window.hwnd()?.0);
    // Preferência oficial: o DWM decide o raio físico. CSS mantém seus 4 px lógicos.
    // SAFETY: HWND válido da janela viva; ponteiro e tamanho correspondem ao enum i32.
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            std::ptr::from_ref(&DWMWCP_ROUNDSMALL).cast(),
            size_of_val(&DWMWCP_ROUNDSMALL) as u32,
        )?;
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn request_small_corners(_: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn glass_diagnostics(window: &WebviewWindow) -> GlassDiagnostics {
    let mut report = GlassDiagnostics {
        backdrop_type: None,
        corner_preference: None,
        errors: Vec::new(),
    };
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::{
            Foundation::HWND,
            Graphics::Dwm::{
                DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_WINDOW_CORNER_PREFERENCE, DwmGetWindowAttribute,
            },
        };
        match window.hwnd() {
            Ok(handle) => {
                for (attribute, output) in [
                    (DWMWA_SYSTEMBACKDROP_TYPE, &mut report.backdrop_type),
                    (
                        DWMWA_WINDOW_CORNER_PREFERENCE,
                        &mut report.corner_preference,
                    ),
                ] {
                    let mut value = 0_i32;
                    // SAFETY: os dois atributos retornam enums de 32 bits; HWND vivo e buffer válido.
                    let result = unsafe {
                        DwmGetWindowAttribute(
                            HWND(handle.0),
                            attribute,
                            (&mut value as *mut i32).cast(),
                            size_of::<i32>() as u32,
                        )
                    };
                    match result {
                        Ok(()) => *output = Some(value),
                        Err(error) => report
                            .errors
                            .push(format!("atributo {}: {error}", attribute.0)),
                    }
                }
            }
            Err(error) => report.errors.push(error.to_string()),
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
        report
            .errors
            .push("diagnóstico DWM requer Windows".to_owned());
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometria_converte_dpi_e_preserva_a_origem_do_monitor() {
        let (size, position) = geometry(
            Mode::Collapsed,
            PhysicalPosition::new(0, 0),
            PhysicalSize::new(1920, 1080),
            1.0,
        );
        assert_eq!(size, PhysicalSize::new(180, 36));
        assert_eq!(position, PhysicalPosition::new(870, 8));
        let (size, position) = geometry(
            Mode::Expanded,
            PhysicalPosition::new(-2560, -100),
            PhysicalSize::new(2560, 1440),
            1.5,
        );
        assert_eq!(size, PhysicalSize::new(510, 345));
        assert_eq!(position, PhysicalPosition::new(-1535, -88));
        let (size, position) = geometry(
            Mode::Expanded,
            PhysicalPosition::new(0, 0),
            PhysicalSize::new(3840, 2160),
            2.0,
        );
        assert_eq!(size, PhysicalSize::new(680, 460));
        assert_eq!(position, PhysicalPosition::new(1580, 16));
    }
}
