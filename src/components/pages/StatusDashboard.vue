<script setup lang="ts">
import { computed } from 'vue'
import {
  CheckCircle, XCircle, RefreshCw,
  Settings, RotateCcw, Terminal, Wrench,
  FolderOpen, Trash2, HardDrive, Download
} from 'lucide-vue-next'
import Card from '../ui/Card.vue'
import Button from '../ui/Button.vue'
import type { EnvironmentStatus } from '../../types/config'

const props = defineProps<{
  envStatus: EnvironmentStatus
  activeTool: string | null
  showToast: (type: 'success' | 'error', message: string) => void
}>()

const emit = defineEmits<{
  openTool: [toolId: string]
  refresh: []
}>()

// 状态卡片数据
const statusCards = computed(() => [
  {
    label: 'OpenClaw',
    installed: props.envStatus.openclaw.installed,
    version: props.envStatus.openclaw.version,
    icon: HardDrive,
    color: 'blue'
  },
  {
    label: 'Node.js',
    installed: props.envStatus.node.installed,
    version: props.envStatus.node.version,
    icon: Download,
    color: 'green'
  },
  {
    label: 'Git',
    installed: props.envStatus.git.installed,
    version: props.envStatus.git.version,
    icon: FolderOpen,
    color: 'orange'
  },
  {
    label: 'fnm',
    installed: props.envStatus.fnm.installed,
    version: props.envStatus.fnm.version,
    icon: Wrench,
    color: 'purple'
  }
])

// 工具宫格
const tools = [
  { id: 'config', label: '模型配置', icon: Settings, desc: '管理服务商和模型' },
  { id: 'restart', label: '重启网关', icon: RotateCcw, desc: '重启 OpenClaw 服务' },
  { id: 'tui', label: '打开 TUI', icon: Terminal, desc: '终端交互界面' },
  { id: 'source', label: '查看源文件', icon: FolderOpen, desc: '打开配置文件目录' },
  { id: 'uninstall', label: '卸载', icon: Trash2, desc: '卸载 OpenClaw' },
]

const colorMap: Record<string, string> = {
  blue: 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800',
  green: 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800',
  orange: 'bg-orange-50 dark:bg-orange-900/20 border-orange-200 dark:border-orange-800',
  purple: 'bg-purple-50 dark:bg-purple-900/20 border-purple-200 dark:border-purple-800',
}
</script>

<template>
  <div class="h-full overflow-y-auto p-6">
    <div class="max-w-4xl mx-auto space-y-6">
      <!-- 标题 + 刷新 -->
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-xl font-bold">设备状态</h2>
          <p class="text-sm text-muted-foreground mt-0.5">
            {{ envStatus.system.os }} / {{ envStatus.system.arch }} · {{ envStatus.system.shell }}
          </p>
        </div>
        <Button variant="outline" size="sm" @click="emit('refresh')">
          <RefreshCw class="w-4 h-4 mr-1.5" />
          刷新
        </Button>
      </div>

      <!-- 状态卡片网格 -->
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
        <div
          v-for="card in statusCards"
          :key="card.label"
          class="rounded-lg border p-3 transition-colors"
          :class="colorMap[card.color]"
        >
          <div class="flex items-center gap-2 mb-2">
            <component :is="card.icon" class="w-4 h-4 text-muted-foreground" />
            <span class="text-sm font-medium">{{ card.label }}</span>
          </div>
          <div class="flex items-center gap-1.5">
            <component
              :is="card.installed ? CheckCircle : XCircle"
              class="w-4 h-4 flex-shrink-0"
              :class="card.installed ? 'text-green-500' : 'text-red-400'"
            />
            <span class="text-sm truncate">
              {{ card.installed ? (card.version || '已安装') : '未安装' }}
            </span>
          </div>
        </div>
      </div>

      <!-- 工具区 -->
      <Card class="p-4">
        <h3 class="text-sm font-medium mb-3 text-muted-foreground">工具</h3>
        <div class="grid grid-cols-3 sm:grid-cols-5 gap-2">
          <button
            v-for="tool in tools"
            :key="tool.id"
            @click="emit('openTool', tool.id)"
            class="flex flex-col items-center gap-1.5 p-3 rounded-lg border hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-center"
            :class="activeTool === tool.id ? 'bg-blue-50 dark:bg-blue-900/20 border-blue-300' : 'border-transparent'"
          >
            <component :is="tool.icon" class="w-5 h-5 text-muted-foreground" />
            <span class="text-xs font-medium">{{ tool.label }}</span>
          </button>
        </div>
      </Card>
    </div>
  </div>
</template>
