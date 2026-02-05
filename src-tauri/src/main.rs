// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;

use commands::{android, ios, harmony, settings};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // Android commands
            android::list_android_emulators,
             android::start_android_emulator,
            android::stop_android_emulator,
            android::delete_android_emulator,
            android::wipe_android_data,
            android::screenshot_android,
            android::start_logcat,
            android::get_logcat_logs,
            android::stop_logcat,
            android::copy_image_to_clipboard,
            android::write_log_file,
            android::get_device_packages,
            // iOS commands
            ios::list_ios_simulators,
            ios::start_ios_simulator,
            ios::stop_ios_simulator,
            ios::delete_ios_simulator,
            ios::wipe_ios_data,
            ios::screenshot_ios,
            // HarmonyOS commands
            harmony::list_harmony_emulators,
            harmony::start_harmony_emulator,
            harmony::stop_harmony_emulator,
            harmony::screenshot_harmony,
            // Settings commands
            settings::get_settings,
            settings::save_settings,
            settings::open_file,
            settings::get_emulator_launch_params,
            settings::save_emulator_launch_params,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
