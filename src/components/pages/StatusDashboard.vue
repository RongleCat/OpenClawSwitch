<script setup lang="ts">
import { computed } from 'vue'
import {
  CheckCircle, XCircle, RefreshCw,
  Settings, RotateCcw, Terminal,
  FolderOpen, Trash2, HardDrive, Download,
  Globe, Wrench
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
  }
])

// 工具宫格
const tools = [
  { id: 'config', label: '模型配置', icon: Settings, desc: '管理服务商和模型' },
  { id: 'restart', label: '重启网关', icon: RotateCcw, desc: '重启 OpenClaw 服务' },
  { id: 'tui', label: '打开 TUI', icon: Terminal, desc: '终端交互界面' },
  { id: 'webui', label: '打开 Web UI', icon: Globe, desc: '打开控制面板' },
  { id: 'doctor', label: '一键修复', icon: Wrench, desc: '诊断并修复问题' },
  { id: 'source', label: '查看源文件', icon: FolderOpen, desc: '打开配置文件目录' },
  { id: 'uninstall', label: '卸载', icon: Trash2, desc: '卸载 OpenClaw' },
]

const colorMap: Record<string, string> = {
  blue: 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800',
  green: 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800',
  orange: 'bg-orange-50 dark:bg-orange-900/20 border-orange-200 dark:border-orange-800',
}
</script>

<template>
  <div class="h-full overflow-y-auto bg-gray-50 p-8">
    <div class="max-w-6xl mx-auto space-y-6">
      <!-- 标题 + 刷新 -->
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-xl font-bold text-gray-900">设备状态</h2>
          <p class="text-sm text-gray-500 mt-1">
            {{ envStatus.system.os }} / {{ envStatus.system.arch }} · {{ envStatus.system.shell }}
          </p>
        </div>
        <Button variant="outline" size="sm" @click="emit('refresh')">
          <RefreshCw class="w-4 h-4 mr-1.5" />
          刷新
        </Button>
      </div>

      <!-- 状态卡片网格 -->
      <div class="grid grid-cols-3 gap-4">
        <div
          v-for="card in statusCards"
          :key="card.label"
          class="rounded-lg border border-gray-200 bg-white p-4 shadow-sm hover:shadow-md transition-shadow"
        >
          <div class="flex items-center gap-2 mb-3">
            <component :is="card.icon" class="w-5 h-5 text-gray-600" />
            <span class="text-sm font-semibold text-gray-900">{{ card.label }}</span>
          </div>
          <div class="flex items-center gap-2">
            <component
              :is="card.installed ? CheckCircle : XCircle"
              class="w-5 h-5 flex-shrink-0"
              :class="card.installed ? 'text-green-500' : 'text-red-400'"
            />
            <span class="text-sm text-gray-700 truncate">
              {{ card.installed ? (card.version || '已安装') : '未安装' }}
            </span>
          </div>
        </div>
      </div>

      <!-- 工具区 -->
      <Card class="p-6">
        <h3 class="text-sm font-semibold text-gray-700 mb-4">工具</h3>
        <div class="grid grid-cols-7 gap-3">
          <button
            v-for="tool in tools"
            :key="tool.id"
            @click="emit('openTool', tool.id)"
            class="flex flex-col items-center gap-2 p-4 rounded-lg border border-gray-200 hover:bg-gray-50 hover:border-gray-300 transition-all text-center"
            :class="activeTool === tool.id ? 'bg-blue-50 border-blue-300' : ''"
          >
            <component :is="tool.icon" class="w-6 h-6 text-gray-600" />
            <span class="text-xs font-medium text-gray-900">{{ tool.label }}</span>
          </button>
        </div>
      </Card>
    </div>
  </div>
</template>
