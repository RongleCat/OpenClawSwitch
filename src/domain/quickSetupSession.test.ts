import { describe, expect, it } from 'vitest'
import {
  QUICK_SETUP_SESSION_STORAGE_KEY,
  QUICK_SETUP_SESSION_TTL_MS,
  clearQuickSetupSession,
  createQuickSetupSessionSnapshot,
  loadQuickSetupSession,
  resolveQuickSetupSessionStepIndex,
  saveQuickSetupSession,
  shouldPersistQuickSetupSession,
  shouldClearQuickSetupSessionAfterInstall,
  shouldClearQuickSetupSessionForEnvironment,
  shouldResumeQuickSetupSession,
  type QuickSetupSessionStorage,
} from './quickSetupSession'

const createMemoryStorage = (): QuickSetupSessionStorage => {
  const state = new Map<string, string>()
  return {
    getItem: (key) => state.get(key) ?? null,
    setItem: (key, value) => {
      state.set(key, value)
    },
    removeItem: (key) => {
      state.delete(key)
    },
  }
}

describe('quick setup session snapshot', () => {
  it('resolves the saved step index', () => {
    expect(resolveQuickSetupSessionStepIndex({ stepId: 'model' })).toBe(0)
    expect(resolveQuickSetupSessionStepIndex({ stepId: 'channel' })).toBe(1)
    expect(resolveQuickSetupSessionStepIndex({ stepId: 'gateway' })).toBe(2)
  })

  it('expires stale sessions', () => {
    expect(shouldResumeQuickSetupSession({ updatedAt: 1000 }, 1000 + QUICK_SETUP_SESSION_TTL_MS + 1)).toBe(false)
    expect(shouldResumeQuickSetupSession({ updatedAt: 1000 }, 1000 + QUICK_SETUP_SESSION_TTL_MS)).toBe(true)
  })

  it('clears resumed quick setup when local openclaw is uninstalled', () => {
    expect(shouldClearQuickSetupSessionForEnvironment('local', false)).toBe(true)
    expect(shouldClearQuickSetupSessionForEnvironment('local', true)).toBe(false)
    expect(shouldClearQuickSetupSessionForEnvironment('ssh', false)).toBe(false)
  })

  it('clears quick setup cache after local install completes', () => {
    expect(shouldClearQuickSetupSessionAfterInstall('local')).toBe(true)
    expect(shouldClearQuickSetupSessionAfterInstall('ssh')).toBe(false)
  })

  it('does not re-persist a quick setup session after install has disabled persistence', () => {
    expect(
      shouldPersistQuickSetupSession({
        restoringSession: false,
        persistenceDisabled: true,
      })
    ).toBe(false)

    expect(
      shouldPersistQuickSetupSession({
        restoringSession: true,
        persistenceDisabled: false,
      })
    ).toBe(false)

    expect(
      shouldPersistQuickSetupSession({
        restoringSession: false,
        persistenceDisabled: false,
      })
    ).toBe(true)
  })
})

describe('quick setup session storage', () => {
  it('saves and restores the quick setup snapshot', () => {
    const storage = createMemoryStorage()
    const snapshot = createQuickSetupSessionSnapshot({
      status: 'awaiting_admin_relaunch',
      stepId: 'gateway',
      savedStepIds: ['model', 'channel'],
      selectedProviderId: 'dashscope-coding',
      providerApiKey: 'sk-test',
      modelQuery: 'qwen3.5-plus',
      selectedChannelId: 'feishu',
      channelIdValue: 'app-id',
      channelSecretValue: 'app-secret',
      browserDefaultProfileEnabled: true,
      toolsFullProfileEnabled: false,
      updatedAt: 1234,
    })

    saveQuickSetupSession(snapshot, storage)

    expect(loadQuickSetupSession(storage, 1234)).toEqual(snapshot)
  })

  it('clears malformed session payloads', () => {
    const storage = createMemoryStorage()
    storage.setItem(QUICK_SETUP_SESSION_STORAGE_KEY, '{bad json')

    expect(loadQuickSetupSession(storage)).toBeNull()
    expect(storage.getItem(QUICK_SETUP_SESSION_STORAGE_KEY)).toBeNull()
  })

  it('removes session when clearing', () => {
    const storage = createMemoryStorage()
    saveQuickSetupSession(
      createQuickSetupSessionSnapshot({
        status: 'in_progress',
        stepId: 'model',
        savedStepIds: [],
        selectedProviderId: 'dashscope-coding',
        providerApiKey: '',
        modelQuery: '',
        selectedChannelId: 'feishu',
        channelIdValue: '',
        channelSecretValue: '',
        browserDefaultProfileEnabled: false,
        toolsFullProfileEnabled: false,
        updatedAt: 1234,
      }),
      storage,
    )

    clearQuickSetupSession(storage)
    expect(storage.getItem(QUICK_SETUP_SESSION_STORAGE_KEY)).toBeNull()
  })
})
