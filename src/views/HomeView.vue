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
        <div class="device-header-left" :class="{ 'sidebar-collapsed': sidebarCollapsed }">
          <h4>{{ t('tabs.' + activeTab) }}</h4>
          <n-button v-if="sidebarCollapsed" text size="small" @click="sidebarCollapsed = false" class="expand-btn">
            <img src="@/assets/silder.svg" class="slider-icon slider-icon-expand" />
          </n-button>
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
          <n-button @click="consoleCollapsed = !consoleCollapsed" size="small" quaternary circle>
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

    <!-- 第三列：控制台 -->
    <transition name="console-slide">
      <div v-if="!consoleCollapsed" class="console-panel">
        <div class="console-header">
          <span>控制台</span>
          <div class="console-header-actions">
            <n-button text size="small" @click="clearConsole">清空</n-button>
            <n-button text size="small" @click="consoleCollapsed = true">
              <template #icon>
                <n-icon>✕</n-icon>
              </template>
            </n-button>
          </div>
        </div>
        <div class="console-content" ref="consoleRef">
          <div v-for="(log, index) in consoleLogs" :key="index" :class="['console-log', log.type]">
            <span class="console-time">{{ log.time }}</span>
            <span class="console-message">{{ log.message }}</span>
          </div>
        </div>
      </div>
    </transition>
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
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'

const { t } = useI18n()
const router = useRouter()
const message = useMessage()
const dialog = useDialog()
const emulatorStore = useEmulatorStore()

const isMacOS = ref(false)
const activeTab = ref(localStorage.getItem('activeTab') || 'android')
const searchText = ref('')
const consoleLogs = ref<Array<{type: string, message: string, time: string}>>([])
const consoleRef = ref<HTMLElement>()
const consoleCollapsed = ref(true)
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

const addConsoleLog = (type: string, message: string) => {
  const time = new Date().toLocaleTimeString()
  consoleLogs.value.push({ type, message, time })
  // 只在错误时自动展开控制台
  if (type === 'error') {
    consoleCollapsed.value = false
  }
  nextTick(() => {
    if (consoleRef.value) {
      consoleRef.value.scrollTop = consoleRef.value.scrollHeight
    }
  })
}

const clearConsole = () => {
  consoleLogs.value = []
}

onMounted(async () => {
  const userAgent = navigator.userAgent.toLowerCase()
  isMacOS.value = userAgent.includes('mac')
  if (!isMacOS.value && activeTab.value === 'ios') {
    activeTab.value = 'android'
  }
  await handleRefresh()
  window.addEventListener('focus', handleRefresh)
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
      await appWindow.setSize(new LogicalSize(1148, currentSize.height))
    }
  } catch (error) {
    console.error('Failed to resize window:', error)
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
    await handleRefresh()
  } catch (error) {
    const errorMsg = typeof error === 'string' ? error : (error instanceof Error ? error.message : JSON.stringify(error))
    addConsoleLog('error', `模拟器启动失败: ${id} - ${errorMsg}`)
    console.error('Start emulator error:', error)
    message.error(errorMsg)
  } finally {
    startingEmulators.value.delete(id)
  }
}

const handleStop = async (id: string) => {
  try {
    stoppingEmulators.value.add(id)
    await emulatorStore.stopEmulator(id)
    message.success(t('messages.stopSuccess'))
    await handleRefresh()
  } catch (error) {
    const errorMsg = typeof error === 'string' ? error : (error instanceof Error ? error.message : JSON.stringify(error))
    addConsoleLog('error', `模拟器关闭失败: ${id} - ${errorMsg}`)
    console.error('Stop emulator error:', error)
    message.error(errorMsg)
  } finally {
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
  } catch (error) {
    const errorMsg = typeof error === 'string' ? error : (error instanceof Error ? error.message : JSON.stringify(error))
    message.error(errorMsg)
  }
}

const handleViewLogs = (id: string) => {
  consoleCollapsed.value = false
  addConsoleLog('info', `查看模拟器日志: ${id}`)
}

const handleCopyId = (id: string) => {
  navigator.clipboard.writeText(id)
  message.success(t('messages.copySuccess'))
}
</script>

<style scoped>
.home-view {
  display: flex;
  height: 100vh;
  overflow: hidden;
  position: relative;
}

/* 第一列：平台侧边栏 */
.platform-sidebar {
  width: 200px;
  background: #f8f9fa;
  border-right: 1px solid #e0e0e0;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.sidebar-header {
  padding: 12px 20px 0 20px;
  border-bottom: 1px solid #e0e0e0;
  display: flex;
  justify-content: space-between;
  height: 61px;
}

.sidebar-header h3 {
  padding-top: 13px;
  font-size: 16px;
  font-weight: 600;
}

.collapse-btn {
  color: #666;
}

.slider-icon {
  width: 24px;
  height: 24px;
  color: #666;
}

.slider-icon-expand {
  transform: rotate(180deg);
}

.platform-list {
  flex: 1;
  padding: 8px;
  overflow-y: auto;
}

.platform-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  margin-bottom: 4px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 14px;
}

.platform-item:hover {
  background: rgba(0, 0, 0, 0.05);
}

.platform-item.active {
  background: #E0E0E0;
  color: #424242;
  font-weight: 500;
}

.platform-icon {
  width: 20px;
  height: 20px;
}

.sidebar-footer {
  padding: 12px;
  border-top: 1px solid #e0e0e0;
}

.settings-button {
  justify-content: flex-start;
  width: 100%;
}

.settings-icon {
  width: 21px;
  height: 21px;
  margin-right: 8px;
}

/* 第二列：设备面板 */
.device-panel {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.device-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #e0e0e0;
  height: 61px;
  padding: 12px 20px 0 20px;
}

.device-header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  transition: margin-left 0.3s ease;
}

.device-header-left.sidebar-collapsed {
  margin-left: 70px;
}

.expand-btn {
  color: #666;
}

.device-header h4 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  padding-top: 3px;
}

.device-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.console-icon {
  width: 22px;
  height: 22px;
}

.device-content {
  flex: 1;
  overflow: hidden;
}

/* 第三列：控制台 */
.console-panel {
  width: 320px;
  border-left: 1px solid #e0e0e0;
  display: flex;
  flex-direction: column;
  background: #f8f9fa;
  flex-shrink: 0;
  position: fixed;
  right: 0;
  top: 0;
  bottom: 0;
  z-index: 1000;
  box-shadow: -2px 0 8px rgba(0, 0, 0, 0.1);
}

.console-slide-enter-active,
.console-slide-leave-active {
  transition: transform 0.3s ease;
}

.console-slide-enter-from,
.console-slide-leave-to {
  transform: translateX(100%);
}

.console-slide-enter-to,
.console-slide-leave-from {
  transform: translateX(0);
}

.console-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid #e0e0e0;
  font-size: 14px;
  font-weight: 500;
  height: 61px;
}

.console-header-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.console-content {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 12px;
  color: #333;
}

.console-log {
  margin-bottom: 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.console-time {
  color: #888;
  font-size: 11px;
}

.console-message {
  word-break: break-word;
}

.console-log.error .console-message {
  color: #d32f2f;
}

.console-log.success .console-message {
  color: #388e3c;
}

.console-log.info .console-message {
  color: #1976d2;
}
</style>
