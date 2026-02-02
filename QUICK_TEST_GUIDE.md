# Quick Test Guide - Settings Persistence

## ✅ Implementation Complete

All code changes have been made. Settings will now persist across app restarts.

## 🚀 Quick Test (3 minutes)

### Step 1: Rebuild the App
```bash
pnpm tauri dev
```

### Step 2: Test Language Persistence
1. Open the app
2. Click "设置" (Settings) at bottom
3. Click "English" button
4. ✅ UI should change to English immediately
5. Close the app (completely exit)
6. Restart: `pnpm tauri dev`
7. ✅ App should open in English

### Step 3: Test Theme Persistence
1. Go to Settings
2. Click "Dark" button
3. ✅ UI should change to dark theme
4. Close and restart the app
5. ✅ App should open in dark theme

## 📁 Where Settings Are Saved

**Windows:**
```
%APPDATA%\oh-emulator-manager\settings.json
```

**To view the file:**
```cmd
type %APPDATA%\oh-emulator-manager\settings.json
```

## 🐛 If Something Goes Wrong

### Issue: Language doesn't change immediately
- Check browser console (F12) for errors
- Verify `src/App.vue` has the watch function

### Issue: Settings don't persist after restart
- Check if settings.json file is created
- Check terminal for Rust errors
- Verify `dirs = "5"` is in `src-tauri/Cargo.toml`

### Issue: Build fails
- Run `cargo clean` in src-tauri folder
- Delete `src-tauri/target` folder
- Rebuild: `pnpm tauri dev`

## 📝 What Changed

### Backend (Rust)
- ✅ Settings now save to JSON file instead of memory
- ✅ File location uses platform-specific config directory
- ✅ Auto-creates directory if it doesn't exist

### Frontend (Vue)
- ✅ Settings load on app startup
- ✅ Language changes update UI immediately
- ✅ All setting changes auto-save

## 🎯 Expected Behavior

1. **First Launch:** Default settings (Chinese, System theme)
2. **Change Language:** UI updates immediately
3. **Restart App:** Language persists
4. **Change Theme:** UI updates immediately
5. **Restart App:** Theme persists
6. **All Settings:** Persist across restarts

## ✨ No Manual Save Button Needed

Settings auto-save when you:
- Click language button
- Click theme button
- Toggle any switch
- Select SDK folder

Everything saves automatically!
