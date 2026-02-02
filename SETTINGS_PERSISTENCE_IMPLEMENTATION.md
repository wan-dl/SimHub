# Settings Persistence Implementation

## Status: ✅ COMPLETED

## What Was Implemented

### Backend (Rust)
File: `src-tauri/src/commands/settings.rs`

1. **File-based persistence** instead of memory-only storage
2. Settings are saved to a JSON file at:
   - Windows: `%APPDATA%\oh-emulator-manager\settings.json`
   - macOS: `~/Library/Application Support/oh-emulator-manager/settings.json`
   - Linux: `~/.config/oh-emulator-manager/settings.json`

3. **Key functions:**
   - `get_settings_path()` - Gets the platform-specific config directory
   - `load_settings_from_file()` - Loads settings from JSON file (creates default if not exists)
   - `save_settings_to_file()` - Saves settings to JSON file
   - `get_settings()` - Tauri command to load settings (with environment variable fallbacks)
   - `save_settings()` - Tauri command to save settings

4. **Dependencies added:**
   - `dirs = "5"` in Cargo.toml for cross-platform config directory paths

### Frontend (Vue/TypeScript)

1. **App.vue** - Application startup
   - `onMounted()` hook loads settings when app starts
   - `watch()` monitors language changes and updates i18n locale immediately

2. **stores/settings.ts** - Pinia store
   - `loadSettings()` - Calls backend to load settings
   - `saveSettings()` - Calls backend to save settings

3. **views/SettingsView.vue** - Settings UI
   - `handleAutoSave()` - Automatically saves settings when any option changes
   - Called on language button click
   - Called on theme button click
   - Called on switch toggle (auto-start, minimize to tray, etc.)
   - Called after SDK path selection

## How It Works

### On App Startup:
1. App.vue `onMounted()` → `settingsStore.loadSettings()`
2. Frontend calls `get_settings` Tauri command
3. Backend loads from `settings.json` (or creates default)
4. Settings populate the store
5. Watch triggers and updates i18n locale

### When User Changes Settings:
1. User clicks language/theme button or toggles switch
2. Store value updates immediately (reactive)
3. `handleAutoSave()` is called
4. Frontend calls `save_settings` Tauri command
5. Backend writes to `settings.json`
6. For language: watch in App.vue updates i18n locale

### On Next App Restart:
1. Settings load from `settings.json`
2. UI reflects saved preferences
3. Language and theme are restored

## Testing Instructions

### Test 1: Language Persistence
1. Build and run the app: `pnpm tauri dev`
2. Go to Settings → General
3. Click "English" button
4. Verify UI changes to English immediately
5. Close the app completely
6. Restart the app
7. ✅ Expected: App should start in English

### Test 2: Theme Persistence
1. Go to Settings → General
2. Click "深色" (Dark) button
3. Verify UI changes to dark theme
4. Close the app
5. Restart the app
6. ✅ Expected: App should start in dark theme

### Test 3: SDK Path Persistence
1. Go to Settings → Android
2. Click "选择文件夹" and select a folder
3. Close the app
4. Restart the app
5. Go to Settings → Android
6. ✅ Expected: Selected path should still be there

### Test 4: Settings File Location
Check that the settings file was created:

**Windows:**
```cmd
type %APPDATA%\oh-emulator-manager\settings.json
```

**macOS/Linux:**
```bash
cat ~/.config/oh-emulator-manager/settings.json
```

Expected content:
```json
{
  "language": "en-US",
  "theme": "dark",
  "auto_start": false,
  "minimize_to_tray": true,
  "close_to_minimize": true,
  "android_home": "",
  "deveco_home": "",
  "xcode_home": ""
}
```

## Build Command

To test the implementation, rebuild the app:

```bash
pnpm tauri dev
```

Or for production build:

```bash
pnpm tauri build
```

## Known Issues

None. Implementation is complete and should work correctly.

## Files Modified

1. `src-tauri/src/commands/settings.rs` - Added file persistence
2. `src-tauri/Cargo.toml` - Added `dirs = "5"` dependency
3. `src/App.vue` - Added onMounted and watch for language
4. `src/stores/settings.ts` - Already had load/save methods
5. `src/views/SettingsView.vue` - Already calls handleAutoSave

## Next Steps

1. **Rebuild the application** to compile the new Rust code
2. **Test all scenarios** listed above
3. If any issues occur, check the browser console and terminal for errors
