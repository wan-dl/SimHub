import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useSettingsStore = defineStore('settings', () => {
  const language = ref<'zh-CN' | 'en-US'>('zh-CN')
  const theme = ref<'light' | 'dark' | 'system'>('system')
  const autoStart = ref(false)
  const minimizeToTray = ref(true)
  const closeToMinimize = ref(true)
  const androidHome = ref('')
  const devecoHome = ref('')
  const xcodeHome = ref('')

  async function loadSettings() {
    try {
      const settings: any = await invoke('get_settings')
      language.value = settings.language
      theme.value = settings.theme
      autoStart.value = settings.auto_start
      minimizeToTray.value = settings.minimize_to_tray
      closeToMinimize.value = settings.close_to_minimize
      androidHome.value = settings.android_home
      devecoHome.value = settings.deveco_home
      xcodeHome.value = settings.xcode_home
    } catch (error) {
      console.error('Failed to load settings:', error)
    }
  }

  async function saveSettings() {
    await invoke('save_settings', {
      settings: {
        language: language.value,
        theme: theme.value,
        auto_start: autoStart.value,
        minimize_to_tray: minimizeToTray.value,
        close_to_minimize: closeToMinimize.value,
        android_home: androidHome.value,
        deveco_home: devecoHome.value,
        xcode_home: xcodeHome.value
      }
    })
  }

  return {
    language,
    theme,
    autoStart,
    minimizeToTray,
    closeToMinimize,
    androidHome,
    devecoHome,
    xcodeHome,
    loadSettings,
    saveSettings
  }
})
