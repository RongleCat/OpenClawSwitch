<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import {
  ChevronDown,
  ExternalLink,
  Monitor,
  Moon,
  RefreshCw,
  Settings2,
  Sun,
  Wifi,
} from 'lucide-vue-next'
import StatusDashboard from './components/pages/StatusDashboard.vue'
import InstallPage from './components/pages/InstallPage.vue'
import ConfigPage from './components/pages/ConfigPage.vue'
import MessageChannelsPage from './components/pages/MessageChannelsPage.vue'
import DiagnosticsPage from './components/pages/DiagnosticsPage.vue'
import SshConnectModal from './components/SshConnectModal.vue'
import SshFingerprintDialog from './components/SshFingerprintDialog.vue'
import Toast from './components/ui/Toast.vue'
import { deriveAppState } from './domain/appState'
import { NAV_ITEMS, type NavPage } from './domain/navigation'
import { isPrimaryModelPlaceholder } from './domain/configValidation'
import appIcon from './assets/app-icon.png'
import type {
  ConfigFileInfo,
  EnvironmentStatus,
  FingerprintInfo,
  OpenClawConfig,
  SshProfile,
} from './types/config'

type EnvMode = 'local' | 'ssh'
type ThemeMode = 'system' | 'light' | 'dark'

interface EnvironmentInfo {
  mode: EnvMode
  label: string
  sshProfile?: SshProfile
}

const environments = ref<EnvironmentInfo[]>([{ mode: 'local', label: '本地环境' }])
const currentEnvIndex = ref(0)
const currentEnv = computed(() => environments.value[currentEnvIndex.value])
const targetMode = ref<EnvMode | null>(null)

const showEnvDropdown = ref(false)
const showSshModal = ref(false)
const showFingerprintDialog = ref(false)
const sshConnected = ref(false)
const sshFingerprint = ref<FingerprintInfo | null>(null)
const sshFingerprintCallback = ref<(() => void) | null>(null)
const themeStorageKey = 'openclawswitch.theme.mode'
const themeMode = ref<ThemeMode>('system')
const themeModeCycle: ThemeMode[] = ['system', 'light', 'dark']

const envStatus = ref<EnvironmentStatus | null>(null)
const configLoaded = ref(false)
const primaryModelValid = ref(false)
const gatewayReachable = ref(false)
const lastActionFailed = ref(false)

const loading = ref(false)
const loadingMessage = ref('加载中...')
const activeNav = ref<NavPage>('overview')

const toast = ref<{ type: 'success' | 'error'; message: string } | null>(null)
let toastTimer: ReturnType<typeof setTimeout> | null = null

const openclawInstalled = computed(() => envStatus.value?.openclaw.installed ?? false)
const envConnected = computed(() => (currentEnv.value.mode === 'local' ? true : sshConnected.value))

const appState = computed(() =>
  deriveAppState({
    envConnected: envConnected.value,
    openclawInstalled: openclawInstalled.value,
    configLoaded: configLoaded.value,
    primaryModelValid: primaryModelValid.value,
    gatewayReachable: gatewayReachable.value,
    lastActionFailed: lastActionFailed.value,
  })
)

type GateState = 'NO_TARGET' | 'NEED_INSTALL' | 'NEED_CONFIG' | null

const gateState = computed<GateState>(() => {
  if (appState.value === 'NO_TARGET') return 'NO_TARGET'
  if (appState.value === 'NEED_INSTALL') return 'NEED_INSTALL'
  if (appState.value === 'NEED_CONFIG') return 'NEED_CONFIG'
  return null
})

const isGateActive = computed(() => gateState.value !== null)

const stateTextMap: Record<string, string> = {
  NO_TARGET: '未连接环境',
  NEED_INSTALL: '待安装',
  NEED_CONFIG: '待配置',
  READY: '可用',
  DEGRADED: '可用但降级',
  ERROR: '错误',
}

const navMeta: Record<NavPage, { title: string; subtitle: string }> = {
  overview: {
    title: '工作台',
    subtitle: '快速确认可用状态与下一步操作。',
  },
  'ai-config': {
    title: '模型配置',
    subtitle: '保存配置并验证生效，避免“假成功”。',
  },
  diagnostics: {
    title: '服务诊断',
    subtitle: '遇到异常时快速定位问题并执行修复动作。',
  },
  channels: {
    title: '消息渠道',
    subtitle: '通知增强能力，不阻塞 OpenClaw 主流程。',
  },
  settings: {
    title: '系统设置',
    subtitle: '管理环境、SSH Profile 与连接偏好。',
  },
}

const pageMeta = computed(() => navMeta[activeNav.value])
const topbarTitle = computed(() =>
  isGateActive.value ? '安装与接入' : pageMeta.value.title
)
const themeModeLabel = computed(() => {
  if (themeMode.value === 'light') return '浅色'
  if (themeMode.value === 'dark') return '深色'
  return '跟随系统'
})
const themeModeIcon = computed(() => {
  if (themeMode.value === 'light') return Sun
  if (themeMode.value === 'dark') return Moon
  return Monitor
})
const themeButtonTitle = computed(() => `主题：${themeModeLabel.value}（点击切换）`)
const globalVersionText = computed(() => envStatus.value?.openclaw.version || '--')
const configStatusText = computed(() => {
  if (!openclawInstalled.value) return '未安装'
  if (!configLoaded.value) return '未加载'
  if (!primaryModelValid.value) return '主模型无效'
  return '配置有效'
})

const isThemeMode = (raw: string | null): raw is ThemeMode =>
  raw === 'system' || raw === 'light' || raw === 'dark'

const applyThemeMode = (mode: ThemeMode) => {
  const root = document.documentElement
  if (mode === 'system') {
    root.removeAttribute('data-theme')
    return
  }
  root.setAttribute('data-theme', mode)
}

const loadThemeMode = () => {
  try {
    const raw = localStorage.getItem(themeStorageKey)
    if (isThemeMode(raw)) {
      themeMode.value = raw
    }
  } catch {
    // ignored
  }
  applyThemeMode(themeMode.value)
}

const cycleThemeMode = () => {
  const currentIndex = themeModeCycle.indexOf(themeMode.value)
  const nextIndex = (currentIndex + 1) % themeModeCycle.length
  themeMode.value = themeModeCycle[nextIndex]
}

watch(themeMode, (mode) => {
  applyThemeMode(mode)
  try {
    localStorage.setItem(themeStorageKey, mode)
  } catch {
    // ignored
  }
})

const showToast = (type: 'success' | 'error', message: string) => {
  if (toastTimer) clearTimeout(toastTimer)
  toast.value = { type, message }
  toastTimer = setTimeout(() => {
    toast.value = null
  }, 3200)
}

const closeToast = () => {
  if (toastTimer) clearTimeout(toastTimer)
  toast.value = null
}

const sleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms))

const runGatewayStartCommand = async () => {
  if (currentEnv.value.mode === 'ssh') {
    if (!sshConnected.value) {
      throw new Error('SSH 未连接，无法启动远程网关')
    }
    await invoke('ssh_start_gateway')
    return
  }
  await invoke('start_gateway')
}

const checkGatewayHealth = async (): Promise<boolean> => {
  try {
    if (currentEnv.value.mode === 'ssh') {
      if (!sshConnected.value) return false
      return await invoke<boolean>('ssh_health_check')
    }
    return await invoke<boolean>('health_check_gateway')
  } catch {
    return false
  }
}

const waitForGatewayReady = async (
  maxAttempts = 30,
  intervalMs = 2000
): Promise<boolean> => {
  for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
    const healthy = await checkGatewayHealth()
    if (healthy) return true

    loadingMessage.value = `正在等待网关启动（${attempt}/${maxAttempts}）...`

    if (attempt % 5 === 0) {
      try {
        await runGatewayStartCommand()
      } catch {
        // ignored
      }
    }

    if (attempt < maxAttempts) {
      await sleep(intervalMs)
    }
  }
  return false
}

const markActionResult = (
  _action: string,
  ok: boolean,
  successMessage: string,
  errorMessage?: string
) => {
  lastActionFailed.value = !ok
  const detail = ok ? successMessage : errorMessage || '操作失败'
  showToast(ok ? 'success' : 'error', detail)
}

const syncConfigSignals = async () => {
  if (!openclawInstalled.value) {
    configLoaded.value = false
    primaryModelValid.value = false
    gatewayReachable.value = false
    return
  }

  try {
    if (currentEnv.value.mode === 'ssh' && sshConnected.value) {
      const results = await invoke<Array<{ path: string }>>('ssh_search_config')
      if (!results.length) {
        configLoaded.value = false
        primaryModelValid.value = false
      } else {
        const raw = await invoke<string>('ssh_read_file', { path: results[0].path })
        const remoteConfig = JSON.parse(raw) as OpenClawConfig
        const primary = remoteConfig.agents?.defaults?.model?.primary
        configLoaded.value = true
        primaryModelValid.value = !isPrimaryModelPlaceholder(primary)
      }
    } else {
      const [config] = await invoke<[OpenClawConfig, ConfigFileInfo]>('load_default_config')
      const primary = config.agents?.defaults?.model?.primary
      configLoaded.value = true
      primaryModelValid.value = !isPrimaryModelPlaceholder(primary)
    }
  } catch {
    configLoaded.value = false
    primaryModelValid.value = false
  }

  try {
    if (currentEnv.value.mode === 'ssh' && sshConnected.value) {
      gatewayReachable.value = await invoke<boolean>('ssh_health_check')
    } else {
      gatewayReachable.value = await invoke<boolean>('health_check_gateway')
    }
  } catch {
    gatewayReachable.value = false
  }
}

const checkEnvironment = async () => {
  loading.value = true
  try {
    if (currentEnv.value.mode === 'ssh') {
      if (!sshConnected.value) {
        envStatus.value = null
        configLoaded.value = false
        primaryModelValid.value = false
        gatewayReachable.value = false
        return
      }
      envStatus.value = await invoke<EnvironmentStatus>('ssh_check_environment')
    } else {
      envStatus.value = await invoke<EnvironmentStatus>('check_environment')
    }

    await syncConfigSignals()
  } catch {
    envStatus.value = null
    configLoaded.value = false
    primaryModelValid.value = false
    gatewayReachable.value = false
    markActionResult('环境检测', false, '', '环境检测失败')
  } finally {
    loading.value = false
  }
}

const pageBlockedReason = (target: NavPage) => {
  if (isGateActive.value) {
    if (target === 'overview') {
      return null
    }
    return '当前处于门禁状态，请先完成前置步骤'
  }

  if (target === 'channels') return null

  if (appState.value === 'NO_TARGET' && !['settings'].includes(target)) {
    return '请先连接环境'
  }

  if (appState.value === 'NEED_INSTALL' && !['settings'].includes(target)) {
    return '请先完成安装'
  }

  if (appState.value === 'NEED_CONFIG' && target === 'overview') {
    return '请先完成模型配置并验证'
  }

  return null
}

const navigateTo = (target: NavPage) => {
  const reason = pageBlockedReason(target)
  if (reason) {
    showToast('error', reason)
    return
  }

  activeNav.value = target
}

watch(
  appState,
  (state) => {
    if (!targetMode.value && state !== 'NO_TARGET') {
      targetMode.value = currentEnv.value.mode
    }

    if (gateState.value === 'NO_TARGET' || gateState.value === 'NEED_INSTALL') {
      activeNav.value = 'overview'
      return
    }

    if (gateState.value === 'NEED_CONFIG') {
      activeNav.value = 'overview'
      return
    }

    // 不做错误态强制跳转，默认保持在工作台。
  },
  { immediate: true }
)

const handleGlobalClick = (event: MouseEvent) => {
  const target = event.target as HTMLElement
  if (!target.closest('.env-dropdown-container')) {
    showEnvDropdown.value = false
  }
}

const loadSshProfiles = async () => {
  try {
    const profiles = await invoke<SshProfile[]>('ssh_load_profiles')
    const next = [{ mode: 'local', label: '本地环境' } as EnvironmentInfo]
    for (const profile of profiles) {
      next.push({
        mode: 'ssh',
        label: profile.name || `${profile.host}:${profile.port}`,
        sshProfile: profile,
      })
    }
    environments.value = next
  } catch {
    environments.value = [{ mode: 'local', label: '本地环境' }]
  }
}

const selectEnvironment = async (index: number) => {
  showEnvDropdown.value = false
  currentEnvIndex.value = index
  envStatus.value = null
  lastActionFailed.value = false

  const selected = environments.value[index]
  targetMode.value = selected.mode
  if (selected.mode === 'local') {
    if (sshConnected.value) {
      try {
        await invoke('ssh_disconnect')
      } catch {
        // ignored
      }
      sshConnected.value = false
    }
    await checkEnvironment()
    return
  }

  if (!sshConnected.value) {
    showSshModal.value = true
    return
  }

  await checkEnvironment()
}

const chooseLocalTarget = async () => {
  targetMode.value = 'local'
  await selectEnvironment(0)
}

const chooseSshTarget = async () => {
  targetMode.value = 'ssh'
  const sshIndex = environments.value.findIndex((item) => item.mode === 'ssh')
  if (sshIndex >= 0) {
    await selectEnvironment(sshIndex)
    return
  }
  showSshModal.value = true
}

const runManualConfig = async () => {
  try {
    await invoke('open_terminal_with_command', {
      command: 'openclaw onboard --install-daemon',
    })
    markActionResult('手动配置', true, '已打开终端执行 openclaw onboard --install-daemon')
  } catch (error) {
    markActionResult('手动配置', false, '', `打开终端失败: ${error}`)
  }
}

const applyDefaultConfig = async () => {
  loading.value = true
  loadingMessage.value = '正在写入默认配置...'
  try {
    await invoke('generate_default_config')
    loadingMessage.value = '正在安装网关服务...'
    await invoke('install_gateway_service')

    loadingMessage.value = '正在启动网关服务...'
    await runGatewayStartCommand()

    loadingMessage.value = '正在监控网关健康状态...'
    const ready = await waitForGatewayReady()
    if (!ready) {
      throw new Error('网关在预期时间内未启动成功，请检查日志后重试')
    }

    loadingMessage.value = '正在进入工作台...'
    await checkEnvironment()
    activeNav.value = 'overview'
    markActionResult('默认配置', true, '默认配置完成，网关已启动，已进入工作台')
  } catch (error) {
    markActionResult('默认配置', false, '', `默认配置执行失败: ${error}`)
  } finally {
    loadingMessage.value = '加载中...'
    loading.value = false
  }
}

const addSshEnvironment = () => {
  showEnvDropdown.value = false
  showSshModal.value = true
}

const handleFingerprint = (info: FingerprintInfo, onConfirm: () => void) => {
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
  if (sshFingerprint.value) {
    try {
      await invoke('ssh_save_fingerprint', { fingerprint: sshFingerprint.value.sha256 })
    } catch {
      // ignored
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
  try {
    await invoke('ssh_disconnect')
  } catch {
    // ignored
  }
  sshConnected.value = false
  showToast('error', '已拒绝 SSH 指纹，连接已取消')
}

const handleSshConnected = async () => {
  sshConnected.value = true
  showSshModal.value = false
  await checkEnvironment()
  showToast('success', 'SSH 连接成功')
}

const openDashboard = async () => {
  try {
    await invoke('open_web_ui')
    markActionResult('打开 Dashboard', true, '已静默打开 Dashboard（携带 token）')
  } catch {
    markActionResult('打开 Dashboard', false, '', '打开 Dashboard 失败')
  }
}

const openToolPanel = async (toolId: string) => {
  if (toolId === 'config') {
    navigateTo('ai-config')
    return
  }

  if (toolId === 'channels') {
    navigateTo('channels')
    return
  }

  if (toolId === 'doctor') {
    navigateTo('diagnostics')
    return
  }

  if (toolId === 'webui') {
    await openDashboard()
    return
  }

  if (toolId === 'restart') {
    try {
      if (currentEnv.value.mode === 'ssh') {
        await invoke('ssh_restart_gateway')
      } else {
        await invoke('restart_gateway')
      }
      markActionResult('重启网关', true, '网关重启命令已发送')
      await checkEnvironment()
    } catch {
      markActionResult('重启网关', false, '', '重启失败')
    }
    return
  }

  if (toolId === 'tui') {
    try {
      await invoke('open_tui')
      markActionResult('打开 TUI', true, '已打开 TUI')
    } catch {
      markActionResult('打开 TUI', false, '', '打开 TUI 失败')
    }
    return
  }

  if (toolId === 'start') {
    try {
      if (currentEnv.value.mode === 'ssh') {
        await invoke('ssh_start_gateway')
      } else {
        await invoke('start_gateway')
      }
      markActionResult('启动网关服务', true, '启动命令已发送')
      await checkEnvironment()
    } catch {
      markActionResult('启动网关服务', false, '', '启动失败')
    }
    return
  }

  if (toolId === 'stop') {
    try {
      if (currentEnv.value.mode === 'ssh') {
        await invoke('ssh_stop_gateway')
      } else {
        await invoke('stop_gateway')
      }
      markActionResult('停止网关服务', true, '停止命令已发送')
      await checkEnvironment()
    } catch {
      markActionResult('停止网关服务', false, '', '停止失败')
    }
    return
  }

  markActionResult('未知动作', false, '', '当前动作暂未开放')
}

const handleInstallComplete = async () => {
  targetMode.value = 'local'
  lastActionFailed.value = false
  await checkEnvironment()
}

onMounted(async () => {
  loadThemeMode()
  document.addEventListener('click', handleGlobalClick)
  await loadSshProfiles()
  await checkEnvironment()
})

onUnmounted(() => {
  document.removeEventListener('click', handleGlobalClick)
  if (toastTimer) clearTimeout(toastTimer)
})
</script>

<template>
  <div class="oc-app-shell">
    <div class="oc-app-window">
      <aside v-if="!isGateActive" class="oc-sidebar">
        <div class="oc-sidebar-header">
          <div class="flex items-center gap-2.5">
            <img
              :src="appIcon"
              alt="OpenClawSwitch"
              class="h-7 w-7 rounded-[8px] border"
              style="border-color: var(--oc-divider-soft);"
            />
            <div class="truncate text-sm font-semibold" style="color: var(--oc-text-primary);">OpenClawSwitch</div>
          </div>
        </div>

        <div class="oc-sidebar-body oc-main-scroll">
          <div class="space-y-1">
            <button
              v-for="item in NAV_ITEMS"
              :key="item.id"
              class="oc-sidebar-item"
              :class="{ 'oc-sidebar-item-active': activeNav === item.id }"
              type="button"
              @click="navigateTo(item.id)"
            >
              <span class="truncate">{{ item.label }}</span>
            </button>
          </div>

          <div
            class="mt-4 rounded-[12px] border p-3 text-xs"
            style="border-color: var(--oc-divider-soft); background: color-mix(in srgb, var(--oc-card-elevated) 86%, transparent 14%); color: var(--oc-text-muted);"
          >
            <div class="mb-1 flex items-center justify-between">
              <span>当前状态</span>
              <span style="color: var(--oc-text-primary);">{{ stateTextMap[appState] }}</span>
            </div>
            <div class="mb-1 flex items-center justify-between">
              <span>OpenClaw</span>
              <span style="color: var(--oc-text-primary);">{{ globalVersionText }}</span>
            </div>
            <div class="flex items-center justify-between">
              <span>配置状态</span>
              <span style="color: var(--oc-text-primary);">{{ configStatusText }}</span>
            </div>
          </div>
        </div>
      </aside>

      <section class="oc-content-shell">
        <header class="oc-topbar">
          <div class="min-w-0">
            <p class="truncate text-sm font-semibold" style="color: var(--oc-text-primary);">
              {{ topbarTitle }}
            </p>
          </div>

          <div class="oc-topbar-actions">
            <button
              class="oc-toolbar-btn h-8 w-8 !px-0"
              type="button"
              aria-label="theme-switcher"
              :title="themeButtonTitle"
              @click="cycleThemeMode"
            >
              <component :is="themeModeIcon" class="h-4 w-4" />
            </button>
            <div class="relative env-dropdown-container">
              <button class="oc-toolbar-btn h-8 min-w-[180px] justify-start" type="button" aria-label="environment-switcher" @click="showEnvDropdown = !showEnvDropdown">
                <component
                  :is="currentEnv.mode === 'local' ? Monitor : Wifi"
                  class="h-4 w-4"
                  :style="{ color: currentEnv.mode === 'local' ? 'var(--oc-accent)' : sshConnected ? 'var(--oc-success)' : 'var(--oc-text-muted)' }"
                />
                <span class="truncate text-sm">{{ currentEnv.label }}</span>
                <ChevronDown class="ml-auto h-4 w-4" :class="{ 'rotate-180': showEnvDropdown }" />
              </button>

              <div
                v-if="showEnvDropdown"
                class="oc-dropdown-menu absolute right-0 top-full z-40 mt-2 w-64"
              >
                <button
                  v-for="(env, index) in environments"
                  :key="`${env.label}-${index}`"
                  class="oc-dropdown-item flex items-center gap-2"
                  :class="{ 'oc-dropdown-item-active': index === currentEnvIndex }"
                  type="button"
                  @click="selectEnvironment(index)"
                >
                  <component :is="env.mode === 'local' ? Monitor : Wifi" class="h-4 w-4" />
                  <span class="truncate">{{ env.label }}</span>
                </button>

                <div class="oc-dropdown-separator" />
                <button
                  class="oc-dropdown-item flex items-center gap-2"
                  style="color: var(--oc-accent);"
                  type="button"
                  aria-label="add-ssh-environment"
                  @click="addSshEnvironment"
                >
                  <Settings2 class="h-4 w-4" />
                  添加 SSH 连接
                </button>
              </div>
            </div>

            <button class="oc-toolbar-btn h-8 w-8 !px-0" type="button" aria-label="refresh-environment" @click="checkEnvironment">
              <RefreshCw class="h-4 w-4" />
            </button>
            <button class="oc-toolbar-btn h-8 px-3 text-sm" type="button" aria-label="open-dashboard" @click="openDashboard">
              <ExternalLink class="h-4 w-4" />
              Dashboard
            </button>
          </div>
        </header>

        <main class="oc-main-area">
          <div class="h-full oc-main-scroll">
            <div
              class="oc-main-scroll-page"
              :class="{ 'oc-main-scroll-page-fixed': !isGateActive && (activeNav === 'channels' || activeNav === 'overview' || activeNav === 'diagnostics' || activeNav === 'ai-config') }"
            >
              <template v-if="isGateActive">
                <div class="oc-panel p-6">
                <h3 class="text-xl font-semibold" style="color: var(--oc-text-primary);">
                  {{
                    gateState === 'NO_TARGET'
                      ? '选择运行环境'
                      : gateState === 'NEED_INSTALL'
                        ? '完成安装'
                        : '完成模型配置'
                  }}
                </h3>
                <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">
                  {{
                    gateState === 'NO_TARGET'
                      ? '请选择本地或 SSH 作为唯一运行环境入口。'
                      : gateState === 'NEED_INSTALL'
                        ? '当前环境已确定，请继续完成安装。'
                        : '环境已安装，下一步是完成配置并验证生效。'
                  }}
                </p>

                <div v-if="gateState === 'NO_TARGET'" class="mt-4 grid gap-3 md:grid-cols-2">
                  <button
                    type="button"
                    class="rounded-[12px] border p-4 text-left transition-colors"
                    :style="{
                      borderColor: targetMode === 'local' ? 'var(--oc-card-border-strong)' : 'var(--oc-card-border)',
                      background: targetMode === 'local' ? 'var(--oc-item-active)' : 'var(--oc-card-elevated)'
                    }"
                    @click="chooseLocalTarget"
                  >
                    <p class="text-base font-semibold" style="color: var(--oc-text-primary);">安装在本地</p>
                    <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">使用本机环境安装并运行 OpenClaw。</p>
                  </button>

                  <button
                    type="button"
                    class="rounded-[12px] border p-4 text-left transition-colors"
                    :style="{
                      borderColor: targetMode === 'ssh' ? 'var(--oc-card-border-strong)' : 'var(--oc-card-border)',
                      background: targetMode === 'ssh' ? 'var(--oc-item-active)' : 'var(--oc-card-elevated)'
                    }"
                    @click="chooseSshTarget"
                  >
                    <p class="text-base font-semibold" style="color: var(--oc-text-primary);">连接 SSH 环境</p>
                    <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">后续所有操作基于该 SSH 环境执行。</p>
                  </button>
                </div>

                <div v-else-if="gateState === 'NEED_INSTALL'" class="mt-4 space-y-3">
                  <div class="rounded-[12px] border px-3 py-2 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-secondary);">
                    当前环境：<strong style="color: var(--oc-text-primary);">{{ targetMode === 'ssh' ? 'SSH' : '本地' }}</strong>
                  </div>

                  <div v-if="targetMode === 'ssh'" class="rounded-[12px] border px-3 py-3 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-secondary);">
                    <p>1. 先通过右上角环境入口建立 SSH 连接</p>
                    <p class="mt-1">2. 在远程执行安装后，点击“重新检测环境”</p>
                    <div class="mt-3 flex flex-wrap gap-2">
                      <button class="oc-toolbar-btn h-9 px-3" type="button" @click="showSshModal = true">
                        连接 SSH
                      </button>
                      <button class="oc-toolbar-btn h-9 px-3" type="button" @click="checkEnvironment">
                        重新检测环境
                      </button>
                    </div>
                  </div>

                  <div class="flex flex-wrap gap-2">
                    <button class="oc-toolbar-btn h-9 px-3" type="button" @click="chooseLocalTarget">改用本地</button>
                    <button class="oc-toolbar-btn h-9 px-3" type="button" @click="chooseSshTarget">改用 SSH</button>
                  </div>
                </div>

                <div v-else class="mt-4 space-y-2">
                  <div class="rounded-[12px] border px-3 py-3 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-secondary);">
                    检测结果为“待配置”，请先完成以下任一操作：
                  </div>
                  <div class="flex flex-wrap gap-2">
                    <button class="oc-toolbar-btn h-10 px-4" type="button" @click="runManualConfig">
                      手动配置
                    </button>
                    <button class="oc-toolbar-btn h-10 px-4" type="button" @click="applyDefaultConfig">
                      使用默认配置
                    </button>
                    <button class="oc-toolbar-btn h-10 px-4" type="button" @click="checkEnvironment">
                      重新检测环境
                    </button>
                  </div>
                </div>
                </div>

                <InstallPage
                  v-if="gateState === 'NEED_INSTALL' && targetMode === 'local'"
                  class="mt-3"
                  :mode="'local'"
                  :env-connected="true"
                  :openclaw-installed="openclawInstalled"
                  @install-complete="handleInstallComplete"
                />
              </template>

              <StatusDashboard
                v-else-if="activeNav === 'overview' && envStatus"
                class="oc-page-root"
                :env-status="envStatus"
                :gateway-reachable="gatewayReachable"
                :env-mode="currentEnv.mode"
                @open-tool="openToolPanel"
              />

              <div v-else-if="activeNav === 'ai-config'" class="oc-page-root">
                <ConfigPage
                  v-if="envStatus && openclawInstalled"
                  class="oc-page-root"
                  :show-toast="showToast"
                  :env-mode="currentEnv.mode"
                  :env-ssh-connected="sshConnected"
                />
                <div v-else class="oc-panel p-6">
                  <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">模型配置不可用</h3>
                  <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">请先在“安装与接入”中完成安装。</p>
                </div>
              </div>

              <DiagnosticsPage
                v-else-if="activeNav === 'diagnostics'"
                class="oc-page-root"
                :app-state="appState === 'ERROR' || appState === 'DEGRADED' ? appState : 'READY'"
                :env-mode="currentEnv.mode"
                :openclaw-installed="openclawInstalled"
                @refresh="checkEnvironment"
              />

              <div v-else-if="activeNav === 'channels'" class="oc-page-root">
                <MessageChannelsPage class="h-full min-h-0" :show-toast="showToast" />
              </div>

              <div v-else class="oc-panel p-6">
                <h3 class="text-xl font-semibold" style="color: var(--oc-text-primary);">系统设置</h3>
                <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">连接相关操作统一从顶部环境入口管理，设置页仅保留偏好项。</p>
              </div>
            </div>
          </div>
        </main>
      </section>
    </div>

    <SshConnectModal
      v-if="showSshModal"
      @close="showSshModal = false"
      @connected="handleSshConnected"
      @fingerprint="handleFingerprint"
    />

    <SshFingerprintDialog
      v-if="showFingerprintDialog && sshFingerprint"
      :fingerprint="sshFingerprint"
      @confirm="confirmFingerprint"
      @reject="rejectFingerprint"
    />

    <Toast v-if="toast" :type="toast.type" :message="toast.message" @close="closeToast" />

    <div v-if="loading" class="fixed inset-0 z-[110] flex items-center justify-center bg-black/30 backdrop-blur-[1px]">
      <div class="flex items-center gap-3 rounded-xl border px-4 py-3" style="border-color: var(--oc-card-border); background: var(--oc-card); box-shadow: var(--oc-shadow-popover);">
        <div class="h-5 w-5 animate-spin rounded-full border-2 border-[var(--oc-accent)] border-t-transparent" />
        <span class="text-sm" style="color: var(--oc-text-primary);">{{ loadingMessage }}</span>
      </div>
    </div>
  </div>
</template>
