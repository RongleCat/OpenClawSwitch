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

const classes = computed(() =>
  cn(
    'fixed bottom-4 right-4 z-50 flex items-center gap-2 px-4 py-3 rounded-lg shadow-lg border max-w-sm animate-in slide-in-from-bottom-4 fade-in duration-300',
    {
      'bg-green-50 border-green-200 text-green-800 dark:bg-green-950 dark:border-green-800 dark:text-green-200': props.type === 'success',
      'bg-red-50 border-red-200 text-red-800 dark:bg-red-950 dark:border-red-800 dark:text-red-200': props.type === 'error',
    },
    props.class
  )
)
</script>

<template>
  <Teleport to="body">
    <div :class="classes">
      <component :is="type === 'success' ? CheckCircle2 : XCircle" class="w-5 h-5 flex-shrink-0" />
      <span class="text-sm flex-1">{{ message }}</span>
      <button @click="emit('close')" class="p-1 hover:bg-black/10 rounded transition-colors">
        <X class="w-4 h-4" />
      </button>
    </div>
  </Teleport>
</template>
