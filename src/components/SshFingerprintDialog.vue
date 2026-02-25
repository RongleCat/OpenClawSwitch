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
  <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-[60]">
    <Card class="w-full max-w-md p-6 m-4">
      <!-- 标题 -->
      <div class="flex items-center gap-3 mb-4">
        <div class="p-2 rounded-full" :class="fingerprint.isKnown
          ? 'bg-green-100 dark:bg-green-900'
          : 'bg-amber-100 dark:bg-amber-900'">
          <ShieldCheck v-if="fingerprint.isKnown" class="w-6 h-6 text-green-600 dark:text-green-400" />
          <ShieldAlert v-else class="w-6 h-6 text-amber-600 dark:text-amber-400" />
        </div>
        <div>
          <h3 class="font-semibold text-lg">
            {{ fingerprint.isKnown ? '已知主机' : '未知主机' }}
          </h3>
          <p class="text-sm text-muted-foreground">{{ fingerprint.host }}</p>
        </div>
      </div>

      <!-- 警告信息 -->
      <div v-if="!fingerprint.isKnown"
           class="mb-4 p-3 rounded-lg bg-amber-50 dark:bg-amber-950 border border-amber-200 dark:border-amber-800">
        <p class="text-sm text-amber-800 dark:text-amber-200">
          首次连接此服务器，请确认以下指纹信息是否正确。
          如果您不确定，请联系服务器管理员验证。
        </p>
      </div>

      <!-- 指纹信息 -->
      <div class="space-y-3 mb-6">
        <div>
          <div class="flex items-center gap-1.5 mb-1">
            <Fingerprint class="w-3.5 h-3.5 text-muted-foreground" />
            <span class="text-xs font-medium text-muted-foreground">SHA-256</span>
          </div>
          <code class="block text-xs bg-gray-100 dark:bg-gray-800 p-2.5 rounded-lg font-mono break-all select-all">
            {{ fingerprint.sha256 }}
          </code>
        </div>
        <div>
          <div class="flex items-center gap-1.5 mb-1">
            <Fingerprint class="w-3.5 h-3.5 text-muted-foreground" />
            <span class="text-xs font-medium text-muted-foreground">MD5</span>
          </div>
          <code class="block text-xs bg-gray-100 dark:bg-gray-800 p-2.5 rounded-lg font-mono break-all select-all">
            {{ fingerprint.md5 }}
          </code>
        </div>
      </div>

      <!-- 操作按钮 -->
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
