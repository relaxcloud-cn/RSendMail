<script setup lang="ts">
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { CircleHelpIcon } from 'lucide-vue-next'
</script>

<template>
  <Dialog>
    <DialogTrigger asChild>
      <Button variant="ghost" size="icon" class="h-8 w-8 text-slate-500 rounded-full hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors">
        <CircleHelpIcon class="h-5 w-5" />
      </Button>
    </DialogTrigger>
    <DialogContent class="sm:max-w-[650px] h-auto max-h-[85vh] overflow-y-auto bg-white/95 dark:bg-slate-950/95 backdrop-blur-xl border border-slate-200 dark:border-slate-800 shadow-2xl">
      <DialogHeader>
        <DialogTitle class="text-xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-600 to-indigo-600 dark:from-blue-400 dark:to-indigo-400">{{ $t('gui.guide.title') }}</DialogTitle>
        <DialogDescription class="text-slate-500 dark:text-slate-400">
          {{ $t('gui.guide.desc') }}
        </DialogDescription>
      </DialogHeader>
      
      <div class="space-y-6 mt-4 text-sm text-slate-700 dark:text-slate-300 isolate">
        <section class="p-4 rounded-lg bg-slate-50 dark:bg-slate-900/50 border border-slate-100 dark:border-slate-800/60 shadow-inner">
          <h3 class="font-semibold text-base text-slate-900 dark:text-white mb-2.5 flex items-center">
             <span class="w-6 h-6 rounded bg-blue-100 dark:bg-blue-900/50 text-blue-600 dark:text-blue-400 flex items-center justify-center mr-2 text-xs">1</span> 
             {{ $t('gui.guide.smtp_title') }}
          </h3>
          <div v-html="$t('gui.guide.smtp_desc')"></div>
        </section>

        <section class="p-4 rounded-lg bg-slate-50 dark:bg-slate-900/50 border border-slate-100 dark:border-slate-800/60 shadow-inner">
          <h3 class="font-semibold text-base text-slate-900 dark:text-white mb-2.5 flex items-center">
             <span class="w-6 h-6 rounded bg-emerald-100 dark:bg-emerald-900/50 text-emerald-600 dark:text-emerald-400 flex items-center justify-center mr-2 text-xs">2</span> 
             {{ $t('gui.guide.modes_title') }}
          </h3>
          <div v-html="$t('gui.guide.modes_desc')"></div>
        </section>

        <section class="p-4 rounded-lg bg-slate-50 dark:bg-slate-900/50 border border-slate-100 dark:border-slate-800/60 shadow-inner">
          <h3 class="font-semibold text-base text-slate-900 dark:text-white mb-2.5 flex items-center">
             <span class="w-6 h-6 rounded bg-amber-100 dark:bg-amber-900/50 text-amber-600 dark:text-amber-400 flex items-center justify-center mr-2 text-xs">3</span> 
             {{ $t('gui.guide.adv_title') }}
          </h3>
          <div v-html="$t('gui.guide.adv_desc')"></div>
        </section>

        <section class="p-4 rounded-lg bg-slate-50 dark:bg-slate-900/50 border border-slate-100 dark:border-slate-800/60 shadow-inner">
          <h3 class="font-semibold text-base text-slate-900 dark:text-white mb-2.5 flex items-center">
             <span class="w-6 h-6 rounded bg-rose-100 dark:bg-rose-900/50 text-rose-600 dark:text-rose-400 flex items-center justify-center mr-2 text-xs">4</span> 
             {{ $t('gui.guide.loop_title') }}
          </h3>
          <div v-html="$t('gui.guide.loop_desc')"></div>
        </section>
      </div>

    </DialogContent>
  </Dialog>
</template>
