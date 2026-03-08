<script setup lang="ts">
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { Settings, Wand2, Loader2, Shield } from 'lucide-vue-next'
import Card from '../ui/Card.vue'
import Button from '../ui/Button.vue'
import {
  formatGatewayInstallError,
  isAdminRequiredGatewayInstallError,
} from '../../domain/postInstallError'
import { waitForGatewayReady } from '../../domain/gatewayStartup'

const emit = defineEmits<{
  complete: []
}>()

const mode = ref<'manual' | 'default' | null>(null)
const configuring = ref(false)
const statusMessage = ref('')
const errorMessage = ref('')
const adminRelaunching = ref(false)
const adminRelaunchMessage = ref('')

const canRelaunchAsAdmin = computed(() =>
  isAdminRequiredGatewayInstallError(errorMessage.value)
)

const resetMessages = () => {
  statusMessage.value = ''
  errorMessage.value = ''
  adminRelaunchMessage.value = ''
}

const startManual = async () => {
  configuring.value = true
  mode.value = 'manual'
  resetMessages()
  statusMessage.value = '正在打开终端...'

  try {
    await invoke<string>('open_terminal_with_command', { command: 'openclaw onboard --install-daemon' })
    statusMessage.value = '已打开终端，请按提示完成配置。'
    setTimeout(() => {
      emit('complete')
    }, 1500)
  } catch (error) {
    errorMessage.value = formatGatewayInstallError(`打开终端失败: ${error}`)
    configuring.value = false
  }
}

const startDefaultConfig = async () => {
  configuring.value = true
  mode.value = 'default'
  resetMessages()
  statusMessage.value = '正在写入默认配置...'

  try {
    await invoke<string>('generate_default_config')
    statusMessage.value = '正在安装网关服务...'
    await invoke<string>('install_gateway_service')
    statusMessage.value = '正在等待网关就绪...'
    const ready = await waitForGatewayReady(
      async () => invoke<boolean>('health_check_gateway'),
      {
        maxAttempts: 20,
        intervalMs: 1000,
      }
    )
    if (!ready) {
      throw new Error('网关服务已安装，但在预期时间内未完成启动')
    }
    statusMessage.value = '默认配置已写入，网关服务安装完成。'
    setTimeout(() => {
      emit('complete')
    }, 1000)
  } catch (error) {
    errorMessage.value = formatGatewayInstallError(`默认配置失败: ${error}`)
    configuring.value = false
  }
}

const relaunchAsAdmin = async () => {
  adminRelaunching.value = true
  adminRelaunchMessage.value = ''

  try {
    adminRelaunchMessage.value = await invoke<string>('relaunch_as_admin')
  } catch (error) {
    errorMessage.value = formatGatewayInstallError(`管理员重启失败: ${error}`)
  } finally {
    adminRelaunching.value = false
  }
}
</script>

<template>
  <div class="oc-page-root flex flex-col">
    <div class="max-w-4xl mx-auto w-full flex flex-col flex-1">
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
          :style="mode === 'default'
            ? 'border-color: var(--oc-success); background: color-mix(in srgb, var(--oc-success) 12%, var(--oc-card) 88%);'
            : ''"
          @click="!configuring && startDefaultConfig()"
        >
          <div class="flex flex-col items-center text-center gap-3">
            <div class="flex h-12 w-12 items-center justify-center rounded-full" style="background: color-mix(in srgb, var(--oc-success) 16%, transparent);">
              <Wand2 class="w-6 h-6" style="color: var(--oc-success);" />
            </div>
            <div>
              <h3 class="mb-1 font-semibold" style="color: var(--oc-text-primary);">使用默认配置</h3>
              <p class="text-sm" style="color: var(--oc-text-muted);">
                写入默认配置文件并安装网关服务
              </p>
            </div>
          </div>
        </Card>
      </div>

      <div class="text-center">
        <div v-if="configuring" class="flex items-center justify-center gap-2 text-sm" style="color: var(--oc-text-muted);">
          <Loader2 class="w-4 h-4 animate-spin" />
          <span>{{ statusMessage }}</span>
        </div>
        <p
          v-if="errorMessage"
          class="mx-auto mt-2 max-w-2xl whitespace-pre-line text-sm"
          style="color: var(--oc-danger);"
        >
          {{ errorMessage }}
        </p>
        <div
          v-if="canRelaunchAsAdmin"
          class="mt-4 flex flex-col items-center gap-3"
        >
          <Button
            variant="default"
            :disabled="adminRelaunching"
            @click="relaunchAsAdmin"
          >
            <Shield class="h-4 w-4" />
            {{ adminRelaunching ? '正在请求管理员权限...' : '以管理员身份重启' }}
          </Button>
          <p
            v-if="adminRelaunchMessage"
            class="max-w-2xl whitespace-pre-line text-sm"
            style="color: var(--oc-text-muted);"
          >
            {{ adminRelaunchMessage }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
