<template>
  <n-config-provider :theme="theme" :locale="locale" :date-locale="dateLocale">
    <n-message-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <router-view />
        </n-notification-provider>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { computed, watch, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { NConfigProvider, NMessageProvider, NDialogProvider, NNotificationProvider, darkTheme, zhCN, dateZhCN, enUS, dateEnUS } from 'naive-ui'
import { useSettingsStore } from '@/stores/settings'
import { useLogsStore } from '@/stores/logs'
import { listen } from '@tauri-apps/api/event'

const settingsStore = useSettingsStore()
const logsStore = useLogsStore()
const { locale: i18nLocale } = useI18n()
const router = useRouter()

// 应用启动时加载设置
onMounted(async () => {
  await settingsStore.loadSettings()
  
  // 监听菜单事件
  await listen('navigate-to-settings', () => {
    router.push('/settings')
  })
  
  // 监听后端日志事件
  await listen('add-log', (event: any) => {
    const { type, message, source } = event.payload
    logsStore.addLog(type, message, source)
  })
})

// 监听语言变化并更新 i18n
watch(() => settingsStore.language, (newLang) => {
  i18nLocale.value = newLang
}, { immediate: true })

const theme = computed(() => {
  if (settingsStore.theme === 'dark') return darkTheme
  if (settingsStore.theme === 'light') return null
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? darkTheme : null
})

const locale = computed(() => settingsStore.language === 'zh-CN' ? zhCN : enUS)
const dateLocale = computed(() => settingsStore.language === 'zh-CN' ? dateZhCN : dateEnUS)
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
  -webkit-user-select: none;
  user-select: none;
}

/* 允许输入框和可编辑内容选择文本 */
input,
textarea,
[contenteditable="true"],
.n-input__input-el {
  -webkit-user-select: text;
  user-select: text;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  overscroll-behavior: none;
  overflow: hidden;
}

#app {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  overscroll-behavior: none;
}
</style>
