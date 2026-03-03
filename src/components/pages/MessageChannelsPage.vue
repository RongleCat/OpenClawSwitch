<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { open as openExternal } from '@tauri-apps/api/shell'
import {
  Send,
  Hash,
  Slack,
  MessageCircle,
  Apple,
  Bell,
  ChevronRight,
  Eye,
  EyeOff,
  Check,
  X,
  Save,
  KeyRound,
  ExternalLink,
  Download,
  Loader2
} from 'lucide-vue-next'
import CommonInputConfirmModal from '../CommonInputConfirmModal.vue'
import Button from '../ui/Button.vue'
import Card from '../ui/Card.vue'
import Input from '../ui/Input.vue'
import TerminalLog from '../ui/TerminalLog.vue'
import type { ConfigFileInfo, InstallLogEvent, OpenClawConfig } from '../../types/config'

type ChannelId =
  | 'telegram'
  | 'discord'
  | 'slack'
  | 'feishu'
  | 'whatsapp'
  | 'imessage'
  | 'wechat'
  | 'dingtalk'

interface ChannelForm {
  token: string
  userId: string
  enabled: boolean
  privateStrategy: 'pair' | 'allow_all' | 'deny'
  groupStrategy: 'whitelist' | 'allow_all' | 'deny'
}

interface ChannelMeta {
  id: ChannelId
  name: string
  icon: any
  iconColor: string
}

type ChannelConfigPanel = 'credentials' | 'strategy'
type ExtensionChannelId = 'feishu' | 'dingtalk'

interface ChannelExtensionStatus {
  feishuInstalled: boolean
  dingtalkInstalled: boolean
}

interface ChannelExtensionInstallStateEvent {
  channelId: string
  status: 'running' | 'success' | 'error'
}

const props = withDefaults(
  defineProps<{
    showToast: (type: 'success' | 'error', message: string) => void
  }>(),
  {}
)

const channelList: ChannelMeta[] = [
  { id: 'feishu', name: '飞书', icon: MessageCircle, iconColor: 'var(--oc-accent)' },
  { id: 'dingtalk', name: '钉钉', icon: Bell, iconColor: 'var(--oc-accent)' },
  { id: 'telegram', name: 'Telegram', icon: Send, iconColor: 'var(--oc-accent)' },
  { id: 'discord', name: 'Discord', icon: Hash, iconColor: 'var(--oc-warning)' },
  { id: 'slack', name: 'Slack', icon: Slack, iconColor: 'var(--oc-text-secondary)' },
  { id: 'whatsapp', name: 'WhatsApp', icon: MessageCircle, iconColor: 'var(--oc-success)' },
  { id: 'imessage', name: 'iMessage', icon: Apple, iconColor: 'var(--oc-text-secondary)' },
  { id: 'wechat', name: '微信', icon: MessageCircle, iconColor: 'var(--oc-success)' }
]

const hints: Record<ChannelId, string> = {
  telegram: '1. 搜索 @BotFather 发送 /newbot 获取 Token 2. 搜索 @userinfobot 获取 User ID',
  discord: '配置 Bot Token 与频道/用户 ID 后即可发送通知。',
  slack: '填写 Bot Token 和用户或频道 ID，支持私聊与频道推送。',
  feishu: '填写飞书 App ID 与 App Secret。',
  whatsapp: '填写接入 Token 和目标号码 ID 进行消息推送。',
  imessage: '填写网关 Token 与目标用户标识进行投递。',
  wechat: '填写企业微信或转发网关 Token 与用户 ID。',
  dingtalk: '填写 Client ID 与 Client Secret。'
}

const storageKey = 'openclawswitch-message-channels'

const defaultForm = (): ChannelForm => ({
  token: '',
  userId: '',
  enabled: false,
  privateStrategy: 'pair',
  groupStrategy: 'whitelist'
})

const channelIds: ChannelId[] = channelList.map(channel => channel.id)

const loadStoredForms = (): Record<ChannelId, ChannelForm> => {
  const initial = channelIds.reduce((acc, id) => {
    acc[id] = defaultForm()
    return acc
  }, {} as Record<ChannelId, ChannelForm>)

  try {
    const raw = localStorage.getItem(storageKey)
    if (!raw) return initial
    const parsed = JSON.parse(raw) as Partial<Record<ChannelId, Partial<ChannelForm>>>

    for (const id of channelIds) {
      if (!parsed[id]) continue
      initial[id] = {
        token: parsed[id]?.token || '',
        userId: parsed[id]?.userId || '',
        enabled: Boolean(parsed[id]?.enabled),
        privateStrategy: (parsed[id]?.privateStrategy as ChannelForm['privateStrategy']) || 'pair',
        groupStrategy: (parsed[id]?.groupStrategy as ChannelForm['groupStrategy']) || 'whitelist'
      }
    }
  } catch {
    return initial
  }

  return initial
}

const forms = ref<Record<ChannelId, ChannelForm>>(loadStoredForms())
const selectedChannelId = ref<ChannelId>(channelList[0].id)
const revealToken = ref(false)
const revealSecret = ref(false)
const selectedPanel = ref<ChannelConfigPanel>('credentials')
const extensionStatus = ref<ChannelExtensionStatus>({
  feishuInstalled: false,
  dingtalkInstalled: false
})
const extensionStatusLoading = ref(false)
const showInstallModal = ref(false)
const installingExtension = ref(false)
const installLogs = ref<InstallLogEvent[]>([])
const installingChannel = ref<ExtensionChannelId | null>(null)
const showPairingCodeModal = ref(false)
const pairingInput = ref('')
const approvingPairing = ref(false)

let unlistenExtensionInstallLog: (() => void) | null = null
let unlistenExtensionInstallState: (() => void) | null = null

watch(
  forms,
  (value) => {
    localStorage.setItem(storageKey, JSON.stringify(value))
  },
  { deep: true }
)

watch(selectedChannelId, () => {
  selectedPanel.value = 'credentials'
  revealToken.value = false
  revealSecret.value = false
  void syncChannelsFromConfig()
})

const selectedChannel = computed(() =>
  channelList.find(channel => channel.id === selectedChannelId.value) || channelList[0]
)
const installingChannelName = computed(() =>
  channelList.find(channel => channel.id === installingChannel.value)?.name || ''
)

const currentForm = computed(() => forms.value[selectedChannelId.value])

const isConfigured = (id: ChannelId) => {
  const form = forms.value[id]
  return form.token.trim().length > 0 && form.userId.trim().length > 0
}

const isExtensionChannel = (id: ChannelId): id is ExtensionChannelId =>
  id === 'feishu' || id === 'dingtalk'

const selectedExtensionInstalled = computed(() => {
  if (selectedChannelId.value === 'feishu') return extensionStatus.value.feishuInstalled
  if (selectedChannelId.value === 'dingtalk') return extensionStatus.value.dingtalkInstalled
  return true
})

const selectedNeedsExtensionInstall = computed(() =>
  isExtensionChannel(selectedChannelId.value) && !selectedExtensionInstalled.value
)

const canConfigureCurrentChannel = computed(() => !selectedNeedsExtensionInstall.value)
const enabledLabel = computed(() => (currentForm.value.enabled ? '已启用' : '已停用'))

const loadLocalConfig = async () =>
  invoke<[OpenClawConfig, ConfigFileInfo]>('load_default_config')

const syncChannelsFromConfig = async () => {
  try {
    const [config] = await loadLocalConfig()
    const root = config as Record<string, unknown>
    const channelsRaw = root.channels as Record<string, Record<string, unknown>> | undefined

    for (const id of channelIds) {
      if (id === 'dingtalk') {
        forms.value[id].enabled = Boolean(
          channelsRaw?.['dingtalk-connector']?.enabled ?? channelsRaw?.dingtalk?.enabled
        )
      } else {
        forms.value[id].enabled = Boolean(channelsRaw?.[id]?.enabled)
      }
    }

    const feishu = channelsRaw?.feishu
    if (typeof feishu?.appId === 'string') {
      forms.value.feishu.token = feishu.appId
    }
    if (typeof feishu?.appSecret === 'string') {
      forms.value.feishu.userId = feishu.appSecret
    }

    const dingtalk = channelsRaw?.['dingtalk-connector'] ?? channelsRaw?.dingtalk
    if (typeof dingtalk?.clientId === 'string') {
      forms.value.dingtalk.token = dingtalk.clientId
    }
    if (typeof dingtalk?.clientSecret === 'string') {
      forms.value.dingtalk.userId = dingtalk.clientSecret
    }
  } catch {
    // ignore, keep local values
  }
}

const persistChannelEnabled = async (channelId: ChannelId, enabled: boolean) => {
  const [config, info] = await loadLocalConfig()
  const mutable = config as Record<string, unknown>
  const channels =
    ((mutable.channels as Record<string, Record<string, unknown>> | undefined) || {})
  if (!channels[channelId]) {
    channels[channelId] = {}
  }
  channels[channelId].enabled = enabled
  mutable.channels = channels

  await invoke('save_config', {
    config: mutable,
    path: info.path
  })
}

const extractPairingCode = (raw: string): string => {
  const compact = raw.trim().replace(/\s+/g, ' ')
  if (!compact) return ''

  const commandMatch = compact.match(/^openclaw\s+pairing\s+approve\s+feishu\s+(.+)$/i)
  const commandCode = commandMatch ? commandMatch[1] : ''

  const candidate = commandCode ? commandCode.trim() : compact
  const firstToken = candidate.split(/\s+/)[0] || ''
  return firstToken.replace(/^['"]|['"]$/g, '').trim()
}

const openPairingModal = () => {
  if (selectedChannelId.value !== 'feishu') return
  pairingInput.value = ''
  showPairingCodeModal.value = true
}

const closePairingModal = () => {
  if (approvingPairing.value) return
  showPairingCodeModal.value = false
}

const submitPairing = async () => {
  const pairingCode = extractPairingCode(pairingInput.value)
  if (!pairingCode) {
    props.showToast('error', '请先填写有效配对码')
    return
  }

  approvingPairing.value = true
  try {
    await invoke<string>('approve_feishu_pairing', { pairingCode })
    showPairingCodeModal.value = false
    pairingInput.value = ''
    props.showToast('success', '配对成功')
  } catch (error) {
    props.showToast('error', String(error))
  } finally {
    approvingPairing.value = false
  }
}

const handlePairingInputChange = (value: string) => {
  pairingInput.value = value
}

const openAppCenter = async () => {
  let url = ''
  if (selectedChannelId.value === 'feishu') {
    url = 'https://open.feishu.cn/app'
  } else if (selectedChannelId.value === 'dingtalk') {
    url = 'https://open-dev.dingtalk.com/fe/app?hash=%23%2Fcorp%2Fapp#/corp/app'
  } else {
    return
  }

  try {
    await openExternal(url)
  } catch (error) {
    props.showToast('error', `打开 ${selectedChannel.value.name}开放平台失败: ${String(error)}`)
  }
}

const toggleChannelEnabled = async () => {
  if (!canConfigureCurrentChannel.value) {
    props.showToast('error', `${selectedChannel.value.name} 扩展未安装，暂不可配置`)
    return
  }

  const channelId = selectedChannelId.value
  const previous = currentForm.value.enabled
  const next = !previous
  currentForm.value.enabled = next

  try {
    if (channelId === 'feishu') {
      await invoke<string>('set_feishu_channel_config', {
        appId: currentForm.value.token.trim(),
        appSecret: currentForm.value.userId.trim(),
        enabled: next
      })
    } else if (channelId === 'dingtalk') {
      await invoke<string>('set_dingtalk_channel_config', {
        clientId: currentForm.value.token.trim(),
        clientSecret: currentForm.value.userId.trim(),
        enabled: next
      })
    } else {
      await persistChannelEnabled(channelId, next)
    }
    await syncChannelsFromConfig()
    props.showToast('success', `${selectedChannel.value.name}已${next ? '启用' : '停用'}`)
  } catch (error) {
    currentForm.value.enabled = previous
    props.showToast('error', String(error))
  }
}

const refreshExtensionStatus = async () => {
  extensionStatusLoading.value = true
  try {
    extensionStatus.value = await invoke<ChannelExtensionStatus>('get_channel_extension_status')
  } catch (error) {
    props.showToast('error', `读取扩展状态失败: ${String(error)}`)
  } finally {
    extensionStatusLoading.value = false
  }
}

const openInstallModal = async () => {
  if (!isExtensionChannel(selectedChannelId.value)) return

  installingChannel.value = selectedChannelId.value
  installLogs.value = []
  showInstallModal.value = true
  installingExtension.value = true

  try {
    await invoke<string>('install_channel_extension', { channelId: installingChannel.value })
    props.showToast('success', `${selectedChannel.value.name} 扩展安装完成`)
    await refreshExtensionStatus()
  } catch (error) {
    props.showToast('error', String(error))
  } finally {
    installingExtension.value = false
  }
}

const closeInstallModal = () => {
  if (installingExtension.value) return
  showInstallModal.value = false
}

const saveConfig = async () => {
  if (!canConfigureCurrentChannel.value) {
    props.showToast('error', `${selectedChannel.value.name} 扩展未安装，暂不可配置`)
    return
  }

  const current = currentForm.value
  if (selectedChannelId.value === 'feishu') {
    if (!current.token.trim() || !current.userId.trim()) {
      props.showToast('error', '请先填写飞书 App ID 和 App Secret')
      return
    }
    try {
      await invoke<string>('set_feishu_channel_config', {
        appId: current.token.trim(),
        appSecret: current.userId.trim(),
        enabled: current.enabled
      })
      await syncChannelsFromConfig()
      props.showToast('success', '飞书配置已保存')
    } catch (error) {
      props.showToast('error', String(error))
    }
    return
  }

  if (selectedChannelId.value === 'dingtalk') {
    if (!current.token.trim() || !current.userId.trim()) {
      props.showToast('error', '请先填写钉钉 Client ID 和 Client Secret')
      return
    }
    try {
      await invoke<string>('set_dingtalk_channel_config', {
        clientId: current.token.trim(),
        clientSecret: current.userId.trim(),
        enabled: current.enabled
      })
      await syncChannelsFromConfig()
      props.showToast('success', '钉钉配置已保存')
    } catch (error) {
      props.showToast('error', String(error))
    }
    return
  }

  try {
    await persistChannelEnabled(selectedChannelId.value, current.enabled)
    await syncChannelsFromConfig()
    props.showToast('success', `${selectedChannel.value.name} 配置已保存`)
  } catch (error) {
    props.showToast('error', String(error))
  }
}

const panelTabs: Array<{ id: ChannelConfigPanel; label: string }> = [
  { id: 'credentials', label: '凭据配置' },
  { id: 'strategy', label: '策略配置' }
]

onMounted(async () => {
  await syncChannelsFromConfig()
  await refreshExtensionStatus()

  unlistenExtensionInstallLog = await listen<InstallLogEvent>('channel-extension-install-log', (event) => {
    if (!installingChannel.value || event.payload.step !== installingChannel.value) return
    installLogs.value.push(event.payload)
  })

  unlistenExtensionInstallState = await listen<ChannelExtensionInstallStateEvent>('channel-extension-install-state', (event) => {
    if (!installingChannel.value || event.payload.channelId !== installingChannel.value) return
    if (event.payload.status === 'success' || event.payload.status === 'error') {
      installingExtension.value = false
    }
  })
})

onUnmounted(() => {
  unlistenExtensionInstallLog?.()
  unlistenExtensionInstallState?.()
})
</script>

<template>
  <div class="oc-page-root">
    <div class="grid h-full min-h-0 grid-cols-1 gap-4 lg:grid-cols-[320px_minmax(0,1fr)]">
      <section class="oc-panel min-h-0 overflow-visible flex flex-col">
        <div class="border-b px-4 py-3" style="border-color: var(--oc-divider-soft);">
          <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">消息渠道</h3>
          <p class="mt-1 text-xs" style="color: var(--oc-text-muted);">在左侧选择渠道，右侧分段菜单切换对应配置表单。</p>
        </div>

        <div class="min-h-0 flex-1 overflow-y-auto p-3">
          <div class="space-y-2">
            <button
              v-for="channel in channelList"
              :key="channel.id"
              type="button"
              class="oc-subpanel w-full border p-3 text-left transition-all duration-200"
              :class="[
                selectedChannelId === channel.id
                  ? 'border-[var(--oc-card-border-strong)]'
                  : 'border-[var(--oc-card-border)]'
              ]"
              :style="{ background: selectedChannelId === channel.id ? 'var(--oc-item-active)' : 'var(--oc-card-elevated)' }"
              @click="selectedChannelId = channel.id"
            >
              <div class="flex items-center gap-3">
                <div class="flex h-9 w-9 items-center justify-center rounded-full border" style="border-color: var(--oc-divider); background: var(--oc-card);">
                  <component :is="channel.icon" class="h-4 w-4" :style="{ color: channel.iconColor }" />
                </div>

                <div class="min-w-0 flex-1">
                  <div class="text-lg font-semibold" style="color: var(--oc-text-primary);">{{ channel.name }}</div>
                  <div class="mt-1 flex items-center gap-2 text-sm">
                    <Check v-if="isConfigured(channel.id)" class="h-3.5 w-3.5" style="color: var(--oc-success);" />
                    <X v-else class="h-3.5 w-3.5" style="color: var(--oc-text-muted);" />
                    <span :style="{ color: isConfigured(channel.id) ? 'var(--oc-success)' : 'var(--oc-text-muted)' }">
                      {{ isConfigured(channel.id) ? '已配置' : '未配置' }}
                    </span>
                    <span style="color: var(--oc-text-quiet);">·</span>
                    <span :style="{ color: forms[channel.id].enabled ? 'var(--oc-success)' : 'var(--oc-text-muted)' }">
                      {{ forms[channel.id].enabled ? '已启用' : '已停用' }}
                    </span>
                  </div>
                </div>

                <ChevronRight class="h-4 w-4" style="color: var(--oc-text-muted);" />
              </div>
            </button>
          </div>
        </div>
      </section>

      <section class="oc-panel min-h-0 overflow-hidden flex flex-col">
        <div class="border-b px-5 py-4" style="border-color: var(--oc-divider-soft);">
          <div class="flex items-start justify-between gap-3">
            <div class="flex items-center gap-3">
              <div class="flex h-10 w-10 items-center justify-center rounded-full border" style="border-color: var(--oc-divider); background: var(--oc-card-elevated);">
                <component :is="selectedChannel.icon" class="h-5 w-5" :style="{ color: selectedChannel.iconColor }" />
              </div>
              <div>
                <h3 class="text-[22px] font-semibold leading-tight" style="color: var(--oc-text-primary);">配置 {{ selectedChannel.name }}</h3>
                <p class="mt-1 text-sm" style="color: var(--oc-text-muted);">{{ hints[selectedChannelId] }}</p>
              </div>
            </div>

            <button
              v-if="isExtensionChannel(selectedChannelId) && !selectedExtensionInstalled"
              class="oc-toolbar-btn h-9 min-w-[104px] px-3 whitespace-nowrap"
              type="button"
              :disabled="installingExtension || extensionStatusLoading"
              @click="openInstallModal"
            >
              <Loader2 v-if="installingExtension && installingChannel === selectedChannelId" class="h-4 w-4 animate-spin" />
              <Download v-else class="h-4 w-4" />
              安装扩展
            </button>
            <div
              v-else
              class="inline-flex h-9 items-center gap-3 rounded-[10px] border px-3 text-sm"
              style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-secondary);"
            >
              <span class="text-xs font-medium" style="color: var(--oc-text-muted);">是否启用</span>
              <button
                type="button"
                class="relative inline-flex h-6 w-11 items-center rounded-full border transition-colors"
                :style="{
                  borderColor: currentForm.enabled ? 'color-mix(in srgb, var(--oc-success) 55%, transparent)' : 'var(--oc-card-border)',
                  background: currentForm.enabled
                    ? 'color-mix(in srgb, var(--oc-success) 28%, transparent)'
                    : 'color-mix(in srgb, var(--oc-card-elevated) 92%, transparent)'
                }"
                :disabled="!canConfigureCurrentChannel"
                @click="toggleChannelEnabled"
              >
                <span
                  class="h-4 w-4 rounded-full border transition-transform"
                  :style="{
                    borderColor: 'var(--oc-card-border)',
                    background: 'var(--oc-card)',
                    transform: currentForm.enabled ? 'translateX(22px)' : 'translateX(2px)'
                  }"
                />
              </button>
              <span class="text-xs" :style="{ color: currentForm.enabled ? 'var(--oc-success)' : 'var(--oc-text-muted)' }">
                {{ enabledLabel }}
              </span>
            </div>
          </div>
        </div>

        <div class="border-b px-4 py-2" style="border-color: var(--oc-divider-soft);">
          <div class="flex flex-wrap gap-2">
            <button
              v-for="tab in panelTabs"
              :key="tab.id"
              type="button"
              class="oc-toolbar-btn h-8 px-3 text-sm"
              :style="tab.id === selectedPanel ? { background: 'var(--oc-item-active)', borderColor: 'var(--oc-card-border-strong)', color: 'var(--oc-text-primary)' } : undefined"
              @click="selectedPanel = tab.id"
            >
              {{ tab.label }}
            </button>
          </div>
        </div>

        <div class="min-h-0 flex-1 overflow-y-auto p-5">
          <div
            v-if="selectedNeedsExtensionInstall"
            class="rounded-[12px] border p-4 text-sm"
            style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-secondary);"
          >
            <p>
              当前渠道 <strong style="color: var(--oc-text-primary);">{{ selectedChannel.name }}</strong> 扩展尚未安装，暂不可配置。
            </p>
            <p class="mt-1">请点击右上角“安装扩展”，安装完成后将自动解锁配置。</p>
          </div>

          <div v-else-if="selectedPanel === 'credentials'" class="space-y-4">
            <div>
              <label class="mb-1.5 flex items-center gap-2 text-sm font-medium" style="color: var(--oc-text-secondary);">
                {{ selectedChannelId === 'feishu'
                  ? '飞书 App ID'
                  : selectedChannelId === 'dingtalk'
                    ? 'Client ID（原 AppKey 和 SuiteKey）'
                    : 'Bot Token' }} <span style="color: var(--oc-danger);">*</span>
                <Check class="h-3.5 w-3.5" style="color: var(--oc-success);" v-if="currentForm.token" />
              </label>
              <div class="relative">
                <Input
                  :type="selectedChannelId === 'feishu' || selectedChannelId === 'dingtalk' ? 'text' : (revealToken ? 'text' : 'password')"
                  :model-value="currentForm.token"
                  :placeholder="selectedChannelId === 'feishu'
                    ? '输入飞书 App ID（如 cli_xxxxx）'
                    : selectedChannelId === 'dingtalk'
                      ? '输入 Client ID'
                      : '输入渠道访问 Token'"
                  :class="selectedChannelId === 'feishu' || selectedChannelId === 'dingtalk' ? '' : 'pr-11'"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.token = value }"
                />
                <button
                  v-if="selectedChannelId !== 'feishu' && selectedChannelId !== 'dingtalk'"
                  type="button"
                  class="absolute inset-y-0 right-0 flex w-10 items-center justify-center transition-colors hover:opacity-80"
                  style="color: var(--oc-text-muted);"
                  :disabled="!canConfigureCurrentChannel"
                  @click="revealToken = !revealToken"
                >
                  <EyeOff v-if="revealToken" class="h-4 w-4" />
                  <Eye v-else class="h-4 w-4" />
                </button>
              </div>
            </div>

            <div>
              <label class="mb-1.5 flex items-center gap-2 text-sm font-medium" style="color: var(--oc-text-secondary);">
                {{ selectedChannelId === 'feishu'
                  ? '飞书 App Secret'
                  : selectedChannelId === 'dingtalk'
                    ? 'Client Secret（原 AppSecret 和 SuiteSecret）'
                    : 'User ID' }} <span style="color: var(--oc-danger);">*</span>
                <Check class="h-3.5 w-3.5" style="color: var(--oc-success);" v-if="currentForm.userId" />
              </label>
              <div v-if="selectedChannelId === 'feishu' || selectedChannelId === 'dingtalk'" class="relative">
                <Input
                  :type="revealSecret ? 'text' : 'password'"
                  :model-value="currentForm.userId"
                  :placeholder="selectedChannelId === 'feishu' ? '输入飞书 App Secret' : '输入 Client Secret'"
                  class="pr-11"
                  :disabled="!canConfigureCurrentChannel"
                  @update:model-value="(value) => { currentForm.userId = value }"
                />
                <button
                  type="button"
                  class="absolute inset-y-0 right-0 flex w-10 items-center justify-center transition-colors hover:opacity-80"
                  style="color: var(--oc-text-muted);"
                  :disabled="!canConfigureCurrentChannel"
                  @click="revealSecret = !revealSecret"
                >
                  <EyeOff v-if="revealSecret" class="h-4 w-4" />
                  <Eye v-else class="h-4 w-4" />
                </button>
              </div>
              <Input
                v-else
                :model-value="currentForm.userId"
                placeholder="输入用户 ID / 会话 ID"
                :disabled="!canConfigureCurrentChannel"
                @update:model-value="(value) => { currentForm.userId = value }"
              />
            </div>
          </div>

          <div v-else-if="selectedPanel === 'strategy'" class="space-y-4">
            <div>
              <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">私聊策略</label>
              <select
                v-model="currentForm.privateStrategy"
                class="oc-select"
                :disabled="!canConfigureCurrentChannel"
              >
                <option value="pair">配对模式</option>
                <option value="allow_all">全部放行</option>
                <option value="deny">关闭私聊</option>
              </select>
            </div>

            <div>
              <label class="mb-1.5 block text-sm font-medium" style="color: var(--oc-text-secondary);">群组策略</label>
              <select
                v-model="currentForm.groupStrategy"
                class="oc-select"
                :disabled="!canConfigureCurrentChannel"
              >
                <option value="whitelist">白名单</option>
                <option value="allow_all">全部群组</option>
                <option value="deny">关闭群组</option>
              </select>
            </div>

            <div class="rounded-[12px] border p-3 text-sm" style="border-color: var(--oc-card-border); background: var(--oc-card-elevated); color: var(--oc-text-muted);">
              策略保存后即生效于该渠道的后续消息路由。
            </div>
          </div>

        </div>

        <div class="border-t px-5 py-3" style="border-color: var(--oc-divider-soft);">
          <div class="flex items-center gap-3">
            <Button class="min-w-[132px]" :disabled="!canConfigureCurrentChannel || installingExtension" @click="saveConfig">
              <Save class="h-4 w-4" />
              保存配置
            </Button>
            <Button
              v-if="selectedChannelId === 'feishu'"
              variant="outline"
              class="min-w-[132px] whitespace-nowrap"
              :disabled="!canConfigureCurrentChannel || installingExtension || approvingPairing"
              @click="openPairingModal"
            >
              <KeyRound class="h-4 w-4" />
              填写配对码
            </Button>
            <Button
              v-if="selectedChannelId === 'feishu' || selectedChannelId === 'dingtalk'"
              variant="outline"
              class="min-w-[132px] whitespace-nowrap"
              @click="openAppCenter"
            >
              <ExternalLink class="h-4 w-4" />
              申请应用
            </Button>
          </div>
        </div>
      </section>
    </div>

    <CommonInputConfirmModal
      v-if="showPairingCodeModal"
      title="填写配对码"
      description="请填写配对码，或直接粘贴完整命令。"
      placeholder="openclaw pairing approve feishu UZM4NXNC"
      note="支持粘贴完整命令，系统会自动提取配对码。"
      :model-value="pairingInput"
      :loading="approvingPairing"
      :confirm-text="approvingPairing ? '配对中...' : '确认配对'"
      @update:model-value="handlePairingInputChange"
      @cancel="closePairingModal"
      @confirm="submitPairing"
    />

    <div v-if="showInstallModal" class="oc-modal-overlay" @click.self="closeInstallModal">
      <Card class="oc-modal-card w-full max-w-3xl max-h-[82vh] flex flex-col p-5">
        <div class="flex items-center justify-between gap-2">
          <h3 class="text-lg font-semibold" style="color: var(--oc-text-primary);">
            安装扩展 {{ installingChannelName || selectedChannel.name }}
          </h3>
          <span
            class="inline-flex items-center rounded-[10px] border px-2 py-1 text-xs"
            :style="{
              borderColor: 'var(--oc-card-border)',
              background: 'var(--oc-card-elevated)',
              color: installingExtension ? 'var(--oc-warning)' : 'var(--oc-success)'
            }"
          >
            {{ installingExtension ? '安装中' : '已结束' }}
          </span>
        </div>

        <div class="mt-3 min-h-0 flex-1">
          <TerminalLog :logs="installLogs" />
        </div>

        <div class="mt-4 flex justify-end gap-2">
          <Button variant="outline" :disabled="installingExtension" @click="closeInstallModal">
            关闭
          </Button>
          <Button
            v-if="!installingExtension && isExtensionChannel(selectedChannelId) && !selectedExtensionInstalled"
            @click="openInstallModal"
          >
            重试安装
          </Button>
        </div>
      </Card>
    </div>
  </div>
</template>
