<template>
  <div class="real-device-card">
    <div class="card-content">
      <div class="device-type-badge" :class="device.type">
        {{ device.type === 'ios' ? 'iOS' : device.type === 'android' ? 'Android' : 'HarmonyOS' }}
      </div>
      <div class="device-info">
        <div class="device-name">{{ device.name }}</div>
        <div class="device-details">
          {{ device.brand }} · <span class="device-serial" @click="handleCopySerial" :title="t('actions.copyId')">{{ device.serial }}</span>
        </div>
      </div>
      <div class="device-status">
        <template v-if="device.type === 'ios'">
          <span class="status-tag" :class="device.trusted ? 'trusted' : 'untrusted'">
            {{ device.trusted ? t('status.trusted') : t('status.untrusted') }}
          </span>
        </template>
        <template v-else>
          <span class="status-tag" :class="device.usb_debugging ? 'debugging' : 'no-debugging'">
            {{ device.usb_debugging ? t('status.usbDebugging') : t('status.usbDebuggingOff') }}
          </span>
        </template>
      </div>
      <div class="device-actions">
        <n-dropdown :options="dropdownOptions" @select="handleAction">
          <n-button size="small" quaternary circle>
            <img src="@/assets/more.svg" class="action-icon" />
          </n-button>
        </n-dropdown>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NButton, NDropdown } from 'naive-ui'
import { useI18n } from 'vue-i18n'

export interface RealDevice {
  type: string
  name: string
  serial: string
  brand: string
  vendor_id: string
  product_id: string
  usb_debugging?: boolean
  trusted?: boolean
}

const props = defineProps<{
  device: RealDevice
}>()

const emit = defineEmits<{
  copyId: [serial: string]
}>()

const { t } = useI18n()

const dropdownOptions = computed(() => [
  { label: t('actions.copyId'), key: 'copyId' },
])

const handleAction = (key: string) => {
  if (key === 'copyId') {
    emit('copyId', props.device.serial)
  }
}

const handleCopySerial = () => {
  emit('copyId', props.device.serial)
}
</script>

<style scoped>
.real-device-card {
  padding: 12px 16px;
  transition: background-color 0.2s;
}

.real-device-card:hover {
  background-color: rgba(0, 0, 0, 0.02);
}

.card-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.device-type-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 4px;
  flex-shrink: 0;
}

.device-type-badge.android {
  background: #e8f5e9;
  color: #2e7d32;
}

.device-type-badge.ios {
  background: #e3f2fd;
  color: #1565c0;
}

.device-type-badge.harmony {
  background: #fff3e0;
  color: #e65100;
}

.device-info {
  flex: 1;
  min-width: 0;
}

.device-name {
  font-size: 14px;
  font-weight: 500;
  color: #333;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.device-details {
  font-size: 11px;
  color: #999;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.device-serial {
  color: #1976d2;
  cursor: pointer;
  transition: color 0.2s;
}

.device-serial:hover {
  color: #1565c0;
  text-decoration: underline;
}

.device-status {
  flex-shrink: 0;
}

.status-tag {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 4px;
}

.status-tag.trusted,
.status-tag.debugging {
  background: #e8f5e9;
  color: #2e7d32;
}

.status-tag.untrusted,
.status-tag.no-debugging {
  background: #fff3e0;
  color: #e65100;
}

.device-actions {
  flex-shrink: 0;
}

.action-icon {
  width: 20px;
  height: 20px;
}
</style>
