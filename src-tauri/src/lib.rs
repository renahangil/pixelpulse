mod cache;
pub mod collectors;
mod commands;
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
            commands::get_glass_diagnostics
        ])
        .on_menu_event(|app, event| {
            if event.id().as_ref() == commands::EXIT_MENU_ID {
                app.exit(0);
            }
        })
        .setup(|app| {
            app.manage(commands::context_menu(app.handle())?);
            let (state, changes) = AppState::new_with_receiver();
            let collector = collectors::Collector::new()?;
            app.manage(state.clone());
            app.manage(window::initialize(app.handle(), state.clone())?);
            app.manage(Sampler::start(
                app.handle().clone(),
                state,
                changes,
                collector,
            )?);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("erro ao executar o PixelPulse")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                // Acorda a espera e aguarda a thread antes de desmontar o aplicativo.
                #[cfg(windows)]
                z_order::unwatch();
                app.state::<Sampler>().shutdown();
            }
        });
}
