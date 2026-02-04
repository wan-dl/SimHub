# ConsolePanel 组件集成指南

## 概述
已将日志查看功能从 HomeView.vue 抽离为独立的 ConsolePanel.vue 组件。

## 修改步骤

### 1. 在 HomeView.vue 中导入 ConsolePanel 组件

```vue
<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, nextTick, watch } from 'vue'
import { NButton, NInput, NIcon, useMessage, useDialog, NSelect } from 'naive-ui'
import { Refresh } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useEmulatorStore } from '@/stores/emulator'
import EmulatorList from '@/components/EmulatorList.vue'
import ConsolePanel from '@/components/ConsolePanel.vue'  // 新增
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
```

### 2. 替换模板中的控制台部分

**原来的代码（删除）：**
```vue
<!-- 第三列：控制台 -->
<transition name="console-slide">
  <div v-if="!consoleCollapsed" class="console-panel" :style="{ width: consolePanelWidth + 'px' }">
    <!-- 大量的控制台相关代码 -->
  </div>
</transition>
```

**新的代码（替换为）：**
```vue
<!-- 第三列：控制台 -->
<console-panel
  ref="consolePanelRef"
  v-model:collapsed="consoleCollapsed"
  :panel-width="consolePanelWidth"
  :device-options="runningDeviceOptions"
  :app-logs="consoleLogs"
  @device-change="handleDeviceChange"
  @add-log="handleAddLog"
/>
```

### 3. 简化 script 部分

**删除以下状态变量：**
- `consoleTab`
- `deviceLogs`
- `logcatProcess`
- `currentLogDevice`
- `selectedLogDevice`
- `showKeywordFilter`
- `logOutputPaused`
- `packageList`
- `timeFilterType`
- `recentMinutes`
- `sinceTime`
- `logFilter`
- `ansiConverter`
- `renderPackageLabel`

**删除以下函数：**
- `startLogcat`
- `stopLogcat`
- `handleDeviceLogTab`
- `handleDeviceChange`
- `loadDevicePackages`
- `toggleLogOutput`
- `clearConsole`
- `exportLogs`
- `filteredDeviceLogs`
- `restartLogcatWithTimeFilter`
- 所有与日志相关的 watch

**保留并简化：**
```typescript
const consolePanelRef = ref<InstanceType<typeof ConsolePanel>>()
const consoleLogs = ref<Array<{type: string, message: string, time: string, path?: string}>>([])

const addConsoleLog = (type: string, message: string, path?: string) => {
  const time = new Date().toLocaleTimeString()
  consoleLogs.value.push({ type, message, time, path })
  // 只在错误时自动展开控制台
  if (type === 'error') {
    consoleCollapsed.value = false
  }
}

const handleAddLog = (type: string, message: string, path?: string) => {
  if (type === 'clear') {
    consoleLogs.value = []
  } else {
    addConsoleLog(type, message, path)
  }
}

const handleViewLogs = async (id: string) => {
  try {
    consoleCollapsed.value = false
    await consolePanelRef.value?.startDeviceLog(id)
    addConsoleLog('info', `正在查看设备 ${id} 的日志`)
  } catch (error) {
    console.error('Error in handleViewLogs:', error)
    message.error(`查看日志失败: ${error}`)
  }
}

const handleDeviceChange = (deviceId: string) => {
  // 设备切换逻辑已在 ConsolePanel 内部处理
  console.log('Device changed to:', deviceId)
}
```

**修改 watch(consoleCollapsed)：**
```typescript
watch(consoleCollapsed, async (collapsed) => {
  try {
    const appWindow = getCurrentWindow()
    const currentSize = await appWindow.innerSize()
    
    if (collapsed) {
      // 控制台折叠时，通过 ConsolePanel 停止 logcat
      await consolePanelRef.value?.stopDeviceLog()
      await appWindow.setSize(new LogicalSize(828, currentSize.height))
    } else {
      const sidebarWidth = sidebarCollapsed.value ? 0 : 200
      const handleWidth = 5
      const minDevicePanelWidth = 50
      const maxConsoleWidth = currentSize.width - sidebarWidth - handleWidth - minDevicePanelWidth
      
      if (consolePanelWidth.value > maxConsoleWidth) {
        consolePanelWidth.value = Math.max(280, maxConsoleWidth)
      }
      
      await appWindow.setSize(new LogicalSize(828 + consolePanelWidth.value, currentSize.height))
    }
  } catch (error) {
    console.error('Failed to resize window:', error)
  }
})
```

**修改 onUnmounted：**
```typescript
onUnmounted(() => {
  window.removeEventListener('focus', handleRefresh)
  consolePanelRef.value?.stopDeviceLog()
})
```

### 4. 删除不需要的导入

删除以下导入：
```typescript
import AnsiToHtml from 'ansi-to-html'
import { NRadioGroup, NRadioButton, NInputNumber, NDatePicker } from 'naive-ui'
```

## 优势

1. **代码组织**：HomeView.vue 从 ~900 行减少到 ~500 行
2. **关注点分离**：日志功能独立，易于维护
3. **可复用性**：ConsolePanel 可在其他地方使用
4. **可测试性**：组件更小，更容易测试
5. **性能**：组件隔离，减少不必要的重渲染

## 文件结构

```
src/
├── components/
│   ├── ConsolePanel.vue       # 新增：控制台面板组件
│   ├── EmulatorCard.vue
│   └── EmulatorList.vue
└── views/
    ├── HomeView.vue           # 简化：使用 ConsolePanel
    └── HomeView.css           # 保持不变
```

## 注意事项

1. ConsolePanel 组件通过 `defineExpose` 暴露了 `startDeviceLog` 和 `stopDeviceLog` 方法
2. 父组件通过 ref 调用这些方法来控制日志
3. 所有日志相关的状态和逻辑都在 ConsolePanel 内部管理
4. 应用日志（consoleLogs）仍在 HomeView 中管理，因为它与其他功能（如截图）相关
