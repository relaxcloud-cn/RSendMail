<script setup lang="ts">
import { ref, watch } from 'vue'
import ConfigPanel from './components/ConfigPanel.vue'
import MonitoringPanel from './components/MonitoringPanel.vue'
import { Button } from '@/components/ui/button'
import { PlayIcon, StopCircleIcon, Settings2Icon, GlobeIcon, DownloadIcon } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { save, open, message } from '@tauri-apps/plugin-dialog'
import { writeTextFile, readTextFile } from '@tauri-apps/plugin-fs'
import HelpDialog from './components/HelpDialog.vue'

const { locale } = useI18n()
const configPanelRef = ref<InstanceType<typeof ConfigPanel> | null>(null)

watch(locale, (value) => {
  window.localStorage.setItem('rsendmail-locale', value)
})

async function handleStart() {
  if (!configPanelRef.value) return;

  if (typeof configPanelRef.value.validate === 'function') {
    const errors = configPanelRef.value.validate();
    if (errors && errors.length > 0) {
      await message('Please fix the following problems:\n\n- ' + errors.join('\n- '), {
        title: 'Missing Required Fields',
        kind: 'error'
      });
      return;
    }
  }

  const config = configPanelRef.value.buildConfig();
  try {
    await invoke('start_sending', { config });
  } catch (err) {
    console.error("Failed to start sending:", err);
  }
}

async function handleStop() {
  try {
    await invoke('stop_sending');
  } catch (err) {
    console.error("Failed to stop engine:", err);
  }
}

async function handleSaveConfig() {
  if (!configPanelRef.value) return;
  try {
    const config = configPanelRef.value.buildConfig();
    const filePath = await save({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      defaultPath: 'rsendmail_config.json'
    });
    if (filePath) {
      await writeTextFile(filePath, JSON.stringify(config, null, 2));
    }
  } catch (err) {
    console.error("Failed to save config:", err);
  }
}

async function handleLoadConfig() {
  try {
    const selected = await open({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      multiple: false
    });
    if (selected && typeof selected === 'string') {
      const contents = await readTextFile(selected);
      const parsed = JSON.parse(contents);
      if (configPanelRef.value && configPanelRef.value.loadConfig) {
        configPanelRef.value.loadConfig(parsed);
      }
    }
  } catch (err) {
    console.error("Failed to load config:", err);
  }
}
</script>

<template>
  <div class="h-screen w-screen bg-slate-50 dark:bg-slate-950 flex flex-col font-sans overflow-hidden">
    <!-- HEADER NAVBAR -->
    <header class="h-14 border-b border-slate-200 dark:border-slate-800 bg-white/70 dark:bg-slate-900/70 backdrop-blur-md flex items-center justify-between px-6 shrink-0 z-10 w-full relative drag-region">
      <div class="flex items-center space-x-3 pointer-events-none">
        <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-blue-500 to-indigo-600 flex items-center justify-center shadow-inner">
           <svg class="w-4 h-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/></svg>
        </div>
        <h1 class="text-lg font-bold text-slate-800 dark:text-slate-100 tracking-tight">RSendMail</h1>
      </div>
      
      <div class="flex items-center space-x-3 pointer-events-auto">
        <div class="flex items-center bg-transparent border border-slate-200 dark:border-slate-800 rounded-md px-2 h-8 text-sm">
          <GlobeIcon class="w-4 h-4 text-slate-500 mr-2" />
          <select v-model="locale" class="bg-transparent border-none outline-none text-slate-700 dark:text-slate-300">
            <option value="zh-CN">简体中文</option>
            <option value="en-US">English</option>
            <option value="zh-TW">繁體中文</option>
            <option value="ja-JP">日本語</option>
          </select>
        </div>
        
        <div class="w-px h-5 bg-slate-200 dark:bg-slate-800 mx-1"></div>
        
        <HelpDialog />
        
        <Button @click="handleLoadConfig" variant="outline" size="sm" class="h-8 gap-2 border-slate-200 dark:border-slate-800">
          <DownloadIcon class="w-4 h-4 text-slate-500" />
          <span>{{ $t('gui.load_config') }}</span>
        </Button>
        <Button @click="handleSaveConfig" variant="outline" size="sm" class="h-8 gap-2 border-slate-200 dark:border-slate-800">
          <Settings2Icon class="w-4 h-4 text-slate-500" />
          <span>{{ $t('gui.save_config') }}</span>
        </Button>
        <div class="w-px h-5 bg-slate-200 dark:bg-slate-800 mx-1"></div>
        <Button @click="handleStop" variant="destructive" size="sm" class="h-8 gap-2">
          <StopCircleIcon class="w-4 h-4" />
          <span>{{ $t('gui.stop_send') }}</span>
        </Button>
        <Button @click="handleStart" size="sm" class="h-8 gap-2 bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-700 hover:to-indigo-700 text-white shadow-md shadow-blue-500/20">
          <PlayIcon class="w-4 h-4 fill-current" />
          <span>{{ $t('gui.start_send') }}</span>
        </Button>
      </div>
    </header>

    <!-- MAIN CONTENT GRID -->
    <main class="flex-1 p-6 grid grid-cols-1 lg:grid-cols-12 gap-6 overflow-hidden">
      <!-- LEFT PANELS: Settings (Takes up 5 cols) -->
      <div class="lg:col-span-5 h-full overflow-hidden flex flex-col">
          <ConfigPanel ref="configPanelRef" />
      </div>

      <!-- RIGHT PANELS: Metrics & Logs (Takes up 7 cols) -->
      <div class="lg:col-span-7 h-full overflow-hidden flex flex-col">
          <MonitoringPanel />
      </div>
    </main>
  </div>
</template>

<style>
/* Custom directive to allow window dragging on MacOS */
.drag-region {
  -webkit-app-region: drag;
}
.pointer-events-auto {
  -webkit-app-region: no-drag;
}
</style>
