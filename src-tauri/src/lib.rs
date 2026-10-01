mod commands;
mod models;

use commands::{config, exetools, launcher, scanner, settings};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scanner::scan_cmd_tools,
            scanner::update_cmd_tool,
            scanner::assign_cmd_tool,
            scanner::read_text_file,
            config::load_menu_config,
            config::add_category,
            config::rename_category,
            config::delete_category,
            settings::load_theme_settings,
            settings::save_theme_settings,
            exetools::load_exe_tools,
            exetools::add_exe_tool,
            exetools::update_exe_tool,
            exetools::remove_exe_tool,
            launcher::launch_tool,
            launcher::open_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
