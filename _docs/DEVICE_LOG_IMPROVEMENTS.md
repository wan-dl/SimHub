# 设备日志功能改进

## 改进内容

### 1. 动态标题
- **原来**：固定显示"控制台"
- **现在**：
  - 程序输出标签：显示"控制台"
  - 设备日志标签：显示"<设备名称> 日志"（如 "Pixel_5_API_30 日志"）

### 2. 日志颜色支持
- **后端改进**：logcat 命令添加 `--color=always` 参数
- **效果**：日志会显示 ANSI 颜色代码（需要前端支持渲染）

### 3. 文本选择支持
- **CSS 改进**：
  - 添加 `user-select: text` 到 `.console-content` 和 `.console-log.device .console-message`
  - 添加 `cursor: text` 让用户知道可以选择文本
- **效果**：用户可以选择和复制日志内容

### 4. 字体大小调整
- **原来**：12px
- **现在**：14px（+2px）
- **更清晰易读**

### 5. 行高调整
- **原来**：1.6
- **现在**：1.8
- **效果**：行间距更舒适，不再紧凑

### 6. 日志过滤功能
新增三种过滤方式：

#### 6.1 按日志级别过滤
- 全部（默认）
- Verbose (V)
- Debug (D)
- Info (I)
- Warning (W)
- Error (E)
- Fatal (F)

#### 6.2 按关键字过滤
- 输入关键字，实时过滤包含该关键字的日志
- 不区分大小写

#### 6.3 按包名过滤
- 输入包名，过滤特定应用的日志
- 支持部分匹配

### 7. 日志导出功能
- **功能**：点击"导出"按钮，将日志保存为文本文件
- **文件名格式**：
  - 程序日志：`app-logs-YYYY-MM-DDTHH-mm-ss.txt`
  - 设备日志：`device-logs-<设备名称>-YYYY-MM-DDTHH-mm-ss.txt`
- **内容**：
  - 程序日志：包含时间戳和消息
  - 设备日志：原始 logcat 输出

### 8. 日志清除功能
- **原有功能保留**：点击"清空"按钮清除当前标签页的日志
- **分别清除**：程序日志和设备日志独立清除

## 技术实现

### 前端改进

#### 新增状态变量
```typescript
const currentLogDevice = ref<string>('')  // 当前查看日志的设备名称
const logFilter = ref({
  level: 'all',      // 日志级别过滤
  keyword: '',       // 关键字过滤
  packageName: ''    // 包名过滤
})
```

#### 新增计算属性
```typescript
const filteredDeviceLogs = computed(() => {
  // 按级别、关键字、包名过滤日志
})
```

#### 新增函数
```typescript
const exportLogs = async () => {
  // 导出日志到文件
}
```

### 后端改进

#### logcat 命令参数
```rust
.args(&["-s", &device_id, "logcat", "-v", "time", "--color=always"])
```

- `-v time`：显示日期和时间
- `--color=always`：启用 ANSI 颜色代码

### 样式改进

#### 控制台内容
```css
.console-content {
  user-select: text;
  cursor: text;
}
```

#### 设备日志
```css
.console-log.device .console-message {
  font-size: 14px;      /* +2px */
  line-height: 1.8;     /* 更舒适的行高 */
  user-select: text;    /* 支持文本选择 */
}
```

#### 过滤器
```css
.console-filters {
  display: flex;
  gap: 8px;
  padding: 8px 16px;
  border-bottom: 1px solid #e0e0e0;
  background: #fafafa;
}
```

## 使用说明

### 查看设备日志
1. 启动模拟器
2. 点击模拟器卡片的"更多"按钮
3. 选择"查看日志"
4. 控制台自动打开并显示"<设备名称> 日志"

### 过滤日志
1. 在设备日志标签页，顶部会显示过滤器
2. 选择日志级别（默认"全部"）
3. 输入关键字（可选）
4. 输入包名（可选）
5. 日志实时过滤

### 导出日志
1. 点击控制台头部的"导出"按钮
2. 选择保存位置和文件名
3. 日志保存为文本文件

### 清空日志
1. 点击控制台头部的"清空"按钮
2. 当前标签页的日志被清除

## 注意事项

1. **ANSI 颜色代码**：
   - 后端已添加 `--color=always` 参数
   - 前端需要额外的库（如 `ansi-to-html`）来渲染颜色
   - 当前显示的是原始 ANSI 代码

2. **过滤性能**：
   - 过滤是在前端进行的
   - 大量日志时可能影响性能
   - 建议定期清空日志

3. **日志级别匹配**：
   - 使用正则表达式 `/\s([VDIWEF])\s/` 匹配日志级别
   - 依赖 logcat 的标准格式

## 相关文件

- `src/views/HomeView.vue` - 主视图，包含所有日志功能
- `src/views/HomeView.css` - 样式文件
- `src-tauri/src/commands/android.rs` - 后端 logcat 实现
