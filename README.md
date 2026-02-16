# OH Emulator Manager

Windows、MacOS跨平台模拟器管理工具 - 可视化管理 iOS、Android、HarmonyOS 模拟器

## 功能特性

- 🎯 统一管理 iOS、Android、HarmonyOS 模拟器
- 🚀 快速启动、关闭、删除模拟器
- 📸 一键截图功能
- 🌍 多语言支持（中文/英文）
- 🎨 深色/浅色主题
- ⚙️ 灵活的 SDK 路径配置

## 技术栈

- **框架**: Tauri 2.x (Rust + WebView)
- **前端**: Vue 3 + TypeScript
- **UI**: Naive UI
- **状态管理**: Pinia
- **样式**: UnoCSS

## 开发环境要求

- Node.js 18+
- Rust 1.70+
- pnpm 8+

### 平台特定要求

#### Windows
- **必需**: Visual Studio 2022 Build Tools
  - 下载：https://visualstudio.microsoft.com/zh-hans/downloads/
  - 选择 "使用 C++ 的桌面开发"
  - 包含 MSVC 和 Windows SDK
- Android SDK (用于 Android 模拟器)
- DevEco Studio 5.x+ (用于 HarmonyOS 模拟器)

#### macOS
- Xcode (用于 iOS 模拟器)
- Android SDK (用于 Android 模拟器)
- DevEco Studio 5.x+ (用于 HarmonyOS 模拟器)

## 快速开始

### 1. 安装依赖

```bash
pnpm install
```

### 2. Windows 用户：安装 Visual Studio Build Tools

如果遇到编译错误，请参考 `fix-windows-build.md` 安装 MSVC 工具链。

### 3. 开发模式

```bash
pnpm tauri:dev
```

⚠️ **首次运行需要 10-30 分钟下载和编译 Rust 依赖**

### 4. 构建生产版本

```bash
pnpm tauri:build
```

## 项目结构

```
oh-emulator-manager/
├── src/                    # 前端源码
│   ├── components/         # Vue 组件
│   ├── views/              # 页面视图
│   ├── stores/             # Pinia 状态
│   ├── i18n/               # 国际化
│   └── router/             # 路由配置
├── src-tauri/              # Tauri 后端
│   └── src/
│       ├── commands/       # Tauri 命令
│       └── db/             # 数据库
└── _docs/                  # 项目文档
```

## 许可证

MIT License
