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
    <console-panel
      ref="consolePanelRef"
      v-model:collapsed="consoleCollapsed"
      :panel-width="consolePanelWidth"
      :device-options="runningDeviceOptions"
      :app-logs="consoleLogs"
      @device-change="handleDeviceChange"
      @add-log="handleAddLog"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, nextTick, watch } from 'vue'
import { NButton, NInput, NIcon, useMessage, useDialog } from 'naive-ui'
import { Refresh } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useEmulatorStore } from '@/stores/emulator'
import EmulatorList from '@/components/EmulatorList.vue'
import ConsolePanel from '@/components/ConsolePanel.vue'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'

const { t } = useI18n()
const router = useRouter()
const message = useMessage()
const dialog = useDialog()
const emulatorStore = useEmulatorStore()

const consolePanelRef = ref<InstanceType<typeof ConsolePanel>>()
const isMacOS = ref(false)
const activeTab = ref(localStorage.getItem('activeTab') || 'android')
const searchText = ref('')
const consoleLogs = ref<Array<{type: string, message: string, time: string, path?: string}>>([])
const consoleCollapsed = ref(true)
const consolePanelWidth = ref(500)
const isResizing = ref(false)
const startingEmulators = ref<Set<string>>(new Set())
const stoppingEmulators = ref<Set<string>>(new Set())
const sidebarCollapsed = ref(false)

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

const handleDeviceChange = (deviceId: string) => {
  // 设备切换逻辑已在 ConsolePanel 内部处理
  console.log('Device changed to:', deviceId)
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
    if (Math.abs(e.deltaX) > Math.abs(e.deltaY)) {
      e.preventDefault()
    }
  }
  
  window.addEventListener('wheel', preventNavigation, { passive: false })
  
  onUnmounted(() => {
    window.removeEventListener('wheel', preventNavigation)
  })
})

onUnmounted(() => {
  window.removeEventListener('focus', handleRefresh)
  consolePanelRef.value?.stopDeviceLog()
})

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
    addConsoleLog('info', `正在启动模拟器: ${id}`)
    await emulatorStore.startEmulator(id)
    addConsoleLog('success', `模拟器启动成功: ${id}`)
    message.success(t('messages.startSuccess'))
    
    let retries = 0
    const maxRetries = 30
    const checkStatus = async () => {
      await handleRefresh()
      const emulator = filteredEmulators.value.find(e => e.id === id || e.name === id)
      
      if (emulator?.status === 'running') {
        startingEmulators.value.delete(id)
        addConsoleLog('success', `模拟器 ${id} 已完全启动`)
        return
      }
      
      retries++
      if (retries < maxRetries) {
        if (retries % 5 === 0) {
          addConsoleLog('info', `等待模拟器启动... (${retries * 2}秒)`)
        }
        setTimeout(checkStatus, 2000)
      } else {
        startingEmulators.value.delete(id)
        addConsoleLog('warning', `模拟器 ${id} 启动超时，但进程可能仍在启动中`)
      }
    }
    
    setTimeout(checkStatus, 2000)
  } catch (error) {
    const errorMsg = typeof error === 'string' ? error : (error instanceof Error ? error.message : JSON.stringify(error))
    addConsoleLog('error', `模拟器启动失败: ${id} - ${errorMsg}`)
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
    
    let retries = 0
    const maxRetries = 10
    const checkStatus = async () => {
      await handleRefresh()
      const emulator = filteredEmulators.value.find(e => e.id === id || e.name === id)
      
      if (emulator?.status === 'stopped' || !emulator) {
        stoppingEmulators.value.delete(id)
        return
      }
      
      retries++
      if (retries < maxRetries) {
        setTimeout(checkStatus, 1000)
      } else {
        stoppingEmulators.value.delete(id)
      }
    }
    
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
    await consolePanelRef.value?.startDeviceLog(id)
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

const startResize = (e: MouseEvent) => {
  isResizing.value = true
  const startX = e.clientX
  const startWidth = consolePanelWidth.value

  const handleMouseMove = (e: MouseEvent) => {
    if (!isResizing.value) return
    
    const deltaX = startX - e.clientX
    const newWidth = startWidth + deltaX
    const windowWidth = window.innerWidth
    const sidebarWidth = sidebarCollapsed.value ? 0 : 200
    const handleWidth = 5
    const maxConsoleWidth = windowWidth - sidebarWidth - handleWidth - 50
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
