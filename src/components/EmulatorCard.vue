<template>
  <n-card class="emulator-card" hoverable>
    <div class="card-content">
      <div class="drag-handle" :class="{ 'running': emulator.status === 'running' }">⋮⋮</div>
      <div class="emulator-info">
        <div class="emulator-details">
          {{ emulator.deviceType }} · {{ emulator.osVersion }}
        </div>
        <div class="emulator-id" @click="$emit('copyId', emulator.id)">{{ emulator.id }}</div>
      </div>
      <div class="emulator-actions">
        <span v-if="emulator.status === 'running'" class="status-running">
          {{ t(`status.${emulator.status}`) }}
        </span>
        <n-button
          v-if="emulator.status === 'stopped'"
          type="primary"
          size="small"
          @click="$emit('start', emulator.id)"
        >
          {{ t("actions.start") }}
        </n-button>
        <n-button v-else type="error" size="small" @click="$emit('stop', emulator.id)">
          {{ t("actions.stop") }}
        </n-button>
        <n-dropdown :options="dropdownOptions" @select="handleAction">
          <n-button size="small">{{ t("actions.more") }} ▼</n-button>
        </n-dropdown>
      </div>
    </div>
  </n-card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { NCard, NButton, NTag, NDropdown } from "naive-ui";
import { useI18n } from "vue-i18n";
import type { Emulator } from "@/stores/emulator";

const props = defineProps<{
  emulator: Emulator;
}>();

const emit = defineEmits<{
  start: [id: string];
  stop: [id: string];
  delete: [id: string];
  wipeData: [id: string];
  screenshot: [id: string];
  viewLogs: [id: string];
  copyId: [id: string];
}>();

const { t } = useI18n();

const dropdownOptions = computed(() => [
  { label: t("actions.copyId"), key: "copyId" },
  { label: t("actions.screenshot"), key: "screenshot" },
  { label: t("actions.viewLogs"), key: "viewLogs" },
  { label: t("actions.wipeData"), key: "wipeData" },
  { label: t("actions.delete"), key: "delete" },
]);

const handleAction = (key: string) => {
  emit(key as any, props.emulator.id);
};
</script>

<style scoped>
.emulator-card {
  margin-bottom: 8px;
}

.card-content {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 1px 0;
}

.drag-handle {
  cursor: grab;
  color: #999;
  font-size: 16px;
  user-select: none;
}

.drag-handle.running {
  color: #18a058;
}

.emulator-info {
  flex: 1;
}

.emulator-details {
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 2px;
}

.emulator-id {
  font-size: 11px;
  color: #999;
  cursor: pointer;
  /* margin-bottom: 6px; */
}

.emulator-id:hover {
  color: #18a058;
}

.emulator-actions {
  display: flex;
  gap: 8px;
}

.emulator-actions .n-button {
  font-size: 13px;
}

.status-running {
  color: #18a058;
  font-size: 13px;
}
</style>
