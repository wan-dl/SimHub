<template>
  <div class="emulator-list">
    <n-spin :show="loading" class="spin-container">
      <div v-if="emulators.length === 0" class="empty-container">
        <n-empty description="暂无模拟器" />
      </div>
      <emulator-card
        v-for="emulator in filteredEmulators"
        :key="emulator.id"
        :emulator="emulator"
        @start="$emit('start', $event)"
        @stop="$emit('stop', $event)"
        @delete="$emit('delete', $event)"
        @wipe-data="$emit('wipeData', $event)"
        @screenshot="$emit('screenshot', $event)"
        @view-logs="$emit('viewLogs', $event)"
        @copy-id="$emit('copyId', $event)"
      />
    </n-spin>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NSpin, NEmpty } from 'naive-ui'
import EmulatorCard from './EmulatorCard.vue'
import type { Emulator } from '@/stores/emulator'

const props = defineProps<{
  emulators: Emulator[]
  loading?: boolean
  searchText?: string
}>()

defineEmits<{
  start: [id: string]
  stop: [id: string]
  delete: [id: string]
  wipeData: [id: string]
  screenshot: [id: string]
  viewLogs: [id: string]
  copyId: [id: string]
}>()

const filteredEmulators = computed(() => {
  if (!props.searchText) return props.emulators
  const text = props.searchText.toLowerCase()
  return props.emulators.filter(e =>
    e.name.toLowerCase().includes(text) ||
    e.deviceType.toLowerCase().includes(text)
  )
})
</script>

<style scoped>
.emulator-list {
  padding: 16px;
  overflow-y: auto;
  height: 100%;
}

.spin-container {
  height: 100%;
}

.empty-container {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
