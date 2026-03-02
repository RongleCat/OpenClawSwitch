<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { Settings, Zap, Loader2 } from 'lucide-vue-next'
import Button from '../ui/Button.vue'
import Card from '../ui/Card.vue'

const emit = defineEmits<{
  complete: []
}>()

const configuring = ref(false)
const configMode = ref<'manual' | 'default' | null>(null)
const statusMessage = ref('')

const handleManualConfig = async () => {
  configuring.value = true
  configMode.value = 'manual'
  statusMessage.value = '正在打开终端...'

  try {
    const command = 'openclaw onboard --install-daemon'
    await invoke<string>('open_terminal_with_command', { command })
    statusMessage.value = '已打开终端，请在终端中完成配置'

    // 等待 2 秒后自动完成
    setTimeout(() => {
      emit('complete')
    }, 2000)
  } catch (e) {
    statusMessage.value = `打开终端失败: ${e}`
    configuring.value = false
  }
}

const handleDefaultConfig = async () => {
  configuring.value = true
  configMode.value = 'default'
  statusMessage.value = '正在生成默认配置...'

  try {
    // 生成配置文件
    await invoke<string>('generate_default_config')
    statusMessage.value = '配置文件已生成，正在安装网关服务...'

    // 安装网关服务
    await invoke<string>('install_gateway_service')
    statusMessage.value = '网关服务已安装，配置完成！'

    // 等待 1 秒后完成
    setTimeout(() => {
      emit('complete')
    }, 1000)
  } catch (e) {
    statusMessage.value = `配置失败: ${e}`
    configuring.value = false
  }
}
</script>

<template>
  <div class="h-full flex flex-col bg-gray-50 p-8">
    <div class="max-w-2xl mx-auto w-full flex flex-col flex-1 justify-center">
      <!-- 标题 -->
      <div class="text-center mb-8">
        <h2 class="text-2xl font-bold text-gray-900 mb-2">安装完成！</h2>
        <p class="text-gray-600">选择配置方式以完成设置</p>
      </div>

      <!-- 配置选项 -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
        <!-- 手动配置 -->
        <Card
          class="p-6 cursor-pointer hover:border-blue-400 transition-all bg-white shadow-sm"
          :class="configMode === 'manual' ? 'border-blue-400 bg-blue-50' : ''"
          @click="!configuring && handleManualConfig()"
        >
          <div class="flex flex-col items-center text-center gap-3">
            <div class="w-12 h-12 rounded-full bg-blue-100 flex items-center justify-center">
              <Settings class="w-6 h-6 text-blue-600" />
            </div>
            <div>
              <h3 class="font-semibold text-gray-900 mb-1">手动配置</h3>
              <p class="text-sm text-gray-600">
                打开终端，通过交互式向导完成配置
              </p>
            </div>
            <div class="text-xs text-gray-500 mt-2">
              推荐有经验的用户使用
            </div>
          </div>
        </Card>

        <!-- 默认配置 -->
        <Card
          class="p-6 cursor-pointer hover:border-green-400 transition-all bg-white shadow-sm"
          :class="configMode === 'default' ? 'border-green-400 bg-green-50' : ''"
          @click="!configuring && handleDefaultConfig()"
        >
          <div class="flex flex-col items-center text-center gap-3">
            <div class="w-12 h-12 rounded-full bg-green-100 flex items-center justify-center">
              <Zap class="w-6 h-6 text-green-600" />
            </div>
            <div>
              <h3 class="font-semibold text-gray-900 mb-1">使用默认配置</h3>
              <p class="text-sm text-gray-600">
                自动生成最精简配置并启动网关服务
              </p>
            </div>
            <div class="text-xs text-gray-500 mt-2">
              推荐快速开始使用
            </div>
          </div>
        </Card>
      </div>

      <!-- 状态消息 -->
      <div v-if="configuring" class="text-center">
        <div class="flex items-center justify-center gap-2 text-sm text-gray-600">
          <Loader2 class="w-4 h-4 animate-spin" />
          <span>{{ statusMessage }}</span>
        </div>
      </div>

      <!-- 说明 -->
      <Card class="p-4 bg-gray-100">
        <p class="text-xs text-gray-700">
          <strong>提示：</strong>默认配置会生成一个最精简的可启动配置，模型配置留空（需要后续手动配置）。
          所有 hooks 功能已启用，网关服务会自动在后台运行。
        </p>
      </Card>
    </div>
  </div>
</template>
