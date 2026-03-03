<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import {
  Activity,
  AlertCircle,
  Stethoscope,
  Wrench,
} from 'lucide-vue-next'
import Button from '../ui/Button.vue'
import { extractDoctorIssues, type DoctorLogLine } from '../../domain/doctorIssues'

interface DoctorStatusEvent {
  running: boolean
  mode?: 'check' | 'fix'
  success?: boolean | null
  exitCode?: number | null
  reason?: string | null
}

const props = defineProps<{
  appState: 'READY' | 'DEGRADED' | 'ERROR'
  envMode: 'local' | 'ssh'
  openclawInstalled: boolean
}>()

const emit = defineEmits<{
  refresh: []
}>()

const logs = ref<DoctorLogLine[]>([])
const doctorRunning = ref(false)
const runningMode = ref<'check' | 'fix' | null>(null)
const statusText = ref('等待执行')
const logContainerRef = ref<HTMLDivElement>()

let unlistenDoctorLine: (() => void) | null = null
let unlistenDoctorStatus: (() => void) | null = null

const isLocalMode = computed(() => props.envMode === 'local')

const canRunDoctor = computed(
  () => props.openclawInstalled && isLocalMode.value && !doctorRunning.value
)

const actionHint = computed(() => {
  if (!props.openclawInstalled) {
    return '当前环境未安装 OpenClaw，暂不可执行服务诊断。'
  }
  if (!isLocalMode.value) {
    return 'SSH 环境暂不支持内置诊断，请在远端执行 openclaw doctor / openclaw doctor --fix。'
  }
  if (doctorRunning.value) {
    return runningMode.value === 'fix' ? '自动修复执行中...' : '服务诊断执行中...'
  }
  if (props.appState === 'ERROR') {
    return '检测到服务异常，建议先执行立即诊断。'
  }
  if (props.appState === 'DEGRADED') {
    return '当前处于降级状态，建议执行诊断并确认关键依赖。'
  }
  return '服务状态可用，可按需执行诊断或自动修复。'
})

const extractedIssues = computed(() => extractDoctorIssues(logs.value))

const runStateLabel = computed(() => {
  if (doctorRunning.value) {
    return runningMode.value === 'fix' ? '自动修复中' : '诊断中'
  }
  return '空闲'
})

const appendDoctorLog = (payload: DoctorLogLine) => {
  logs.value.push(payload)
  if (logs.value.length > 1200) {
    logs.value.shift()
  }
}

const levelColor = (value: string) => {
  if (value === 'error') return 'var(--oc-danger)'
  if (value === 'warn') return 'var(--oc-warning)'
  if (value === 'success') return 'var(--oc-success)'
  return 'var(--oc-text-secondary)'
}

const levelBadgeStyle = (value: 'error' | 'warn') => {
  if (value === 'error') {
    return {
      color: 'var(--oc-danger)',
      background: 'color-mix(in srgb, var(--oc-danger) 18%, transparent)',
      borderColor: 'color-mix(in srgb, var(--oc-danger) 35%, transparent)',
    }
  }
  return {
    color: 'var(--oc-warning)',
    background: 'color-mix(in srgb, var(--oc-warning) 18%, transparent)',
    borderColor: 'color-mix(in srgb, var(--oc-warning) 35%, transparent)',
  }
}

const formatTime = (value: number) =>
  new Date(value).toLocaleTimeString('zh-CN', { hour12: false })

const runDoctor = async (fix: boolean) => {
  if (!canRunDoctor.value) {
    return
  }

  statusText.value = fix ? '开始执行自动修复...' : '开始执行服务诊断...'

  try {
    const started = await invoke<boolean>('start_openclaw_doctor', { fix })
    if (!started) {
      doctorRunning.value = true
      statusText.value = '已有诊断任务在运行'
      appendDoctorLog({
        message: '已有诊断任务在运行，请稍候完成后再试。',
        level: 'warn',
        timestamp: Date.now(),
      })
      return
    }

    logs.value = []
    doctorRunning.value = true
    runningMode.value = fix ? 'fix' : 'check'
  } catch (error) {
    doctorRunning.value = false
    statusText.value = '启动失败'
    appendDoctorLog({
      message: `启动诊断失败: ${String(error)}`,
      level: 'error',
      timestamp: Date.now(),
    })
  }
}

watch(
  () => logs.value.length,
  async () => {
    await nextTick()
    if (logContainerRef.value) {
      logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight
    }
  }
)

watch(doctorRunning, (running, prev) => {
  if (prev && !running) {
    emit('refresh')
  }
})

onMounted(async () => {
  unlistenDoctorLine = await listen<DoctorLogLine>('openclaw-doctor-line', (event) => {
    appendDoctorLog(event.payload)
  })

  unlistenDoctorStatus = await listen<DoctorStatusEvent>('openclaw-doctor-status', (event) => {
    doctorRunning.value = event.payload.running
    runningMode.value = event.payload.mode || null
    if (event.payload.reason) {
      statusText.value = event.payload.reason
    }
  })

  try {
    doctorRunning.value = await invoke<boolean>('is_openclaw_doctor_running')
    if (doctorRunning.value) {
      statusText.value = '已有诊断任务在运行'
    }
  } catch {
    doctorRunning.value = false
  }
})

onUnmounted(() => {
  unlistenDoctorLine?.()
  unlistenDoctorStatus?.()
})
</script>

<template>
  <div class="oc-page-root flex min-h-0 flex-1 flex-col gap-2">
    <section class="oc-panel shrink-0 p-4">
      <div class="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">服务诊断</h3>
          <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">{{ actionHint }}</p>
          <p class="mt-2 text-xs" style="color: var(--oc-text-quiet);">
            状态：{{ runStateLabel }} · {{ statusText }}
          </p>
        </div>

        <div class="flex flex-wrap items-center gap-2">
          <Button :disabled="!canRunDoctor" @click="runDoctor(false)">
            <Stethoscope class="h-4 w-4" />
            立即诊断
          </Button>
          <Button variant="secondary" :disabled="!canRunDoctor" @click="runDoctor(true)">
            <Wrench class="h-4 w-4" />
            自动修复
          </Button>
        </div>
      </div>
    </section>

    <div class="grid min-h-0 flex-1 gap-2 xl:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
      <section class="oc-panel min-h-0 flex flex-col">
        <div class="flex items-center justify-between border-b px-4 py-3" style="border-color: var(--oc-divider-soft);">
          <div class="flex items-center gap-2 text-sm" style="color: var(--oc-text-secondary);">
            <Activity class="h-4 w-4" />
            <span class="font-semibold">实时诊断日志</span>
            <span style="color: var(--oc-text-muted);">({{ logs.length }} 行)</span>
          </div>
        </div>

        <div class="flex min-h-0 flex-1 px-4 pb-4 pt-3">
          <div
            ref="logContainerRef"
            class="min-h-0 flex-1 overflow-auto rounded-[12px] border p-3 font-mono text-xs leading-6"
            style="border-color: var(--oc-card-border); background: color-mix(in srgb, var(--oc-card-elevated) 88%, transparent);"
          >
            <div v-if="!openclawInstalled" class="py-8 text-center" style="color: var(--oc-text-muted);">
              当前环境未安装 OpenClaw，暂不可执行诊断。
            </div>
            <div v-else-if="!isLocalMode" class="py-8 text-center" style="color: var(--oc-text-muted);">
              SSH 环境暂不支持内置诊断，请在远端执行 openclaw doctor。
            </div>
            <div v-else-if="logs.length === 0" class="py-8 text-center" style="color: var(--oc-text-muted);">
              等待诊断输出...
            </div>
            <div v-else>
              <div v-for="(line, index) in logs" :key="`${line.timestamp}-${index}`" class="whitespace-pre-wrap break-words">
                <span class="mr-2 select-none text-[11px]" style="color: var(--oc-text-quiet);">{{ formatTime(line.timestamp) }}</span>
                <span :style="{ color: levelColor(line.level) }">{{ line.message }}</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section class="oc-panel min-h-0 flex flex-col">
        <div class="flex items-center justify-between border-b px-4 py-3" style="border-color: var(--oc-divider-soft);">
          <div class="flex items-center gap-2 text-sm" style="color: var(--oc-text-secondary);">
            <AlertCircle class="h-4 w-4" />
            <span class="font-semibold">提取问题</span>
            <span style="color: var(--oc-text-muted);">({{ extractedIssues.length }})</span>
          </div>
        </div>

        <div class="min-h-0 flex-1 overflow-y-auto overflow-x-hidden px-4 pb-4 pt-3">
          <div v-if="extractedIssues.length === 0" class="rounded-[12px] border p-3 text-sm" style="border-color: var(--oc-card-border); color: var(--oc-text-muted);">
            当前输出中未提取到明显错误。
          </div>

          <div v-else class="space-y-2">
            <article
              v-for="issue in extractedIssues"
              :key="issue.id"
              class="rounded-[12px] border p-3"
              style="border-color: var(--oc-card-border); background: color-mix(in srgb, var(--oc-card-elevated) 82%, transparent);"
            >
              <div class="mb-2 flex items-center justify-between gap-2">
                <span
                  class="rounded-full border px-2 py-0.5 text-[11px] font-semibold"
                  :style="levelBadgeStyle(issue.level)"
                >
                  {{ issue.level === 'error' ? '错误' : '警告' }}
                </span>
                <span class="text-[11px]" style="color: var(--oc-text-quiet);">{{ formatTime(issue.timestamp) }}</span>
              </div>
              <p class="break-words text-sm font-medium" :style="{ color: issue.level === 'error' ? 'var(--oc-danger)' : 'var(--oc-warning)' }">
                {{ issue.message }}
              </p>
              <pre class="mt-2 max-w-full overflow-x-auto rounded-[10px] border px-2 py-2 text-xs leading-5 whitespace-pre" style="border-color: var(--oc-divider-soft); color: var(--oc-text-secondary);">{{ issue.context }}</pre>
            </article>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
