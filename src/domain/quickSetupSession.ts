import {
  QUICK_SETUP_STEPS,
  type QuickSetupChannelId,
  type QuickSetupProviderId,
  type QuickSetupStepId,
} from './quickSetupGuide'

export const QUICK_SETUP_SESSION_STORAGE_KEY = 'openclawswitch.quick-setup.session'
export const QUICK_SETUP_SESSION_VERSION = 1
export const QUICK_SETUP_SESSION_TTL_MS = 24 * 60 * 60 * 1000

export type QuickSetupSessionStatus = 'in_progress' | 'awaiting_admin_relaunch'
export type QuickSetupSessionEnvMode = 'local' | 'ssh'
export type QuickSetupModelSelectionMode = 'auto' | 'manual'

export interface QuickSetupSessionPersistenceInput {
  restoringSession: boolean
  persistenceDisabled: boolean
}

export interface QuickSetupSessionSnapshot {
  version: number
  status: QuickSetupSessionStatus
  stepId: QuickSetupStepId
  savedStepIds: QuickSetupStepId[]
  selectedProviderId: QuickSetupProviderId
  providerApiKey: string
  modelQuery: string
  modelSelectionMode: QuickSetupModelSelectionMode
  customProviderName: string
  customProviderBaseUrl: string
  selectedChannelId: QuickSetupChannelId
  channelIdValue: string
  channelSecretValue: string
  browserDefaultProfileEnabled: boolean
  toolsFullProfileEnabled: boolean
  updatedAt: number
}

export interface QuickSetupSessionStorage {
  getItem(key: string): string | null
  setItem(key: string, value: string): void
  removeItem(key: string): void
}

const validStepIds = new Set<QuickSetupStepId>(QUICK_SETUP_STEPS.map((step) => step.id))
const validChannelIds = new Set<QuickSetupChannelId>(['feishu', 'wecom', 'qq', 'dingtalk'])
const validProviderIds = new Set<QuickSetupProviderId>([
  'dashscope-coding',
  'tencent-coding',
  'deepseek',
  'dashscope',
  'hunyuan',
  'custom',
])
const validModelSelectionModes = new Set<QuickSetupModelSelectionMode>(['auto', 'manual'])

const resolveStorage = (storage?: QuickSetupSessionStorage | null) => {
  if (storage) return storage
  if (typeof window !== 'undefined' && window.localStorage) return window.localStorage
  return null
}

const sanitizeSavedSteps = (value: unknown): QuickSetupStepId[] => {
  if (!Array.isArray(value)) return []
  return Array.from(new Set(value.filter((stepId): stepId is QuickSetupStepId => validStepIds.has(stepId as QuickSetupStepId))))
}

export const createQuickSetupSessionSnapshot = (
  input: Omit<QuickSetupSessionSnapshot, 'version' | 'updatedAt'> & { updatedAt?: number }
): QuickSetupSessionSnapshot => ({
  version: QUICK_SETUP_SESSION_VERSION,
  status: input.status,
  stepId: validStepIds.has(input.stepId) ? input.stepId : 'model',
  savedStepIds: sanitizeSavedSteps(input.savedStepIds),
  selectedProviderId: validProviderIds.has(input.selectedProviderId) ? input.selectedProviderId : 'dashscope-coding',
  providerApiKey: input.providerApiKey,
  modelQuery: input.modelQuery,
  modelSelectionMode: validModelSelectionModes.has(input.modelSelectionMode) ? input.modelSelectionMode : 'auto',
  customProviderName: input.customProviderName,
  customProviderBaseUrl: input.customProviderBaseUrl,
  selectedChannelId: validChannelIds.has(input.selectedChannelId) ? input.selectedChannelId : 'feishu',
  channelIdValue: input.channelIdValue,
  channelSecretValue: input.channelSecretValue,
  browserDefaultProfileEnabled: input.browserDefaultProfileEnabled,
  toolsFullProfileEnabled: input.toolsFullProfileEnabled,
  updatedAt: input.updatedAt ?? Date.now(),
})

export const resolveQuickSetupSessionStepIndex = (snapshot: Pick<QuickSetupSessionSnapshot, 'stepId'>) =>
  Math.max(QUICK_SETUP_STEPS.findIndex((step) => step.id === snapshot.stepId), 0)

export const shouldResumeQuickSetupSession = (
  snapshot: Pick<QuickSetupSessionSnapshot, 'updatedAt'> | null | undefined,
  now = Date.now()
) => Boolean(snapshot && now - snapshot.updatedAt <= QUICK_SETUP_SESSION_TTL_MS)

export const shouldClearQuickSetupSessionForEnvironment = (
  envMode: QuickSetupSessionEnvMode,
  openclawInstalled: boolean
) => envMode === 'local' && !openclawInstalled

export const shouldClearQuickSetupSessionAfterInstall = (envMode: QuickSetupSessionEnvMode) =>
  envMode === 'local'

export const shouldPersistQuickSetupSession = (input: QuickSetupSessionPersistenceInput) =>
  !input.restoringSession && !input.persistenceDisabled

export const saveQuickSetupSession = (snapshot: QuickSetupSessionSnapshot, storage?: QuickSetupSessionStorage | null) => {
  const targetStorage = resolveStorage(storage)
  if (!targetStorage) return
  targetStorage.setItem(QUICK_SETUP_SESSION_STORAGE_KEY, JSON.stringify(snapshot))
}

export const loadQuickSetupSession = (
  storage?: QuickSetupSessionStorage | null,
  now = Date.now()
): QuickSetupSessionSnapshot | null => {
  const targetStorage = resolveStorage(storage)
  if (!targetStorage) return null

  const raw = targetStorage.getItem(QUICK_SETUP_SESSION_STORAGE_KEY)
  if (!raw) return null

  try {
    const parsed = JSON.parse(raw) as Partial<QuickSetupSessionSnapshot>
    const snapshot = createQuickSetupSessionSnapshot({
      status: parsed.status === 'awaiting_admin_relaunch' ? 'awaiting_admin_relaunch' : 'in_progress',
      stepId: (parsed.stepId as QuickSetupStepId) ?? 'model',
      savedStepIds: parsed.savedStepIds ?? [],
      selectedProviderId: (parsed.selectedProviderId as QuickSetupProviderId) ?? 'dashscope-coding',
      providerApiKey: typeof parsed.providerApiKey === 'string' ? parsed.providerApiKey : '',
      modelQuery: typeof parsed.modelQuery === 'string' ? parsed.modelQuery : '',
      modelSelectionMode: (parsed.modelSelectionMode as QuickSetupModelSelectionMode) ?? 'auto',
      customProviderName: typeof parsed.customProviderName === 'string' ? parsed.customProviderName : '',
      customProviderBaseUrl: typeof parsed.customProviderBaseUrl === 'string' ? parsed.customProviderBaseUrl : '',
      selectedChannelId: (parsed.selectedChannelId as QuickSetupChannelId) ?? 'feishu',
      channelIdValue: typeof parsed.channelIdValue === 'string' ? parsed.channelIdValue : '',
      channelSecretValue: typeof parsed.channelSecretValue === 'string' ? parsed.channelSecretValue : '',
      browserDefaultProfileEnabled: Boolean(parsed.browserDefaultProfileEnabled),
      toolsFullProfileEnabled: Boolean(parsed.toolsFullProfileEnabled),
      updatedAt: typeof parsed.updatedAt === 'number' ? parsed.updatedAt : 0,
    })

    if (!shouldResumeQuickSetupSession(snapshot, now)) {
      targetStorage.removeItem(QUICK_SETUP_SESSION_STORAGE_KEY)
      return null
    }

    return snapshot
  } catch {
    targetStorage.removeItem(QUICK_SETUP_SESSION_STORAGE_KEY)
    return null
  }
}

export const clearQuickSetupSession = (storage?: QuickSetupSessionStorage | null) => {
  const targetStorage = resolveStorage(storage)
  if (!targetStorage) return
  targetStorage.removeItem(QUICK_SETUP_SESSION_STORAGE_KEY)
}
