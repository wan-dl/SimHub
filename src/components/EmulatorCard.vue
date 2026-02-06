<template>
  <div class="emulator-card">
    <div class="card-content">
      <div class="drag-handle" :class="{ 'running': emulator.status === 'running' }">⋮⋮</div>
      <div class="emulator-info">
        <div class="emulator-name" :class="{ 'running': emulator.status === 'running' }">{{ emulator.name }}</div>
        <div class="emulator-details" :class="{ 'running': emulator.status === 'running' }">
          <template v-if="emulator.status === 'running'">
            {{ emulator.id }}
            <template v-if="emulator.osVersion">
              · {{ emulator.osVersion }}
            </template>
          </template>
          <template v-else>
            {{ emulator.deviceType }}
            <template v-if="emulator.osVersion">
              · {{ emulator.osVersion }}
            </template>
          </template>
        </div>
      </div>
      <div class="emulator-actions">
        <n-button
          v-if="emulator.status === 'stopped'"
          size="small"
          :loading="isStarting"
          :disabled="isStarting"
          @click="$emit('start', emulator.id)"
          quaternary
          circle
          class="start-button"
        >
          <img src="@/assets/start.svg" class="action-icon" />
        </n-button>
        <n-button v-else size="small" :loading="isStopping" :disabled="isStopping" @click="$emit('stop', emulator.id)" quaternary circle class="stop-button">
          <img src="@/assets/closed.svg" class="action-icon" />
        </n-button>
        <n-button
          v-if="emulator.type === 'android' || emulator.type === 'harmony'"
          size="small"
          quaternary
          circle
          class="edit-button"
          @click="$emit('edit', emulator.id, emulator.type)"
          title="编辑启动参数"
        >
          <img src="@/assets/edit.svg" class="action-icon" />
        </n-button>
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
import { computed } from "vue";
import { NButton, NDropdown } from "naive-ui";
import { useI18n } from "vue-i18n";
import type { Emulator } from "@/stores/emulator";

const props = defineProps<{
  emulator: Emulator;
  isStarting?: boolean;
  isStopping?: boolean;
}>();

const emit = defineEmits<{
  start: [id: string];
  stop: [id: string];
  delete: [id: string];
  wipeData: [id: string];
  screenshot: [id: string];
  viewLogs: [id: string];
  copyId: [id: string];
  edit: [id: string, type: string];
}>();

const { t } = useI18n();

const isStopped = computed(() => props.emulator.status === 'stopped');
const isIOS = computed(() => props.emulator.type === 'ios');

const dropdownOptions = computed(() => [
  { 
    label: t("actions.copyId"), 
    key: "copyId",
    disabled: isIOS.value ? false : isStopped.value
  },
  { 
    label: t("actions.screenshot"), 
    key: "screenshot",
    disabled: isStopped.value
  },
  { 
    label: t("actions.viewLogs"), 
    key: "viewLogs",
    disabled: isStopped.value
  },
  { label: t("actions.wipeData"), key: "wipeData" },
  { label: t("actions.delete"), key: "delete" },
]);

const handleAction = (key: string) => {
  emit(key as any, props.emulator.id);
};
</script>

<style scoped>
.emulator-card {
  padding: 12px 16px;
  transition: background-color 0.2s;
}

.emulator-card:hover {
  background-color: rgba(0, 0, 0, 0.02);
}

.card-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.drag-handle {
  cursor: grab;
  color: #999;
  font-size: 14px;
  user-select: none;
}

.drag-handle.running {
  color: #18a058;
}

.emulator-info {
  flex: 1;
}

.emulator-name {
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 2px;
  color: #333;
  transition: color 0.2s;
}

.emulator-name.running {
  color: #18a058;
}

.emulator-details {
  font-size: 11px;
  color: #999;
  transition: color 0.2s;
}

.emulator-details.running {
  color: #36ad6a;
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

.start-button {
  color: #424242;
}

.stop-button {
  color: #424242;
}

.edit-button {
  color: #424242;
}

.edit-button:hover {
  color: #333;
}

.action-icon {
  width: 20px;
  height: 20px;
}
</style>
