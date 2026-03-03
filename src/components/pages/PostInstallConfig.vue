<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { Settings, Wand2, Loader2 } from 'lucide-vue-next'
import Card from '../ui/Card.vue'
import QuickConfigWizard from './QuickConfigWizard.vue'

const emit = defineEmits<{
  complete: []
}>()

const mode = ref<'manual' | 'wizard' | null>(null)
const configuring = ref(false)
const statusMessage = ref('')

const startManual = async () => {
  configuring.value = true
  mode.value = 'manual'
  statusMessage.value = '正在打开终端...'

  try {
    await invoke<string>('open_terminal_with_command', { command: 'openclaw onboard --install-daemon' })
    statusMessage.value = '已打开终端，请按提示完成配置。'
    setTimeout(() => {
      emit('complete')
    }, 1500)
  } catch (error) {
    statusMessage.value = `打开终端失败: ${error}`
    configuring.value = false
  }
}

const startWizard = () => {
  mode.value = 'wizard'
}

const backToSelector = () => {
  if (configuring.value) return
  mode.value = null
  statusMessage.value = ''
}
</script>

<template>
  <div class="oc-page-root flex flex-col">
    <div class="max-w-4xl mx-auto w-full flex flex-col flex-1">
      <template v-if="mode === 'wizard'">
        <div class="mb-3">
          <button class="oc-toolbar-btn h-9 px-3 text-sm" type="button" @click="backToSelector">
            返回配置方式选择
          </button>
        </div>
        <QuickConfigWizard @complete="emit('complete')" />
      </template>

      <template v-else>
        <div class="text-center mb-8 mt-6">
          <h2 class="mb-2 text-2xl font-bold" style="color: var(--oc-text-primary);">安装完成，继续完成可用配置</h2>
          <p style="color: var(--oc-text-muted);">请选择配置路径，确保主模型有效并可启动网关。</p>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
          <Card
            class="cursor-pointer p-6 transition-all"
            :style="mode === 'manual'
              ? 'border-color: var(--oc-accent); background: color-mix(in srgb, var(--oc-accent) 12%, var(--oc-card) 88%);'
              : ''"
            @click="!configuring && startManual()"
          >
            <div class="flex flex-col items-center text-center gap-3">
              <div class="flex h-12 w-12 items-center justify-center rounded-full" style="background: color-mix(in srgb, var(--oc-accent) 16%, transparent);">
                <Settings class="w-6 h-6" style="color: var(--oc-accent);" />
              </div>
              <div>
                <h3 class="mb-1 font-semibold" style="color: var(--oc-text-primary);">手动配置</h3>
                <p class="text-sm" style="color: var(--oc-text-muted);">
                  打开终端，使用 OpenClaw 官方命令继续配置
                </p>
              </div>
            </div>
          </Card>

          <Card
            class="cursor-pointer p-6 transition-all"
            :style="mode === 'wizard'
              ? 'border-color: var(--oc-success); background: color-mix(in srgb, var(--oc-success) 12%, var(--oc-card) 88%);'
              : ''"
            @click="startWizard"
          >
            <div class="flex flex-col items-center text-center gap-3">
              <div class="flex h-12 w-12 items-center justify-center rounded-full" style="background: color-mix(in srgb, var(--oc-success) 16%, transparent);">
                <Wand2 class="w-6 h-6" style="color: var(--oc-success);" />
              </div>
              <div>
                <h3 class="mb-1 font-semibold" style="color: var(--oc-text-primary);">快速配置向导</h3>
                <p class="text-sm" style="color: var(--oc-text-muted);">
                  3 步配置 Provider 和主模型，并自动保存验证
                </p>
              </div>
            </div>
          </Card>
        </div>

        <div v-if="configuring" class="text-center">
          <div class="flex items-center justify-center gap-2 text-sm" style="color: var(--oc-text-muted);">
            <Loader2 class="w-4 h-4 animate-spin" />
            <span>{{ statusMessage }}</span>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>
