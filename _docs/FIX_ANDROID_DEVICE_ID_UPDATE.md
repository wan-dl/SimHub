# Android 模拟器设备 ID 更新问题修复

## 问题描述

Android 模拟器启动后，设备 ID 显示数据没有更新。具体表现为：
- 模拟器未启动时，ID 为 AVD 名称（如 `Pixel_5_API_30`）
- 模拟器启动后，ID 应该更新为设备序列号（如 `emulator-5554`）
- 但实际上 ID 没有及时更新，导致后续操作（截图、停止等）可能失败

## 根本原因

在 `list_android_emulators` 函数中：

1. **初始状态**：所有模拟器的 ID 都设置为 AVD 名称
2. **adb 检查**：通过 `adb devices` 获取运行中的设备，并更新 ID 为设备序列号
3. **进程检查**：通过进程列表检查模拟器是否运行，但只更新了 `status`，没有更新 `id`

问题出在第 3 步：当模拟器刚启动时，adb 可能还没连接上，但进程已经存在。此时通过进程检查发现模拟器运行，但 ID 仍然是 AVD 名称，而不是设备序列号。

## 修复方案

### 1. 优化 adb 设备检查逻辑

使用 HashMap 来存储设备序列号和 AVD 名称的映射关系：

```rust
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
```

### 2. 增强进程检查逻辑

当通过进程检查发现模拟器运行时，也尝试从 adb 获取设备序列号：

```rust
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
```

## 测试步骤

1. 启动一个 Android 模拟器
2. 在前端查看模拟器列表，确认状态显示为"运行中"
3. 点击"复制 ID"，确认复制的是设备序列号（如 `emulator-5554`），而不是 AVD 名称
4. 尝试截图功能，确认能正常工作
5. 尝试停止模拟器，确认能正常关闭

## 预期结果

- ✅ 模拟器启动后，ID 正确更新为设备序列号
- ✅ 复制 ID 功能复制的是设备序列号
- ✅ 截图、停止等操作使用正确的设备 ID
- ✅ 即使 adb 连接延迟，也能在后续刷新中正确更新 ID

## 相关文件

- `src-tauri/src/commands/android.rs` - Android 模拟器命令实现
- `src/components/EmulatorCard.vue` - 模拟器卡片组件
- `src/views/HomeView.vue` - 主视图，包含模拟器列表和操作逻辑
