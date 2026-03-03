<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import type { InstallLogEvent } from '../../types/config'

const props = defineProps<{
  logs: InstallLogEvent[]
}>()

const containerRef = ref<HTMLDivElement>()

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
    case 'success':
      return 'var(--oc-success)'
    case 'error':
      return 'var(--oc-danger)'
    case 'warn':
      return 'var(--oc-warning)'
    default:
      return 'var(--oc-text-secondary)'
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
    class="h-full min-h-0 overflow-auto rounded-[12px] border p-4 font-mono text-xs"
    style="border-color: var(--oc-card-border); background: color-mix(in srgb, var(--oc-card-elevated) 88%, transparent);"
  >
    <div v-if="logs.length === 0" class="py-8 text-center" style="color: var(--oc-text-muted);">
      等待安装开始...
    </div>
    <div v-for="(log, i) in logs" :key="i" class="leading-6">
      <span class="mr-2 select-none text-[11px]" style="color: var(--oc-text-quiet);">
        {{ new Date(log.timestamp).toLocaleTimeString('zh-CN', { hour12: false }) }}
      </span>
      <span class="mr-1" :style="{ color: levelColor(log.level) }">{{ levelPrefix(log.level) }}</span>
      <span :style="{ color: levelColor(log.level) }">{{ log.message }}</span>
    </div>
    <div class="mt-1 inline-block h-4 w-2 animate-pulse rounded-[2px]" style="background: color-mix(in srgb, var(--oc-success) 68%, transparent);" />
  </div>
</template>
