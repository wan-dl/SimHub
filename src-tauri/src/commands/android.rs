use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Mutex;
use tokio;
use tauri::Emitter;
use arboard::{Clipboard, ImageData};
use image::GenericImageView;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader as TokioBufReader};

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
            
            // 创建一个映射来存储设备序列号到 AVD 名称的关系
            let mut serial_to_avd: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            
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
                                let avd_name = avd_name.trim().to_string();
                                serial_to_avd.insert(serial.to_string(), avd_name);
                            }
                        }
                    }
                }
            }
            
            // 更新模拟器状态和 ID
            for emu in &mut emulators {
                for (serial, avd_name) in &serial_to_avd {
                    if emu.name == *avd_name {
                        emu.status = "running".to_string();
                        emu.id = serial.clone();
                        break;
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
                        // 尝试通过 adb 获取设备序列号
                        if let Ok(adb_output) = Command::new(&adb_path).arg("devices").output() {
                            let devices = String::from_utf8_lossy(&adb_output.stdout);
                            for line in devices.lines() {
                                if line.contains("emulator-") && line.contains("device") {
                                    let parts: Vec<&str> = line.split_whitespace().collect();
                                    if let Some(serial) = parts.first() {
                                        // 验证这个设备是否对应当前模拟器
                                        if let Ok(avd_output) = Command::new(&adb_path)
                                            .args(["-s", serial, "emu", "avd", "name"])
                                            .output() 
                                        {
                                            let avd_name = String::from_utf8_lossy(&avd_output.stdout);
                                            if avd_name.trim() == emu.name {
                                                emu.id = serial.to_string();
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
                        // 尝试通过 adb 获取设备序列号
                        if let Ok(adb_output) = Command::new(&adb_path).arg("devices").output() {
                            let devices = String::from_utf8_lossy(&adb_output.stdout);
                            for line in devices.lines() {
                                if line.contains("emulator-") && line.contains("device") {
                                    let parts: Vec<&str> = line.split_whitespace().collect();
                                    if let Some(serial) = parts.first() {
                                        // 验证这个设备是否对应当前模拟器
                                        if let Ok(avd_output) = Command::new(&adb_path)
                                            .args(["-s", serial, "emu", "avd", "name"])
                                            .output() 
                                        {
                                            let avd_name = String::from_utf8_lossy(&avd_output.stdout);
                                            if avd_name.trim() == emu.name {
                                                emu.id = serial.to_string();
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
        }
    }

    Ok(emulators)
}

#[tauri::command]
pub async fn start_android_emulator(id: String, app: tauri::AppHandle) -> Result<(), String> {
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
    
    // Load launch parameters
    let params = crate::commands::settings::get_emulator_launch_params_sync(&id, "android");
    
    // Set up environment variables
    let mut cmd = Command::new(&emulator_path);
    cmd.arg("-avd")
        .arg(&id)
        .env("ANDROID_HOME", &android_home)
        .env("ANDROID_SDK_ROOT", &android_home)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    
    // Apply launch parameters
    if params.no_window {
        cmd.arg("-no-window");
    }
    
    if !params.dns_server.is_empty() {
        cmd.arg("-dns-server").arg(&params.dns_server);
    }
    
    if !params.gps_longitude.is_empty() && !params.gps_latitude.is_empty() {
        cmd.arg("-gps").arg(format!("{},{}", params.gps_longitude, params.gps_latitude));
    }
    
    if let Some(memory) = params.memory {
        cmd.arg("-memory").arg(memory.to_string());
    }
    
    if !params.http_proxy.is_empty() {
        cmd.arg("-http-proxy").arg(&params.http_proxy);
    }
    
    // 直接打印 Command 对象，确保日志与实际执行的命令一致
    let cmd_str = format!("{:?}", cmd);
    
    // 发送启动命令到前端控制台
    let _ = app.emit("add-log", serde_json::json!({
        "type": "command",
        "message": cmd_str.clone(),
        "source": "app"
    }));
    
    println!("启动 Android 模拟器命令: {}", cmd_str);
    
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
pub async fn wipe_android_data(id: String, app: tauri::AppHandle) -> Result<(), String> {
    // Get ANDROID_HOME from settings or environment
    let android_home = crate::commands::settings::get_android_home()
        .ok_or_else(|| "Android SDK path not configured. Please set it in Settings.".to_string())?;
    
    // Get AVD directory path
    let avd_home = std::env::var("ANDROID_AVD_HOME")
        .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.android/avd", h)))
        .or_else(|_| std::env::var("USERPROFILE").map(|h| format!("{}/.android/avd", h)))
        .map_err(|_| "Cannot determine AVD home directory".to_string())?;
    
    // 先读取 .ini 文件获取真实的 AVD 路径
    let ini_path = std::path::Path::new(&avd_home).join(format!("{}.ini", id));
    
    let avd_path = if ini_path.exists() {
        // 读取 .ini 文件内容
        let ini_content = std::fs::read_to_string(&ini_path)
            .map_err(|e| format!("Failed to read AVD ini file: {}", e))?;
        
        // 查找 path= 行
        let mut real_path = None;
        for line in ini_content.lines() {
            if line.starts_with("path=") {
                real_path = Some(line.trim_start_matches("path=").trim().to_string());
                break;
            }
        }
        
        if let Some(path) = real_path {
            std::path::PathBuf::from(path)
        } else {
            // 如果 .ini 文件中没有 path，回退到默认方式
            std::path::Path::new(&avd_home).join(format!("{}.avd", id))
        }
    } else {
        // 如果 .ini 文件不存在，使用默认方式
        std::path::Path::new(&avd_home).join(format!("{}.avd", id))
    };
    
    // 输出操作命令到日志
    let cmd_str = format!("清除 Android 模拟器数据: AVD={}, INI={:?}, 实际路径={:?}", id, ini_path, avd_path);
    println!("{}", cmd_str);
    let _ = app.emit("add-log", serde_json::json!({
        "type": "command",
        "message": cmd_str,
        "source": "app"
    }));
    
    if !avd_path.exists() {
        let error_msg = format!("AVD directory not found: {:?}", avd_path);
        println!("Error: {}", error_msg);
        let _ = app.emit("add-log", serde_json::json!({
            "type": "error",
            "message": error_msg.clone(),
            "source": "app"
        }));
        return Err(error_msg);
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
            let delete_msg = format!("删除文件: {:?}", file_path);
            println!("{}", delete_msg);
            let _ = app.emit("add-log", serde_json::json!({
                "type": "info",
                "message": delete_msg,
                "source": "app"
            }));
            
            if file_path.is_dir() {
                std::fs::remove_dir_all(&file_path)
                    .map_err(|e| {
                        let error_msg = format!("Failed to delete {}: {}", file, e);
                        println!("Error: {}", error_msg);
                        let _ = app.emit("add-log", serde_json::json!({
                            "type": "error",
                            "message": error_msg.clone(),
                            "source": "app"
                        }));
                        error_msg
                    })?;
            } else {
                std::fs::remove_file(&file_path)
                    .map_err(|e| {
                        let error_msg = format!("Failed to delete {}: {}", file, e);
                        println!("Error: {}", error_msg);
                        let _ = app.emit("add-log", serde_json::json!({
                            "type": "error",
                            "message": error_msg.clone(),
                            "source": "app"
                        }));
                        error_msg
                    })?;
            }
            deleted_count += 1;
        }
    }
    
    if deleted_count == 0 {
        // No user data files found, AVD might be clean already
        let info_msg = format!("模拟器 {} 数据已经是干净的，无需清除", id);
        println!("{}", info_msg);
        let _ = app.emit("add-log", serde_json::json!({
            "type": "info",
            "message": info_msg,
            "source": "app"
        }));
        return Ok(());
    }
    
    let success_msg = format!("成功清除 {} 个数据文件", deleted_count);
    println!("{}", success_msg);
    let _ = app.emit("add-log", serde_json::json!({
        "type": "success",
        "message": success_msg,
        "source": "app"
    }));

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
pub async fn start_logcat(device_id: String, time_filter: Option<String>) -> Result<(), String> {
    // 先停止之前的 logcat
    stop_logcat().await?;
    
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
    
    // 清空缓冲区
    {
        let mut buffer = LOGCAT_BUFFER.lock().unwrap();
        buffer.clear();
    }
    
    // 构建 logcat 命令参数
    let mut args = vec!["-s".to_string(), device_id.clone(), "logcat".to_string(), "-v".to_string(), "color".to_string()];
    
    // 添加时间过滤参数
    // time_filter 格式：
    // - "recent:5" 表示最近5分钟
    // - "since:2024-02-04T10:30:00Z" 表示从某个时间点开始
    if let Some(filter) = time_filter {
        if filter.starts_with("recent:") {
            // 最近 N 分钟
            if let Ok(minutes) = filter.strip_prefix("recent:").unwrap_or("").parse::<i64>() {
                // 计算时间点
                let now = chrono::Local::now();
                let since = now - chrono::Duration::minutes(minutes);
                let time_str = since.format("%m-%d %H:%M:%S.000").to_string();
                args.push("-t".to_string());
                args.push(time_str);
            }
        } else if filter.starts_with("since:") {
            // 从某个时间点开始
            if let Some(time_str) = filter.strip_prefix("since:") {
                // 解析时间字符串
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(time_str) {
                    let local_dt = dt.with_timezone(&chrono::Local);
                    let time_str = local_dt.format("%m-%d %H:%M:%S.000").to_string();
                    args.push("-t".to_string());
                    args.push(time_str);
                }
            }
        }
    }
    
    // 转换为字符串引用
    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    
    // 启动 logcat 进程
    let mut child = tokio::process::Command::new(&adb_path)
        .args(&args_refs)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start logcat: {}", e))?;
    
    // 获取 stdout
    let stdout = child.stdout.take()
        .ok_or_else(|| "Failed to capture stdout".to_string())?;
    
    // 标记 logcat 正在运行
    {
        let mut running = LOGCAT_RUNNING.lock().unwrap();
        *running = true;
    }
    
    // 在后台线程中持续读取日志
    tokio::spawn(async move {
        let reader = TokioBufReader::new(stdout);
        let mut lines = reader.lines();
        
        while let Ok(Some(line)) = lines.next_line().await {
            // 检查是否应该停止
            {
                let running = LOGCAT_RUNNING.lock().unwrap();
                if !*running {
                    break;
                }
            }
            
            let mut buffer = LOGCAT_BUFFER.lock().unwrap();
            
            // 限制缓冲区大小
            if buffer.len() >= 1000 {
                buffer.remove(0);
            }
            
            buffer.push(line);
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
    // 标记停止
    {
        let mut running = LOGCAT_RUNNING.lock().unwrap();
        *running = false;
    }
    
    // 清空缓冲区
    {
        let mut buffer = LOGCAT_BUFFER.lock().unwrap();
        buffer.clear();
    }
    
    // 注意：由于我们使用 tokio::process::Command，进程会在任务结束时自动清理
    // 我们通过设置 LOGCAT_RUNNING = false 来让读取循环退出
    
    Ok(())
}

#[tauri::command]
pub async fn copy_image_to_clipboard(path: String) -> Result<(), String> {
    // 读取图片文件
    let img = image::open(&path)
        .map_err(|e| format!("Failed to open image: {}", e))?;
    
    // 获取图片尺寸和像素数据
    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8();
    let pixels = rgba.into_raw();
    
    // 创建 ImageData
    let img_data = ImageData {
        width: width as usize,
        height: height as usize,
        bytes: std::borrow::Cow::from(pixels),
    };
    
    // 复制到剪贴板
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to access clipboard: {}", e))?;
    
    clipboard.set_image(img_data)
        .map_err(|e| format!("Failed to copy image to clipboard: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn write_log_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write log file: {}", e))?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub is_system: bool,
}

#[tauri::command]
pub async fn get_device_packages(device_id: String) -> Result<Vec<PackageInfo>, String> {
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
    
    // 获取所有包名
    let output = Command::new(&adb_path)
        .args(&["-s", &device_id, "shell", "pm", "list", "packages"])
        .output()
        .map_err(|e| format!("Failed to list packages: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to get packages: {}", stderr));
    }
    
    let packages_output = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();
    
    // 获取系统包列表
    let system_output = Command::new(&adb_path)
        .args(&["-s", &device_id, "shell", "pm", "list", "packages", "-s"])
        .output()
        .map_err(|e| format!("Failed to list system packages: {}", e))?;
    
    let system_packages_output = String::from_utf8_lossy(&system_output.stdout);
    let system_packages: std::collections::HashSet<String> = system_packages_output
        .lines()
        .filter_map(|line| {
            line.strip_prefix("package:")
                .map(|s| s.trim().to_string())
        })
        .collect();
    
    // 解析所有包名
    for line in packages_output.lines() {
        if let Some(package_name) = line.strip_prefix("package:") {
            let package_name = package_name.trim().to_string();
            let is_system = system_packages.contains(&package_name);
            
            packages.push(PackageInfo {
                name: package_name,
                is_system,
            });
        }
    }
    
    // 排序：第三方包在前，系统包在后
    packages.sort_by(|a, b| {
        match (a.is_system, b.is_system) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });
    
    Ok(packages)
}
