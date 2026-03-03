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

const styleMap = computed(() => {
  if (props.variant === 'destructive') {
    return {
      borderColor: 'color-mix(in srgb, var(--oc-danger) 62%, transparent)',
      background: 'color-mix(in srgb, var(--oc-danger) 12%, transparent)',
      color: 'var(--oc-danger)'
    }
  }

  if (props.variant === 'success') {
    return {
      borderColor: 'color-mix(in srgb, var(--oc-success) 62%, transparent)',
      background: 'color-mix(in srgb, var(--oc-success) 12%, transparent)',
      color: 'var(--oc-success)'
    }
  }

  return {
    borderColor: 'var(--oc-card-border)',
    background: 'var(--oc-card-elevated)',
    color: 'var(--oc-text-secondary)'
  }
})

const classes = computed(() =>
  cn(
    'relative w-full rounded-[12px] border p-4 text-sm',
    props.class
  )
)
</script>

<template>
  <div :class="classes" :style="styleMap">
    <slot />
  </div>
</template>
