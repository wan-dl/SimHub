<template>
  <transition name="console-slide">
    <div v-if="!collapsed" class="console-panel" :style="{ width: panelWidth + 'px' }">
      <div class="console-header">
        <div class="console-header-left">
          <n-button text size="small" @click="$emit('update:collapsed', true)" class="collapse-btn">
            <img src="@/assets/silder.svg" class="slider-icon" />
          </n-button>
          <span class="console-title">
            {{ consoleTab === 'app' ? '控制台' : '设备日志' }}
          </span>
          <!-- 设备选择下拉列表 -->
          <n-select
            v-if="consoleTab === 'device'"
            v-model:value="selectedDevice"
            size="small"
            :options="deviceOptions"
            :placeholder="deviceOptions.length > 0 ? '选择设备' : '无运行中的设备'"
            :disabled="deviceOptions.length === 0"
            @update:value="handleDeviceChange"
            style="width: 200px; margin-left: 12px"
          />
        </div>
      </div>
      
      <!-- 设备日志过滤器 -->
      <div v-if="consoleTab === 'device'" class="console-filters">
        <n-select
          v-model:value="logFilter.level"
          size="small"
          :options="logLevelOptions"
          style="width: 100px"
        />
        <n-select
          v-model:value="logFilter.packageName"
          size="small"
          :options="packageList"
          filterable
          placeholder="包名过滤"
          clearable
          :render-label="renderPackageLabel"
          style="flex: 1; max-width: 250px"
        />
        <n-input
          v-if="showKeywordFilter"
          v-model:value="logFilter.keyword"
          size="small"
          placeholder="关键字过滤"
          clearable
          style="flex: 1; max-width: 200px"
        />
        <n-button 
          text 
          size="small" 
          @click="showKeywordFilter = !showKeywordFilter" 
          :title="showKeywordFilter ? '隐藏关键字过滤' : '显示关键字过滤'" 
          class="filter-action-btn"
          :class="{ active: showKeywordFilter }"
        >
          <img src="@/assets/search.svg" class="action-icon" />
        </n-button>
        <n-button 
          text 
          size="small" 
          @click="toggleLogOutput" 
          :title="logOutputPaused ? '继续输出' : '暂停输出'" 
          class="filter-action-btn"
          :class="{ active: logOutputPaused }"
        >
          <img :src="logOutputPaused ? '/src/assets/play.svg' : '/src/assets/pause.svg'" class="action-icon" />
        </n-button>
        <n-button text size="small" @click="exportLogs" title="导出日志" class="filter-action-btn">
          <img src="@/assets/export.svg" class="action-icon" />
        </n-button>
        <n-button text size="small" @click="clearLogs" title="清空日志" class="filter-action-btn">
          <img src="@/assets/clear.svg" class="action-icon" />
        </n-button>
      </div>
      
      <!-- 时间过滤器 -->
      <div v-if="consoleTab === 'device'" class="console-time-filters">
        <img src="@/assets/time.svg" class="time-icon" />
        <n-radio-group v-model:value="timeFilterType" size="small">
          <n-radio-button value="all">全部</n-radio-button>
          <n-radio-button value="recent">最近</n-radio-button>
          <n-radio-button value="since">时间点之后</n-radio-button>
        </n-radio-group>
        <n-input-number
          v-if="timeFilterType === 'recent'"
          v-model:value="recentMinutes"
          size="small"
          :min="1"
          :max="60"
          style="width: 80px"
        />
        <span v-if="timeFilterType === 'recent'" class="time-unit">分钟</span>
        <n-date-picker
          v-if="timeFilterType === 'since'"
          v-model:value="sinceTime"
          type="datetime"
          size="small"
          clearable
          placeholder="选择时间"
          style="width: 200px"
          :is-date-disabled="(ts: number) => ts > Date.now()"
        />
      </div>

      <div class="console-body">
        <div class="console-content" ref="consoleRef">
          <div v-if="consoleTab === 'app'" class="console-logs">
            <div v-for="(log, index) in appLogs" :key="index" :class="['console-log', log.type]">
              <span class="console-time">{{ log.time }}</span>
              <span class="console-message">{{ log.message }}</span>
              <div v-if="log.path" class="console-path-container">
                <span class="console-path" @click="openScreenshot(log.path)">
                  {{ log.path }}
                </span>
                <n-button 
                  text 
                  size="tiny" 
                  @click="copyPathToClipboard(log.path)"
                  class="copy-path-btn"
                  title="复制路径"
                >
                  <img src="@/assets/copy.svg" class="btn-icon" />
                </n-button>
                <n-button 
                  text 
                  size="tiny" 
                  @click="copyImageToClipboard(log.path)"
                  class="copy-image-btn"
                  title="复制图片"
                >
                  <img src="@/assets/image.svg" class="btn-icon" />
                </n-button>
              </div>
            </div>
          </div>
          <div v-else-if="consoleTab === 'device'" class="console-logs">
            <div v-for="(log, index) in filteredDeviceLogs" :key="index" class="console-log device">
              <span class="console-message" v-html="log.html"></span>
            </div>
            <div v-if="filteredDeviceLogs.length === 0 && deviceLogs.length === 0" class="console-empty">
              <span>暂无设备日志</span>
              <span class="console-hint">启动模拟器后将显示 logcat 日志</span>
            </div>
            <div v-else-if="filteredDeviceLogs.length === 0 && deviceLogs.length > 0" class="console-empty">
              <span>没有匹配的日志</span>
              <span class="console-hint">尝试调整过滤条件</span>
            </div>
          </div>
        </div>
        <div class="console-tabs">
          <div 
            :class="['console-tab', { active: consoleTab === 'app' }]"
            @click="consoleTab = 'app'"
            title="程序输出"
          >
            <img src="@/assets/app-log.svg" class="tab-icon" />
          </div>
          <div 
            :class="['console-tab', { active: consoleTab === 'device' }]"
            @click="handleDeviceLogTab"
            title="设备日志"
          >
            <img src="@/assets/device-log.svg" class="tab-icon" />
          </div>
        </div>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch, h } from 'vue'
import { NButton, NInput, NSelect, NRadioGroup, NRadioButton, NInputNumber, NDatePicker, useMessage } from 'naive-ui'
import AnsiToHtml from 'ansi-to-html'

const ansiConverter = new AnsiToHtml({
  fg: '#333',
  bg: '#f8f9fa',
  newline: false,
  escapeXML: true,
  stream: false
})

interface Props {
  collapsed: boolean
  panelWidth: number
  deviceOptions: Array<{label: string, value: string}>
  appLogs: Array<{type: string, message: string, time: string, path?: string}>
}

interface Emits {
  (e: 'update:collapsed', value: boolean): void
  (e: 'device-change', deviceId: string): void
  (e: 'add-log', type: string, message: string, path?: string): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const message = useMessage()

const consoleRef = ref<HTMLElement>()
const consoleTab = ref<'app' | 'device'>('app')
const selectedDevice = ref<string>('')
const showKeywordFilter = ref(false)
const logOutputPaused = ref(false)
const packageList = ref<Array<{label: string, value: string, isSystem: boolean}>>([])
const timeFilterType = ref<'all' | 'recent' | 'since'>('all')
const recentMinutes = ref(5)
const sinceTime = ref<number | null>(null)
const deviceLogs = ref<Array<{message: string, timestamp?: number}>>([])
const logcatProcess = ref<any>(null)

const logFilter = ref({
  level: 'all',
  keyword: '',
  packageName: ''
})

const logLevelOptions = [
  { label: '全部', value: 'all' },
  { label: 'Verbose', value: 'V' },
  { label: 'Debug', value: 'D' },
  { label: 'Info', value: 'I' },
  { label: 'Warning', value: 'W' },
  { label: 'Error', value: 'E' },
  { label: 'Fatal', value: 'F' }
]

const renderPackageLabel = (option: any) => {
  if (option.value === '') {
    return h('span', { style: { fontWeight: 'bold' } }, option.label)
  }
  return h('div', { style: { display: 'flex', alignItems: 'center', gap: '8px' } }, [
    h('span', option.label),
    option.isSystem ? h('span', { 
      style: { 
        fontSize: '10px', 
        color: '#999', 
        background: '#f0f0f0', 
        padding: '2px 6px', 
        borderRadius: '3px' 
      } 
    }, '系统') : null
  ])
}

const filteredDeviceLogs = computed(() => {
  let logs = deviceLogs.value
  
  // 时间过滤已经在后端通过 adb logcat -t 参数完成
  
  // 按日志级别过滤
  if (logFilter.value.level !== 'all') {
    logs = logs.filter(log => {
      const match = log.message.match(/\s([VDIWEF])\s/)
      return match && match[1] === logFilter.value.level
    })
  }
  
  // 按关键字过滤
  if (logFilter.value.keyword) {
    const keyword = logFilter.value.keyword.toLowerCase()
    logs = logs.filter(log => log.message.toLowerCase().includes(keyword))
  }
  
  // 按包名过滤
  if (logFilter.value.packageName) {
    const packageName = logFilter.value.packageName.toLowerCase()
    logs = logs.filter(log => log.message.toLowerCase().includes(packageName))
  }
  
  // 转换 ANSI 颜色代码为 HTML
  return logs.map(log => ({
    ...log,
    html: ansiConverter.toHtml(log.message)
  }))
})

const handleDeviceChange = async (deviceId: string) => {
  emit('device-change', deviceId)
  await stopLogcat()
  deviceLogs.value = []
  await startLogcat(deviceId)
  await loadDevicePackages(deviceId)
}

const handleDeviceLogTab = async () => {
  consoleTab.value = 'device'
  if (props.deviceOptions.length > 0 && !selectedDevice.value) {
    selectedDevice.value = props.deviceOptions[0].value
    await startLogcat(selectedDevice.value)
    await loadDevicePackages(selectedDevice.value)
  }
}

const toggleLogOutput = () => {
  logOutputPaused.value = !logOutputPaused.value
}

const clearLogs = () => {
  if (consoleTab.value === 'app') {
    // 清空应用日志需要通过父组件
    emit('add-log', 'clear', '')
  } else {
    deviceLogs.value = []
  }
}

const exportLogs = async () => {
  try {
    const { save } = await import('@tauri-apps/plugin-dialog')
    const logs = consoleTab.value === 'app' 
      ? props.appLogs.map(log => `[${log.time}] ${log.message}`).join('\n')
      : deviceLogs.value.map(log => log.message).join('\n')
    
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5)
    const defaultName = consoleTab.value === 'app' 
      ? `app-logs-${timestamp}.txt`
      : `device-logs-${selectedDevice.value}-${timestamp}.txt`
    
    const filePath = await save({
      defaultPath: defaultName,
      filters: [{
        name: 'Text Files',
        extensions: ['txt']
      }]
    })
    
    if (filePath) {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('write_log_file', { path: filePath, content: logs })
      message.success('日志已导出')
    }
  } catch (error) {
    console.error('Failed to export logs:', error)
    message.error('导出失败')
  }
}

const startLogcat = async (deviceId: string, timeFilter?: string) => {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('start_logcat', { deviceId, timeFilter: timeFilter || null })
    
    logOutputPaused.value = false
    
    const fetchLogs = async () => {
      try {
        const logs: string[] = await invoke('get_logcat_logs', { deviceId })
        
        if (!logOutputPaused.value) {
          const isAtBottom = consoleRef.value 
            ? (consoleRef.value.scrollHeight - consoleRef.value.scrollTop - consoleRef.value.clientHeight) < 50
            : true
          
          logs.forEach(log => {
            deviceLogs.value.push({ 
              message: log,
              timestamp: Date.now()
            })
          })
          
          if (deviceLogs.value.length > 1000) {
            deviceLogs.value = deviceLogs.value.slice(-1000)
          }
          
          if (isAtBottom) {
            nextTick(() => {
              if (consoleRef.value && consoleTab.value === 'device') {
                consoleRef.value.scrollTop = consoleRef.value.scrollHeight
              }
            })
          }
        }
      } catch (error) {
        console.error('Failed to fetch logcat:', error)
      }
    }
    
    logcatProcess.value = setInterval(fetchLogs, 1000)
  } catch (error) {
    console.error('Failed to start logcat:', error)
    emit('add-log', 'error', `无法启动设备日志: ${error}`)
  }
}

const stopLogcat = async () => {
  if (logcatProcess.value) {
    clearInterval(logcatProcess.value)
    logcatProcess.value = null
  }
  
  logOutputPaused.value = false
  
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('stop_logcat')
  } catch (error) {
    console.error('Failed to stop logcat:', error)
  }
}

const restartLogcatWithTimeFilter = async () => {
  if (!selectedDevice.value) return
  
  await stopLogcat()
  deviceLogs.value = []
  
  let timeFilter: string | undefined = undefined
  if (timeFilterType.value === 'recent') {
    timeFilter = `recent:${recentMinutes.value}`
  } else if (timeFilterType.value === 'since' && sinceTime.value) {
    const date = new Date(sinceTime.value)
    timeFilter = `since:${date.toISOString()}`
  }
  
  await startLogcat(selectedDevice.value, timeFilter)
}

const loadDevicePackages = async (deviceId: string) => {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const packages: Array<{name: string, is_system: boolean}> = await invoke('get_device_packages', { deviceId })
    
    packageList.value = packages.map(pkg => ({
      label: pkg.name,
      value: pkg.name,
      isSystem: pkg.is_system
    }))
  } catch (error) {
    console.error('Failed to load packages:', error)
    packageList.value = []
  }
}

const openScreenshot = async (path: string) => {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('open_file', { path })
  } catch (error) {
    console.error('Failed to open screenshot:', error)
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

watch([timeFilterType, recentMinutes, sinceTime], () => {
  restartLogcatWithTimeFilter()
})

watch(() => props.collapsed, async (collapsed) => {
  if (collapsed) {
    await stopLogcat()
  }
})

// 暴露方法给父组件
defineExpose({
  startDeviceLog: async (deviceId: string) => {
    consoleTab.value = 'device'
    selectedDevice.value = deviceId
    deviceLogs.value = []
    await stopLogcat()
    await startLogcat(deviceId)
    await loadDevicePackages(deviceId)
  },
  stopDeviceLog: stopLogcat
})
</script>

<style scoped>
@import '../views/HomeView.css';
</style>
