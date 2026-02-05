<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

interface Props {
  variant?: 'default' | 'destructive' | 'success'
  class?: string
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default'
})

const classes = computed(() =>
  cn(
    'relative w-full rounded-lg border p-4',
    {
      'bg-background text-foreground': props.variant === 'default',
      'border-destructive/50 text-destructive dark:border-destructive [&>svg]:text-destructive': props.variant === 'destructive',
      'border-green-500/50 text-green-600 dark:border-green-500 [&>svg]:text-green-600': props.variant === 'success',
    },
    props.class
  )
)
</script>

<template>
  <div :class="classes">
    <slot />
  </div>
</template>
