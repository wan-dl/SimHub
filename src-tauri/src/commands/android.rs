use serde::{Deserialize, Serialize};
use std::process::Command;
use tokio;

#[derive(Debug, Serialize, Deserialize)]
pub struct AndroidEmulator {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub os_version: String,
    pub status: String,
}

#[tauri::command]
pub async fn list_android_emulators() -> Result<Vec<AndroidEmulator>, String> {
    // Get ANDROID_HOME from settings or environment
    let android_home = crate::commands::settings::get_android_home()
        .ok_or_else(|| "Android SDK path not configured. Please set it in Settings.".to_string())?;
    
    let emulator_exe = if cfg!(target_os = "windows") {
        "emulator.exe"
    } else {
        "emulator"
    };
    
    let emulator_path = std::path::Path::new(&android_home)
        .join("emulator")
        .join(emulator_exe);
    
    let output = Command::new(&emulator_path)
        .arg("-list-avds")
        .env("ANDROID_HOME", &android_home)
        .env("ANDROID_SDK_ROOT", &android_home)
        .output()
        .map_err(|e| format!("Failed to execute emulator command: {}", e))?;

    if !output.status.success() {
        return Err("Failed to list Android emulators".to_string());
    }

    let avd_list = String::from_utf8_lossy(&output.stdout);
    let mut emulators = Vec::new();

    for line in avd_list.lines() {
        let name = line.trim();
        if !name.is_empty() {
            emulators.push(AndroidEmulator {
                id: name.to_string(),
                name: name.to_string(),
                device_type: name.to_string(),
                os_version: "".to_string(),
                status: "stopped".to_string(),
            });
        }
    }

    // Check running emulators using adb devices
    let adb_exe = if cfg!(target_os = "windows") {
        "adb.exe"
    } else {
        "adb"
    };
    
    let adb_path = std::path::Path::new(&android_home)
        .join("platform-tools")
        .join(adb_exe);
    
    if adb_path.exists() {
        if let Ok(adb_output) = Command::new(&adb_path).arg("devices").output() {
            let devices = String::from_utf8_lossy(&adb_output.stdout);
            
            for line in devices.lines() {
                if line.contains("emulator-") && line.contains("device") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(serial) = parts.first() {
                        // Query the AVD name for this emulator
                        if let Ok(avd_output) = Command::new(&adb_path)
                            .args(["-s", serial, "emu", "avd", "name"])
                            .output() 
                        {
                            let output_str = String::from_utf8_lossy(&avd_output.stdout);
                            if let Some(avd_name) = output_str.lines().next() {
                                let avd_name = avd_name.trim();
                                // Find matching emulator and update status
                                for emu in &mut emulators {
                                    if emu.name == avd_name {
                                        emu.status = "running".to_string();
                                        emu.id = serial.to_string(); // Use device serial as ID
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(emulators)
}

#[tauri::command]
pub async fn start_android_emulator(id: String) -> Result<(), String> {
    // Get ANDROID_HOME from settings or environment
    let android_home = crate::commands::settings::get_android_home()
        .ok_or_else(|| "Android SDK path not configured. Please set it in Settings.".to_string())?;
    
    let emulator_exe = if cfg!(target_os = "windows") {
        "emulator.exe"
    } else {
        "emulator"
    };
    
    let emulator_path = std::path::Path::new(&android_home)
        .join("emulator")
        .join(emulator_exe);
    
    if !emulator_path.exists() {
        return Err(format!("Emulator not found at: {:?}. Please check your Android SDK path in Settings.", emulator_path));
    }
    
    // Set up environment variables
    let mut cmd = Command::new(&emulator_path);
    cmd.arg("-avd")
        .arg(&id)
        .env("ANDROID_HOME", &android_home)
        .env("ANDROID_SDK_ROOT", &android_home)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to start emulator: {}", e))?;

    // Wait a moment to capture any immediate errors
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Try to get the exit status (non-blocking check)
    match child.try_wait() {
        Ok(Some(status)) => {
            // Process exited, capture error output
            let output = child.wait_with_output()
                .map_err(|e| format!("Failed to read output: {}", e))?;
            
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            if !status.success() {
                let error_msg = if !stderr.is_empty() {
                    stderr.to_string()
                } else if !stdout.is_empty() {
                    stdout.to_string()
                } else {
                    format!("Emulator exited with status: {}", status)
                };
                return Err(error_msg);
            }
        }
        Ok(None) => {
            // Process is still running, which is good
        }
        Err(e) => {
            return Err(format!("Failed to check process status: {}", e));
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_android_emulator(id: String) -> Result<(), String> {
    // Get ANDROID_HOME from settings or environment
    let android_home = crate::commands::settings::get_android_home()
        .ok_or_else(|| "Android SDK path not configured. Please set it in Settings.".to_string())?;
    
    let adb_exe = if cfg!(target_os = "windows") {
        "adb.exe"
    } else {
        "adb"
    };
    
    let adb_path = std::path::Path::new(&android_home)
        .join("platform-tools")
        .join(adb_exe);
    
    // 首先获取当前设备列表
    let devices_output = Command::new(&adb_path)
        .args(&["devices"])
        .output()
        .map_err(|e| format!("Failed to get devices: {}", e))?;

    let devices = String::from_utf8_lossy(&devices_output.stdout);
    let mut target_serial = None;
    
    // 查找目标模拟器
    for line in devices.lines() {
        if line.contains(&id) || line.contains(&format!("emulator-{}", id)) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(serial) = parts.first() {
                target_serial = Some(serial.to_string());
                break;
            }
        }
    }
    
    let serial = target_serial.ok_or_else(|| format!("Emulator '{}' not found in running devices", id))?;
    
    // 发送关闭命令
    let kill_output = Command::new(&adb_path)
        .args(&["-s", &serial, "emu", "kill"])
        .output()
        .map_err(|e| format!("Failed to stop emulator {}: {}", serial, e))?;
    
    if !kill_output.status.success() {
        let stderr = String::from_utf8_lossy(&kill_output.stderr);
        return Err(format!("Failed to stop emulator {}: {}", serial, stderr));
    }
    
    // Wait a moment for the emulator to shut down
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    Ok(())
}

#[tauri::command]
pub async fn delete_android_emulator(id: String) -> Result<(), String> {
    // Get ANDROID_HOME from settings or environment
    let android_home = crate::commands::settings::get_android_home()
        .ok_or_else(|| "Android SDK path not configured. Please set it in Settings.".to_string())?;
    
    let avdmanager_exe = if cfg!(target_os = "windows") {
        "avdmanager.bat"
    } else {
        "avdmanager"
    };
    
    let avdmanager_path = std::path::Path::new(&android_home)
        .join("cmdline-tools")
        .join("latest")
        .join("bin")
        .join(avdmanager_exe);
    
    Command::new(&avdmanager_path)
        .args(&["delete", "avd", "-n", &id])
        .output()
        .map_err(|e| format!("Failed to delete emulator: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn wipe_android_data(id: String) -> Result<(), String> {
    // Get ANDROID_HOME from settings or environment
    let android_home = crate::commands::settings::get_android_home()
        .ok_or_else(|| "Android SDK path not configured. Please set it in Settings.".to_string())?;
    
    let emulator_exe = if cfg!(target_os = "windows") {
        "emulator.exe"
    } else {
        "emulator"
    };
    
    let emulator_path = std::path::Path::new(&android_home)
        .join("emulator")
        .join(emulator_exe);
    
    Command::new(&emulator_path)
        .args(&["-avd", &id, "-wipe-data"])
        .spawn()
        .map_err(|e| format!("Failed to wipe data: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn screenshot_android(id: String) -> Result<String, String> {
    // Get ANDROID_HOME from settings or environment
    let android_home = crate::commands::settings::get_android_home()
        .ok_or_else(|| "Android SDK path not configured. Please set it in Settings.".to_string())?;
    
    let adb_exe = if cfg!(target_os = "windows") {
        "adb.exe"
    } else {
        "adb"
    };
    
    let adb_path = std::path::Path::new(&android_home)
        .join("platform-tools")
        .join(adb_exe);
    
    let output = Command::new(&adb_path)
        .args(&["devices"])
        .output()
        .map_err(|e| format!("Failed to get devices: {}", e))?;

    let devices = String::from_utf8_lossy(&output.stdout);
    
    for line in devices.lines() {
        if line.contains(&id) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(serial) = parts.first() {
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                let filename = format!("screenshot_{}_{}.png", id, timestamp);
                
                Command::new(&adb_path)
                    .args(&["-s", serial, "exec-out", "screencap", "-p"])
                    .output()
                    .map_err(|e| format!("Failed to take screenshot: {}", e))?;

                return Ok(filename);
            }
        }
    }

    Err("Emulator not found".to_string())
}
