use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref SETTINGS: Mutex<Settings> = Mutex::new(Settings::default());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub language: String,
    pub theme: String,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub close_to_minimize: bool,
    pub android_home: String,
    pub deveco_home: String,
    pub xcode_home: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "zh-CN".to_string(),
            theme: "system".to_string(),
            auto_start: false,
            minimize_to_tray: true,
            close_to_minimize: true,
            android_home: String::new(),
            deveco_home: String::new(),
            xcode_home: String::new(),
        }
    }
}

pub fn get_android_home() -> Option<String> {
    let settings = SETTINGS.lock().ok()?;
    if !settings.android_home.is_empty() {
        return Some(settings.android_home.clone());
    }
    
    // Fallback to environment variables
    std::env::var("ANDROID_HOME")
        .or_else(|_| std::env::var("ANDROID_SDK_ROOT"))
        .ok()
}

#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    let mut settings = SETTINGS.lock().map_err(|e| e.to_string())?;

    // Initialize from environment if not set
    if settings.android_home.is_empty() {
        if let Ok(android_home) = std::env::var("ANDROID_HOME") {
            settings.android_home = android_home;
        } else if let Ok(android_sdk_root) = std::env::var("ANDROID_SDK_ROOT") {
            settings.android_home = android_sdk_root;
        }
    }

    if settings.deveco_home.is_empty() {
        if let Ok(deveco_home) = std::env::var("DEVECO_SDK_HOME") {
            settings.deveco_home = deveco_home;
        }
    }

    if cfg!(target_os = "macos") && settings.xcode_home.is_empty() {
        if let Ok(xcode_home) = std::env::var("DEVELOPER_DIR") {
            settings.xcode_home = xcode_home;
        } else {
            settings.xcode_home = "/Applications/Xcode.app/Contents/Developer".to_string();
        }
    }

    Ok(settings.clone())
}

#[tauri::command]
pub async fn save_settings(settings: Settings) -> Result<(), String> {
    let mut stored_settings = SETTINGS.lock().map_err(|e| e.to_string())?;
    *stored_settings = settings;
    Ok(())
}
