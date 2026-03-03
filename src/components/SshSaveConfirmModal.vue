<script setup lang="ts">
import Button from './ui/Button.vue'
import Card from './ui/Card.vue'
import type { JsonDiffSummary } from '../domain/jsonDiff'

defineProps<{
  targetPath: string
  summary: JsonDiffSummary
}>()

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()
</script>

<template>
  <div class="oc-modal-overlay" @click.self="emit('cancel')">
    <Card class="oc-modal-card w-full max-w-2xl p-6">
      <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">确认写入远程配置</h3>
      <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">
        目标路径：<code>{{ targetPath }}</code>
      </p>

      <div class="mt-4 grid gap-3 sm:grid-cols-2">
        <div class="rounded-[12px] border p-3" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
          <p class="text-xs" style="color: var(--oc-text-muted);">变化条目</p>
          <p class="mt-1 text-xl font-semibold" style="color: var(--oc-text-primary);">{{ summary.changed }}</p>
          <p class="mt-1 text-xs" style="color: var(--oc-text-muted);">新增 {{ summary.added }} · 删除 {{ summary.removed }}</p>
        </div>

        <div class="rounded-[12px] border p-3" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
          <p class="text-xs" style="color: var(--oc-text-muted);">文件大小</p>
          <p class="mt-1 text-xl font-semibold" style="color: var(--oc-text-primary);">{{ summary.oldSize }} → {{ summary.newSize }}</p>
          <p class="mt-1 text-xs" style="color: var(--oc-text-muted);">单位：字符数</p>
        </div>
      </div>

      <div class="mt-4">
        <p class="mb-2 text-xs font-medium" style="color: var(--oc-text-muted);">变更字段（最多显示 12 个）</p>
        <div class="flex flex-wrap gap-2">
          <span
            v-for="key in summary.changedKeys.slice(0, 12)"
            :key="key"
            class="rounded-full border px-2 py-1 text-xs"
            style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-secondary);"
          >
            {{ key }}
          </span>
          <span v-if="summary.changedKeys.length === 0" class="text-xs" style="color: var(--oc-text-muted);">
            未发现字段级差异（可能是格式变化）。
          </span>
        </div>
      </div>

      <div class="mt-6 flex justify-end gap-2">
        <Button variant="outline" @click="emit('cancel')">取消</Button>
        <Button @click="emit('confirm')">确认写入</Button>
      </div>
    </Card>
  </div>
</template>
