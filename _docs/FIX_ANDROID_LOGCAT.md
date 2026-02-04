# Android 模拟器查看日志功能修复

## 问题描述

点击"查看日志"按钮后没有反应，设备日志标签页没有显示任何日志内容。

## 问题分析

### 前端问题

`handleViewLogs` 函数实现不完整：
```typescript
const handleViewLogs = (id: string) => {
  consoleCollapsed.value = false
  addConsoleLog('info', `查看模拟器日志: ${id}`)
}
```

只是打开了控制台并添加了一条消息，但没有：
1. 切换到设备日志标签页
2. 启动 logcat 进程

### 后端问题

`start_logcat` 函数有严重的实现问题：

1. **使用 `.output()` 而不是 `.spawn()`**
   - `.output()` 会等待命令完成才返回
   - `logcat` 是持续运行的命令，永远不会自动结束
   - 导致命令阻塞，无法获取实时日志

2. **在 tokio::spawn 中使用同步 Command**
   - 阻塞了异步运行时
   - 影响其他异步操作

3. **没有持续读取日志流**
   - 只读取一次就结束了
   - 无法获取实时日志更新

## 修复方案

### 1. 前端修复

修改 `handleViewLogs` 函数，完整实现查看日志功能：

```typescript
const handleViewLogs = async (id: string) => {
  consoleCollapsed.value = false
  consoleTab.value = 'device'
  
  // 清空之前的日志
  deviceLogs.value = []
  
  // 如果已经有 logcat 在运行，先停止
  if (logcatProcess.value) {
    await stopLogcat()
  }
  
  // 启动新的 logcat
  await startLogcat(id)
  addConsoleLog('info', `正在查看设备 ${id} 的日志`)
}
```

### 2. 后端修复

#### 2.1 添加进程管理

```rust
use std::process::{Child, Stdio};
use std::io::{BufRead, BufReader};

static LOGCAT_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
```

#### 2.2 重写 start_logcat 函数

```rust
#[tauri::command]
pub async fn start_logcat(device_id: String) -> Result<(), String> {
    // 先停止之前的 logcat
    stop_logcat().await?;
    
    // ... 获取 adb 路径 ...
    
    // 启动 logcat 进程（使用 spawn 而不是 output）
    let mut child = Command::new(&adb_path)
        .args(&["-s", &device_id, "logcat", "-v", "brief"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start logcat: {}", e))?;
    
    // 获取 stdout
    let stdout = child.stdout.take()
        .ok_or_else(|| "Failed to capture stdout".to_string())?;
    
    // 保存进程句柄
    {
        let mut process = LOGCAT_PROCESS.lock().unwrap();
        *process = Some(child);
    }
    
    // 在后台线程中持续读取日志
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        
        for line in reader.lines() {
            // 检查是否应该停止
            {
                let running = LOGCAT_RUNNING.lock().unwrap();
                if !*running {
                    break;
                }
            }
            
            if let Ok(line) = line {
                let mut buffer = LOGCAT_BUFFER.lock().unwrap();
                
                // 限制缓冲区大小
                if buffer.len() >= 1000 {
                    buffer.remove(0);
                }
                
                buffer.push(line);
            }
        }
    });
    
    Ok(())
}
```

#### 2.3 改进 stop_logcat 函数

```rust
#[tauri::command]
pub async fn stop_logcat() -> Result<(), String> {
    // 标记停止
    {
        let mut running = LOGCAT_RUNNING.lock().unwrap();
        *running = false;
    }
    
    // 终止进程
    {
        let mut process = LOGCAT_PROCESS.lock().unwrap();
        if let Some(mut child) = process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
    
    // 清空缓冲区
    {
        let mut buffer = LOGCAT_BUFFER.lock().unwrap();
        buffer.clear();
    }
    
    Ok(())
}
```

## 关键改进

1. **使用 `.spawn()` 启动进程**
   - 立即返回，不阻塞
   - 可以持续运行

2. **使用 BufReader 逐行读取**
   - 实时读取日志流
   - 不会阻塞

3. **正确的进程管理**
   - 保存进程句柄
   - 可以正确终止进程

4. **前端完整实现**
   - 切换到设备日志标签页
   - 清空旧日志
   - 启动新的 logcat

## 测试步骤

1. 启动一个 Android 模拟器
2. 点击模拟器卡片的"更多"按钮
3. 选择"查看日志"
4. 确认：
   - 控制台自动打开
   - 切换到"设备日志"标签页
   - 开始显示实时日志
   - 日志持续更新
5. 切换到其他模拟器的日志，确认能正确切换
6. 关闭控制台或切换标签页，确认 logcat 进程正确停止

## 预期结果

- ✅ 点击"查看日志"后立即打开控制台
- ✅ 自动切换到设备日志标签页
- ✅ 实时显示 logcat 日志
- ✅ 日志持续更新
- ✅ 可以查看不同设备的日志
- ✅ 关闭时正确清理进程

## 相关文件

- `src/views/HomeView.vue` - 前端日志查看逻辑
- `src-tauri/src/commands/android.rs` - 后端 logcat 实现
