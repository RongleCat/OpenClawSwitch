<script setup lang="ts">
import { computed } from 'vue'
import { FolderOpen, FileJson, HardDrive, Cloud, RefreshCw } from 'lucide-vue-next'
import Button from './ui/Button.vue'
import type { ConfigFileInfo } from '@/types/config'

interface Props {
  fileInfo: ConfigFileInfo | null
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const emit = defineEmits<{
  selectDirectory: []
  selectFile: []
  reload: []
}>()

const modeLabel = computed(() => {
  if (!props.fileInfo) return ''
  return props.fileInfo.mode === 'local' ? '本地模式' : '远程模式'
})

const modeIcon = computed(() => {
  if (!props.fileInfo) return HardDrive
  return props.fileInfo.mode === 'local' ? HardDrive : Cloud
})

const modeColor = computed(() => {
  if (!props.fileInfo) return 'text-gray-500'
  return props.fileInfo.mode === 'local'
    ? 'text-green-600 dark:text-green-400'
    : 'text-blue-600 dark:text-blue-400'
})
</script>

<template>
  <div class="space-y-3">
    <!-- 操作按钮 -->
    <div class="flex flex-wrap gap-2">
      <Button
        variant="outline"
        size="sm"
        @click="emit('selectDirectory')"
        :disabled="loading"
        class="flex-1 min-w-[140px]"
      >
        <FolderOpen class="w-4 h-4" />
        选择配置目录
      </Button>
      <Button
        variant="outline"
        size="sm"
        @click="emit('selectFile')"
        :disabled="loading"
        class="flex-1 min-w-[140px]"
      >
        <FileJson class="w-4 h-4" />
        选择配置文件
      </Button>
      <Button
        v-if="fileInfo"
        variant="ghost"
        size="sm"
        @click="emit('reload')"
        :disabled="loading"
        title="重新加载配置"
      >
        <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': loading }" />
      </Button>
    </div>

    <!-- 当前文件信息 -->
    <div v-if="fileInfo" class="rounded-lg border bg-muted/50 p-3 space-y-2">
      <div class="flex items-center gap-2">
        <component :is="modeIcon" class="w-4 h-4" :class="modeColor" />
        <span class="text-sm font-medium" :class="modeColor">{{ modeLabel }}</span>
        <span class="text-xs px-2 py-0.5 rounded-full bg-background border">
          {{ fileInfo.fileName }}
        </span>
      </div>
      <div class="text-xs text-muted-foreground truncate" :title="fileInfo.path">
        {{ fileInfo.path }}
      </div>
    </div>

    <!-- 未加载状态 -->
    <div v-else class="rounded-lg border border-dashed p-4 text-center text-muted-foreground">
      <FileJson class="w-8 h-8 mx-auto mb-2 opacity-50" />
      <p class="text-sm">请选择配置目录或文件</p>
      <p class="text-xs mt-1">支持 openclaw.json 和 clawdbot.json</p>
    </div>
  </div>
</template>
