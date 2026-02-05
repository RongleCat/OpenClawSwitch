<script setup lang="ts">
import { type HTMLAttributes, computed } from 'vue'
import { cn } from '@/lib/utils'

interface Props {
  type?: string
  placeholder?: string
  modelValue?: string
  class?: HTMLAttributes['class']
  autocomplete?: string
  autocorrect?: string
  autocapitalize?: string
  spellcheck?: string | boolean
  lang?: string
  inputmode?: string
}

const props = defineProps<Props>()
const emit = defineEmits(['update:modelValue'])

const classes = computed(() =>
  cn(
    'flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
    props.class
  )
)

const handleInput = (event: Event) => {
  emit('update:modelValue', (event.target as HTMLInputElement).value)
}
</script>

<template>
  <input
    :type="type || 'text'"
    :placeholder="placeholder"
    :value="modelValue"
    :class="classes"
    :autocomplete="autocomplete"
    :autocorrect="autocorrect"
    :autocapitalize="autocapitalize"
    :spellcheck="spellcheck"
    :lang="lang"
    :inputmode="inputmode"
    @input="handleInput"
  />
</template>
