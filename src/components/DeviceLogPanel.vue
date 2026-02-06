<template>
  <transition name="console-slide">
    <div v-if="!collapsed" class="console-panel" :style="{ width: panelWidth + 'px' }">
      <div class="console-header">
        <div class="console-header-left">
          <n-button text size="small" @click="$emit('update:collapsed', true)" class="collapse-btn">
            <img src="@/assets/silder.svg" class="slider-icon" />
          </n-button>
          <span class="console-title">设备日志</span>
          <n-select
            :value="selectedDevice"
            @update:value="$emit('update:selectedDevice', $event)"
            size="small"
            :options="deviceOptions"
            :placeholder="deviceOptions.length > 0 ? '选择设备' : '无运行中的设备'"
            :disabled="deviceOptions.length === 0"
            style="width: 200px; margin-left: 12px"
          />
        </div>
      </div>
      
      <!-- 设备日志过滤器 -->
      <div class="console-filters">
        <n-select
          :value="filter.level"
          @update:value="updateFilter('level', $event)"
          size="small"
          :options="levelOptions"
          style="width: 100px"
        />
        <n-select
          :value="filter.packageName"
          @update:value="updateFilter('packageName', $event)"
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
          :value="filter.keyword"
          @update:value="updateFilter('keyword', $event)"
          size="small"
          placeholder="关键字过滤"
          clearable
          style="flex: 1; max-width: 200px"
        />
        <n-button 
          text 
          size="small" 
          @click="$emit('update:showKeywordFilter', !showKeywordFilter)" 
          :title="showKeywordFilter ? '隐藏关键字过滤' : '显示关键字过滤'" 
          class="filter-action-btn"
          :class="{ active: showKeywordFilter }"
        >
          <img src="@/assets/search.svg" class="action-icon" />
        </n-button>
        <n-button 
          text 
          size="small" 
          @click="togglePause" 
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
      <div class="console-time-filters">
        <n-radio-group :value="timeFilterType" @update:value="$emit('update:timeFilterType', $event)" size="small">
          <n-radio-button value="all">全部</n-radio-button>
          <n-radio-button value="recent">最近</n-radio-button>
        </n-radio-group>
        <n-input-number
          v-if="timeFilterType === 'recent'"
          :value="recentMinutes"
          @update:value="(value: number | null) => value && emit('update:recentMinutes', value)"
          size="small"
          :min="1"
          :max="60"
          style="width: 80px"
        />
        <span v-if="timeFilterType === 'recent'" class="time-unit">分钟</span>
        <n-date-picker
          v-if="timeFilterType === 'since'"
          :value="sinceTime"
          @update:value="(value: number | null) => emit('update:sinceTime', value)"
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
          <div class="console-logs">
            <div v-for="(log, index) in filteredLogs" :key="index" class="console-log device">
              <span class="console-message" v-html="log.html"></span>
            </div>
            <div v-if="filteredLogs.length === 0 && deviceLogs.length === 0" class="console-empty">
              <span>暂无设备日志</span>
              <span class="console-hint">启动模拟器后将显示 logcat 日志</span>
            </div>
            <div v-else-if="filteredLogs.length === 0 && deviceLogs.length > 0" class="console-empty">
              <span>没有匹配的日志</span>
              <span class="console-hint">尝试调整过滤条件</span>
            </div>
          </div>
        </div>
        <div class="console-tabs">
          <div 
            class="console-tab"
            @click="$emit('switchToApp')"
            title="程序输出"
          >
            <img src="@/assets/app-log.svg" class="tab-icon" />
          </div>
          <div 
            class="console-tab active"
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
import { ref, computed, nextTick, watch, h, onUnmounted } from 'vue'
import { NButton, NInput, NSelect, NRadioGroup, NRadioButton, NInputNumber, NDatePicker, useMessage } from 'naive-ui'
import AnsiToHtml from 'ansi-to-html'

const ansiConverter = new AnsiToHtml({
  fg: '#333',
  bg: '#f8f9fa',
  newline: false,
  escapeXML: true,
  stream: false
})

const message = useMessage()

interface LogEntry {
  message: string
  timestamp?: number
}

interface LogFilter {
  level: string
  keyword: string
  packageName: string
}

interface Props {
  collapsed: boolean
  panelWidth: number
  selectedDevice: string
  deviceOptions: Array<{ label: string; value: string }>
  filter: LogFilter
  showKeywordFilter: boolean
  timeFilterType: 'all' | 'recent' | 'since'
  recentMinutes: number
  sinceTime: number | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:collapsed': [value: boolean]
  'update:selectedDevice': [value: string]
  'update:showKeywordFilter': [value: boolean]
  'update:timeFilterType': [value: string]
  'update:recentMinutes': [value: number]
  'update:sinceTime': [value: number | null]
  'updateFilter': [key: string, value: string]
  'switchToApp': []
  'logError': [message: string]
}>()

const consoleRef = ref<HTMLElement>()
const deviceLogs = ref<LogEntry[]>([])
const logOutputPaused = ref(false)
const packageList = ref<Array<{label: string, value: string, isSystem: boolean}>>([])
const logcatProcess = ref<any>(null)

const levelOptions = [
  { label: '全部', value: 'all' },
  { label: 'Verbose', value: 'V' },
  { label: 'Debug', value: 'D' },
  { label: 'Info', value: 'I' },
  { label: 'Warning', value: 'W' },
  { label: 'Error', value: 'E' },
  { label: 'Fatal', value: 'F' }
]

const renderPackageLabel = (option: any) => {
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

const updateFilter = (key: string, value: string) => {
  emit('updateFilter', key, value)
}

const filteredLogs = computed(() => {
  let logs = deviceLogs.value
  
  // 按日志级别过滤
  if (props.filter.level !== 'all') {
    logs = logs.filter(log => {
      const match = log.message.match(/\s([VDIWEF])\s/)
      return match && match[1] === props.filter.level
    })
  }
  
  // 按关键字过滤
  if (props.filter.keyword) {
    const keyword = props.filter.keyword.toLowerCase()
    logs = logs.filter(log => log.message.toLowerCase().includes(keyword))
  }
  
  // 按包名过滤
  if (props.filter.packageName) {
    const packageName = props.filter.packageName.toLowerCase()
    logs = logs.filter(log => log.message.toLowerCase().includes(packageName))
  }
  
  // 转换 ANSI 颜色代码为 HTML
  return logs.map(log => ({
    ...log,
    html: ansiConverter.toHtml(log.message)
  }))
})

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

const startLogcat = async (deviceId: string, timeFilter?: string) => {
  try {
    console.log('startLogcat: Starting for device', deviceId, 'with time filter:', timeFilter)
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('start_logcat', { deviceId, timeFilter: timeFilter || null })
    console.log('startLogcat: Backend start_logcat completed')
    
    logOutputPaused.value = false
    
    const fetchLogs = async () => {
      try {
        const logs: string[] = await invoke('get_logcat_logs', { deviceId })
        console.log('fetchLogs: Received', logs.length, 'log lines')
        
        if (!logOutputPaused.value) {
          logs.forEach(log => {
            deviceLogs.value.push({ 
              message: log,
              timestamp: Date.now()
            })
          })
          
          if (deviceLogs.value.length > 1000) {
            deviceLogs.value = deviceLogs.value.slice(-1000)
          }
        }
      } catch (error) {
        console.error('fetchLogs: Failed to fetch logcat:', error)
      }
    }
    
    console.log('startLogcat: Setting up interval to fetch logs')
    logcatProcess.value = setInterval(fetchLogs, 1000)
    console.log('startLogcat: Interval set, process ID:', logcatProcess.value)
  } catch (error) {
    console.error('startLogcat: Failed to start logcat:', error)
    emit('logError', `无法启动设备日志: ${error}`)
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
  if (!props.selectedDevice) return
  
  await stopLogcat()
  deviceLogs.value = []
  
  let timeFilter: string | undefined = undefined
  if (props.timeFilterType === 'recent') {
    timeFilter = `recent:${props.recentMinutes}`
  } else if (props.timeFilterType === 'since' && props.sinceTime) {
    const date = new Date(props.sinceTime)
    timeFilter = `since:${date.toISOString()}`
  }
  
  await startLogcat(props.selectedDevice, timeFilter)
}

const togglePause = () => {
  logOutputPaused.value = !logOutputPaused.value
}

const clearLogs = () => {
  deviceLogs.value = []
}

const exportLogs = async () => {
  try {
    const { save } = await import('@tauri-apps/plugin-dialog')
    const logs = deviceLogs.value.map(log => log.message).join('\n')
    
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5)
    const defaultName = `device-logs-${props.selectedDevice}-${timestamp}.txt`
    
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

// 监听设备变化
watch(() => props.selectedDevice, async (newDevice, oldDevice) => {
  if (newDevice && newDevice !== oldDevice) {
    await stopLogcat()
    deviceLogs.value = []
    await startLogcat(newDevice)
    await loadDevicePackages(newDevice)
  }
})

// 监听时间过滤参数变化
watch([() => props.timeFilterType, () => props.recentMinutes, () => props.sinceTime], () => {
  restartLogcatWithTimeFilter()
})

// 监听折叠状态
watch(() => props.collapsed, async (collapsed) => {
  if (collapsed) {
    await stopLogcat()
  }
})

// 自动滚动到底部
watch(() => deviceLogs.value.length, () => {
  if (!logOutputPaused.value) {
    nextTick(() => {
      if (consoleRef.value) {
        const isAtBottom = consoleRef.value.scrollHeight - consoleRef.value.scrollTop - consoleRef.value.clientHeight < 50
        if (isAtBottom) {
          consoleRef.value.scrollTop = consoleRef.value.scrollHeight
        }
      }
    })
  }
})

// 组件卸载时清理
onUnmounted(() => {
  stopLogcat()
})

defineExpose({
  consoleRef,
  startLogcat,
  stopLogcat,
  loadDevicePackages
})
</script>

<style scoped src="../views/HomeView.css"></style>
