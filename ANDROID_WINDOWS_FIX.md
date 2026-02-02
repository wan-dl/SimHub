# Android 模拟器 Windows 路径修复

## 问题
在 Windows 上启动 Android 模拟器时报错：
```
Emulator not found at: "D:\\Program Files\\Android\\sdk\\emulator\\emulator"
```

## 原因
代码使用了 Unix 风格的可执行文件名，但在 Windows 上需要使用 `.exe` 扩展名：
- ❌ `emulator` → ✅ `emulator.exe`
- ❌ `adb` → ✅ `adb.exe`
- ❌ `avdmanager` → ✅ `avdmanager.bat`

## 修复内容

### 1. `list_android_emulators` - 列出模拟器
- ✅ 使用 `emulator.exe` (Windows) 或 `emulator` (Unix)
- ✅ 使用 `adb.exe` (Windows) 或 `adb` (Unix)
- ✅ 移除了不兼容的 `ps aux` 命令，改用 `adb devices`

### 2. `start_android_emulator` - 启动模拟器
- ✅ 使用 `emulator.exe` (Windows) 或 `emulator` (Unix)
- ✅ 检查文件是否存在后再启动

### 3. `stop_android_emulator` - 停止模拟器
- ✅ 使用 `adb.exe` (Windows) 或 `adb` (Unix)
- ✅ 移除了不兼容的 `ps aux` 命令

### 4. `delete_android_emulator` - 删除模拟器
- ✅ 使用 `avdmanager.bat` (Windows) 或 `avdmanager` (Unix)
- ✅ 使用完整路径：`ANDROID_HOME/cmdline-tools/latest/bin/avdmanager`

### 5. `wipe_android_data` - 清除数据
- ✅ 使用 `emulator.exe` (Windows) 或 `emulator` (Unix)

### 6. `screenshot_android` - 截图
- ✅ 使用 `adb.exe` (Windows) 或 `adb` (Unix)

## 使用方法

### 1. 配置 Android SDK 路径
1. 打开应用
2. 点击"设置"
3. 进入"Android"标签
4. 点击"选择文件夹"
5. 选择你的 Android SDK 路径，例如：
   - `D:\Program Files\Android\sdk`
   - `C:\Users\你的用户名\AppData\Local\Android\Sdk`

### 2. 验证 SDK 结构
确保你的 Android SDK 包含以下文件：
```
Android SDK/
├── emulator/
│   └── emulator.exe          ← 必需
├── platform-tools/
│   └── adb.exe               ← 必需
└── cmdline-tools/
    └── latest/
        └── bin/
            └── avdmanager.bat ← 删除模拟器时需要
```

### 3. 重新构建应用
```bash
pnpm tauri dev
```

### 4. 测试
1. 点击"刷新"按钮
2. 应该能看到你的 Android 模拟器列表
3. 点击"启动"按钮启动模拟器

## 常见问题

### Q: 仍然提示找不到模拟器
**A:** 检查以下几点：
1. Android SDK 路径是否正确
2. `emulator.exe` 是否存在于 `SDK路径/emulator/` 目录下
3. 重启应用后再试

### Q: 能列出模拟器但无法启动
**A:** 可能的原因：
1. 模拟器配置损坏
2. 缺少系统镜像
3. 查看控制台错误信息

### Q: 如何查看详细错误
**A:** 
1. 应用底部有"控制台"面板
2. 点击展开查看详细日志
3. 错误会自动展开控制台

## 技术细节

### 跨平台可执行文件检测
```rust
let emulator_exe = if cfg!(target_os = "windows") {
    "emulator.exe"
} else {
    "emulator"
};
```

### 完整路径构建
```rust
let emulator_path = std::path::Path::new(&android_home)
    .join("emulator")
    .join(emulator_exe);
```

这样可以确保在所有平台上都使用正确的路径分隔符和文件名。

## 文件修改
- ✅ `src-tauri/src/commands/android.rs` - 所有 Android 命令函数

## 下一步
重新构建应用并测试 Android 模拟器功能。
