<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Button } from '@/components/ui/button'
import { TerminalIcon, DownloadIcon, Trash2Icon } from 'lucide-vue-next'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'

// Reactive Stats
const totalEmails = ref(0)
const successCount = ref(0)
const errorCount = ref(0)
const elapsedTime = ref("00:00:00")

interface LogItem {
  id: number
  time: string
  level: 'INFO' | 'SUCCESS' | 'ERROR' | 'WARN' | string
  message: string
}

const logs = ref<LogItem[]>([
  { id: 1, time: new Date().toLocaleTimeString('en-US', { hour12: false }), level: 'INFO', message: 'System ready. Waiting for tasks...' }
])
const logScrollArea = ref<any>(null)

const levelColors: Record<string, string> = {
  INFO: 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400',
  SUCCESS: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400',
  ERROR: 'bg-rose-100 text-rose-700 dark:bg-rose-900/30 dark:text-rose-400',
  WARN: 'bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400',
}

function addLog(level: string, message: string) {
  const time = new Date().toLocaleTimeString('en-US', { hour12: false });
  logs.value.push({
    id: logs.value.length + 1,
    level,
    message,
    time
  })
  if (logs.value.length > 500) {
    logs.value.shift()
  }
  nextTick(() => {
    if (logScrollArea.value) {
      const viewport = logScrollArea.value.$el.querySelector('[data-radix-scroll-area-viewport]');
      if (viewport) {
        viewport.scrollTop = viewport.scrollHeight;
      }
    }
  })
}

function handleClearLogs() {
  logs.value = [];
  addLog('INFO', 'Logs cleared.');
}

async function handleExportLogs() {
  try {
    const filePath = await save({
      filters: [{ name: 'Log Text', extensions: ['txt'] }],
      defaultPath: 'rsendmail_logs.txt'
    });
    if (filePath) {
      const text = logs.value.map(l => `[${l.time}] [${l.level}] ${l.message}`).join('\n');
      await writeTextFile(filePath, text);
      addLog('INFO', `Logs exported to ${filePath}`);
    }
  } catch (err) {
    console.error("Failed to export logs:", err);
    addLog('ERROR', `Failed to export logs: ${err}`);
  }
}

let unlistenSend: UnlistenFn | null = null;
let unlistenProgress: UnlistenFn | null = null;
let unlistenStats: UnlistenFn | null = null;

onMounted(async () => {
  unlistenSend = await listen<{level: string, message: string}>('send-event', (event) => {
    addLog(event.payload.level, event.payload.message);
  });

  unlistenProgress = await listen<{sent: number, success: number, fail: number}>('progress-event', (event) => {
    totalEmails.value = event.payload.sent;
    successCount.value = event.payload.success;
    errorCount.value = event.payload.fail;
  });

  unlistenStats = await listen<{qps: number, elapsed: string}>('stats-event', (event) => {
    elapsedTime.value = event.payload.elapsed;
  });
})

onUnmounted(() => {
  if (unlistenSend) unlistenSend();
  if (unlistenProgress) unlistenProgress();
  if (unlistenStats) unlistenStats();
})
</script>

<template>
  <div class="h-full flex flex-col space-y-4">
    <!-- METRICS GRID -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 shrink-0">
      <Card class="border-slate-200 dark:border-slate-800 bg-gradient-to-b from-white to-slate-50/50 dark:from-slate-900 dark:to-slate-900/50 shadow-sm transition-all hover:-translate-y-1 hover:shadow-md duration-200">
        <CardContent class="p-4 flex flex-col items-center justify-center">
          <p class="text-xs font-medium text-slate-500 uppercase tracking-wider mb-1">{{ $t('gui.total') }}</p>
          <p class="text-3xl font-extrabold text-slate-900 dark:text-slate-100">{{ totalEmails.toLocaleString() }}</p>
        </CardContent>
      </Card>

      <Card class="border-emerald-100 dark:border-emerald-900/30 bg-gradient-to-b from-emerald-50/50 to-emerald-100/30 dark:from-emerald-950/20 dark:to-emerald-900/10 shadow-sm transition-all hover:-translate-y-1 hover:shadow-md duration-200">
        <CardContent class="p-4 flex flex-col items-center justify-center">
          <p class="text-xs font-medium text-emerald-600 dark:text-emerald-500 uppercase tracking-wider mb-1">{{ $t('gui.success') }}</p>
          <p class="text-3xl font-extrabold text-emerald-600 dark:text-emerald-400">{{ successCount.toLocaleString() }}</p>
        </CardContent>
      </Card>

      <Card class="border-rose-100 dark:border-rose-900/30 bg-gradient-to-b from-rose-50/50 to-rose-100/30 dark:from-rose-950/20 dark:to-rose-900/10 shadow-sm transition-all hover:-translate-y-1 hover:shadow-md duration-200">
        <CardContent class="p-4 flex flex-col items-center justify-center">
          <p class="text-xs font-medium text-rose-600 dark:text-rose-500 uppercase tracking-wider mb-1">{{ $t('gui.failed') }}</p>
          <p class="text-3xl font-extrabold text-rose-600 dark:text-rose-400">{{ errorCount.toLocaleString() }}</p>
        </CardContent>
      </Card>

      <Card class="border-slate-200 dark:border-slate-800 shadow-sm transition-all hover:-translate-y-1 hover:shadow-md duration-200">
        <CardContent class="p-4 flex flex-col items-center justify-center">
          <p class="text-xs font-medium text-slate-500 uppercase tracking-wider mb-1">{{ $t('gui.elapsed_time') }}</p>
          <p class="text-3xl font-extrabold text-indigo-600 dark:text-indigo-400">{{ elapsedTime }}</p>
        </CardContent>
      </Card>
    </div>

    <!-- TERMINAL VIEW -->
    <Card class="flex-1 flex flex-col min-h-[300px] border-slate-200 dark:border-slate-800 shadow-sm overflow-hidden">
      <CardHeader class="pb-2 flex flex-row items-center justify-between space-y-0 h-14 bg-slate-50 dark:bg-slate-900/50 border-b border-slate-100 dark:border-slate-800 shrink-0">
        <div class="flex items-center">
          <TerminalIcon class="w-4 h-4 mr-2 text-slate-500" />
          <CardTitle class="text-sm font-semibold tracking-tight">{{ $t('gui.send_log') }}</CardTitle>
        </div>
        <div class="flex items-center space-x-2">
          <Button @click="handleClearLogs" variant="ghost" size="sm" class="h-8 gap-1 text-slate-500 hover:text-slate-700 dark:hover:text-slate-300">
            <Trash2Icon class="w-3 h-3" />
            <span class="text-xs">{{ $t('gui.clear') }}</span>
          </Button>
          <Button @click="handleExportLogs" variant="outline" size="sm" class="h-8 gap-1 bg-white dark:bg-slate-950">
            <DownloadIcon class="w-3 h-3 text-slate-500" />
            <span class="text-xs">{{ $t('gui.export_log') }}</span>
          </Button>
        </div>
      </CardHeader>
      
      <CardContent class="flex-1 p-0 overflow-hidden bg-[#FAFAFA] dark:bg-[#121212]">
        <ScrollArea ref="logScrollArea" class="h-full w-full">
          <div class="p-4 font-mono text-xs space-y-2">
            <div 
              v-for="log in logs" 
              :key="log.id" 
              class="flex items-start space-x-3 py-1 border-b border-transparent hover:bg-slate-100/50 dark:hover:bg-slate-800/50 transition-colors rounded px-2"
            >
              <span class="text-slate-400 shrink-0 w-16">{{ log.time }}</span>
              <span :class="['px-1.5 py-0.5 rounded text-[9px] font-bold shrink-0 w-14 text-center', levelColors[log.level] || levelColors.INFO]">
                {{ log.level }}
              </span>
              <span class="text-slate-700 dark:text-slate-300 break-all">{{ log.message }}</span>
            </div>
            
            <div v-if="logs.length === 0" class="h-full flex items-center justify-center text-slate-400 italic">
              No logs available.
            </div>
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  </div>
</template>
