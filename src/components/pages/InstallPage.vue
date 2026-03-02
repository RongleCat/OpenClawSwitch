<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { CheckCircle, XCircle, Loader2, Circle } from 'lucide-vue-next'
import Button from '../ui/Button.vue'
import Card from '../ui/Card.vue'
import TerminalLog from '../ui/TerminalLog.vue'
import PostInstallConfig from './PostInstallConfig.vue'
import type { InstallLogEvent, InstallProgressEvent, InstallDownloadEvent, InstallStepTimingEvent } from '../../types/config'

const emit = defineEmits<{
  installComplete: []
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
    case 'success': return 'text-green-500'
    case 'error': return 'text-red-500'
    case 'running': return 'text-blue-500'
    default: return 'text-gray-300 dark:text-gray-600'
  }
}

const formatDuration = (ms: number) => {
  if (ms < 1000) return `${ms}ms`
  const sec = Math.floor(ms / 1000)
  if (sec < 60) return `${sec}s`
  const min = Math.floor(sec / 60)
  return `${min}m`
}
</script>

<template>
  <!-- 安装后配置界面 -->
  <PostInstallConfig
    v-if="showPostConfig"
    @complete="handlePostConfigComplete"
  />

  <!-- 安装界面 -->
  <div v-else class="h-full flex flex-col bg-gray-50 p-8">
    <div class="max-w-4xl mx-auto w-full flex flex-col flex-1 min-h-0">
      <!-- 标题栏 -->
      <div class="mb-6 flex-shrink-0">
        <h2 class="text-xl font-bold text-gray-900">安装 OpenClaw</h2>
        <p class="text-sm text-gray-600">自动检测环境并安装所有依赖</p>
      </div>

      <!-- 步骤进度 -->
      <Card class="p-5 mb-4 flex-shrink-0 bg-white shadow-sm">
        <div class="flex items-center gap-2">
          <template v-for="(step, i) in steps" :key="i">
            <div class="flex flex-col gap-0.5">
              <div class="flex items-center gap-2">
                <component
                  :is="stepIcon(step.status)"
                  class="w-5 h-5 flex-shrink-0"
                  :class="[stepColor(step.status), { 'animate-spin': step.status === 'running' }]"
                />
                <span
                  class="text-sm whitespace-nowrap"
                  :class="step.status === 'running' ? 'font-medium text-blue-600' : step.status === 'success' ? 'text-green-600' : step.status === 'error' ? 'text-red-600' : 'text-gray-500'"
                >
                  {{ step.name }}
                </span>
              </div>
              <div v-if="step.duration > 0" class="text-xs text-gray-500 ml-7">
                {{ formatDuration(step.duration) }}
              </div>
            </div>
            <div v-if="i < steps.length - 1" class="flex-1 h-px bg-gray-200 min-w-4" />
          </template>
        </div>

        <!-- 下载进度条 -->
        <div v-if="downloadProgress" class="mt-4 pt-4 border-t border-gray-200">
          <div class="flex items-center justify-between text-xs text-gray-600 mb-2">
            <span>下载中...</span>
            <span>{{ downloadProgress.speed }} · {{ (downloadProgress.downloaded / 1048576).toFixed(1) }}MB / {{ (downloadProgress.total / 1048576).toFixed(1) }}MB</span>
          </div>
          <div class="w-full bg-gray-200 rounded-full h-2 overflow-hidden">
            <div
              class="bg-blue-500 h-full transition-all duration-300"
              :style="{ width: `${downloadProgress.percent}%` }"
            />
          </div>
        </div>
      </Card>

      <!-- 终端日志 -->
      <div class="flex-1 min-h-0 mb-4">
        <TerminalLog :logs="logs" />
      </div>

      <!-- 操作按钮 -->
      <div class="flex items-center justify-between flex-shrink-0">
        <div class="text-sm">
          <template v-if="installComplete">
            <span class="text-green-600 font-medium">安装完成!</span>
          </template>
          <template v-else-if="installError">
            <span class="text-red-500">{{ installError }}</span>
          </template>
        </div>
        <div class="flex gap-2">
          <Button
            v-if="!installing && !installComplete"
            @click="startInstall"
            class="bg-blue-600 hover:bg-blue-700 text-white"
          >
            {{ installError ? '重试安装' : '开始安装' }}
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
