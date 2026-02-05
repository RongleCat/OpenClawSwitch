<script setup lang="ts">
import { ref, computed } from 'vue'
import { cn } from '@/lib/utils'
import { Trash2, Star, Shield, ChevronDown, ChevronUp, Cpu, Brain, Zap, Plus, X } from 'lucide-vue-next'
import Button from './ui/Button.vue'
import type { ProviderInfo, ModelInfo } from '@/types/config'

interface Props {
  provider: ProviderInfo
  isPrimary?: boolean
  isFallback?: boolean
  class?: string
}

const props = withDefaults(defineProps<Props>(), {
  isPrimary: false,
  isFallback: false
})

const emit = defineEmits<{
  setPrimary: [modelPath: string]
  setFallback: [modelPath: string]
  addModel: [providerName: string]
  removeModel: [providerName: string, modelId: string]
  delete: []
}>()

const showModels = ref(false)

const handleRemoveModel = (modelId: string) => {
  emit('removeModel', props.provider.name, modelId)
}

const statusColor = computed(() => {
  if (props.isPrimary) return 'border-l-4 border-l-blue-500 bg-blue-50/50 dark:bg-blue-950/50'
  if (props.isFallback) return 'border-l-4 border-l-amber-500 bg-amber-50/50 dark:bg-amber-950/50'
  return 'border-l-4 border-l-gray-200 dark:border-l-gray-700'
})

const statusText = computed(() => {
  if (props.isPrimary) return '主要'
  if (props.isFallback) return '备用'
  return ''
})

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
  <div :class="cn('rounded-lg border p-3 transition-all hover:shadow-sm', statusColor, props.class)">
    <div class="flex items-start justify-between gap-3">
      <!-- 提供商信息 -->
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2 mb-1">
          <h3 class="font-semibold truncate">{{ provider.name }}</h3>
          <span v-if="statusText" class="px-1.5 py-0.5 rounded text-xs font-medium"
                :class="isPrimary ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-200' : 'bg-amber-100 text-amber-700 dark:bg-amber-900 dark:text-amber-200'">
            {{ statusText }}
          </span>
        </div>
        <div class="text-xs text-muted-foreground space-y-0.5">
          <p class="truncate">{{ provider.baseUrl }}</p>
          <div class="flex items-center gap-3">
            <span :class="provider.hasApiKey ? 'text-green-600 dark:text-green-400' : ''">
              Key: {{ provider.hasApiKey ? '✓' : '-' }}
            </span>
            <span v-if="provider.api">API: {{ provider.api }}</span>
          </div>
        </div>

        <!-- 模型列表 -->
        <div class="mt-2">
          <button @click="showModels = !showModels"
                  class="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground">
            <Cpu class="w-3 h-3" />
            <span>{{ provider.modelCount }} 个模型</span>
            <component :is="showModels ? ChevronUp : ChevronDown" class="w-3 h-3" />
          </button>

          <div v-if="showModels" class="mt-2 space-y-1 pl-2 border-l border-gray-200 dark:border-gray-700">
            <div v-for="model in provider.models" :key="model.id"
                 class="flex items-center justify-between gap-2 py-1 px-2 rounded text-xs bg-white/50 dark:bg-black/20 group">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-1">
                  <span class="font-medium truncate">{{ model.name || model.id }}</span>
                  <Brain v-if="model.reasoning" class="w-3 h-3 text-purple-500 flex-shrink-0" />
                  <span v-if="model.contextWindow" class="text-muted-foreground flex items-center gap-0.5 flex-shrink-0">
                    <Zap class="w-2.5 h-2.5" />{{ formatContextWindow(model.contextWindow) }}
                  </span>
                </div>
                <div class="text-muted-foreground truncate">{{ model.id }}</div>
              </div>
              <div class="flex gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                <Button variant="ghost" size="sm" @click="handleSetPrimary(model)" title="主要" class="h-6 w-6 p-0">
                  <Star class="w-3 h-3" />
                </Button>
                <Button variant="ghost" size="sm" @click="handleSetFallback(model)" title="备用" class="h-6 w-6 p-0">
                  <Shield class="w-3 h-3" />
                </Button>
                <Button variant="ghost" size="sm" @click="handleRemoveModel(model.id)" title="删除" class="h-6 w-6 p-0 text-destructive">
                  <X class="w-3 h-3" />
                </Button>
              </div>
            </div>

            <!-- 添加模型按钮 -->
            <button @click="emit('addModel', provider.name)"
                    class="flex items-center gap-1 py-1 px-2 text-xs text-muted-foreground hover:text-foreground w-full text-left">
              <Plus class="w-3 h-3" />
              添加模型
            </button>
          </div>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="flex flex-col gap-1 flex-shrink-0">
        <Button v-if="!isPrimary" variant="outline" size="sm" @click="handleSetPrimary()" class="h-7 text-xs">
          <Star class="w-3 h-3" />
          主要
        </Button>
        <Button v-if="!isFallback && !isPrimary" variant="outline" size="sm" @click="handleSetFallback()" class="h-7 text-xs">
          <Shield class="w-3 h-3" />
          备用
        </Button>
        <Button v-if="isFallback" variant="outline" size="sm" @click="emit('setFallback', '')" class="h-7 text-xs">
          取消
        </Button>
        <Button variant="ghost" size="sm" @click="emit('delete')" class="h-7 text-xs text-destructive">
          <Trash2 class="w-3 h-3" />
        </Button>
      </div>
    </div>
  </div>
</template>
