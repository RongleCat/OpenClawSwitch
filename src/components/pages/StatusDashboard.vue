<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { Play, Square, RotateCcw, Stethoscope, ChevronDown, ChevronUp, RefreshCw, Wrench, Loader2 } from 'lucide-vue-next'
import {
  resolveGatewayQuickActionGridColumns,
  resolveGatewayQuickActionState,
  shouldShowInstallGatewayServiceAction,
} from '../../domain/gatewayServiceAction'
import { resolveAsyncButtonLabel, resolveAsyncButtonState } from '../../domain/asyncButtonState'
import type { EnvironmentStatus } from '../../types/config'

interface DashboardLogEvent {
  message: string
  level: 'info' | 'warn' | 'error' | 'success'
  timestamp: number
}

interface DashboardLogStatusEvent {
  running: boolean
  reason?: string | null
}

const props = withDefaults(defineProps<{
  envStatus: EnvironmentStatus
  gatewayReachable: boolean
  envMode: 'local' | 'ssh'
  isWindows?: boolean
  gatewayServiceInstalled?: boolean
  pendingToolId?: string | null
}>(), {
  isWindows: false,
  gatewayServiceInstalled: true,
  pendingToolId: null,
})

const emit = defineEmits<{
  openTool: [toolId: string]
}>()

const logExpanded = ref(true)
const logs = ref<DashboardLogEvent[]>([])
const logsFollowing = ref(false)
const refreshingLogs = ref(false)
const logContainerRef = ref<HTMLDivElement>()

let unlistenLogLine: (() => void) | null = null
let unlistenLogStatus: (() => void) | null = null

const serviceStatus = computed(() => {
  if (!props.envStatus.openclaw.installed) {
    return { label: '未安装', color: 'var(--oc-danger)', dotClass: 'oc-status-danger' }
  }
  if (props.gatewayReachable) {
    return { label: '运行中', color: 'var(--oc-success)', dotClass: 'oc-status-success' }
  }
  return { label: '已安装未运行', color: 'var(--oc-warning)', dotClass: 'oc-status-warning' }
})

const serviceSummary = computed(() => {
  const version = props.envStatus.openclaw.version || '--'
  const node = props.envStatus.node.installed ? `v${props.envStatus.node.version || '--'}` : '未安装'
  const gateway = props.gatewayReachable ? '运行中' : '未运行'
  const environment = props.envMode === 'ssh' ? 'SSH' : '本地'
  return `OpenClaw ${version} · Node.js ${node} · 网关 ${gateway} · 环境 ${environment}`
})

const quickActions = computed(() => {
  const installed = props.envStatus.openclaw.installed
  const running = props.gatewayReachable
  const actions = [
    {
      id: running ? 'stop' : 'start',
      label: running ? '停止服务' : '启动服务',
      icon: running ? Square : Play,
      color: running ? 'var(--oc-danger)' : 'var(--oc-success)',
      disabled: !installed,
    },
    {
      id: 'restart',
      label: '重启服务',
      icon: RotateCcw,
      color: 'var(--oc-warning)',
      disabled: !installed || !running,
    },
    {
      id: 'doctor',
      label: '服务诊断',
      icon: Stethoscope,
      color: 'var(--oc-accent)',
      disabled: !installed || props.envMode !== 'local',
    },
  ]

  if (
    shouldShowInstallGatewayServiceAction({
      isWindows: props.isWindows,
      envMode: props.envMode,
      gatewayServiceInstalled: props.gatewayServiceInstalled,
      gatewayReachable: props.gatewayReachable,
    })
  ) {
    actions.push({
      id: 'install-service',
      label: '安装网关服务',
      icon: Wrench,
      color: 'var(--oc-accent)',
      disabled: !installed,
    })
  }

  return actions.map((action) => ({
    ...action,
    ...resolveGatewayQuickActionState({
      actionId: action.id,
      baseDisabled: action.disabled,
      pendingActionId: props.pendingToolId,
    }),
  }))
})

const quickActionGridColumns = computed(() =>
  resolveGatewayQuickActionGridColumns(quickActions.value.length)
)

const resolveQuickActionLabel = (actionId: string, label: string, loading: boolean) => {
  if (!loading) {
    return label
  }

  const loadingLabels: Record<string, string> = {
    start: '启动中...',
    stop: '停止中...',
    restart: '重启中...',
    'install-service': '安装中...',
  }

  return loadingLabels[actionId] || `${label}...`
}

const refreshLogsButtonState = computed(() =>
  resolveAsyncButtonState({
    loading: refreshingLogs.value,
    baseDisabled: props.envMode !== 'local' || logsFollowing.value,
  })
)

const refreshLogsButtonLabel = computed(() =>
  resolveAsyncButtonLabel({
    loading: refreshingLogs.value,
    label: '刷新',
    loadingLabel: '刷新中...',
  })
)

const levelColor = (level: string) => {
  if (level === 'error') return 'var(--oc-danger)'
  if (level === 'warn') return 'var(--oc-warning)'
  if (level === 'success') return 'var(--oc-success)'
  return 'var(--oc-text-secondary)'
}

const formatTime = (value: number) =>
  new Date(value).toLocaleTimeString('zh-CN', { hour12: false })

const appendLog = (payload: DashboardLogEvent) => {
  logs.value.push(payload)
  if (logs.value.length > 600) {
    logs.value.shift()
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

const startLogFollow = async () => {
  if (props.envMode !== 'local' || !props.envStatus.openclaw.installed) {
    return
  }
  try {
    await invoke<boolean>('start_openclaw_logs_follow')
  } catch (error) {
    appendLog({
      message: `启动日志跟踪失败: ${String(error)}`,
      level: 'error',
      timestamp: Date.now(),
    })
  }
}

const refreshLogs = async () => {
  if (refreshLogsButtonState.value.disabled) {
    return
  }

  refreshingLogs.value = true
  try {
    await startLogFollow()
  } finally {
    refreshingLogs.value = false
  }
}

watch(
  () => [props.envMode, props.envStatus.openclaw.installed],
  async ([mode, installed], [prevMode, prevInstalled]) => {
    if (mode !== 'local') {
      logsFollowing.value = false
      return
    }
    if (installed && (prevMode !== 'local' || !prevInstalled)) {
      await startLogFollow()
    }
  }
)

watch(
  () => props.gatewayReachable,
  async (reachable, previous) => {
    if (
      reachable &&
      !previous &&
      props.envMode === 'local' &&
      props.envStatus.openclaw.installed &&
      !logsFollowing.value
    ) {
      await startLogFollow()
    }
  }
)

onMounted(async () => {
  unlistenLogLine = await listen<DashboardLogEvent>('openclaw-log-line', (event) => {
    appendLog(event.payload)
  })

  unlistenLogStatus = await listen<DashboardLogStatusEvent>('openclaw-log-status', (event) => {
    logsFollowing.value = event.payload.running
  })

  await startLogFollow()
})

onUnmounted(() => {
  unlistenLogLine?.()
  unlistenLogStatus?.()
})
</script>

<template>
  <div class="oc-page-root flex flex-col gap-2">
    <section class="oc-panel shrink-0 p-4">
      <div class="flex items-center justify-between gap-3">
        <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">服务状态</h3>
        <span class="inline-flex items-center gap-2 text-sm font-medium" :style="{ color: serviceStatus.color }">
          <span class="oc-status-dot" :class="serviceStatus.dotClass" />
          {{ serviceStatus.label }}
        </span>
      </div>
      <p class="mt-2 text-sm" style="color: var(--oc-text-secondary);">{{ serviceSummary }}</p>
    </section>

    <section class="oc-panel shrink-0 p-4">
      <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">快捷操作</h3>
      <div
        class="mt-3 grid grid-cols-2 gap-2"
        :class="{
          'md:grid-cols-1': quickActionGridColumns === 1,
          'md:grid-cols-2': quickActionGridColumns === 2,
          'md:grid-cols-3': quickActionGridColumns === 3,
          'md:grid-cols-4': quickActionGridColumns === 4,
        }"
      >
        <button
          v-for="item in quickActions"
          :key="item.id"
          class="oc-subpanel px-3 py-3 text-center transition-all duration-200 disabled:cursor-not-allowed disabled:opacity-45"
          :style="{ color: item.color }"
          :disabled="item.disabled"
          :aria-busy="item.loading"
          @click="emit('openTool', item.id)"
        >
          <div class="mb-2 flex justify-center">
            <span
              class="flex h-9 w-9 items-center justify-center rounded-full border"
              style="border-color: var(--oc-divider); background: color-mix(in srgb, var(--oc-card-elevated) 78%, transparent);"
            >
              <component :is="item.loading ? Loader2 : item.icon" :class="['h-4 w-4', item.loading ? 'animate-spin' : '']" />
            </span>
          </div>
          <div class="text-sm font-semibold">{{ resolveQuickActionLabel(item.id, item.label, item.loading) }}</div>
        </button>
      </div>
    </section>

    <section class="oc-panel flex min-h-0 flex-1 flex-col">
      <div class="flex items-center justify-between border-b px-4 py-3" style="border-color: var(--oc-divider-soft);">
        <div class="flex items-center gap-2 text-sm" style="color: var(--oc-text-secondary);">
          <span class="font-semibold">实时日志</span>
          <span style="color: var(--oc-text-muted);">({{ logs.length }} 行)</span>
          <span class="rounded-full px-2 py-0.5 text-[11px]" style="background: color-mix(in srgb, var(--oc-card-elevated) 82%, transparent); color: var(--oc-text-muted);">
            {{ logsFollowing ? '跟踪中' : '已停止' }}
          </span>
        </div>

        <div class="flex items-center gap-2">
          <button
            class="oc-toolbar-btn h-8 px-3 text-xs"
            type="button"
            :disabled="refreshLogsButtonState.disabled"
            :aria-busy="refreshLogsButtonState.loading"
            @click="refreshLogs"
          >
            <component :is="refreshLogsButtonState.loading ? Loader2 : RefreshCw" :class="['h-3.5 w-3.5', refreshLogsButtonState.loading ? 'animate-spin' : '']" />
            {{ refreshLogsButtonLabel }}
          </button>
          <button class="oc-toolbar-btn h-8 w-8 !px-0" type="button" @click="logExpanded = !logExpanded">
            <ChevronUp v-if="logExpanded" class="h-4 w-4" />
            <ChevronDown v-else class="h-4 w-4" />
          </button>
        </div>
      </div>

      <div v-if="logExpanded" class="flex min-h-0 flex-1 px-4 pb-4 pt-3">
        <div
          ref="logContainerRef"
          class="min-h-0 flex-1 overflow-auto rounded-[12px] border p-3 font-mono text-xs leading-6"
          style="border-color: var(--oc-card-border); background: color-mix(in srgb, var(--oc-card-elevated) 88%, transparent);"
        >
          <div v-if="envMode !== 'local'" class="py-8 text-center" style="color: var(--oc-text-muted);">
            SSH 环境暂不支持内置实时跟踪，请在远端执行 openclaw logs --follow。
          </div>
          <div v-else-if="!envStatus.openclaw.installed" class="py-8 text-center" style="color: var(--oc-text-muted);">
            未检测到 OpenClaw，暂无法读取实时日志。
          </div>
          <div v-else-if="logs.length === 0" class="py-8 text-center" style="color: var(--oc-text-muted);">
            等待日志输出...
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
  </div>
</template>
