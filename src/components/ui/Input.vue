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
    'oc-input disabled:cursor-not-allowed disabled:opacity-45',
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
