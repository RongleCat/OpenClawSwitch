<script setup lang="ts">
import { type HTMLAttributes, computed } from 'vue'
import { cn } from '@/lib/utils'

type InputSpellcheck = boolean | 'true' | 'false'
type InputMode = 'none' | 'text' | 'tel' | 'url' | 'email' | 'numeric' | 'decimal' | 'search'

interface Props {
  type?: string
  placeholder?: string
  modelValue?: string | number
  class?: HTMLAttributes['class']
  autocomplete?: string
  autocorrect?: string
  autocapitalize?: string
  spellcheck?: InputSpellcheck
  lang?: string
  inputmode?: InputMode
  modelModifiers?: {
    number?: boolean
    trim?: boolean
  }
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
  let value = (event.target as HTMLInputElement).value
  if (props.modelModifiers?.trim) {
    value = value.trim()
  }

  if (props.modelModifiers?.number) {
    const numberValue = Number.parseFloat(value)
    emit('update:modelValue', Number.isNaN(numberValue) ? value : numberValue)
    return
  }

  emit('update:modelValue', value)
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
