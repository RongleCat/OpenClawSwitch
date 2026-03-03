<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'
import { CheckCircle2, XCircle, X } from 'lucide-vue-next'

interface Props {
  type?: 'success' | 'error'
  message: string
  class?: string
}

const props = withDefaults(defineProps<Props>(), {
  type: 'success'
})

const emit = defineEmits<{
  close: []
}>()

const styleMap = computed(() => {
  if (props.type === 'error') {
    return {
      borderColor: 'color-mix(in srgb, var(--oc-danger) 60%, transparent)',
      background: 'color-mix(in srgb, var(--oc-danger) 14%, var(--oc-card) 86%)',
      color: 'var(--oc-danger)'
    }
  }

  return {
    borderColor: 'color-mix(in srgb, var(--oc-success) 60%, transparent)',
    background: 'color-mix(in srgb, var(--oc-success) 14%, var(--oc-card) 86%)',
    color: 'var(--oc-success)'
  }
})

const classes = computed(() =>
  cn(
    'oc-toast-card fixed bottom-5 right-5 z-[120] flex max-w-sm items-center gap-2 rounded-[12px] border px-4 py-3 text-sm backdrop-blur-md animate-in slide-in-from-bottom-4 fade-in duration-300',
    props.class
  )
)
</script>

<template>
  <Teleport to="body">
    <div :class="classes" :style="styleMap">
      <component :is="type === 'success' ? CheckCircle2 : XCircle" class="h-5 w-5 flex-shrink-0" />
      <span class="flex-1">{{ message }}</span>
      <button @click="emit('close')" class="rounded p-1 transition-colors hover:bg-black/10 dark:hover:bg-white/10">
        <X class="h-4 w-4" />
      </button>
    </div>
  </Teleport>
</template>
