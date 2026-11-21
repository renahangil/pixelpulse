use crate::{
    cache::{AppState, Mode},
    collectors::{Inventory, Snapshot},
    settings::{Language, Preferences, Settings},
    window,
};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

#[tauri::command]
pub fn set_mode(
    mode: Mode,
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> Result<(), String> {
    island_only(&window)?;
    let previous = state.mode();
    // Switch the sampling interval after the window has accepted its new size.
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

/// Document visibility complements the native window's visibility and minimized state.
#[tauri::command]
pub fn set_ui_visibility(
    visible: bool,
    state: State<'_, AppState>,
    window: WebviewWindow,
) -> Result<(), String> {
    island_only(&window)?;
    state.set_ui_visible(visible)
}

#[tauri::command]
pub fn get_glass_diagnostics(window: WebviewWindow) -> window::GlassDiagnostics {
    window::glass_diagnostics(&window)
}

pub const EXIT_MENU_ID: &str = "encerrar-pixelpulse";
pub const SETTINGS_MENU_ID: &str = "system-diagnostics";

#[tauri::command]
pub async fn open_context_menu(
    window: WebviewWindow,
    settings: State<'_, Settings>,
) -> Result<(), String> {
    use tauri::menu::{Menu, MenuItem};
    island_only(&window)?;
    let (diagnostics, exit) = settings.preferences().language.menu();
    let menu = (|| {
        let diagnostics =
            MenuItem::with_id(&window, SETTINGS_MENU_ID, diagnostics, true, None::<&str>)?;
        let exit = MenuItem::with_id(&window, EXIT_MENU_ID, exit, true, None::<&str>)?;
        Menu::with_items(&window, &[&diagnostics, &exit])
    })()
    .map_err(|error: tauri::Error| error.to_string())?;
    window.popup_menu(&menu).map_err(|error| error.to_string())
}

#[derive(serde::Serialize)]
pub struct Configuration {
    preferences: Preferences,
    inventory: Option<Inventory>,
    volume_limit: usize,
}

#[tauri::command]
pub fn get_settings(
    app: AppHandle,
    settings: State<'_, Settings>,
) -> Result<Configuration, String> {
    let preferences = settings.preferences();
    let inventory = settings
        .inventory
        .read()
        .map_err(|error| error.to_string())?
        .clone();
    let volume_limit = app
        .get_webview_window("main")
        .map(|window| window::extra_limit(&window))
        .unwrap_or_default();
    Ok(Configuration {
        preferences,
        inventory,
        volume_limit,
    })
}

#[tauri::command]
pub async fn set_language(
    window: WebviewWindow,
    app: AppHandle,
    settings: State<'_, Settings>,
    language: Language,
) -> Result<Preferences, String> {
    settings_only(&window)?;
    let next = settings.update(|value| {
        value.language = language;
        Ok(())
    })?;
    let _ = app.emit("preferences-changed", &next);
    Ok(next)
}

#[tauri::command]
pub async fn set_volume(
    window: WebviewWindow,
    app: AppHandle,
    settings: State<'_, Settings>,
    state: State<'_, AppState>,
    id: String,
    selected: bool,
) -> Result<Preferences, String> {
    settings_only(&window)?;
    let allowed: Vec<_> = settings
        .inventory
        .read()
        .map_err(|error| error.to_string())?
        .as_ref()
        .ok_or("Hardware discovery is still running")?
        .volumes
        .iter()
        .filter(|volume| !volume.system)
        .map(|volume| volume.id.clone())
        .collect();
    if !allowed.contains(&id) {
        return Err("Volume is unavailable".into());
    }
    let limit = app
        .get_webview_window("main")
        .map(|window| window::extra_limit(&window))
        .unwrap_or_default();
    let next = settings.update(|prefs| {
        if selected
            && !prefs.volumes.contains(&id)
            && prefs
                .volumes
                .iter()
                .filter(|id| allowed.contains(id))
                .count()
                >= limit
        {
            return Err("No more rows fit on this screen".into());
        }
        prefs.volumes.retain(|item| item != &id);
        if selected {
            prefs.volumes.push(id);
        }
        Ok(())
    })?;
    state.request_sample();
    let _ = app.emit("preferences-changed", &next);
    Ok(next)
}

#[tauri::command]
pub async fn get_startup(window: WebviewWindow) -> Result<bool, String> {
    startup_change(window, None).await
}

#[tauri::command]
pub async fn set_startup(window: WebviewWindow, enabled: bool) -> Result<bool, String> {
    startup_change(window, Some(enabled)).await
}

async fn startup_change(window: WebviewWindow, enabled: Option<bool>) -> Result<bool, String> {
    settings_only(&window)?;
    tauri::async_runtime::spawn_blocking(move || crate::startup::update(enabled))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn resize_island(
    window: WebviewWindow,
    state: State<'_, AppState>,
    height: u32,
) -> Result<(), String> {
    island_only(&window)?;
    if state.mode() == Mode::Expanded {
        window::resize_expanded(&window, height).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn close_settings(window: WebviewWindow) -> Result<(), String> {
    settings_only(&window)?;
    window.close().map_err(|error| error.to_string())
}

fn island_only(window: &WebviewWindow) -> Result<(), String> {
    (window.label() == "main")
        .then_some(())
        .ok_or_else(|| "Island window required".into())
}

fn settings_only(window: &WebviewWindow) -> Result<(), String> {
    (window.label() == "settings")
        .then_some(())
        .ok_or_else(|| "Settings window required".into())
}
