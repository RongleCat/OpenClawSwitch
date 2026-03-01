<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { CheckCircle, XCircle, Loader2, Circle } from 'lucide-vue-next'
import Button from '../ui/Button.vue'
import Card from '../ui/Card.vue'
import TerminalLog from '../ui/TerminalLog.vue'
import type { InstallLogEvent, InstallProgressEvent } from '../../types/config'

const emit = defineEmits<{
  installComplete: []
}>()

const logs = ref<InstallLogEvent[]>([])
const installing = ref(false)
const installComplete = ref(false)
const installError = ref<string | null>(null)

const steps = ref([
  { name: '环境检测', status: 'pending' as 'pending' | 'running' | 'success' | 'error' },
  { name: '安装 Git', status: 'pending' as 'pending' | 'running' | 'success' | 'error' },
  { name: '安装 fnm', status: 'pending' as 'pending' | 'running' | 'success' | 'error' },
  { name: '安装 Node.js', status: 'pending' as 'pending' | 'running' | 'success' | 'error' },
  { name: '安装 OpenClaw', status: 'pending' as 'pending' | 'running' | 'success' | 'error' },
  { name: '验证安装', status: 'pending' as 'pending' | 'running' | 'success' | 'error' },
])

let unlistenLog: (() => void) | null = null
let unlistenProgress: (() => void) | null = null

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
})

onUnmounted(() => {
  unlistenLog?.()
  unlistenProgress?.()
})

const startInstall = async () => {
  installing.value = true
  installError.value = null
  installComplete.value = false
  logs.value = []

  // 重置步骤状态
  steps.value.forEach(s => s.status = 'pending')

  try {
    await invoke<string>('run_full_install')
    installComplete.value = true
  } catch (e) {
    installError.value = String(e)
  } finally {
    installing.value = false
  }
}

const retryInstall = () => {
  startInstall()
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
</script>

<template>
  <div class="h-full flex flex-col p-6">
    <div class="max-w-4xl mx-auto w-full flex flex-col flex-1 min-h-0">
      <!-- 标题栏 -->
      <div class="mb-6 flex-shrink-0">
        <h2 class="text-xl font-bold">安装 OpenClaw</h2>
        <p class="text-sm text-muted-foreground">自动检测环境并安装所有依赖</p>
      </div>

      <!-- 步骤进度 -->
      <Card class="p-4 mb-4 flex-shrink-0">
        <div class="flex items-center gap-2">
          <template v-for="(step, i) in steps" :key="i">
            <div class="flex items-center gap-1.5">
              <component
                :is="stepIcon(step.status)"
                class="w-5 h-5 flex-shrink-0"
                :class="[stepColor(step.status), { 'animate-spin': step.status === 'running' }]"
              />
              <span
                class="text-sm whitespace-nowrap"
                :class="step.status === 'running' ? 'font-medium text-blue-600' : step.status === 'success' ? 'text-green-600' : step.status === 'error' ? 'text-red-600' : 'text-muted-foreground'"
              >
                {{ step.name }}
              </span>
            </div>
            <div v-if="i < steps.length - 1" class="flex-1 h-px bg-gray-200 dark:bg-gray-700 min-w-4" />
          </template>
        </div>
      </Card>

      <!-- 终端日志 -->
      <div class="flex-1 min-h-0 mb-4">
        <TerminalLog :logs="logs" />
      </div>

      <!-- 操作按钮 -->
      <div class="flex items-center justify-between flex-shrink-0">
        <div class="text-sm text-muted-foreground">
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
          <Button
            v-if="installComplete"
            @click="emit('installComplete')"
            class="bg-green-600 hover:bg-green-700 text-white"
          >
            完成，进入配置
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
