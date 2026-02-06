<template>
  <div class="home-view">
    <!-- 第一列：平台选择 -->
    <div v-if="!sidebarCollapsed" class="platform-sidebar">
      <div class="sidebar-header">
        <h3>{{ t('app.title') }}</h3>
        <n-button text size="small" @click="sidebarCollapsed = true" class="collapse-btn">
          <img src="@/assets/silder.svg" class="slider-icon" />
        </n-button>
      </div>
      <div class="platform-list">
        <div 
          v-if="isMacOS"
          :class="['platform-item', { active: activeTab === 'ios' }]"
          @click="handleTabChange('ios')"
        >
          <img src="@/assets/iOS.svg" class="platform-icon" />
          <span>{{ t('tabs.ios') }}</span>
        </div>
        <div 
          :class="['platform-item', { active: activeTab === 'android' }]"
          @click="handleTabChange('android')"
        >
          <img src="@/assets/android.svg" class="platform-icon" />
          <span>{{ t('tabs.android') }}</span>
        </div>
        <div 
          :class="['platform-item', { active: activeTab === 'harmony' }]"
          @click="handleTabChange('harmony')"
        >
          <img src="@/assets/phone.svg" class="platform-icon" />
          <span>{{ t('tabs.harmony') }}</span>
        </div>
      </div>
      <div class="sidebar-footer">
        <n-button text @click="router.push('/settings')" class="settings-button">
          <img src="@/assets/settings.svg" class="settings-icon" />
          {{ t('actions.settings') }}
        </n-button>
      </div>
    </div>

    <!-- 第二列：设备列表 -->
    <div class="device-panel">
      <div class="device-header">
        <div 
          class="device-header-left" 
          :class="{ 'sidebar-collapsed': sidebarCollapsed }"
          :style="{ marginLeft: (sidebarCollapsed && isMacOS) ? '70px' : '0' }"
        >
          
          <n-button v-if="sidebarCollapsed" text size="small" @click="sidebarCollapsed = false" class="expand-btn">
            <img src="@/assets/silder.svg" class="slider-icon slider-icon-expand" />
          </n-button>
          <h4>{{ t('tabs.' + activeTab) }}</h4>
        </div>
        <div class="device-actions">
          <n-input
            v-model:value="searchText"
            :placeholder="t('actions.search')"
            clearable
            size="small"
            style="width: 180px"
          />
          <n-button @click="handleRefresh" size="small" quaternary circle>
            <template #icon>
              <n-icon :component="Refresh" />
            </template>
          </n-button>
          <n-button v-if="consoleCollapsed" @click="consoleCollapsed = false" size="small" quaternary circle>
            <img src="@/assets/console.svg" class="console-icon" />
          </n-button>
        </div>
      </div>
      <div class="device-content">
        <emulator-list
          :emulators="filteredEmulators"
          :loading="emulatorStore.loading"
          :search-text="searchText"
          :starting-emulators="startingEmulators"
          :stopping-emulators="stoppingEmulators"
          @start="handleStart"
          @stop="handleStop"
          @delete="handleDelete"
          @wipe-data="handleWipeData"
          @screenshot="handleScreenshot"
          @view-logs="handleViewLogs"
          @copy-id="handleCopyId"
          @edit="handleEdit"
        />
      </div>
    </div>

    <!-- 拖动分隔条 -->
    <div 
      v-if="!consoleCollapsed"
      class="resize-handle"
      @mousedown="startResize"
    ></div>

    <!-- 第三列：控制台 -->
    <!-- 应用日志面板 -->
    <app-log-panel
      v-if="consoleTab === 'app'"
      v-model:collapsed="consoleCollapsed"
      :panel-width="consolePanelWidth"
      :logs="consoleLogs"
      @open-screenshot="openScreenshot"
      @copy-path="copyPathToClipboard"
      @copy-image="copyImageToClipboard"
      @switch-to-device="handleDeviceLogTab"
    />

    <!-- 设备日志面板 -->
    <device-log-panel
      v-if="consoleTab === 'device'"
      ref="deviceLogPanelRef"
      v-model:collapsed="consoleCollapsed"
      v-model:selected-device="selectedLogDevice"
      v-model:show-keyword-filter="showKeywordFilter"
      v-model:time-filter-type="timeFilterType"
      v-model:recent-minutes="recentMinutes"
      v-model:since-time="sinceTime"
      :panel-width="consolePanelWidth"
      :device-options="runningDeviceOptions"
      :filter="logFilter"
      @update-filter="updateLogFilter"
      @switch-to-app="consoleTab = 'app'"
      @log-error="addConsoleLog('error', $event)"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { NButton, NInput, NIcon, useMessage, useDialog } from 'naive-ui'
import { Refresh } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useEmulatorStore } from '@/stores/emulator'
import { useLogsStore } from '@/stores/logs'
import EmulatorList from '@/components/EmulatorList.vue'
import AppLogPanel from '@/components/AppLogPanel.vue'
import DeviceLogPanel from '@/components/DeviceLogPanel.vue'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'

const { t } = useI18n()
const router = useRouter()
const message = useMessage()
const dialog = useDialog()
const emulatorStore = useEmulatorStore()
const logsStore = useLogsStore()

const isMacOS = ref(false)
const activeTab = ref(localStorage.getItem('activeTab') || 'android')
const searchText = ref('')
const consoleCollapsed = ref(true)
const consoleTab = ref<'app' | 'device'>('app')
const consolePanelWidth = ref(500)
const isResizing = ref(false)
const startingEmulators = ref<Set<string>>(new Set())
const stoppingEmulators = ref<Set<string>>(new Set())
const sidebarCollapsed = ref(false)
const selectedLogDevice = ref<string>('')
const showKeywordFilter = ref(false)
const timeFilterType = ref<'all' | 'recent' | 'since'>('all')
const recentMinutes = ref(5)
const sinceTime = ref<number | null>(null)
const logFilter = ref({
  level: 'all',
  keyword: '',
  packageName: ''
})

// 从全局日志 store 获取应用日志
const consoleLogs = computed(() => {
  return logsStore.getLogsBySource('app').map(log => ({
    type: log.type,
    message: log.message,
    time: new Date(log.timestamp).toLocaleTimeString(),
    path: undefined // 如果需要路径，可以扩展 LogEntry
  }))
})
const deviceLogPanelRef = ref<any>(null)

const filteredEmulators = computed(() => {
  const emulators = emulatorStore.emulators.filter(emulator => emulator.type === activeTab.value)
  return emulators.sort((a, b) => {
    if (a.status === 'running' && b.status !== 'running') return -1
    if (a.status !== 'running' && b.status === 'running') return 1
    return 0
  })
})

const runningDeviceOptions = computed(() => {
  return filteredEmulators.value
    .filter(e => e.status === 'running')
    .map(e => ({
      label: e.name,
      value: e.id
    }))
})

const updateLogFilter = (key: string, value: string) => {
  logFilter.value[key as keyof typeof logFilter.value] = value
}

const addConsoleLog = (type: string, message: string, path?: string) => {
  logsStore.addLog(type as any, message, 'app')
  
  // 只在错误时自动展开控制台
  if (type === 'error') {
    consoleCollapsed.value = false
  }
}

const handleDeviceLogTab = async () => {
  consoleTab.value = 'device'
  // 如果有运行中的模拟器，开始收集日志
  const runningEmulator = filteredEmulators.value.find(e => e.status === 'running')
  if (runningEmulator && deviceLogPanelRef.value) {
    selectedLogDevice.value = runningEmulator.id
  }
}

onMounted(async () => {
  const userAgent = navigator.userAgent.toLowerCase()
  isMacOS.value = userAgent.includes('mac')
  if (!isMacOS.value && activeTab.value === 'ios') {
    activeTab.value = 'android'
  }
  await handleRefresh()
  window.addEventListener('focus', handleRefresh)
  
  // 禁用触摸板滑动导航
  const preventNavigation = (e: WheelEvent) => {
    // 检测是否是水平滑动
    if (Math.abs(e.deltaX) > Math.abs(e.deltaY)) {
      e.preventDefault()
    }
  }
  
  window.addEventListener('wheel', preventNavigation, { passive: false })
  
  // 清理函数
  onUnmounted(() => {
    window.removeEventListener('wheel', preventNavigation)
  })
})

onUnmounted(() => {
  window.removeEventListener('focus', handleRefresh)
})

watch(consoleCollapsed, async (collapsed) => {
  try {
    const appWindow = getCurrentWindow()
    const currentSize = await appWindow.innerSize()
    
    if (collapsed) {
      await appWindow.setSize(new LogicalSize(828, currentSize.height))
    } else {
      // 确保控制台宽度不会导致第二列小于 50px
      const sidebarWidth = sidebarCollapsed.value ? 0 : 200
      const handleWidth = 5
      const minDevicePanelWidth = 50
      const maxConsoleWidth = currentSize.width - sidebarWidth - handleWidth - minDevicePanelWidth
      
      // 如果当前控制台宽度太大，调整它
      if (consolePanelWidth.value > maxConsoleWidth) {
        consolePanelWidth.value = Math.max(280, maxConsoleWidth)
      }
      
      await appWindow.setSize(new LogicalSize(828 + consolePanelWidth.value, currentSize.height))
    }
  } catch (error) {
    console.error('Failed to resize window:', error)
  }
})

// 监听控制台宽度变化，同步调整窗口大小
watch(consolePanelWidth, async (newWidth) => {
  if (!consoleCollapsed.value) {
    try {
      const appWindow = getCurrentWindow()
      const currentSize = await appWindow.innerSize()
      await appWindow.setSize(new LogicalSize(828 + newWidth, currentSize.height))
    } catch (error) {
      console.error('Failed to resize window:', error)
    }
  }
})

const handleTabChange = async (tab: string) => {
  activeTab.value = tab
  localStorage.setItem('activeTab', tab)
  await emulatorStore.fetchEmulators(tab as any)
}

const handleRefresh = async () => {
  await emulatorStore.fetchEmulators(activeTab.value as any)
}

const handleStart = async (id: string) => {
  try {
    startingEmulators.value.add(id)
    logsStore.addLog('info', `正在启动模拟器: ${id}`, 'app')
    await emulatorStore.startEmulator(id)
    logsStore.addLog('success', `模拟器启动成功: ${id}`, 'app')
    message.success(t('messages.startSuccess'))
    
    // 启动后轮询检查状态，确保UI更新
    let retries = 0
    const maxRetries = 30 // 增加到30次，每次2秒，总共60秒
    const checkStatus = async () => {
      await handleRefresh()
      const emulator = filteredEmulators.value.find(e => e.id === id || e.name === id)
      
      if (emulator?.status === 'running') {
        // 状态已更新为运行中
        startingEmulators.value.delete(id)
        logsStore.addLog('success', `模拟器 ${id} 已完全启动`, 'app')
        return
      }
      
      retries++
      if (retries < maxRetries) {
        // 继续轮询，每2秒检查一次
        if (retries % 5 === 0) {
          // 每10秒输出一次进度
          logsStore.addLog('info', `等待模拟器启动... (${retries * 2}秒)`, 'app')
        }
        setTimeout(checkStatus, 2000)
      } else {
        // 超时，停止轮询
        startingEmulators.value.delete(id)
        logsStore.addLog('warning', `模拟器 ${id} 启动超时，但进程可能仍在启动中`, 'app')
      }
    }
    
    // 延迟2秒后开始检查
    setTimeout(checkStatus, 2000)
  } catch (error) {
    const errorMsg = typeof error === 'string' ? error : (error instanceof Error ? error.message : JSON.stringify(error))
    logsStore.addLog('error', `模拟器启动失败: ${id} - ${errorMsg}`, 'app')
    console.error('Start emulator error:', error)
    message.error(errorMsg)
    startingEmulators.value.delete(id)
  }
}

const handleStop = async (id: string) => {
  try {
    stoppingEmulators.value.add(id)
    await emulatorStore.stopEmulator(id)
    message.success(t('messages.stopSuccess'))
    
    // 停止后轮询检查状态，确保UI更新
    let retries = 0
    const maxRetries = 10
    const checkStatus = async () => {
      await handleRefresh()
      const emulator = filteredEmulators.value.find(e => e.id === id || e.name === id)
      
      if (emulator?.status === 'stopped' || !emulator) {
        // 状态已更新为停止或模拟器不存在
        stoppingEmulators.value.delete(id)
        return
      }
      
      retries++
      if (retries < maxRetries) {
        // 继续轮询
        setTimeout(checkStatus, 1000)
      } else {
        // 超时，停止轮询
        stoppingEmulators.value.delete(id)
      }
    }
    
    // 延迟1秒后开始检查
    setTimeout(checkStatus, 1000)
  } catch (error) {
    const errorMsg = typeof error === 'string' ? error : (error instanceof Error ? error.message : JSON.stringify(error))
    addConsoleLog('error', `模拟器关闭失败: ${id} - ${errorMsg}`)
    console.error('Stop emulator error:', error)
    message.error(errorMsg)
    stoppingEmulators.value.delete(id)
  }
}

const handleDelete = async (id: string) => {
  dialog.warning({
    title: t('dialogs.deleteTitle'),
    content: t('dialogs.deleteContent', { id }),
    positiveText: t('actions.delete'),
    negativeText: t('settings.cancel'),
    onPositiveClick: async () => {
      try {
        await emulatorStore.deleteEmulator(id)
        message.success(t('messages.deleteSuccess'))
        await handleRefresh()
      } catch (error) {
        const errorMsg = typeof error === 'string' ? error : (error instanceof Error ? error.message : JSON.stringify(error))
        message.error(errorMsg)
      }
    }
  })
}

const handleWipeData = async (id: string) => {
  try {
    await emulatorStore.wipeData(id)
    message.success(t('messages.wipeDataSuccess'))
  } catch (error) {
    message.error(t('messages.error'))
  }
}

const handleScreenshot = async (id: string) => {
  try {
    const path = await emulatorStore.takeScreenshot(id)
    message.success(t('messages.screenshotSaved', { path }))
    // 将截图路径输出到控制台，标记为可点击的链接
    addConsoleLog('screenshot', `截图已保存`, path)
    consoleCollapsed.value = false
  } catch (error) {
    const errorMsg = typeof error === 'string' ? error : (error instanceof Error ? error.message : JSON.stringify(error))
    message.error(errorMsg)
    addConsoleLog('error', `截图失败: ${errorMsg}`)
  }
}

const handleViewLogs = async (id: string) => {
  try {
    consoleCollapsed.value = false
    consoleTab.value = 'device'
    selectedLogDevice.value = id
    addConsoleLog('info', `正在查看设备 ${id} 的日志`)
  } catch (error) {
    console.error('Error in handleViewLogs:', error)
    message.error(`查看日志失败: ${error}`)
  }
}

const handleCopyId = (id: string) => {
  navigator.clipboard.writeText(id)
  message.success(t('messages.copySuccess'))
}

const handleEdit = (id: string, type: string) => {
  router.push({
    name: 'emulator-edit',
    params: { id },
    query: { type }
  })
}

const openScreenshot = async (path: string) => {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('open_file', { path })
  } catch (error) {
    console.error('Failed to open screenshot:', error)
    // 如果打开失败，尝试复制路径到剪贴板
    try {
      await navigator.clipboard.writeText(path)
      message.info('无法打开图片，路径已复制到剪贴板')
    } catch {
      message.error('无法打开图片')
    }
  }
}

const copyPathToClipboard = async (path: string) => {
  try {
    await navigator.clipboard.writeText(path)
    message.success('路径已复制到剪贴板')
  } catch (error) {
    console.error('Failed to copy path:', error)
    message.error('复制失败')
  }
}

const copyImageToClipboard = async (path: string) => {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('copy_image_to_clipboard', { path })
    message.success('图片已复制到剪贴板')
  } catch (error) {
    console.error('Failed to copy image:', error)
    const errorMsg = typeof error === 'string' ? error : (error instanceof Error ? error.message : '复制失败')
    message.error(errorMsg)
  }
}

const startResize = (e: MouseEvent) => {
  isResizing.value = true
  const startX = e.clientX
  const startWidth = consolePanelWidth.value

  const handleMouseMove = (e: MouseEvent) => {
    if (!isResizing.value) return
    
    // 计算新宽度（从右边拖动，所以是减法）
    const deltaX = startX - e.clientX
    const newWidth = startWidth + deltaX
    
    // 获取窗口总宽度
    const windowWidth = window.innerWidth
    
    // 计算侧边栏宽度（如果展开）
    const sidebarWidth = sidebarCollapsed.value ? 0 : 200
    
    // 计算拖动手柄宽度
    const handleWidth = 5
    
    // 计算第二列可用的最大宽度（确保至少保留 50px）
    const maxConsoleWidth = windowWidth - sidebarWidth - handleWidth - 50
    
    // 限制最小和最大宽度
    const minWidth = 280
    const maxWidth = Math.min(1200, maxConsoleWidth)
    
    if (newWidth >= minWidth && newWidth <= maxWidth) {
      consolePanelWidth.value = newWidth
    }
  }

  const handleMouseUp = () => {
    isResizing.value = false
    document.removeEventListener('mousemove', handleMouseMove)
    document.removeEventListener('mouseup', handleMouseUp)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
  }

  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
  document.body.style.cursor = 'ew-resize'
  document.body.style.userSelect = 'none'
}
</script>

<style scoped src="./HomeView.css"></style>
