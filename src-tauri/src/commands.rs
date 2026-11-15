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
