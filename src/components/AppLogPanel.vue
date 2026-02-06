<template>
  <transition name="console-slide">
    <div v-if="!collapsed" class="console-panel" :style="{ width: panelWidth + 'px' }">
      <div class="console-header">
        <div class="console-header-left">
          <n-button text size="small" @click="$emit('update:collapsed', true)" class="collapse-btn">
            <img src="@/assets/silder.svg" class="slider-icon" />
          </n-button>
          <span class="console-title">操作日志</span>
        </div>
      </div>
      
      <div class="console-body">
        <div class="console-content" ref="consoleRef">
          <div class="console-logs">
            <div v-for="(log, index) in logs" :key="index" :class="['console-log', log.type]">
              <span class="console-time">{{ log.time }}</span>
              <span class="console-message">{{ log.message }}</span>
              <div v-if="log.path" class="console-path-container">
                <span class="console-path" @click="$emit('openScreenshot', log.path)">
                  {{ log.path }}
                </span>
                <n-button 
                  text 
                  size="tiny" 
                  @click="$emit('copyPath', log.path)"
                  class="copy-path-btn"
                  title="复制路径"
                >
                  <img src="@/assets/copy.svg" class="btn-icon" />
                </n-button>
                <n-button 
                  text 
                  size="tiny" 
                  @click="$emit('copyImage', log.path)"
                  class="copy-image-btn"
                  title="复制图片"
                >
                  <img src="@/assets/image.svg" class="btn-icon" />
                </n-button>
              </div>
            </div>
          </div>
        </div>
        <div class="console-tabs">
          <div 
            class="console-tab active"
            title="程序输出"
          >
            <img src="@/assets/app-log.svg" class="tab-icon" />
          </div>
          <div 
            class="console-tab"
            @click="$emit('switchToDevice')"
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
import { ref, watch, nextTick } from 'vue'
import { NButton } from 'naive-ui'

interface LogEntry {
  type: string
  message: string
  time: string
  path?: string
}

interface Props {
  collapsed: boolean
  panelWidth: number
  logs: LogEntry[]
}

const props = defineProps<Props>()

defineEmits<{
  'update:collapsed': [value: boolean]
  'openScreenshot': [path: string]
  'copyPath': [path: string]
  'copyImage': [path: string]
  'switchToDevice': []
}>()

const consoleRef = ref<HTMLElement>()

// 自动滚动到底部
watch(() => props.logs.length, () => {
  nextTick(() => {
    if (consoleRef.value) {
      consoleRef.value.scrollTop = consoleRef.value.scrollHeight
    }
  })
})

defineExpose({
  consoleRef
})
</script>

<style scoped src="../views/HomeView.css"></style>
