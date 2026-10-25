mod application;
mod domain;
mod infrastructure;

use application::commands::*;
use infrastructure::database::Database;
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState(pub Mutex<Database>);

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let database = Database::open(data_dir.join("openstudio.sqlite"))?;
            app.manage(AppState(Mutex::new(database)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap,
            list_projects,
            get_project,
            create_project,
            update_project,
            delete_project,
            list_brand_kits,
            save_brand_kit,
            delete_brand_kit,
            list_providers,
            save_provider,
            remove_provider,
            activate_provider,
            check_local_providers,
            generate_version,
            generate_brand_kit,
            export_version,
            render_version_svg
        ])
        .run(tauri::generate_context!())
        .expect("error while running OpenStudio");
}
