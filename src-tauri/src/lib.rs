mod cache;
pub mod collectors;
mod commands;
mod settings;
mod startup;
mod window;
#[cfg(windows)]
mod z_order;

use cache::{AppState, sampler::Sampler};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::set_mode,
            commands::open_context_menu,
            commands::get_snapshot,
            commands::get_glass_mode,
            commands::set_ui_visibility,
            commands::get_glass_diagnostics,
            commands::get_settings,
            commands::set_language,
            commands::set_volume,
            commands::get_startup,
            commands::set_startup,
            commands::resize_island,
            commands::close_settings
        ])
        .on_menu_event(|app, event| {
            if event.id().as_ref() == commands::EXIT_MENU_ID {
                app.exit(0);
            } else if event.id().as_ref() == commands::SETTINGS_MENU_ID {
                let app = app.clone();
                tauri::async_runtime::spawn_blocking(move || {
                    if let Err(error) = window::open_settings(&app) {
                        eprintln!("[settings] {error}");
                    }
                });
            }
        })
        .setup(|app| {
            app.manage(settings::Settings::new(
                app.path().home_dir().ok().as_deref(),
            ));
            let (state, changes) = AppState::new_with_receiver();
            app.manage(state.clone());
            app.manage(window::initialize(app.handle(), state.clone())?);
            app.manage(Sampler::start(app.handle().clone(), state, changes)?);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("erro ao executar o PixelPulse")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                // Wake the sampler and join its thread before tearing down the app.
                #[cfg(windows)]
                z_order::unwatch();
                app.state::<Sampler>().shutdown();
            }
        });
}
