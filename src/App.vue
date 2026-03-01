<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import StatusDashboard from './components/pages/StatusDashboard.vue'
import InstallPage from './components/pages/InstallPage.vue'
import ConfigPage from './components/pages/ConfigPage.vue'
import SshConnectModal from './components/SshConnectModal.vue'
import SshFingerprintDialog from './components/SshFingerprintDialog.vue'
import Toast from './components/ui/Toast.vue'
import { ChevronDown, Monitor, Plus, Wifi, WifiOff } from 'lucide-vue-next'
import type { EnvironmentStatus, FingerprintInfo, SshProfile } from './types/config'

// ============================================================================
// 环境管理
// ============================================================================

type EnvMode = 'local' | 'ssh'

interface EnvironmentInfo {
  mode: EnvMode
  label: string
  sshProfile?: SshProfile
}

const environments = ref<EnvironmentInfo[]>([
  { mode: 'local', label: '本地' }
])
const currentEnvIndex = ref(0)
const currentEnv = computed(() => environments.value[currentEnvIndex.value])
const showEnvDropdown = ref(false)

// ============================================================================
// 应用状态
// ============================================================================

const envStatus = ref<EnvironmentStatus | null>(null)
const openclawInstalled = computed(() => envStatus.value?.openclaw.installed ?? false)
const loading = ref(false)

// 当前打开的工具面板
const activeToolPanel = ref<string | null>(null)

// SSH 状态
const showSshModal = ref(false)
const showFingerprintDialog = ref(false)
const sshConnected = ref(false)
const sshFingerprint = ref<FingerprintInfo | null>(null)
const sshFingerprintCallback = ref<(() => void) | null>(null)

// Toast
const toast = ref<{ type: 'success' | 'error'; message: string } | null>(null)
let toastTimer: ReturnType<typeof setTimeout> | null = null

const showToast = (type: 'success' | 'error', message: string) => {
  if (toastTimer) clearTimeout(toastTimer)
  toast.value = { type, message }
  toastTimer = setTimeout(() => { toast.value = null }, 4000)
}

const closeToast = () => {
  if (toastTimer) clearTimeout(toastTimer)
  toast.value = null
}

// ============================================================================
// 环境检测
// ============================================================================

const checkEnvironment = async () => {
  loading.value = true
  try {
    if (currentEnv.value.mode === 'ssh' && sshConnected.value) {
      envStatus.value = await invoke<EnvironmentStatus>('ssh_check_environment')
    } else {
      envStatus.value = await invoke<EnvironmentStatus>('check_environment')
    }
  } catch (e) {
    console.error('环境检测失败:', e)
  } finally {
    loading.value = false
  }
}

const selectEnvironment = async (index: number) => {
  showEnvDropdown.value = false
  currentEnvIndex.value = index
  activeToolPanel.value = null
  envStatus.value = null

  const env = environments.value[index]
  if (env.mode === 'local') {
    // 断开 SSH（如果已连接）
    if (sshConnected.value) {
      try { await invoke('ssh_disconnect') } catch {}
      sshConnected.value = false
    }
    await checkEnvironment()
  } else if (env.mode === 'ssh') {
    // SSH 模式：如果已连接则直接检测，否则需要先连接
    if (sshConnected.value) {
      await checkEnvironment()
    } else if (env.sshProfile) {
      // 自动发起连接
      showSshModal.value = true
    }
  }
}

// ============================================================================
// SSH 连接
// ============================================================================

const addSshEnvironment = () => {
  showSshModal.value = true
}

const handleFingerprint = (info: FingerprintInfo, onConfirm: () => void) => {
  // 如果指纹已知，直接确认
  if (info.isKnown) {
    onConfirm()
    return
  }
  sshFingerprint.value = info
  sshFingerprintCallback.value = onConfirm
  showFingerprintDialog.value = true
}

const confirmFingerprint = async () => {
  showFingerprintDialog.value = false
  // 保存指纹到 known_hosts
  if (sshFingerprint.value) {
    try {
      await invoke('ssh_save_fingerprint', { fingerprint: sshFingerprint.value.sha256 })
    } catch (e) {
      console.error('保存指纹失败:', e)
    }
  }
  sshFingerprintCallback.value?.()
  sshFingerprint.value = null
  sshFingerprintCallback.value = null
}

const rejectFingerprint = async () => {
  showFingerprintDialog.value = false
  sshFingerprint.value = null
  sshFingerprintCallback.value = null
  await invoke('ssh_disconnect')
  showToast('error', '已拒绝连接')
}

const handleSshConnected = async () => {
  sshConnected.value = true
  showSshModal.value = false
  showToast('success', 'SSH 连接成功')
  await checkEnvironment()
}

// ============================================================================
// 工具面板
// ============================================================================

const openToolPanel = (toolId: string) => {
  activeToolPanel.value = activeToolPanel.value === toolId ? null : toolId
}

const closeToolPanel = () => {
  activeToolPanel.value = null
}

// ============================================================================
// 安装完成
// ============================================================================

const handleInstallComplete = async () => {
  await checkEnvironment()
  showToast('success', 'OpenClaw 安装成功!')
}

// ============================================================================
// SSH 配置加载
// ============================================================================

const loadSshProfiles = async () => {
  try {
    const profiles = await invoke<SshProfile[]>('ssh_load_profiles')
    // 添加已保存的 SSH 环境
    for (const profile of profiles) {
      if (!environments.value.find(e => e.sshProfile?.id === profile.id)) {
        environments.value.push({
          mode: 'ssh',
          label: profile.name || `${profile.host}:${profile.port}`,
          sshProfile: profile
        })
      }
    }
  } catch {}
}

// ============================================================================
// 生命周期
// ============================================================================

onMounted(async () => {
  await loadSshProfiles()
  await checkEnvironment()
})
</script>

<template>
  <div class="h-screen flex flex-col bg-gray-50 dark:bg-gray-900 overflow-hidden">
    <!-- 顶部栏 -->
    <header class="flex-shrink-0 border-b bg-white dark:bg-gray-800 px-4 py-2.5">
      <div class="flex items-center justify-between">
        <!-- 左侧：环境选择器 -->
        <div class="flex items-center gap-3">
          <div class="relative">
            <button
              @click="showEnvDropdown = !showEnvDropdown"
              class="flex items-center gap-2 px-3 py-1.5 rounded-lg border bg-white dark:bg-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors min-w-[160px]"
            >
              <component
                :is="currentEnv.mode === 'local' ? Monitor : Wifi"
                class="w-4 h-4"
                :class="currentEnv.mode === 'local' ? 'text-blue-500' : sshConnected ? 'text-green-500' : 'text-gray-400'"
              />
              <span class="text-sm font-medium truncate">{{ currentEnv.label }}</span>
              <ChevronDown class="w-4 h-4 ml-auto text-gray-400" :class="{ 'rotate-180': showEnvDropdown }" />
            </button>

            <!-- 环境下拉列表 -->
            <div
              v-if="showEnvDropdown"
              class="absolute top-full left-0 mt-1 w-56 bg-white dark:bg-gray-800 border rounded-lg shadow-lg z-50"
            >
              <div
                v-for="(env, i) in environments"
                :key="i"
                @click="selectEnvironment(i)"
                class="flex items-center gap-2 px-3 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer text-sm"
                :class="{ 'bg-blue-50 dark:bg-blue-900/20': i === currentEnvIndex }"
              >
                <component
                  :is="env.mode === 'local' ? Monitor : Wifi"
                  class="w-4 h-4"
                  :class="env.mode === 'local' ? 'text-blue-500' : 'text-green-500'"
                />
                <span class="truncate">{{ env.label }}</span>
              </div>
              <div class="border-t">
                <button
                  @click="addSshEnvironment"
                  class="flex items-center gap-2 px-3 py-2 w-full hover:bg-gray-100 dark:hover:bg-gray-700 text-sm text-blue-600"
                >
                  <Plus class="w-4 h-4" />
                  添加 SSH 连接
                </button>
              </div>
            </div>
          </div>

          <h1 class="font-bold text-lg text-gray-700 dark:text-gray-300">OpenClawSwitch</h1>
        </div>

        <!-- 右侧：状态指示 -->
        <div class="flex items-center gap-2 text-sm text-muted-foreground">
          <span v-if="envStatus" class="flex items-center gap-1">
            <span
              class="w-2 h-2 rounded-full"
              :class="openclawInstalled ? 'bg-green-500' : 'bg-red-400'"
            />
            {{ openclawInstalled ? `OpenClaw ${envStatus.openclaw.version || ''}` : '未安装' }}
          </span>
        </div>
      </div>
    </header>

    <!-- 主内容区域 -->
    <main class="flex-1 overflow-hidden flex">
      <!-- 主面板 -->
      <div class="flex-1 overflow-hidden">
        <!-- 未安装 → 安装页面 -->
        <InstallPage
          v-if="envStatus && !openclawInstalled"
          @install-complete="handleInstallComplete"
        />

        <!-- 已安装 → 状态面板 -->
        <StatusDashboard
          v-else-if="envStatus && openclawInstalled"
          :env-status="envStatus"
          :active-tool="activeToolPanel"
          :show-toast="showToast"
          @open-tool="openToolPanel"
          @refresh="checkEnvironment"
        />

        <!-- 加载中 -->
        <div v-else class="flex items-center justify-center h-full">
          <div class="text-center text-muted-foreground">
            <div class="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-3" />
            <p class="text-sm">正在检测环境...</p>
          </div>
        </div>
      </div>

      <!-- 工具全屏面板 -->
      <transition name="fade">
        <div
          v-if="activeToolPanel === 'config'"
          class="absolute inset-0 bg-white dark:bg-gray-800 z-50 flex flex-col"
        >
          <ConfigPage
            :show-toast="showToast"
            :env-mode="currentEnv.mode"
            :env-ssh-connected="sshConnected"
            @close="closeToolPanel"
          />
        </div>
      </transition>
    </main>

    <!-- SSH 连接弹窗 -->
    <SshConnectModal
      v-if="showSshModal"
      @close="showSshModal = false"
      @connected="handleSshConnected"
      @fingerprint="handleFingerprint"
    />

    <!-- SSH 指纹确认 -->
    <SshFingerprintDialog
      v-if="showFingerprintDialog && sshFingerprint"
      :fingerprint="sshFingerprint"
      @confirm="confirmFingerprint"
      @reject="rejectFingerprint"
    />

    <!-- Toast -->
    <Toast v-if="toast" :type="toast.type" :message="toast.message" @close="closeToast" />

    <!-- Loading 遮罩 -->
    <div v-if="loading" class="fixed inset-0 bg-black/20 flex items-center justify-center z-[100]">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-4 shadow-xl flex items-center gap-3">
        <div class="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
        <span class="text-sm font-medium">加载中...</span>
      </div>
    </div>
  </div>
</template>

<style>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
