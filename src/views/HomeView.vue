<template>
  <div class="home-view">
    <div class="header">
      <div class="logo">
        <h3>{{ t('app.title') }}</h3>
      </div>
      <div class="header-tabs">
        <n-tabs v-model:value="activeTab" type="card" @update:value="handleTabChange">
          <n-tab-pane v-if="isMacOS" name="ios">
            <template #tab>
              <div class="tab-content">
                <img src="@/assets/iOS.svg" class="tab-icon" />
                {{ t('tabs.ios') }}
              </div>
            </template>
          </n-tab-pane>
          <n-tab-pane name="android">
            <template #tab>
              <div class="tab-content">
                <img src="@/assets/android.svg" class="tab-icon" />
                {{ t('tabs.android') }}
              </div>
            </template>
          </n-tab-pane>
          <n-tab-pane name="harmony" :tab="t('tabs.harmony')" />
        </n-tabs>
      </div>
      <div class="header-actions">
        <n-button text @click="router.push('/settings')">
          <template #icon>
            <n-icon :component="Settings" />
          </template>
          {{ t('actions.settings') }}
        </n-button>
      </div>
    </div>

    <div class="toolbar">
      <div class="toolbar-actions">
        <n-input
          v-model:value="searchText"
          :placeholder="t('actions.search')"
          clearable
          style="width: 200px"
        />
        <n-button @click="handleRefresh">
          <template #icon>
            <n-icon :component="Refresh" />
          </template>
          {{ t('actions.refresh') }}
        </n-button>
      </div>
    </div>

    <div class="content-wrapper">
      <emulator-list
        :emulators="filteredEmulators"
        :loading="emulatorStore.loading"
        :search-text="searchText"
        @start="handleStart"
        @stop="handleStop"
        @delete="handleDelete"
        @wipe-data="handleWipeData"
        @screenshot="handleScreenshot"
        @view-logs="handleViewLogs"
        @copy-id="handleCopyId"
      />
    </div>

    <div class="console-panel" :class="{ collapsed: consoleCollapsed }">
      <div class="console-header" @click="consoleCollapsed = !consoleCollapsed">
        <span>控制台</span>
        <div class="console-header-actions">
          <n-button text size="small" @click.stop="clearConsole">清空</n-button>
          <span class="collapse-icon">{{ consoleCollapsed ? '▲' : '▼' }}</span>
        </div>
      </div>
      <div v-show="!consoleCollapsed" class="console-content" ref="consoleRef">
        <div v-for="(log, index) in consoleLogs" :key="index" :class="['console-log', log.type]">
          <span class="console-time">{{ log.time }}</span>
          <span class="console-message">{{ log.message }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, nextTick } from 'vue'
import { NTabs, NTabPane, NButton, NInput, NIcon, useMessage } from 'naive-ui'
import { Settings, Refresh } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useEmulatorStore } from '@/stores/emulator'
import EmulatorList from '@/components/EmulatorList.vue'

const { t } = useI18n()
const router = useRouter()
const message = useMessage()
const emulatorStore = useEmulatorStore()

const isMacOS = ref(false)
const activeTab = ref(localStorage.getItem('activeTab') || 'android')
const searchText = ref('')
const consoleLogs = ref<Array<{type: string, message: string, time: string}>>([])
const consoleRef = ref<HTMLElement>()
const consoleCollapsed = ref(true)

const filteredEmulators = computed(() => {
  return emulatorStore.emulators.filter(emulator => emulator.type === activeTab.value)
})

const addConsoleLog = (type: string, message: string) => {
  const time = new Date().toLocaleTimeString()
  consoleLogs.value.push({ type, message, time })
  // 只在错误时自动展开控制台
  if (type === 'error' && consoleCollapsed.value) {
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
  // Check platform
  isMacOS.value = navigator.platform.toLowerCase().includes('mac')
  if (!isMacOS.value && activeTab.value === 'ios') {
    activeTab.value = 'android'
  }
  await handleRefresh()
})

const handleTabChange = async (tab: string) => {
  localStorage.setItem('activeTab', tab)
  await emulatorStore.fetchEmulators(tab as any)
}

const handleRefresh = async () => {
  await emulatorStore.fetchEmulators(activeTab.value as any)
}

const handleStart = async (id: string) => {
  try {
    addConsoleLog('info', `正在启动模拟器: ${id}`)
    await emulatorStore.startEmulator(id)
    addConsoleLog('success', `模拟器启动成功: ${id}`)
    message.success(t('messages.startSuccess'))
    // 等待模拟器被adb检测到后再刷新
    setTimeout(async () => {
      await handleRefresh()
    }, 10000)
  } catch (error) {
    const errorMsg = typeof error === 'string' ? error : (error instanceof Error ? error.message : JSON.stringify(error))
    addConsoleLog('error', `模拟器启动失败: ${id} - ${errorMsg}`)
    console.error('Start emulator error:', error)
    message.error(errorMsg)
  }
}

const handleStop = async (id: string) => {
  try {
    await emulatorStore.stopEmulator(id)
    message.success(t('messages.stopSuccess'))
    // 刷新列表以更新状态
    await handleRefresh()
  } catch (error) {
    const errorMsg = typeof error === 'string' ? error : (error instanceof Error ? error.message : JSON.stringify(error))
    addConsoleLog('error', `模拟器关闭失败: ${id} - ${errorMsg}`)
    console.error('Stop emulator error:', error)
    message.error(errorMsg)
  }
}

const handleDelete = async (id: string) => {
  try {
    await emulatorStore.deleteEmulator(id)
    message.success(t('messages.deleteSuccess'))
    await handleRefresh()
  } catch (error) {
    message.error(t('messages.error'))
  }
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
    await emulatorStore.takeScreenshot(id)
    message.success(t('messages.screenshotSuccess'))
  } catch (error) {
    message.error(t('messages.error'))
  }
}

const handleViewLogs = (id: string) => {
  // TODO: Open log viewer dialog
  console.log('View logs:', id)
}

const handleCopyId = (id: string) => {
  navigator.clipboard.writeText(id)
  message.success(t('messages.copySuccess'))
}
</script>

<style scoped>
.home-view {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.header {
  display: flex;
  flex-direction: row;
  justify-content: center;
  align-items: center;
  padding: 16px 24px;
  /* border-bottom: 1px solid #e0e0e0; */
}

.logo h1 {
  font-size: 20px;
  font-weight: 600;
}

.header-tabs {
  flex: 1;
  display: flex;
  justify-content: flex-end;
  align-items: center;
  padding-right: 20px;
}

.header-tabs :deep(.n-tabs) {
  width: auto;
  height: 36px;
  display: flex;
  align-items: center;
}

.header-tabs :deep(.n-tabs-nav) {
  border-bottom: none !important;
  background: #f0f0f0;
  border-radius: 20px;
  padding: 3px;
  width: auto;
  display: inline-flex;
  margin: 0;
  height: 36px;
  align-items: center;
}

.header-tabs :deep(.n-tabs-nav-scroll-content) {
  display: flex;
  align-items: center;
}

.header-tabs :deep(.n-tabs-tab) {
  border: none !important;
  background: transparent !important;
  border-radius: 16px !important;
  margin: 0 1px;
  transition: all 0.2s ease;
  height: 30px;
  line-height: 30px;
  padding: 0 16px;
}

.header-tabs :deep(.n-tabs-tab--active) {
  background: white !important;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.header-tabs :deep(.n-tabs-tab--active .n-tabs-tab__label) {
  color: #18a058 !important;
}

.tab-content {
  display: flex;
  align-items: center;
  gap: 6px;
}

.tab-icon {
  width: 16px;
  height: 16px;
}

.toolbar {
  display: flex;
  justify-content: flex-start;
  align-items: center;
  padding: 0px 24px;
  /* border-bottom: 1px solid #e0e0e0; */
}

.toolbar-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}

.content-wrapper {
  flex: 1;
  overflow: hidden;
}

.console-panel {
  border-top: 1px solid #e0e0e0;
  display: flex;
  flex-direction: column;
  transition: height 0.3s ease;
  flex-shrink: 0;
  margin-top: auto;
}

.console-panel.collapsed {
  height: auto;
}

.console-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  background: #f5f5f5;
  /* border-bottom: 1px solid #e0e0e0; */
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  user-select: none;
}

.console-header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.collapse-icon {
  font-size: 12px;
  color: #666;
}

.console-content {
  height: 160px;
  overflow-y: auto;
  padding: 8px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 12px;
  background: #f5f5f5;
  color: #333;
}

.console-log {
  margin-bottom: 4px;
  display: flex;
  gap: 8px;
}

.console-time {
  color: #888;
  min-width: 80px;
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
