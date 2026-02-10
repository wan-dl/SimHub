use serde::{Deserialize, Serialize};
use std::process::Command;
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize)]
pub struct HarmonyEmulator {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub os_version: String,
    pub status: String,
}

fn get_emulator_path() -> Result<std::path::PathBuf, String> {
    let emulator_path = crate::commands::settings::get_harmony_emulator_path()
        .ok_or_else(|| "Emulator path not configured. Please set DevEco Studio path or Emulator path in Settings.".to_string())?;
    
    let path = std::path::PathBuf::from(&emulator_path);
    
    if !path.exists() {
        return Err(format!("Emulator not found at: {:?}. Please check your settings.", path));
    }
    
    Ok(path)
}

fn get_hdc_path() -> Result<std::path::PathBuf, String> {
    let hdc_path = crate::commands::settings::get_harmony_hdc_path()
        .ok_or_else(|| "hdc path not configured. Please set DevEco Studio path or hdc path in Settings.".to_string())?;
    
    let path = std::path::PathBuf::from(&hdc_path);
    
    if !path.exists() {
        return Err(format!("hdc not found at: {:?}. Please check your settings.", path));
    }
    
    Ok(path)
}

#[tauri::command]
pub async fn list_harmony_emulators() -> Result<Vec<HarmonyEmulator>, String> {
    let emulator_path = get_emulator_path()?;
    
    let output = Command::new(&emulator_path)
        .arg("-list")
        .output()
        .map_err(|e| format!("Failed to execute Emulator command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to list HarmonyOS emulators: {}", stderr));
    }

    let device_list = String::from_utf8_lossy(&output.stdout);
    let mut emulators = Vec::new();

    for line in device_list.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("List") || line.starts_with("---") {
            continue;
        }
        
        // Parse emulator info from the list output
        // Format may vary, adjust parsing as needed
        if !line.is_empty() {
            let name = line.to_string();
            emulators.push(HarmonyEmulator {
                id: name.clone(),
                name: name.clone(),
                device_type: "HarmonyOS Device".to_string(),
                os_version: "HarmonyOS NEXT".to_string(),
                status: "stopped".to_string(),
            });
        }
    }

    // Check running emulators using hdc
    if let Ok(hdc_path) = get_hdc_path() {
        if let Ok(hdc_output) = Command::new(&hdc_path).arg("list").arg("targets").output() {
            let devices = String::from_utf8_lossy(&hdc_output.stdout);
            for line in devices.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with("[") {
                    // Mark matching emulator as running
                    for emu in &mut emulators {
                        if line.contains(&emu.name) || emu.name.contains(line) {
                            emu.status = "running".to_string();
                            emu.id = line.to_string();
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(emulators)
}

#[tauri::command]
pub async fn start_harmony_emulator(id: String, app: tauri::AppHandle) -> Result<(), String> {
    let emulator_path = get_emulator_path()?;
    
    let emulator_location = crate::commands::settings::get_harmony_emulator_location()
        .ok_or_else(|| "Local Emulator Location not configured. Please set it in Settings.".to_string())?;
    
    let image_location = crate::commands::settings::get_harmony_image_location()
        .ok_or_else(|| "Local Image Location not configured. Please set it in Settings.".to_string())?;
    
    let mut cmd = Command::new(&emulator_path);
    cmd.args(&["-hvd", &id, "-path", &emulator_location, "-imageRoot", &image_location]);
    
    let _ = app.emit("add-log", serde_json::json!({
        "type": "command",
        "message": format!("{:?}", cmd),
        "source": "app"
    }));
    
    cmd.spawn()
        .map_err(|e| format!("Failed to start emulator: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn stop_harmony_emulator(id: String) -> Result<(), String> {
    let hdc_path = get_hdc_path()?;
    
    // Use hdc to kill the emulator
    let output = Command::new(&hdc_path)
        .args(&["-t", &id, "shell", "reboot", "-p"])
        .output()
        .map_err(|e| format!("Failed to stop emulator: {}", e))?;

    if !output.status.success() {
        // Try alternative method - kill via emulator command
        let emulator_path = get_emulator_path()?;
        Command::new(&emulator_path)
            .args(&["-kill", &id])
            .output()
            .map_err(|e| format!("Failed to stop emulator: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn screenshot_harmony(id: String) -> Result<String, String> {
    let hdc_path = get_hdc_path()?;
    
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("screenshot_{}_{}.png", id, timestamp);
    let remote_path = "/data/local/tmp/screenshot.png";
    
    // Get screenshot directory from settings
    let screenshot_dir = crate::commands::settings::get_screenshot_dir()
        .ok_or_else(|| "Cannot find screenshot directory".to_string())?;
    let local_path = std::path::Path::new(&screenshot_dir).join(&filename);

    let output = Command::new(&hdc_path)
        .args(&["-t", &id, "shell", "snapshot_display", "-f", remote_path])
        .output()
        .map_err(|e| format!("Failed to take screenshot: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Screenshot failed: {}", stderr));
    }

    let output = Command::new(&hdc_path)
        .args(&["-t", &id, "file", "recv", remote_path, local_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Failed to download screenshot: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to download screenshot: {}", stderr));
    }

    Ok(local_path.to_string_lossy().to_string())
}
