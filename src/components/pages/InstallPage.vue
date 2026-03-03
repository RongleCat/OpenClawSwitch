<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { CheckCircle, XCircle, Loader2, Circle } from 'lucide-vue-next'
import Button from '../ui/Button.vue'
import Card from '../ui/Card.vue'
import TerminalLog from '../ui/TerminalLog.vue'
import PostInstallConfig from './PostInstallConfig.vue'
import { getOnboardingPrimaryAction } from '../../domain/onboardingActions'
import type { InstallLogEvent, InstallProgressEvent, InstallDownloadEvent, InstallStepTimingEvent } from '../../types/config'

const props = withDefaults(
  defineProps<{
    mode?: 'local' | 'ssh'
    envConnected?: boolean
    openclawInstalled?: boolean
  }>(),
  {
    mode: 'local',
    envConnected: true,
    openclawInstalled: false,
  }
)

const emit = defineEmits<{
  installComplete: []
  requestConnectTarget: []
  goConfig: []
}>()

const logs = ref<InstallLogEvent[]>([])
const installing = ref(false)
const installComplete = ref(false)
const installError = ref<string | null>(null)
const showPostConfig = ref(false)

// 下载进度
const downloadProgress = ref<InstallDownloadEvent | null>(null)

// 步骤耗时
const stepTimings = ref<Map<string, InstallStepTimingEvent>>(new Map())

const steps = ref([
  { name: '环境检测', status: 'pending' as 'pending' | 'running' | 'success' | 'error', duration: 0 },
  { name: '安装 Git', status: 'pending' as 'pending' | 'running' | 'success' | 'error', duration: 0 },
  { name: '安装 Node.js', status: 'pending' as 'pending' | 'running' | 'success' | 'error', duration: 0 },
  { name: '安装 OpenClaw', status: 'pending' as 'pending' | 'running' | 'success' | 'error', duration: 0 },
  { name: '验证安装', status: 'pending' as 'pending' | 'running' | 'success' | 'error', duration: 0 },
])

let unlistenLog: (() => void) | null = null
let unlistenProgress: (() => void) | null = null
let unlistenDownload: (() => void) | null = null
let unlistenTiming: (() => void) | null = null

const primaryAction = computed(() =>
  getOnboardingPrimaryAction({
    mode: props.mode,
    openclawInstalled: props.openclawInstalled,
    envConnected: props.envConnected,
  })
)

onMounted(async () => {
  // 监听安装日志
  unlistenLog = await listen<InstallLogEvent>('install-log', (event) => {
    logs.value.push(event.payload)
  })

  // 监听安装进度
  unlistenProgress = await listen<InstallProgressEvent>('install-progress', (event) => {
    const { currentStep, stepName, status } = event.payload
    const idx = currentStep - 1
    if (idx >= 0 && idx < steps.value.length) {
      steps.value[idx].status = status as any
    }
  })

  // 监听下载进度
  unlistenDownload = await listen<InstallDownloadEvent>('install-download', (event) => {
    downloadProgress.value = event.payload
  })

  // 监听步骤耗时
  unlistenTiming = await listen<InstallStepTimingEvent>('install-step-timing', (event) => {
    const timing = event.payload
    stepTimings.value.set(timing.step, timing)

    // 更新对应步骤的耗时显示
    const stepMap: Record<string, number> = {
      'check': 0,
      'install_git': 1,
      'install_fnm': 2,  // fnm 和 node 都映射到步骤 2
      'install_node': 2,
      'install_openclaw': 3,
      'verify': 4,
    }
    const idx = stepMap[timing.step]
    if (idx !== undefined && idx < steps.value.length) {
      steps.value[idx].duration = timing.duration
    }
  })
})

onUnmounted(() => {
  unlistenLog?.()
  unlistenProgress?.()
  unlistenDownload?.()
  unlistenTiming?.()
})

const startInstall = async () => {
  if (primaryAction.value !== 'run_full_install') {
    if (primaryAction.value === 'connect_target') {
      emit('requestConnectTarget')
      return
    }
    if (primaryAction.value === 'go_config') {
      emit('goConfig')
      return
    }
    installError.value = 'SSH 模式下请在远程环境完成安装后继续配置'
    return
  }

  installing.value = true
  installError.value = null
  installComplete.value = false
  showPostConfig.value = false
  logs.value = []
  downloadProgress.value = null
  stepTimings.value.clear()

  // 重置步骤状态
  steps.value.forEach(s => {
    s.status = 'pending'
    s.duration = 0
  })

  try {
    await invoke<string>('run_full_install')
    installComplete.value = true
    showPostConfig.value = true
  } catch (e) {
    installError.value = String(e)
  } finally {
    installing.value = false
  }
}

const retryInstall = () => {
  startInstall()
}

const handlePostConfigComplete = () => {
  emit('installComplete')
}

const stepIcon = (status: string) => {
  switch (status) {
    case 'success': return CheckCircle
    case 'error': return XCircle
    case 'running': return Loader2
    default: return Circle
  }
}

const stepColor = (status: string) => {
  switch (status) {
    case 'success':
      return 'var(--oc-success)'
    case 'error':
      return 'var(--oc-danger)'
    case 'running':
      return 'var(--oc-accent)'
    default:
      return 'var(--oc-text-muted)'
  }
}

const formatDuration = (ms: number) => {
  if (ms < 1000) return `${ms}ms`
  const sec = Math.floor(ms / 1000)
  if (sec < 60) return `${sec}s`
  const min = Math.floor(sec / 60)
  return `${min}m`
}

const primaryActionText = computed(() => {
  if (installError.value) return '重试安装'
  if (primaryAction.value === 'connect_target') return '连接环境'
  if (primaryAction.value === 'show_remote_guide') return '查看远程安装指南'
  if (primaryAction.value === 'go_config') return '下一步去配置'
  return '开始安装'
})
</script>

<template>
  <PostInstallConfig
    v-if="showPostConfig"
    @complete="handlePostConfigComplete"
  />

  <div v-else class="oc-page-root flex flex-col">
    <div class="max-w-4xl mx-auto w-full flex flex-col flex-1 min-h-0">
      <div class="mb-6 flex-shrink-0">
        <h2 class="text-xl font-bold" style="color: var(--oc-text-primary);">安装 OpenClaw</h2>
        <p class="text-sm" style="color: var(--oc-text-muted);">自动检测环境并安装所有依赖</p>
      </div>

      <Card class="p-5 mb-4 flex-shrink-0">
        <div class="flex items-center gap-2">
          <template v-for="(step, i) in steps" :key="i">
            <div class="flex flex-col gap-0.5">
              <div class="flex items-center gap-2">
                <component
                  :is="stepIcon(step.status)"
                  class="w-5 h-5 flex-shrink-0"
                  :class="{ 'animate-spin': step.status === 'running' }"
                  :style="{ color: stepColor(step.status) }"
                />
                <span
                  class="text-sm whitespace-nowrap"
                  :style="{ color: stepColor(step.status), fontWeight: step.status === 'running' ? 600 : 500 }"
                >
                  {{ step.name }}
                </span>
              </div>
              <div v-if="step.duration > 0" class="ml-7 text-xs" style="color: var(--oc-text-muted);">
                {{ formatDuration(step.duration) }}
              </div>
            </div>
            <div v-if="i < steps.length - 1" class="h-px min-w-4 flex-1" style="background: var(--oc-divider-soft);" />
          </template>
        </div>

        <div v-if="downloadProgress" class="mt-4 border-t pt-4" style="border-color: var(--oc-divider-soft);">
          <div class="mb-2 flex items-center justify-between text-xs" style="color: var(--oc-text-muted);">
            <span>下载中...</span>
            <span>{{ downloadProgress.speed }} · {{ (downloadProgress.downloaded / 1048576).toFixed(1) }}MB / {{ (downloadProgress.total / 1048576).toFixed(1) }}MB</span>
          </div>
          <div class="h-2 w-full overflow-hidden rounded-full" style="background: var(--oc-divider-soft);">
            <div
              class="h-full transition-all duration-300"
              style="background: var(--oc-accent);"
              :style="{ width: `${downloadProgress.percent}%` }"
            />
          </div>
        </div>
      </Card>

      <div class="flex-1 min-h-0 mb-4">
        <TerminalLog :logs="logs" />
      </div>

      <div class="flex items-center justify-between flex-shrink-0">
        <div class="text-sm">
          <template v-if="installComplete">
            <span class="font-medium" style="color: var(--oc-success);">安装完成!</span>
          </template>
          <template v-else-if="installError">
            <span style="color: var(--oc-danger);">{{ installError }}</span>
          </template>
        </div>
        <div class="flex gap-2">
          <Button
            v-if="!installing && !installComplete"
            @click="startInstall"
          >
            {{ primaryActionText }}
          </Button>
          <Button
            v-if="installError && !installing"
            variant="outline"
            @click="retryInstall"
          >
            重试
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
