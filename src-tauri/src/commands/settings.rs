use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

fn get_settings_path() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| "Failed to get config directory".to_string())?;
    let app_dir = config_dir.join("oh-emulator-manager");
    
    // 确保目录存在
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    
    Ok(app_dir.join("settings.json"))
}

fn load_settings_from_file() -> Result<Settings, String> {
    let settings_path = get_settings_path()?;
    
    if !settings_path.exists() {
        return Ok(Settings::default());
    }
    
    let content = fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;
    
    let settings: Settings = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings: {}", e))?;
    
    Ok(settings)
}

fn save_settings_to_file(settings: &Settings) -> Result<(), String> {
    let settings_path = get_settings_path()?;
    
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    
    fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;
    
    Ok(())
}

pub fn get_android_home() -> Option<String> {
    if let Ok(settings) = load_settings_from_file() {
        if !settings.android_home.is_empty() {
            return Some(settings.android_home);
        }
    }
    
    // Fallback to environment variables
    std::env::var("ANDROID_HOME")
        .or_else(|_| std::env::var("ANDROID_SDK_ROOT"))
        .ok()
}

#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    let mut settings = load_settings_from_file()?;

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

    Ok(settings)
}

#[tauri::command]
pub async fn save_settings(settings: Settings) -> Result<(), String> {
    save_settings_to_file(&settings)?;
    Ok(())
}
