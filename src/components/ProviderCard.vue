<script setup lang="ts">
import { ref } from 'vue'
import { cn } from '@/lib/utils'
import { Trash2, Star, Shield, ChevronDown, ChevronUp, Cpu, Brain, Zap, Plus, X, Pencil } from 'lucide-vue-next'
import Button from './ui/Button.vue'
import type { ProviderInfo, ModelInfo } from '@/types/config'

interface Props {
  provider: ProviderInfo
  containsPrimary?: boolean
  class?: string
}

const props = withDefaults(defineProps<Props>(), {
  containsPrimary: false
})

const emit = defineEmits<{
  setPrimary: [modelPath: string]
  setFallback: [modelPath: string]
  addModel: [providerName: string]
  removeModel: [providerName: string, modelId: string]
  edit: []
  delete: []
}>()

const showModels = ref(false)

const handleRemoveModel = (modelId: string) => {
  emit('removeModel', props.provider.name, modelId)
}

const formatContextWindow = (value?: number): string => {
  if (!value) return ''
  if (value >= 1000000) return `${(value / 1000000).toFixed(1)}M`
  if (value >= 1000) return `${(value / 1000).toFixed(0)}K`
  return value.toString()
}

const handleSetPrimary = (model?: ModelInfo) => {
  const modelId = model?.id || 'default'
  emit('setPrimary', `${props.provider.name}/${modelId}`)
}

const handleSetFallback = (model?: ModelInfo) => {
  const modelId = model?.id || 'default'
  emit('setFallback', `${props.provider.name}/${modelId}`)
}
</script>

<template>
  <div
    :class="cn('oc-subpanel oc-provider-card p-4 transition-all', containsPrimary ? 'oc-provider-card-active' : '', props.class)"
  >
    <div class="flex items-start justify-between gap-3">
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2 mb-1">
          <h3 class="font-semibold truncate" style="color: var(--oc-text-primary);">{{ provider.name }}</h3>
          <Star v-if="containsPrimary" class="w-4 h-4 flex-shrink-0" style="color: var(--oc-accent);" title="包含主模型" />
        </div>
        <div class="text-xs space-y-0.5" style="color: var(--oc-text-muted);">
          <p class="truncate">{{ provider.baseUrl }}</p>
          <div class="flex items-center gap-3">
            <span :style="{ color: provider.hasApiKey ? 'var(--oc-success)' : 'var(--oc-text-muted)' }">
              Key: {{ provider.hasApiKey ? '✓' : '-' }}
            </span>
            <span v-if="provider.api">API: {{ provider.api }}</span>
          </div>
        </div>

        <div class="mt-3">
          <button @click="showModels = !showModels"
                  class="flex items-center gap-1 text-xs transition-colors"
                  style="color: var(--oc-text-muted);">
            <Cpu class="w-3 h-3" />
            <span>{{ provider.modelCount }} 个模型</span>
            <component :is="showModels ? ChevronUp : ChevronDown" class="w-3 h-3" />
          </button>

          <div v-if="showModels" class="mt-2 space-y-1 pl-2 border-l-2" style="border-color: var(--oc-divider);">
            <div v-for="model in provider.models" :key="model.id"
                 class="group flex items-center justify-between gap-2 rounded-[9px] border px-2 py-1.5 text-xs"
                 style="border-color: var(--oc-divider-soft); background: color-mix(in srgb, var(--oc-card-elevated) 82%, transparent);">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-1">
                  <span class="truncate font-medium" style="color: var(--oc-text-primary);">{{ model.name || model.id }}</span>
                  <Brain v-if="model.reasoning" class="w-3 h-3 flex-shrink-0" style="color: var(--oc-accent);" />
                  <span v-if="model.contextWindow" class="flex flex-shrink-0 items-center gap-0.5" style="color: var(--oc-text-muted);">
                    <Zap class="w-2.5 h-2.5" />{{ formatContextWindow(model.contextWindow) }}
                  </span>
                </div>
                <div class="truncate" style="color: var(--oc-text-muted);">{{ model.id }}</div>
              </div>
              <div class="flex gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                <Button variant="ghost" size="sm" @click="handleSetPrimary(model)" title="主要" class="h-6 w-6 p-0">
                  <Star class="w-3 h-3" />
                </Button>
                <Button variant="ghost" size="sm" @click="handleSetFallback(model)" title="备用" class="h-6 w-6 p-0">
                  <Shield class="w-3 h-3" />
                </Button>
                <Button variant="ghost" size="sm" @click="handleRemoveModel(model.id)" title="删除" class="h-6 w-6 p-0" style="color: var(--oc-danger);">
                  <X class="w-3 h-3" />
                </Button>
              </div>
            </div>

            <button @click="emit('addModel', provider.name)"
                    class="flex w-full items-center gap-1 px-2 py-1 text-left text-xs transition-colors"
                    style="color: var(--oc-text-secondary);">
              <Plus class="w-3 h-3" />
              添加模型
            </button>
          </div>
        </div>
      </div>

      <div class="flex flex-col gap-1 flex-shrink-0">
        <Button variant="ghost" size="sm" @click="emit('edit')" class="h-7 text-xs" title="编辑">
          <Pencil class="w-3 h-3" />
        </Button>
        <Button variant="ghost" size="sm" @click="emit('delete')" class="h-7 text-xs" style="color: var(--oc-danger);" title="删除">
          <Trash2 class="w-3 h-3" />
        </Button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.oc-provider-card {
  overflow: visible;
  box-shadow: none;
}

.oc-provider-card-active {
  border-color: color-mix(in srgb, var(--oc-input-focus) 78%, var(--oc-card-border) 22%);
  box-shadow: none;
}
</style>
