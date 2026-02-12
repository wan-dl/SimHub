// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod utils;

use commands::{android, ios, harmony, settings, usb_device};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Wry,
};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 创建托盘菜单
            let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            // 创建系统托盘图标
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("SimHub")
                .menu(&menu)
                .on_menu_event(|app: &AppHandle<Wry>, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray: &tauri::tray::TrayIcon<Wry>, event| {
                    if let TrayIconEvent::DoubleClick { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // macOS 上点击关闭按钮时隐藏窗口而不是退出
                #[cfg(target_os = "macos")]
                {
                    api.prevent_close();
                    let _ = window.hide();
                }
                
                // 其他平台根据设置决定
                #[cfg(not(target_os = "macos"))]
                {
                    if let Ok(s) = settings::load_settings() {
                        if s.minimize_to_tray {
                            api.prevent_close();
                            let _ = window.hide();
                        }
                    }
                }
            }
        })
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
            settings::check_path_exists,
            settings::get_emulator_launch_params,
            settings::save_emulator_launch_params,
            // USB device commands
            usb_device::list_usb_devices,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, _event| {
            // macOS: 点击 Dock 图标时显示窗口
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = _event {
                if let Some(window) = _app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
        });
}
