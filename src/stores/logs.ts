import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface LogEntry {
  id: string
  type: 'info' | 'success' | 'warning' | 'error' | 'command'
  message: string
  timestamp: number
  source?: string // 'app' | 'device' | 'system'
  deviceId?: string
}

export const useLogsStore = defineStore('logs', () => {
  const logs = ref<LogEntry[]>([])
  let logIdCounter = 0

  function addLog(
    type: LogEntry['type'],
    message: string,
    source: string = 'app',
    deviceId?: string
  ) {
    const log: LogEntry = {
      id: `log-${Date.now()}-${logIdCounter++}`,
      type,
      message,
      timestamp: Date.now(),
      source,
      deviceId
    }
    
    logs.value.push(log)
    
    // 限制日志数量，保留最近 5000 条
    if (logs.value.length > 5000) {
      logs.value = logs.value.slice(-5000)
    }
    
    return log
  }

  function clearLogs(source?: string, deviceId?: string) {
    if (source && deviceId) {
      logs.value = logs.value.filter(
        log => !(log.source === source && log.deviceId === deviceId)
      )
    } else if (source) {
      logs.value = logs.value.filter(log => log.source !== source)
    } else {
      logs.value = []
    }
  }

  function getLogsBySource(source: string, deviceId?: string) {
    if (deviceId) {
      return logs.value.filter(
        log => log.source === source && log.deviceId === deviceId
      )
    }
    return logs.value.filter(log => log.source === source)
  }

  return {
    logs,
    addLog,
    clearLogs,
    getLogsBySource
  }
})
