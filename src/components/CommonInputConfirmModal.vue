<script setup lang="ts">
import Button from './ui/Button.vue'
import Card from './ui/Card.vue'
import Input from './ui/Input.vue'

interface Props {
  title: string
  description?: string
  modelValue: string
  placeholder?: string
  confirmText?: string
  cancelText?: string
  loading?: boolean
  note?: string
}

const props = withDefaults(defineProps<Props>(), {
  description: '',
  placeholder: '',
  confirmText: '确认',
  cancelText: '取消',
  loading: false,
  note: ''
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  confirm: []
  cancel: []
}>()

const handleInput = (value: string) => {
  emit('update:modelValue', value)
}
</script>

<template>
  <div class="oc-modal-overlay" @click.self="!props.loading && emit('cancel')">
    <Card class="oc-modal-card w-full max-w-lg p-6">
      <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">{{ props.title }}</h3>
      <p v-if="props.description" class="mt-1 text-sm" style="color: var(--oc-text-muted);">
        {{ props.description }}
      </p>

      <div class="mt-4">
        <Input
          :model-value="props.modelValue"
          :placeholder="props.placeholder"
          :disabled="props.loading"
          @update:model-value="handleInput"
        />
      </div>

      <p v-if="props.note" class="mt-2 text-xs" style="color: var(--oc-text-quiet);">
        {{ props.note }}
      </p>

      <div class="mt-5 flex justify-end gap-2">
        <Button variant="outline" :disabled="props.loading" @click="emit('cancel')">
          {{ props.cancelText }}
        </Button>
        <Button :disabled="props.loading || !props.modelValue.trim()" @click="emit('confirm')">
          {{ props.confirmText }}
        </Button>
      </div>
    </Card>
  </div>
</template>
