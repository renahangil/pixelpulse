use crate::{
    cache::{AppState, Mode},
    collectors::Snapshot,
};
use tauri::State;

#[tauri::command]
pub fn set_mode(mode: Mode, state: State<'_, AppState>) -> Result<(), String> {
    state.set_mode(mode)
}

#[tauri::command]
pub fn get_snapshot(state: State<'_, AppState>) -> Result<Option<Snapshot>, String> {
    // Só copia o último resultado; clicar ou ler a UI nunca consulta os sensores.
    state.snapshot()
}
