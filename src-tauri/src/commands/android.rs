use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Mutex;
use tokio;

static LOGCAT_BUFFER: Mutex<Vec<String>> = Mutex::new(Vec::new());
static LOGCAT_RUNNING: Mutex<bool> = Mutex::new(false);

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
    
    // 额外检查：通过进程名称判断模拟器是否在运行
    // 这样即使 adb 还没连接上，只要进程存在就显示为运行状态
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("wmic")
            .args(&["process", "get", "CommandLine", "/format:csv"])
            .output() 
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for emu in &mut emulators {
                if emu.status == "stopped" {
                    // 检查是否有包含该模拟器名称的进程
                    if output_str.contains(&format!("-avd {}", emu.name)) || 
                       output_str.contains(&format!("-avd \"{}\"", emu.name)) {
                        emu.status = "running".to_string();
                    }
                }
            }
        }
    }
    
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        if let Ok(output) = Command::new("ps")
            .args(&["aux"])
            .output() 
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for emu in &mut emulators {
                if emu.status == "stopped" {
                    // 检查是否有包含该模拟器名称的进程
                    if output_str.contains(&format!("-avd {}", emu.name)) {
                        emu.status = "running".to_string();
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

    // Wait for initial startup and check for immediate errors
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // Check if process crashed immediately
    match child.try_wait() {
        Ok(Some(status)) => {
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
            // Process is running, wait for emulator to be ready
            for _ in 0..30 { // Wait up to 30 seconds
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                
                // Check if emulator is ready by listing running emulators
                let adb_path = std::path::Path::new(&android_home)
                    .join("platform-tools")
                    .join(if cfg!(target_os = "windows") { "adb.exe" } else { "adb" });
                
                if let Ok(output) = Command::new(&adb_path)
                    .args(&["devices"])
                    .output() {
                    let devices_output = String::from_utf8_lossy(&output.stdout);
                    if devices_output.contains(&id) || devices_output.lines().any(|line| line.contains("device") && !line.contains("List")) {
                        break; // Emulator is ready
                    }
                }
                
                // Check if process is still running
                if let Ok(Some(_)) = child.try_wait() {
                    return Err("Emulator process terminated unexpectedly".to_string());
                }
            }
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
    
    // Check if force kill is enabled
    let force_kill = crate::commands::settings::get_android_force_kill();
    
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
    
    if force_kill {
        println!("Force kill mode enabled for emulator: {}", id);
        
        // 先找到模拟器进程的PID
        #[cfg(target_os = "windows")]
        {
            // Windows: 使用 wmic 查找进程
            println!("Finding process with wmic...");
            
            let output = Command::new("wmic")
                .args(&["process", "get", "ProcessId,CommandLine", "/format:csv"])
                .output()
                .map_err(|e| format!("Failed to find emulator process: {}", e))?;
            
            let output_str = String::from_utf8_lossy(&output.stdout);
            println!("Process list output (filtered):");
            
            // 查找包含 qemu-system 或 emulator 且包含模拟器ID的进程
            let mut pids = Vec::new();
            for line in output_str.lines() {
                if (line.contains("qemu-system") || line.contains("emulator.exe")) && 
                   (line.contains(&id) || line.contains(&serial)) {
                    println!("Found matching process: {}", line);
                    
                    // CSV 格式：Node,CommandLine,ProcessId
                    // 提取最后一个字段作为PID
                    let parts: Vec<&str> = line.split(',').collect();
                    if let Some(pid_str) = parts.last() {
                        let pid_str = pid_str.trim();
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            pids.push(pid);
                        }
                    }
                }
            }
            
            if pids.is_empty() {
                println!("Cannot find process for emulator: {}, trying alternative method", id);
                
                // 尝试查找所有模拟器进程
                for line in output_str.lines() {
                    if line.contains("qemu-system") || (line.contains("emulator") && line.contains("-avd")) {
                        println!("Found emulator process: {}", line);
                        
                        let parts: Vec<&str> = line.split(',').collect();
                        if let Some(pid_str) = parts.last() {
                            let pid_str = pid_str.trim();
                            if let Ok(pid) = pid_str.parse::<u32>() {
                                pids.push(pid);
                            }
                        }
                    }
                }
            }
            
            if pids.is_empty() {
                return Err(format!("Cannot find process for emulator: {}", id));
            }
            
            // 强杀找到的进程
            for pid in pids {
                let kill_cmd = format!("taskkill /F /PID {}", pid);
                println!("Executing: {}", kill_cmd);
                
                let output = Command::new("taskkill")
                    .args(&["/F", "/PID", &pid.to_string()])
                    .output()
                    .map_err(|e| format!("Failed to kill process {}: {}", pid, e))?;
                
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("Kill failed: {}", stderr);
                } else {
                    println!("Successfully killed process: {}", pid);
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            // macOS: 使用 ps 查找进程
            let find_cmd = vec!["ps", "aux"];
            println!("Finding process: {:?}", find_cmd);
            
            let output = Command::new("ps")
                .args(&["aux"])
                .output()
                .map_err(|e| format!("Failed to find emulator process: {}", e))?;
            
            let output_str = String::from_utf8_lossy(&output.stdout);
            println!("Process list output (filtered):");
            
            // 查找包含 qemu-system 或 emulator 且包含模拟器ID的进程
            let mut pids = Vec::new();
            for line in output_str.lines() {
                if (line.contains("qemu-system") || line.contains("/emulator")) && 
                   (line.contains(&id) || line.contains(&serial)) {
                    println!("Found matching process: {}", line);
                    
                    // 提取PID（第二列）
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 1 {
                        if let Ok(pid) = parts[1].parse::<u32>() {
                            pids.push(pid);
                        }
                    }
                }
            }
            
            if pids.is_empty() {
                println!("Cannot find process for emulator: {}, trying alternative method", id);
                
                // 尝试查找所有 qemu-system 进程
                for line in output_str.lines() {
                    if line.contains("qemu-system") || (line.contains("emulator") && line.contains("-avd")) {
                        println!("Found emulator process: {}", line);
                        
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() > 1 {
                            if let Ok(pid) = parts[1].parse::<u32>() {
                                pids.push(pid);
                            }
                        }
                    }
                }
            }
            
            if pids.is_empty() {
                return Err(format!("Cannot find process for emulator: {}", id));
            }
            
            // 强杀找到的进程
            for pid in pids {
                let kill_cmd = format!("kill -9 {}", pid);
                println!("Executing: {}", kill_cmd);
                
                let output = Command::new("kill")
                    .args(&["-9", &pid.to_string()])
                    .output()
                    .map_err(|e| format!("Failed to kill process {}: {}", pid, e))?;
                
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("Kill failed: {}", stderr);
                } else {
                    println!("Successfully killed process: {}", pid);
                }
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            // Linux: 使用 ps 查找进程
            let find_cmd = vec!["ps", "aux"];
            println!("Finding process: {:?}", find_cmd);
            
            let output = Command::new("ps")
                .args(&["aux"])
                .output()
                .map_err(|e| format!("Failed to find emulator process: {}", e))?;
            
            let output_str = String::from_utf8_lossy(&output.stdout);
            println!("Process list output (filtered):");
            
            // 查找包含 qemu-system 或 emulator 且包含模拟器ID的进程
            let mut pids = Vec::new();
            for line in output_str.lines() {
                if (line.contains("qemu-system") || line.contains("/emulator")) && 
                   (line.contains(&id) || line.contains(&serial)) {
                    println!("Found matching process: {}", line);
                    
                    // 提取PID（第二列）
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() > 1 {
                        if let Ok(pid) = parts[1].parse::<u32>() {
                            pids.push(pid);
                        }
                    }
                }
            }
            
            if pids.is_empty() {
                println!("Cannot find process for emulator: {}, trying alternative method", id);
                
                // 尝试查找所有模拟器进程
                for line in output_str.lines() {
                    if line.contains("qemu-system") || (line.contains("emulator") && line.contains("-avd")) {
                        println!("Found emulator process: {}", line);
                        
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() > 1 {
                            if let Ok(pid) = parts[1].parse::<u32>() {
                                pids.push(pid);
                            }
                        }
                    }
                }
            }
            
            if pids.is_empty() {
                return Err(format!("Cannot find process for emulator: {}", id));
            }
            
            // 强杀找到的进程
            for pid in pids {
                let kill_cmd = format!("kill -9 {}", pid);
                println!("Executing: {}", kill_cmd);
                
                let output = Command::new("kill")
                    .args(&["-9", &pid.to_string()])
                    .output()
                    .map_err(|e| format!("Failed to kill process {}: {}", pid, e))?;
                
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("Kill failed: {}", stderr);
                } else {
                    println!("Successfully killed process: {}", pid);
                }
            }
        }
    } else {
        // 正常关闭方式
        println!("Normal shutdown mode for emulator: {}", id);
        let kill_cmd = format!("{:?} -s {} emu kill", adb_path, serial);
        println!("Executing: {}", kill_cmd);
        
        let kill_output = Command::new(&adb_path)
            .args(&["-s", &serial, "emu", "kill"])
            .output()
            .map_err(|e| format!("Failed to stop emulator {}: {}", serial, e))?;
        
        if !kill_output.status.success() {
            let stderr = String::from_utf8_lossy(&kill_output.stderr);
            println!("Normal shutdown failed: {}", stderr);
            return Err(format!("Failed to stop emulator {}: {}", serial, stderr));
        }
        
        println!("Normal shutdown successful");
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
    
    let output = Command::new(&avdmanager_path)
        .args(&["delete", "avd", "-n", &id])
        .output()
        .map_err(|e| format!("Failed to delete emulator: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let error_msg = if !stderr.is_empty() {
            stderr.to_string()
        } else if !stdout.is_empty() {
            stdout.to_string()
        } else {
            format!("Failed to delete AVD: {}", id)
        };
        return Err(error_msg);
    }

    Ok(())
}

#[tauri::command]
pub async fn wipe_android_data(id: String) -> Result<(), String> {
    // Get ANDROID_HOME from settings or environment
    let android_home = crate::commands::settings::get_android_home()
        .ok_or_else(|| "Android SDK path not configured. Please set it in Settings.".to_string())?;
    
    // Get AVD directory path
    let avd_home = std::env::var("ANDROID_AVD_HOME")
        .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.android/avd", h)))
        .or_else(|_| std::env::var("USERPROFILE").map(|h| format!("{}/.android/avd", h)))
        .map_err(|_| "Cannot determine AVD home directory".to_string())?;
    
    let avd_path = std::path::Path::new(&avd_home).join(format!("{}.avd", id));
    
    if !avd_path.exists() {
        return Err(format!("AVD directory not found: {:?}", avd_path));
    }
    
    // Delete userdata files to wipe data (without starting emulator)
    let files_to_delete = [
        "userdata-qemu.img",
        "userdata-qemu.img.qcow2", 
        "cache.img",
        "cache.img.qcow2",
        "snapshots",
    ];
    
    let mut deleted_count = 0;
    for file in &files_to_delete {
        let file_path = avd_path.join(file);
        if file_path.exists() {
            if file_path.is_dir() {
                std::fs::remove_dir_all(&file_path)
                    .map_err(|e| format!("Failed to delete {}: {}", file, e))?;
            } else {
                std::fs::remove_file(&file_path)
                    .map_err(|e| format!("Failed to delete {}: {}", file, e))?;
            }
            deleted_count += 1;
        }
    }
    
    if deleted_count == 0 {
        // No user data files found, AVD might be clean already
        return Ok(());
    }

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
    
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("screenshot_{}_{}.png", id, timestamp);
    
    // Get screenshot directory from settings
    let screenshot_dir = crate::commands::settings::get_screenshot_dir()
        .ok_or_else(|| "Cannot find screenshot directory".to_string())?;
    let path = std::path::Path::new(&screenshot_dir).join(&filename);
    
    // Take screenshot and save to file
    let output = Command::new(&adb_path)
        .args(&["-s", &id, "exec-out", "screencap", "-p"])
        .output()
        .map_err(|e| format!("Failed to take screenshot: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Screenshot failed: {}", stderr));
    }

    // Write screenshot data to file
    std::fs::write(&path, &output.stdout)
        .map_err(|e| format!("Failed to save screenshot: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn start_logcat(device_id: String) -> Result<(), String> {
    let android_home = crate::commands::settings::get_android_home()
        .ok_or_else(|| "Android SDK path not configured".to_string())?;
    
    let adb_exe = if cfg!(target_os = "windows") {
        "adb.exe"
    } else {
        "adb"
    };
    
    let adb_path = std::path::Path::new(&android_home)
        .join("platform-tools")
        .join(adb_exe);
    
    // 标记 logcat 正在运行
    {
        let mut running = LOGCAT_RUNNING.lock().unwrap();
        *running = true;
    }
    
    // 清空缓冲区
    {
        let mut buffer = LOGCAT_BUFFER.lock().unwrap();
        buffer.clear();
    }
    
    // 在后台线程中运行 logcat
    let adb_path_clone = adb_path.clone();
    let device_id_clone = device_id.clone();
    
    tokio::spawn(async move {
        let output = Command::new(&adb_path_clone)
            .args(&["-s", &device_id_clone, "logcat", "-v", "brief"])
            .output();
        
        if let Ok(output) = output {
            let logs = String::from_utf8_lossy(&output.stdout);
            let mut buffer = LOGCAT_BUFFER.lock().unwrap();
            
            for line in logs.lines() {
                if buffer.len() >= 1000 {
                    buffer.remove(0);
                }
                buffer.push(line.to_string());
            }
        }
    });
    
    Ok(())
}

#[tauri::command]
pub async fn get_logcat_logs(_device_id: String) -> Result<Vec<String>, String> {
    let mut buffer = LOGCAT_BUFFER.lock().unwrap();
    let logs = buffer.clone();
    buffer.clear();
    Ok(logs)
}

#[tauri::command]
pub async fn stop_logcat() -> Result<(), String> {
    let mut running = LOGCAT_RUNNING.lock().unwrap();
    *running = false;
    
    let mut buffer = LOGCAT_BUFFER.lock().unwrap();
    buffer.clear();
    
    Ok(())
}
