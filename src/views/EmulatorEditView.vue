<template>
  <div class="emulator-edit-view">
    <div class="edit-header">
      <div class="header-left">
        <n-button text @click="goBack">
          <template #icon>
            <n-icon :component="ArrowBack" />
          </template>
        </n-button>
        <h2>{{ emulatorId }} 模拟器启动参数</h2>
      </div>
    </div>
    <div class="edit-content">
      <div class="content-wrapper" v-if="loaded">
        <n-form label-placement="left" label-width="0">
          <!-- Android 专属参数 -->
          <template v-if="emulatorType === 'android'">
            <n-form-item>
              <div class="form-item-content">
                <div class="param-header">
                  <span class="param-name">-wipe-data</span>
                  <n-switch v-model:value="wipeData" />
                  <span class="form-item-desc">清除数据启动</span>
                </div>
              </div>
            </n-form-item>

            <n-form-item>
              <div class="form-item-content">
                <div class="param-header">
                  <span class="param-name">-no-snapshot</span>
                  <n-switch v-model:value="noSnapshot" />
                  <span class="form-item-desc">冷启动（不使用快照）</span>
                </div>
              </div>
            </n-form-item>

            <n-form-item>
              <div class="form-item-content">
                <div class="param-header">
                  <span class="param-name">-no-window</span>
                  <n-switch v-model:value="noWindow" />
                  <span class="form-item-desc">无图形界面启动</span>
                </div>
              </div>
            </n-form-item>

            <n-form-item>
              <div class="form-item-content">
                <div class="param-header">
                  <span class="param-name">-dns-server</span>
                  <n-input 
                    v-model:value="dnsServer"
                    placeholder="例如: 8.8.8.8"
                    style="width: 200px"
                  />
                  <span class="form-item-desc">DNS 服务器地址</span>
                </div>
              </div>
            </n-form-item>

            <n-form-item>
              <div class="form-item-content">
                <div class="param-header">
                  <span class="param-name">-gps</span>
                  <div class="gps-inputs">
                    <n-input 
                      v-model:value="gpsLongitude"
                      placeholder="经度"
                      style="width: 120px"
                    />
                    <span class="gps-separator">,</span>
                    <n-input 
                      v-model:value="gpsLatitude"
                      placeholder="纬度"
                      style="width: 120px"
                    />
                  </div>
                  <span class="form-item-desc">GPS 位置</span>
                </div>
              </div>
            </n-form-item>

            <n-form-item>
              <div class="form-item-content">
                <div class="param-header">
                  <span class="param-name">-memory</span>
                  <n-input-number 
                    v-model:value="memory"
                    placeholder="4096"
                    :min="512"
                    :step="512"
                    style="width: 150px"
                  />
                  <span class="form-item-desc">内存大小 (MB)</span>
                </div>
              </div>
            </n-form-item>

            <n-form-item>
              <div class="form-item-content">
                <div class="param-header">
                  <span class="param-name">-http-proxy</span>
                  <n-input 
                    v-model:value="httpProxy"
                    placeholder="例如: http://proxy:port"
                    style="width: 250px"
                  />
                  <span class="form-item-desc">HTTP 代理</span>
                </div>
              </div>
            </n-form-item>

            <n-form-item>
              <div class="form-item-content">
                <div class="param-header">
                  <span class="param-name">-cores</span>
                  <n-input-number 
                    v-model:value="cores"
                    placeholder="4"
                    :min="1"
                    :max="16"
                    style="width: 150px"
                  />
                  <span class="form-item-desc">CPU 核心数</span>
                </div>
              </div>
            </n-form-item>

            <n-form-item>
              <div class="form-item-content" style="padding-bottom: 90px;">
                <div class="param-header">
                  <span class="param-name">-gpu</span>
                  <n-select 
                    v-model:value="gpu"
                    :options="gpuOptions"
                    placeholder="选择 GPU 模式"
                    style="width: 200px"
                  />
                  <span class="form-item-desc">GPU 模式</span>
                </div>
              </div>
            </n-form-item>
          </template>

          <!-- 鸿蒙专属参数 -->
          <template v-if="emulatorType === 'harmony'">
            <!-- 鸿蒙参数待添加 -->
          </template>
        </n-form>
      </div>
    </div>
    
    <div class="edit-footer">
      <n-button type="primary" @click="handleSave">
        <template #icon>
          <n-icon :component="SaveOutline" />
        </template>
        保存
      </n-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { NButton, NIcon, NForm, NFormItem, NSwitch, NInput, NInputNumber, NSelect, useMessage } from 'naive-ui'
import { ArrowBack, SaveOutline } from '@vicons/ionicons5'

const router = useRouter()
const route = useRoute()
const message = useMessage()

const emulatorId = route.params.id as string
const emulatorType = route.query.type as string

const noWindow = ref(false)
const wipeData = ref(false)
const noSnapshot = ref(false)
const dnsServer = ref('')
const gpsLongitude = ref('')
const gpsLatitude = ref('')
const memory = ref<number | null>(null)
const httpProxy = ref('')
const cores = ref<number | null>(null)
const gpu = ref('auto')
const loaded = ref(false)

const gpuOptions = [
  { label: 'auto', value: 'auto' },
  { label: 'host', value: 'host' },
  { label: 'swiftshader_indirect', value: 'swiftshader_indirect' },
  { label: 'angle_indirect', value: 'angle_indirect' },
  { label: 'guest', value: 'guest' },
  { label: 'off', value: 'off' }
]

// 调试信息
console.log('Emulator ID:', emulatorId)
console.log('Emulator Type:', emulatorType)

const goBack = () => {
  router.push('/')
}

const loadSettings = async () => {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const settings = await invoke('get_emulator_launch_params', { 
      emulatorId,
      emulatorType 
    })
    if (settings) {
      noWindow.value = (settings as any).no_window || false
      wipeData.value = (settings as any).wipe_data || false
      noSnapshot.value = (settings as any).no_snapshot || false
      dnsServer.value = (settings as any).dns_server || ''
      gpsLongitude.value = (settings as any).gps_longitude || ''
      gpsLatitude.value = (settings as any).gps_latitude || ''
      memory.value = (settings as any).memory || null
      httpProxy.value = (settings as any).http_proxy || ''
      cores.value = (settings as any).cores || null
      gpu.value = (settings as any).gpu || 'auto'
    }
  } catch (error) {
    console.error('Failed to load settings:', error)
  } finally {
    loaded.value = true
  }
}

const handleSave = async () => {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('save_emulator_launch_params', {
      emulatorId,
      emulatorType,
      params: {
        no_window: noWindow.value,
        wipe_data: wipeData.value,
        no_snapshot: noSnapshot.value,
        dns_server: dnsServer.value,
        gps_longitude: gpsLongitude.value,
        gps_latitude: gpsLatitude.value,
        memory: memory.value,
        http_proxy: httpProxy.value,
        cores: cores.value,
        gpu: gpu.value
      }
    })
    message.success('保存成功')
  } catch (error) {
    console.error('Failed to save settings:', error)
    message.error('保存失败')
  }
}

onMounted(() => {
  loadSettings()
})
</script>

<style scoped>
.emulator-edit-view {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #ffffff;
}

.edit-header {
  display: flex;
  align-items: center;
  padding: 16px 24px;
  border-bottom: 1px solid #e0e0e0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.header-left h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: #333;
}

.edit-content {
  flex: 1;
  padding: 24px;
  padding-bottom: 120px;
  overflow-y: auto;
  display: flex;
  justify-content: center;
}

.content-wrapper {
  width: 100%;
  max-width: 800px;
}

.emulator-info {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 32px;
  padding-bottom: 16px;
  border-bottom: 1px solid #e0e0e0;
}

.emulator-info h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #333;
}

.emulator-type {
  padding: 4px 12px;
  background: #f0f0f0;
  border-radius: 12px;
  font-size: 12px;
  color: #666;
}

.form-item-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
}

.param-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.param-name {
  font-size: 14px;
  font-weight: 500;
  color: #333;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  min-width: 120px;
  text-align: right;
}

.form-item-desc {
  font-size: 13px;
  color: #666;
}

.gps-inputs {
  display: flex;
  align-items: center;
  gap: 8px;
}

.gps-separator {
  font-size: 14px;
  color: #666;
}

.scope-radio-group {
  margin-left: 88px;
}

.scope-radio-group > .n-radio {
  font-size: 13px;
}

.input-hint {
  font-size: 12px;
  color: #999;
  margin-left: 12px;
}

.edit-footer {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 16px 24px;
  background: #ffffff;
  border-top: 1px solid #e0e0e0;
  display: flex;
  justify-content: flex-end;
  z-index: 100;
}

.edit-footer :deep(.n-button.n-button--primary-type) {
  background-color: #1976d2 !important;
  border-color: #1976d2 !important;
  border-radius: 8px !important;
  color: #ffffff !important;
}

.edit-footer :deep(.n-button.n-button--primary-type:hover) {
  background-color: #1565c0 !important;
  border-color: #1565c0 !important;
}

.edit-footer :deep(.n-button.n-button--primary-type:active) {
  background-color: #0d47a1 !important;
  border-color: #0d47a1 !important;
}

.edit-footer :deep(.n-button.n-button--primary-type:focus) {
  background-color: #1976d2 !important;
  border-color: #1976d2 !important;
}

.edit-footer :deep(.n-button.n-button--primary-type .n-button__border) {
  border-radius: 8px !important;
  border-color: #1976d2 !important;
}

.edit-footer :deep(.n-button.n-button--primary-type .n-button__state-border) {
  border-radius: 8px !important;
  border-color: #1976d2 !important;
}

.edit-footer :deep(.n-button.n-button--primary-type .n-button__content) {
  color: #ffffff !important;
}

/* 移除 focus 时的绿色边框 */
.edit-footer :deep(.n-button.n-button--primary-type:focus .n-button__border),
.edit-footer :deep(.n-button.n-button--primary-type:focus .n-button__state-border) {
  border-color: #1976d2 !important;
}

.edit-footer :deep(.n-button.n-button--primary-type:hover .n-button__border),
.edit-footer :deep(.n-button.n-button--primary-type:hover .n-button__state-border) {
  border-color: #1565c0 !important;
}

.edit-footer :deep(.n-button.n-button--primary-type:active .n-button__border),
.edit-footer :deep(.n-button.n-button--primary-type:active .n-button__state-border) {
  border-color: #0d47a1 !important;
}

/* 统一输入组件高度 */
.emulator-edit-view :deep(.n-input),
.emulator-edit-view :deep(.n-input-number),
.emulator-edit-view :deep(.n-base-selection) {
  height: 32px !important;
  min-height: 32px !important;
}

.emulator-edit-view :deep(.n-input-number .n-input) {
  height: auto !important;
  min-height: auto !important;
}

.emulator-edit-view :deep(.n-base-selection-label) {
  height: 30px !important;
  line-height: 30px !important;
}
</style>
