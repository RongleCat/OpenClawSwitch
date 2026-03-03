<script setup lang="ts">
import Button from './ui/Button.vue'
import Card from './ui/Card.vue'
import { ShieldAlert, ShieldCheck, Fingerprint } from 'lucide-vue-next'
import type { FingerprintInfo } from '@/types/config'

interface Props {
  fingerprint: FingerprintInfo
}

defineProps<Props>()

const emit = defineEmits<{
  confirm: []
  reject: []
}>()
</script>

<template>
  <div class="oc-modal-overlay">
    <Card class="oc-modal-card w-full max-w-md p-6">
      <div class="flex items-center gap-3 mb-4">
        <div
          class="rounded-full p-2"
          :style="{
            background: fingerprint.isKnown
              ? 'color-mix(in srgb, var(--oc-success) 20%, transparent)'
              : 'color-mix(in srgb, var(--oc-warning) 20%, transparent)'
          }"
        >
          <ShieldCheck v-if="fingerprint.isKnown" class="w-6 h-6" style="color: var(--oc-success);" />
          <ShieldAlert v-else class="w-6 h-6" style="color: var(--oc-warning);" />
        </div>
        <div>
          <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">
            {{ fingerprint.isKnown ? '已知主机' : '未知主机' }}
          </h3>
          <p class="text-sm" style="color: var(--oc-text-muted);">{{ fingerprint.host }}</p>
        </div>
      </div>

      <div v-if="!fingerprint.isKnown"
           class="mb-4 rounded-[11px] border p-3"
           style="border-color: color-mix(in srgb, var(--oc-warning) 58%, transparent); background: color-mix(in srgb, var(--oc-warning) 12%, transparent); color: var(--oc-warning);">
        <p class="text-sm">
          首次连接此服务器，请确认以下指纹信息是否正确。
          如果您不确定，请联系服务器管理员验证。
        </p>
      </div>

      <div class="space-y-3 mb-6">
        <div>
          <div class="flex items-center gap-1.5 mb-1">
            <Fingerprint class="w-3.5 h-3.5" style="color: var(--oc-text-muted);" />
            <span class="text-xs font-medium" style="color: var(--oc-text-muted);">SHA-256</span>
          </div>
          <code class="block rounded-[10px] border p-2.5 font-mono text-xs break-all select-all" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-primary);">
            {{ fingerprint.sha256 }}
          </code>
        </div>
        <div>
          <div class="flex items-center gap-1.5 mb-1">
            <Fingerprint class="w-3.5 h-3.5" style="color: var(--oc-text-muted);" />
            <span class="text-xs font-medium" style="color: var(--oc-text-muted);">MD5</span>
          </div>
          <code class="block rounded-[10px] border p-2.5 font-mono text-xs break-all select-all" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-primary);">
            {{ fingerprint.md5 }}
          </code>
        </div>
      </div>

      <div class="flex justify-end gap-2">
        <Button variant="ghost" @click="emit('reject')">
          拒绝
        </Button>
        <Button @click="emit('confirm')">
          <ShieldCheck class="w-4 h-4" />
          信任并继续
        </Button>
      </div>
    </Card>
  </div>
</template>
