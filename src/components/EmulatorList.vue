<template>
  <div class="emulator-list">
    <n-spin :show="loading" class="spin-container">
      <div v-if="emulators.length === 0" class="empty-container">
        <n-empty description="暂无模拟器" />
      </div>
      <div v-else class="emulator-items">
        <emulator-card
          v-for="(emulator, index) in filteredEmulators"
          :key="emulator.id"
          :class="{ 'stripe': index % 2 === 1 }"
          :emulator="emulator"
          :is-starting="props.startingEmulators?.has(emulator.id) || false"
          :is-stopping="props.stoppingEmulators?.has(emulator.id) || false"
          @start="$emit('start', $event)"
          @stop="$emit('stop', $event)"
          @delete="$emit('delete', $event)"
          @wipe-data="$emit('wipeData', $event)"
          @screenshot="$emit('screenshot', $event)"
          @view-logs="$emit('viewLogs', $event)"
          @copy-id="$emit('copyId', $event)"
          @edit="(id, type) => $emit('edit', id, type)"
        />
      </div>
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
  startingEmulators?: Set<string>
  stoppingEmulators?: Set<string>
}>()

defineEmits<{
  start: [id: string]
  stop: [id: string]
  delete: [id: string]
  wipeData: [id: string]
  screenshot: [id: string]
  viewLogs: [id: string]
  copyId: [id: string]
  edit: [id: string, type: string]
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
  overflow-y: auto;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.spin-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.emulator-items {
  flex: 1;
}

.emulator-items :deep(.emulator-card.stripe) {
  background-color: rgba(0, 0, 0, 0.02);
}

.emulator-items :deep(.emulator-card.stripe:hover) {
  background-color: rgba(0, 0, 0, 0.04);
}

.empty-container {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 300px;
}
</style>
