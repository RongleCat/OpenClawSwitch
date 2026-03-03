<script setup lang="ts">
import { type HTMLAttributes, computed } from 'vue'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/lib/utils'

const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-[11px] border text-sm font-medium transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--oc-input-focus)] focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-45',
  {
    variants: {
      variant: {
        default:
          'border-[var(--oc-accent)] bg-[var(--oc-accent)] text-white shadow-[var(--oc-shadow-control)] hover:opacity-90',
        destructive:
          'border-[var(--oc-danger)] bg-[var(--oc-danger)] text-white hover:opacity-90',
        outline:
          'border-[var(--oc-card-border)] bg-[var(--oc-card)] text-[var(--oc-text-secondary)] hover:border-[var(--oc-card-border-strong)] hover:text-[var(--oc-text-primary)]',
        secondary:
          'border-[var(--oc-divider)] bg-[var(--oc-card-elevated)] text-[var(--oc-text-primary)] hover:opacity-95',
        ghost:
          'border-transparent bg-transparent text-[var(--oc-text-secondary)] hover:bg-[var(--oc-item-hover)] hover:text-[var(--oc-text-primary)]',
        link: 'border-transparent bg-transparent text-[var(--oc-accent)] underline-offset-4 hover:underline',
      },
      size: {
        default: 'h-10 px-4 py-2',
        sm: 'h-8 rounded-[10px] px-3 text-xs',
        lg: 'h-11 rounded-[12px] px-8',
        icon: 'h-10 w-10',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'default',
    },
  }
)

interface Props {
  variant?: VariantProps<typeof buttonVariants>['variant']
  size?: VariantProps<typeof buttonVariants>['size']
  class?: HTMLAttributes['class']
}

const props = defineProps<Props>()

const classes = computed(() =>
  cn(buttonVariants({ variant: props.variant, size: props.size }), props.class)
)
</script>

<template>
  <button :class="classes">
    <slot />
  </button>
</template>
