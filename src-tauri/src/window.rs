//! Geometria nativa da ilha: não sobra uma área transparente interceptando cliques.

use crate::cache::{AppState, Mode};
use std::io;
use tauri::{
    AppHandle, LogicalSize, Manager, PhysicalPosition, PhysicalSize, WebviewWindow, WindowEvent,
};

const TOP_MARGIN: f64 = 8.0;

// A comparação local decide o padrão; a variável de ambiente permite testar o plano B.
const DEFAULT_ACRYLIC: bool = true;

#[derive(Clone, Copy, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GlassMode {
    Acrylic,
    Fallback,
}

fn configure_glass(window: &WebviewWindow) -> GlassMode {
    let prefer_acrylic = match std::env::var("PIXELPULSE_GLASS").as_deref() {
        Ok("acrylic") => true,
        Ok("fallback") => false,
        _ => DEFAULT_ACRYLIC,
    };
    if prefer_acrylic {
        // A assinatura atual aceita RGBA; alpha 72/255 deixa a tinta discreta.
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
        Mode::Expanded => LogicalSize::new(340.0, 230.0),
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
    let (size, position) = geometry(
        mode,
        *monitor.position(),
        *monitor.size(),
        monitor.scale_factor(),
    );
    // Posicionar primeiro permite ao Windows ajustar o monitor/DPI antes do tamanho.
    if window.outer_position()? != position {
        window.set_position(position)?;
    }
    if window.inner_size()? != size {
        window.set_size(size)?;
    }
    Ok(())
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
    window.show()?;
    Ok(glass)
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
