<script setup lang="ts">
import { ref } from 'vue'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { open } from '@tauri-apps/plugin-dialog'
import { FolderOpenIcon } from 'lucide-vue-next'

// SMTP Settings
const smtpServer = ref('')
const smtpPort = ref('25')
const useTls = ref(false)
const skipCert = ref(false)
const authRequired = ref(false)
const username = ref('')
const password = ref('')

// Send Settings
const sender = ref('')
const recipients = ref('')
const subject = ref('')

const sendMode = ref<'eml' | 'single' | 'dir'>('eml')
const emlDir = ref('')
const emlExtension = ref('eml')
const attachmentFile = ref('')
const attachmentDir = ref('')
const textTemplate = ref('')
const htmlTemplate = ref('')

async function handleBrowse(type: 'eml' | 'single' | 'dir' | 'failed' | 'log') {
  try {
    const selected = await open({
      directory: type !== 'single' && type !== 'log',
      multiple: false
    });
    if (selected && typeof selected === 'string') {
      if (type === 'eml') emlDir.value = selected;
      else if (type === 'single') attachmentFile.value = selected;
      else if (type === 'dir') attachmentDir.value = selected;
      else if (type === 'failed') failedDir.value = selected;
      else if (type === 'log') logFile.value = selected;
    }
  } catch (err) {
    console.error("Dialog error:", err);
  }
}

// Advanced Settings
const processes = ref('auto')
const batchSize = ref(1)
const smtpTimeout = ref(30)
const loopMode = ref(false)
const repeatCount = ref(1)
const loopInterval = ref(1)
const sendIntervalMs = ref(0)
const retryInterval = ref(5)
const failedDir = ref('')
const keepHeaders = ref(false)
const modifyHeaders = ref(false)
const envelopeCcBcc = ref(false)
const anonymizeEmails = ref(false)
const anonymizeDomain = ref('example.com')
const logLevel = ref('info')
const logFile = ref('')

function parseInteger(value: string | number, fallback: number) {
  const parsed = typeof value === 'number' ? value : Number.parseInt(value, 10)
  return Number.isFinite(parsed) ? parsed : fallback
}

function handleTestConnection() {
  alert("Test Connection: Sending check payload to backend... (Feature mapping pending real SMTP ping)");
}

defineExpose({
  buildConfig() {
    return {
      smtp_server: smtpServer.value,
      port: parseInteger(smtpPort.value, 25),
      from: sender.value || null,
      to: recipients.value || null,
      envelope_cc_bcc: envelopeCcBcc.value,
      dir: sendMode.value === 'eml' && emlDir.value ? emlDir.value : null,
      attachment: sendMode.value === 'single' && attachmentFile.value ? attachmentFile.value : null,
      attachment_dir: sendMode.value === 'dir' && attachmentDir.value ? attachmentDir.value : null,
      extension: emlExtension.value || "eml",
      processes: processes.value,
      batch_size: parseInteger(batchSize.value, 1),
      smtp_timeout: parseInteger(smtpTimeout.value, 30),
      log_level: logLevel.value,
      keep_headers: keepHeaders.value,
      anonymize_emails: anonymizeEmails.value,
      anonymize_domain: anonymizeDomain.value,
      modify_headers: modifyHeaders.value,
      loop: loopMode.value,
      repeat: parseInteger(repeatCount.value, 1),
      loop_interval: parseInteger(loopInterval.value, 1),
      retry_interval: parseInteger(retryInterval.value, 5),
      subject_template: subject.value || null,
      text_template: textTemplate.value || null,
      html_template: htmlTemplate.value || null,
      email_send_interval_ms: parseInteger(sendIntervalMs.value, 0),
      auth_mode: authRequired.value,
      username: username.value || null,
      password: password.value || null,
      use_tls: useTls.value,
      accept_invalid_certs: skipCert.value,
      failed_emails_dir: failedDir.value || null,
      log_file: logFile.value || null,
    }
  },
  loadConfig(c: any) {
    if (!c) return;
    smtpServer.value = c.smtp_server || '';
    smtpPort.value = (c.port || 25).toString();
    sender.value = c.from || '';
    recipients.value = c.to || '';
    
    if (c.dir) {
      sendMode.value = 'eml';
      emlDir.value = c.dir;
    } else if (c.attachment) {
      sendMode.value = 'single';
      attachmentFile.value = c.attachment;
    } else if (c.attachment_dir) {
      sendMode.value = 'dir';
      attachmentDir.value = c.attachment_dir;
    }
    
    emlExtension.value = c.extension || 'eml';
    textTemplate.value = c.text_template || '';

    processes.value = c.processes || 'auto';
    batchSize.value = c.batch_size || 1;
    smtpTimeout.value = c.smtp_timeout || 30;
    loopMode.value = c.loop || false;
    repeatCount.value = c.repeat || 1;
    loopInterval.value = c.loop_interval || 1;
    sendIntervalMs.value = c.email_send_interval_ms || 0;
    retryInterval.value = c.retry_interval || 5;
    keepHeaders.value = c.keep_headers || false;
    modifyHeaders.value = c.modify_headers || false;
    envelopeCcBcc.value = c.envelope_cc_bcc || false;
    anonymizeEmails.value = c.anonymize_emails || false;
    anonymizeDomain.value = c.anonymize_domain || 'example.com';
    logLevel.value = c.log_level || 'info';
    logFile.value = c.log_file || '';
    
    subject.value = c.subject_template || '';
    authRequired.value = c.auth_mode || false;
    username.value = c.username || '';
    password.value = c.password || '';
    useTls.value = c.use_tls || false;
    skipCert.value = c.accept_invalid_certs || false;
    failedDir.value = c.failed_emails_dir || '';
    htmlTemplate.value = c.html_template || '';
  },
  validate() {
    const errors: string[] = [];
    if (!smtpServer.value.trim()) errors.push('SMTP Server Address is required.');
    
    if (sendMode.value === 'eml') {
      if (!emlDir.value.trim()) errors.push('EML Directory is required for EML Mode.');
    } else {
      if (!sender.value.trim()) errors.push('Sender is required.');
      if (!recipients.value.trim()) errors.push('Recipient is required.');
      
      if (sendMode.value === 'single' && !attachmentFile.value.trim()) {
        errors.push('Attachment File is required for Single Attachment Mode.');
      }
      if (sendMode.value === 'dir' && !attachmentDir.value.trim()) {
        errors.push('Attachment Directory is required for Directory Attachment Mode.');
      }
    }

    if (authRequired.value) {
      if (!username.value.trim()) errors.push('Username is required when Authentication is enabled.');
      if (!password.value.trim()) errors.push('Password is required when Authentication is enabled.');
    }

    return errors;
  }
})
</script>

<template>
  <div class="h-full flex flex-col space-y-6">
    <Tabs defaultValue="smtp" class="w-full">
      <TabsList class="grid w-full grid-cols-3 mb-4">
        <TabsTrigger value="smtp">{{ $t('gui.smtp_server') }}</TabsTrigger>
        <TabsTrigger value="mode">{{ $t('gui.send_mode') }}</TabsTrigger>
        <TabsTrigger value="advanced">{{ $t('gui.advanced_options') }}</TabsTrigger>
      </TabsList>

      <!-- SMTP CONFIGURATION -->
      <TabsContent value="smtp" class="space-y-4">
        <Card class="border-slate-200 dark:border-slate-800 shadow-sm transition-all hover:shadow-md">
          <CardHeader>
            <CardTitle class="text-lg text-blue-600 dark:text-blue-400">{{ $t('gui.smtp_server') }}</CardTitle>
            <CardDescription>{{ $t('gui.server_address') }} Settings.</CardDescription>
          </CardHeader>
          <CardContent class="space-y-4">
            <div class="grid grid-cols-4 gap-4">
              <div class="col-span-3 space-y-2">
                <Label for="server" class="flex items-center gap-1">{{ $t('gui.server_address') }} <span class="text-red-500">*</span></Label>
                <Input id="server" v-model="smtpServer" placeholder="smtp.example.com" />
              </div>
              <div class="space-y-2">
                <Label for="port">{{ $t('gui.port') }}</Label>
                <Input id="port" v-model="smtpPort" placeholder="25" />
              </div>
            </div>

            <div class="flex items-center space-x-6 pt-2 pb-2">
              <div class="flex items-center space-x-2">
                <Switch id="tls" :checked="useTls" @update:checked="useTls = $event" />
                <Label for="tls">{{ $t('gui.use_tls') }}</Label>
              </div>
              <div class="flex items-center space-x-2">
                <Switch id="skipCert" :checked="skipCert" @update:checked="skipCert = $event" />
                <Label for="skipCert">{{ $t('gui.accept_invalid_certs') }}</Label>
              </div>
            </div>
            
            <div class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-800">
              <div class="flex items-center space-x-2 mb-2">
                <Switch id="auth" :checked="authRequired" @update:checked="authRequired = $event" />
                <Label for="auth" class="font-semibold">{{ $t('gui.auth_required') }}</Label>
              </div>
              
              <div v-if="authRequired" class="grid grid-cols-2 gap-4 animate-in fade-in slide-in-from-top-2 duration-300">
                <div class="space-y-2">
                  <Label for="username" class="flex items-center gap-1">{{ $t('gui.username') }} <span class="text-red-500">*</span></Label>
                  <Input id="username" v-model="username" />
                </div>
                <div class="space-y-2">
                  <Label for="password" class="flex items-center gap-1">{{ $t('gui.password') }} <span class="text-red-500">*</span></Label>
                  <Input id="password" type="password" v-model="password" />
                </div>
              </div>
            </div>

            <div class="pt-4 border-t border-slate-100 dark:border-slate-800 flex justify-end">
              <Button @click="handleTestConnection" variant="secondary" size="sm" class="h-9 gap-2">
                {{ $t('gui.test_connection') }}
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card class="border-slate-200 dark:border-slate-800 shadow-sm transition-all hover:shadow-md">
          <CardHeader>
            <CardTitle class="text-lg text-blue-600 dark:text-blue-400">Routing Details</CardTitle>
          </CardHeader>
          <CardContent class="space-y-4">
             <div class="space-y-2">
                <Label for="sender" class="flex items-center gap-1">
                  {{ $t('gui.sender') }} 
                  <span v-if="sendMode !== 'eml'" class="text-red-500">*</span>
                </Label>
                <Input id="sender" v-model="sender" placeholder="noreply@domain.com" />
              </div>
              <div class="space-y-2">
                <Label for="recipients" class="flex items-center gap-1">
                  {{ $t('gui.recipient') }} 
                  <span v-if="sendMode !== 'eml'" class="text-red-500">*</span> 
                  <span class="text-slate-400 font-normal ml-1">{{ $t('gui.recipient_hint') }}</span>
                </Label>
                <Input id="recipients" v-model="recipients" placeholder="user1@a.com, user2@b.com" />
              </div>
          </CardContent>
        </Card>
      </TabsContent>

      <!-- SEND MODE CONFIGURATION -->
      <TabsContent value="mode">
        <Card class="border-slate-200 dark:border-slate-800 shadow-sm">
           <CardHeader>
            <CardTitle class="text-lg text-blue-600 dark:text-blue-400">{{ $t('gui.send_mode') }}</CardTitle>
            <CardDescription>Select how emails should be constructed and sent.</CardDescription>
          </CardHeader>
          <CardContent class="space-y-4">
             <div class="space-y-2 mb-4">
                <select v-model="sendMode" class="w-full h-10 px-3 py-2 rounded-md border border-slate-200 bg-white text-sm dark:border-slate-800 dark:bg-slate-950">
                  <option value="eml">{{ $t('gui.eml_batch') }}</option>
                  <option value="single">{{ $t('gui.single_attachment') }}</option>
                  <option value="dir">{{ $t('gui.dir_attachment') }}</option>
                </select>
             </div>

             <div v-if="sendMode === 'eml'" class="space-y-4 animate-in fade-in">
                <div class="space-y-2">
                  <Label class="flex items-center gap-1">{{ $t('gui.eml_directory') }} <span class="text-red-500">*</span></Label>
                  <div class="flex space-x-2">
                    <Input v-model="emlDir" readonly class="flex-1" />
                    <Button @click="handleBrowse('eml')" variant="outline"><FolderOpenIcon class="w-4 h-4 mr-2" />{{ $t('gui.browse') }}</Button>
                  </div>
                </div>
                <div class="space-y-2">
                  <Label>{{ $t('gui.extension') }}</Label>
                  <Input v-model="emlExtension" class="w-24" />
                </div>
             </div>

             <div v-else-if="sendMode === 'single'" class="space-y-4 animate-in fade-in">
                <div class="space-y-2">
                  <Label class="flex items-center gap-1">{{ $t('gui.attachment_file') }} <span class="text-red-500">*</span></Label>
                  <div class="flex space-x-2">
                    <Input v-model="attachmentFile" readonly class="flex-1" />
                    <Button @click="handleBrowse('single')" variant="outline"><FolderOpenIcon class="w-4 h-4 mr-2" />{{ $t('gui.browse') }}</Button>
                  </div>
                </div>
             </div>

             <div v-else-if="sendMode === 'dir'" class="space-y-4 animate-in fade-in">
                <div class="space-y-2">
                  <Label class="flex items-center gap-1">{{ $t('gui.attachment_directory') }} <span class="text-red-500">*</span></Label>
                  <div class="flex space-x-2">
                    <Input v-model="attachmentDir" readonly class="flex-1" />
                    <Button @click="handleBrowse('dir')" variant="outline"><FolderOpenIcon class="w-4 h-4 mr-2" />{{ $t('gui.browse') }}</Button>
                  </div>
                </div>
             </div>

             <div v-if="sendMode !== 'eml'" class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-800">
                <div class="space-y-2">
                  <Label for="subject">{{ $t('gui.email_subject') }}</Label>
                  <Input id="subject" v-model="subject" />
                  <p class="text-xs text-slate-500">{{ $t('gui.filename_hint') }}</p>
                </div>
                <div class="space-y-2">
                  <Label for="body">{{ $t('gui.email_body') }} (Text)</Label>
                  <Input id="body" v-model="textTemplate" />
                </div>
                <div class="space-y-2">
                  <Label for="htmlBody">{{ $t('gui.email_body') }} (HTML)</Label>
                  <Input id="htmlBody" v-model="htmlTemplate" />
                </div>
             </div>
          </CardContent>
        </Card>
      </TabsContent>

      <!-- ADVANCED CONFIGURATION -->
      <TabsContent value="advanced">
         <Card class="border-slate-200 dark:border-slate-800 shadow-sm">
           <CardHeader>
            <CardTitle class="text-lg text-blue-600 dark:text-blue-400">{{ $t('gui.advanced_options') }}</CardTitle>
          </CardHeader>
          <CardContent class="space-y-6">
             <!-- Performance -->
             <div class="space-y-4">
                <h3 class="text-sm font-semibold text-slate-900 dark:text-slate-100">{{ $t('gui.performance') }}</h3>
                <div class="grid grid-cols-2 gap-4">
                  <div class="space-y-2">
                    <Label>{{ $t('gui.processes') }}</Label>
                    <Input v-model="processes" placeholder="auto" />
                  </div>
                  <div class="space-y-2">
                    <Label>{{ $t('gui.batch_size') }}</Label>
                    <Input type="number" v-model="batchSize" />
                  </div>
                  <div class="space-y-2">
                    <Label>{{ $t('gui.timeout') }}</Label>
                    <Input type="number" v-model="smtpTimeout" />
                  </div>
                  <div class="space-y-2">
                    <Label>{{ $t('gui.send_interval') }}</Label>
                    <Input type="number" v-model="sendIntervalMs" />
                  </div>
                </div>
             </div>

             <!-- Looping -->
             <div class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-800">
                <h3 class="text-sm font-semibold text-slate-900 dark:text-slate-100">{{ $t('gui.loop_settings') }}</h3>
                <div class="flex items-center space-x-2 mb-2">
                  <Switch id="loop" :checked="loopMode" @update:checked="loopMode = $event" />
                  <Label for="loop">{{ $t('gui.infinite_loop') }}</Label>
                </div>
                <div class="grid grid-cols-2 gap-4">
                  <div class="space-y-2">
                    <Label>{{ $t('gui.repeat_count') }}</Label>
                    <Input type="number" v-model="repeatCount" :disabled="loopMode" />
                  </div>
                  <div class="space-y-2">
                    <Label>{{ $t('gui.loop_interval') }}</Label>
                    <Input type="number" v-model="loopInterval" />
                  </div>
                  <div class="space-y-2">
                    <Label>{{ $t('gui.retry_interval') || 'Retry Interval' }}</Label>
                    <Input type="number" v-model="retryInterval" />
                  </div>
                </div>
             </div>

             <!-- Email Processing -->
             <div v-if="sendMode === 'eml'" class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-800 animate-in fade-in">
                <h3 class="text-sm font-semibold text-slate-900 dark:text-slate-100">{{ $t('gui.email_processing') }}</h3>
                <div class="grid grid-cols-2 gap-4">
                  <div class="flex items-center space-x-2">
                    <Switch id="keepHeaders" :checked="keepHeaders" @update:checked="keepHeaders = $event" />
                    <Label for="keepHeaders">{{ $t('gui.keep_headers') }}</Label>
                  </div>
                  <div class="flex items-center space-x-2">
                    <Switch id="modifyHeaders" :checked="modifyHeaders" @update:checked="modifyHeaders = $event" />
                    <Label for="modifyHeaders">{{ $t('gui.modify_headers') }}</Label>
                  </div>
                  <div class="flex items-center space-x-2">
                    <Switch id="envelopeCcBcc" :checked="envelopeCcBcc" @update:checked="envelopeCcBcc = $event" />
                    <Label for="envelopeCcBcc">{{ $t('gui.envelope_cc_bcc') }}</Label>
                  </div>
                  <div class="flex flex-col space-y-2">
                    <div class="flex items-center space-x-2">
                      <Switch id="anonymizeEmails" :checked="anonymizeEmails" @update:checked="anonymizeEmails = $event" />
                      <Label for="anonymizeEmails">{{ $t('gui.anonymize_emails') }}</Label>
                    </div>
                    <div v-if="anonymizeEmails" class="pt-2 animate-in fade-in slide-in-from-top-2">
                      <Input v-model="anonymizeDomain" placeholder="example.com" class="h-8 text-xs" />
                    </div>
                  </div>
                </div>
             </div>

             <!-- Logging -->
             <div class="space-y-4 pt-4 border-t border-slate-100 dark:border-slate-800">
                <h3 class="text-sm font-semibold text-slate-900 dark:text-slate-100">{{ $t('gui.logging') }}</h3>
                
                <div class="grid grid-cols-2 gap-4">
                  <div class="space-y-2">
                    <Label>{{ $t('gui.log_level') }}</Label>
                    <select v-model="logLevel" class="w-full h-10 px-3 py-2 rounded-md border border-slate-200 bg-white text-sm dark:border-slate-800 dark:bg-slate-950">
                      <option value="trace">Trace</option>
                      <option value="debug">Debug</option>
                      <option value="info">Info</option>
                      <option value="warn">Warn</option>
                      <option value="error">Error</option>
                    </select>
                  </div>
                  <div class="space-y-2">
                    <Label>{{ $t('gui.log_file') || 'Log File' }}</Label>
                    <div class="flex space-x-2">
                      <Input v-model="logFile" placeholder="Optional..." readonly class="flex-1" />
                      <Button @click="handleBrowse('log')" variant="outline"><FolderOpenIcon class="w-4 h-4 mr-2" />{{ $t('gui.browse') }}</Button>
                    </div>
                  </div>
                </div>

                <div class="space-y-2 pt-2">
                  <Label>{{ $t('gui.failed_emails_dir') }}</Label>
                  <div class="flex space-x-2">
                    <Input v-model="failedDir" readonly class="flex-1" />
                    <Button @click="handleBrowse('failed')" variant="outline"><FolderOpenIcon class="w-4 h-4 mr-2" />{{ $t('gui.browse') }}</Button>
                  </div>
               </div>
             </div>
          </CardContent>
        </Card>
      </TabsContent>
    </Tabs>
  </div>
</template>
