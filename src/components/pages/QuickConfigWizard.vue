<script setup lang="ts">
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { CheckCircle2, Loader2 } from 'lucide-vue-next'
import Button from '../ui/Button.vue'
import Card from '../ui/Card.vue'
import Input from '../ui/Input.vue'
import Label from '../ui/Label.vue'
import { isPrimaryModelPlaceholder } from '../../domain/configValidation'
import type { ConfigFileInfo, OpenClawConfig } from '../../types/config'

const emit = defineEmits<{
  complete: []
}>()

interface Preset {
  name: string
  displayName: string
  baseUrl: string
}

const presets: Preset[] = [
  { name: 'deepseek', displayName: 'DeepSeek', baseUrl: 'https://api.deepseek.com' },
  { name: 'siliconflow', displayName: '硅基流动', baseUrl: 'https://api.siliconflow.cn/v1' },
  { name: 'openai', displayName: 'OpenAI', baseUrl: 'https://api.openai.com/v1' },
]

const selectedPreset = ref<Preset>(presets[0])
const apiKey = ref('')
const primaryModelId = ref('')
const validating = ref(false)
const statusMessage = ref('')
const errorMessage = ref('')

const primaryModelPath = computed(() => {
  const modelId = primaryModelId.value.trim()
  return modelId ? `${selectedPreset.value.name}/${modelId}` : ''
})

const primaryModelInvalid = computed(() =>
  isPrimaryModelPlaceholder(primaryModelPath.value)
)

const canValidate = computed(() =>
  !validating.value &&
  apiKey.value.trim().length > 0 &&
  primaryModelId.value.trim().length > 0 &&
  !primaryModelInvalid.value
)

const saveAndValidate = async () => {
  if (!canValidate.value) return

  validating.value = true
  statusMessage.value = '正在写入配置...'
  errorMessage.value = ''

  try {
    const [config, info] = await invoke<[OpenClawConfig, ConfigFileInfo]>('load_default_config')

    let next = await invoke<OpenClawConfig>('upsert_provider', {
      config,
      name: selectedPreset.value.name,
      baseUrl: selectedPreset.value.baseUrl,
      apiKey: apiKey.value.trim(),
      api: null,
    })

    next = await invoke<OpenClawConfig>('set_primary_model', {
      config: next,
      modelPath: primaryModelPath.value,
    })

    await invoke('save_config', {
      config: next,
      path: info.path,
    })

    statusMessage.value = '正在启动网关服务...'
    await invoke('install_gateway_service')

    statusMessage.value = '验证通过，正在进入工作台...'
    emit('complete')
  } catch (error) {
    errorMessage.value = `配置失败: ${error}`
  } finally {
    validating.value = false
  }
}
</script>

<template>
  <Card class="p-6">
    <div class="mb-4">
      <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">快速配置向导（3 步）</h3>
      <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">完成后自动保存并验证，确保不是 placeholder 配置。</p>
    </div>

    <div class="mb-4 grid gap-3 md:grid-cols-3">
      <div class="rounded-[12px] border p-3" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
        <p class="text-xs font-medium" style="color: var(--oc-text-muted);">Step 1</p>
        <p class="mt-1 text-sm" style="color: var(--oc-text-primary);">选择 Provider 模板</p>
      </div>
      <div class="rounded-[12px] border p-3" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
        <p class="text-xs font-medium" style="color: var(--oc-text-muted);">Step 2</p>
        <p class="mt-1 text-sm" style="color: var(--oc-text-primary);">设置主模型</p>
      </div>
      <div class="rounded-[12px] border p-3" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated);">
        <p class="text-xs font-medium" style="color: var(--oc-text-muted);">Step 3</p>
        <p class="mt-1 text-sm" style="color: var(--oc-text-primary);">保存并验证</p>
      </div>
    </div>

    <div class="space-y-4">
      <div>
        <Label class="mb-1.5 block text-sm">Provider 模板</Label>
        <div class="grid gap-2 sm:grid-cols-3">
          <button
            v-for="preset in presets"
            :key="preset.name"
            type="button"
            class="rounded-[10px] border px-3 py-2 text-left text-sm transition-colors"
            :style="{
              borderColor: selectedPreset.name === preset.name ? 'var(--oc-card-border-strong)' : 'var(--oc-card-border)',
              background: selectedPreset.name === preset.name ? 'var(--oc-item-active)' : 'var(--oc-card)'
            }"
            @click="selectedPreset = preset"
          >
            <p style="color: var(--oc-text-primary);">{{ preset.displayName }}</p>
            <p class="mt-0.5 text-xs" style="color: var(--oc-text-muted);">{{ preset.baseUrl }}</p>
          </button>
        </div>
      </div>

      <div>
        <Label class="mb-1.5 block text-sm">API Key</Label>
        <Input v-model="apiKey" type="password" placeholder="输入 API Key" />
      </div>

      <div>
        <Label class="mb-1.5 block text-sm">主模型 ID</Label>
        <Input v-model="primaryModelId" placeholder="例如：deepseek-chat" />
        <p class="mt-1 text-xs" style="color: var(--oc-text-muted);">
          完整路径：{{ primaryModelPath || `${selectedPreset.name}/...` }}
        </p>
        <p v-if="primaryModelInvalid && primaryModelId.trim()" class="mt-1 text-xs" style="color: var(--oc-danger);">
          主模型不能是 placeholder
        </p>
      </div>
    </div>

    <div class="mt-6 flex items-center justify-between gap-3">
      <div class="text-xs" style="color: var(--oc-text-muted);">
        {{ statusMessage || '配置将写入默认配置文件并立即进行生效校验。' }}
      </div>
      <Button :disabled="!canValidate" @click="saveAndValidate">
        <Loader2 v-if="validating" class="h-4 w-4 animate-spin" />
        <CheckCircle2 v-else class="h-4 w-4" />
        保存并验证
      </Button>
    </div>

    <p v-if="errorMessage" class="mt-3 text-sm" style="color: var(--oc-danger);">
      {{ errorMessage }}
    </p>
  </Card>
</template>
