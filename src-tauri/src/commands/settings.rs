use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub language: String,
    pub theme: String,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub android_home: String,
    pub deveco_home: String,
    pub harmony_image_location: String,
    pub harmony_emulator_location: String,
    pub harmony_emulator_path: String,
    pub harmony_hdc_path: String,
    pub xcode_home: String,
    pub screenshot_dir: String,
    pub android_force_kill: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let screenshot_dir = dirs::picture_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        
        Self {
            language: "zh-CN".to_string(),
            theme: "system".to_string(),
            auto_start: false,
            minimize_to_tray: true,
            android_home: String::new(),
            deveco_home: String::new(),
            harmony_image_location: String::new(),
            harmony_emulator_location: String::new(),
            harmony_emulator_path: String::new(),
            harmony_hdc_path: String::new(),
            xcode_home: String::new(),
            screenshot_dir,
            android_force_kill: false,
        }
    }
}

fn get_settings_path() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| "Failed to get config directory".to_string())?;
    let app_dir = config_dir.join("SimHub");
    
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

/// 公开的加载设置函数，供 main.rs 中窗口关闭事件使用
pub fn load_settings() -> Result<Settings, String> {
    load_settings_from_file()
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
    if let Ok(android_home) = std::env::var("ANDROID_HOME") {
        return Some(android_home);
    }
    
    if let Ok(android_sdk_root) = std::env::var("ANDROID_SDK_ROOT") {
        return Some(android_sdk_root);
    }
    
    // macOS: Try default Android SDK location
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let default_path = format!("{}/Library/Android/sdk", home);
            let path = std::path::Path::new(&default_path);
            if path.exists() && path.is_dir() {
                return Some(default_path);
            }
        }
    }
    
    None
}

pub fn get_android_force_kill() -> bool {
    if let Ok(settings) = load_settings_from_file() {
        return settings.android_force_kill;
    }
    false
}

pub fn get_screenshot_dir() -> Option<String> {
    if let Ok(settings) = load_settings_from_file() {
        if !settings.screenshot_dir.is_empty() {
            return Some(settings.screenshot_dir);
        }
    }
    
    // Fallback to picture directory
    dirs::picture_dir().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn check_path_exists(path: String) -> Result<bool, String> {
    if path.trim().is_empty() {
        return Ok(true); // Empty path is considered valid (optional field)
    }
    
    let path_obj = std::path::Path::new(&path);
    Ok(path_obj.exists())
}

#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    Ok(())
}

pub fn get_deveco_home() -> Option<String> {
    if let Ok(settings) = load_settings_from_file() {
        if !settings.deveco_home.is_empty() {
            return Some(settings.deveco_home);
        }
    }
    
    // Fallback to environment variable
    if let Ok(val) = std::env::var("DEVECO_SDK_HOME") {
        return Some(val);
    }
    
    // macOS 默认路径
    #[cfg(target_os = "macos")]
    {
        let default_path = "/Applications/DevEco-Studio.app";
        if std::path::Path::new(default_path).exists() {
            return Some(default_path.to_string());
        }
    }
    
    None
}

pub fn get_harmony_emulator_location() -> Option<String> {
    if let Ok(settings) = load_settings_from_file() {
        if !settings.harmony_emulator_location.is_empty() {
            return Some(settings.harmony_emulator_location);
        }
    }
    
    // macOS 默认路径: $HOME/.Huawei/Emulator/deployed
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let default_path = format!("{}/.Huawei/Emulator/deployed", home);
            if std::path::Path::new(&default_path).exists() {
                return Some(default_path);
            }
        }
    }
    
    None
}

pub fn get_harmony_image_location() -> Option<String> {
    if let Ok(settings) = load_settings_from_file() {
        if !settings.harmony_image_location.is_empty() {
            return Some(settings.harmony_image_location);
        }
    }
    
    // macOS 默认路径: $HOME/Library/Huawei/Sdk/system-image
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let default_path = format!("{}/Library/Huawei/Sdk/system-image", home);
            if std::path::Path::new(&default_path).exists() {
                return Some(default_path);
            }
        }
    }
    
    None
}

pub fn get_harmony_emulator_path() -> Option<String> {
    if let Ok(settings) = load_settings_from_file() {
        // 优先使用用户配置的路径
        if !settings.harmony_emulator_path.is_empty() {
            return Some(settings.harmony_emulator_path);
        }
        // 否则从 DevEco Studio 路径拼接
        if !settings.deveco_home.is_empty() {
            let emulator_path = if cfg!(target_os = "macos") {
                format!("{}/Contents/tools/emulator/Emulator", settings.deveco_home)
            } else if cfg!(target_os = "windows") {
                format!("{}/tools/emulator/Emulator.exe", settings.deveco_home)
            } else {
                format!("{}/tools/emulator/Emulator", settings.deveco_home)
            };
            return Some(emulator_path);
        }
    }
    
    // 从 get_deveco_home() 获取（含默认路径 fallback）
    if let Some(deveco_home) = get_deveco_home() {
        let emulator_path = if cfg!(target_os = "macos") {
            format!("{}/Contents/tools/emulator/Emulator", deveco_home)
        } else if cfg!(target_os = "windows") {
            format!("{}/tools/emulator/Emulator.exe", deveco_home)
        } else {
            format!("{}/tools/emulator/Emulator", deveco_home)
        };
        return Some(emulator_path);
    }
    
    None
}

pub fn get_harmony_hdc_path() -> Option<String> {
    if let Ok(settings) = load_settings_from_file() {
        // 优先使用用户配置的路径
        if !settings.harmony_hdc_path.is_empty() {
            return Some(settings.harmony_hdc_path);
        }
        // 否则从 DevEco Studio 路径拼接
        if !settings.deveco_home.is_empty() {
            let hdc_path = if cfg!(target_os = "macos") {
                format!("{}/Contents/sdk/default/openharmony/toolchains/hdc", settings.deveco_home)
            } else if cfg!(target_os = "windows") {
                format!("{}/sdk/default/openharmony/toolchains/hdc.exe", settings.deveco_home)
            } else {
                format!("{}/sdk/default/openharmony/toolchains/hdc", settings.deveco_home)
            };
            return Some(hdc_path);
        }
    }
    
    // 从 get_deveco_home() 获取（含默认路径 fallback）
    if let Some(deveco_home) = get_deveco_home() {
        let hdc_path = if cfg!(target_os = "macos") {
            format!("{}/Contents/sdk/default/openharmony/toolchains/hdc", deveco_home)
        } else if cfg!(target_os = "windows") {
            format!("{}/sdk/default/openharmony/toolchains/hdc.exe", deveco_home)
        } else {
            format!("{}/sdk/default/openharmony/toolchains/hdc", deveco_home)
        };
        return Some(hdc_path);
    }
    
    None
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

    // Initialize screenshot_dir if empty
    if settings.screenshot_dir.is_empty() {
        if let Some(picture_dir) = dirs::picture_dir() {
            settings.screenshot_dir = picture_dir.to_string_lossy().to_string();
        }
    }

    Ok(settings)
}

#[tauri::command]
pub async fn save_settings(settings: Settings) -> Result<(), String> {
    save_settings_to_file(&settings)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorLaunchParams {
    pub no_window: bool,
    pub wipe_data: bool,
    pub no_snapshot: bool,
    pub dns_server: String,
    pub gps_longitude: String,
    pub gps_latitude: String,
    pub memory: Option<i32>,
    pub http_proxy: String,
    pub cores: Option<i32>,
    pub gpu: String,
}

impl Default for EmulatorLaunchParams {
    fn default() -> Self {
        Self {
            no_window: false,
            wipe_data: false,
            no_snapshot: false,
            dns_server: String::new(),
            gps_longitude: String::new(),
            gps_latitude: String::new(),
            memory: None,
            http_proxy: String::new(),
            cores: None,
            gpu: "auto".to_string(),
        }
    }
}

fn get_emulator_params_path(emulator_id: &str, emulator_type: &str) -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| "Failed to get config directory".to_string())?;
    let app_dir = config_dir.join("SimHub").join("emulator_params");
    
    // 确保目录存在
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create emulator params directory: {}", e))?;
    
    let filename = format!("{}_{}.json", emulator_type, emulator_id);
    Ok(app_dir.join(filename))
}

#[tauri::command]
pub async fn get_emulator_launch_params(emulator_id: String, emulator_type: String) -> Result<EmulatorLaunchParams, String> {
    let params_path = get_emulator_params_path(&emulator_id, &emulator_type)?;
    
    if params_path.exists() {
        let content = fs::read_to_string(&params_path)
            .map_err(|e| format!("Failed to read emulator params file: {}", e))?;
        let params: EmulatorLaunchParams = serde_json::from_str(&content)
            .unwrap_or_default();
        Ok(params)
    } else {
        Ok(EmulatorLaunchParams::default())
    }
}

#[tauri::command]
pub async fn save_emulator_launch_params(
    emulator_id: String,
    emulator_type: String,
    params: EmulatorLaunchParams
) -> Result<(), String> {
    let params_path = get_emulator_params_path(&emulator_id, &emulator_type)?;
    let content = serde_json::to_string_pretty(&params)
        .map_err(|e| format!("Failed to serialize params: {}", e))?;
    fs::write(&params_path, content)
        .map_err(|e| format!("Failed to write params file: {}", e))?;
    Ok(())
}

pub fn get_emulator_launch_params_sync(emulator_id: &str, emulator_type: &str) -> EmulatorLaunchParams {
    let params_path = match get_emulator_params_path(emulator_id, emulator_type) {
        Ok(path) => path,
        Err(_) => return EmulatorLaunchParams::default(),
    };
    
    if params_path.exists() {
        if let Ok(content) = fs::read_to_string(&params_path) {
            if let Ok(params) = serde_json::from_str::<EmulatorLaunchParams>(&content) {
                return params;
            }
        }
    }
    
    EmulatorLaunchParams::default()
}
