<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import type { InstallLogEvent } from '../../types/config'

const props = defineProps<{
  logs: InstallLogEvent[]
}>()

const containerRef = ref<HTMLDivElement>()

// 自动滚动到底部
watch(
  () => props.logs.length,
  async () => {
    await nextTick()
    if (containerRef.value) {
      containerRef.value.scrollTop = containerRef.value.scrollHeight
    }
  }
)

const levelColor = (level: string) => {
  switch (level) {
    case 'success': return 'text-green-400'
    case 'error': return 'text-red-400'
    case 'warn': return 'text-yellow-400'
    default: return 'text-gray-300'
  }
}

const levelPrefix = (level: string) => {
  switch (level) {
    case 'success': return '✓'
    case 'error': return '✗'
    case 'warn': return '⚠'
    default: return '›'
  }
}
</script>

<template>
  <div
    ref="containerRef"
    class="bg-gray-900 rounded-lg p-4 font-mono text-sm overflow-auto h-full min-h-0"
  >
    <div v-if="logs.length === 0" class="text-gray-500 text-center py-8">
      等待安装开始...
    </div>
    <div v-for="(log, i) in logs" :key="i" class="leading-6">
      <span class="text-gray-600 text-xs mr-2 select-none">
        {{ new Date(log.timestamp).toLocaleTimeString('zh-CN', { hour12: false }) }}
      </span>
      <span :class="levelColor(log.level)" class="mr-1">{{ levelPrefix(log.level) }}</span>
      <span :class="levelColor(log.level)">{{ log.message }}</span>
    </div>
    <!-- 光标闪烁 -->
    <div class="inline-block w-2 h-4 bg-green-400 animate-pulse mt-1" />
  </div>
</template>
