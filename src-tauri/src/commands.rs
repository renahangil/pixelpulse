use crate::{
    cache::{AppState, Mode},
    collectors::Snapshot,
    window,
};
use tauri::{State, WebviewWindow};

#[tauri::command]
pub fn set_mode(
    mode: Mode,
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> Result<(), String> {
    let previous = state.mode();
    // Só anunciamos o novo ritmo depois de ajustar a janela de verdade.
    window::apply_mode(&window, mode).map_err(|error| error.to_string())?;
    if let Err(error) = state.set_mode(mode) {
        if let Err(rollback) = window::apply_mode(&window, previous) {
            eprintln!("[janela] falha ao restaurar tamanho: {rollback}");
        }
        return Err(error);
    }
    Ok(())
}

#[tauri::command]
pub fn get_snapshot(state: State<'_, AppState>) -> Result<Option<Snapshot>, String> {
    state.snapshot()
}

#[tauri::command]
pub fn get_glass_mode(state: State<'_, window::GlassMode>) -> window::GlassMode {
    *state.inner()
}

/// Visibilidade do documento complementa is_visible/is_minimized da janela nativa.
#[tauri::command]
pub fn set_ui_visibility(visible: bool, state: State<'_, AppState>) -> Result<(), String> {
    state.set_ui_visible(visible)
}

#[tauri::command]
pub fn get_glass_diagnostics(window: WebviewWindow) -> window::GlassDiagnostics {
    window::glass_diagnostics(&window)
}

pub const EXIT_MENU_ID: &str = "encerrar-pixelpulse";

pub fn context_menu(app: &tauri::AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    let exit = tauri::menu::MenuItem::with_id(
        app,
        EXIT_MENU_ID,
        "Encerrar PixelPulse",
        true,
        None::<&str>,
    )?;
    tauri::menu::Menu::with_items(app, &[&exit])
}

#[tauri::command]
pub async fn open_context_menu(
    window: WebviewWindow,
    menu: State<'_, tauri::menu::Menu<tauri::Wry>>,
) -> Result<(), String> {
    window
        .popup_menu(menu.inner())
        .map_err(|error| error.to_string())
}
