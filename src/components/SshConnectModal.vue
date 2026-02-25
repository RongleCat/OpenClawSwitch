<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'
import Button from './ui/Button.vue'
import Input from './ui/Input.vue'
import Label from './ui/Label.vue'
import Card from './ui/Card.vue'
import {
  Monitor, Key, Lock, Trash2, Star, Plus, Loader2, Wifi, WifiOff, X
} from 'lucide-vue-next'
import type { SshProfile, SshAuthMode, FingerprintInfo } from '@/types/config'

const emit = defineEmits<{
  close: []
  connected: []
  fingerprint: [info: FingerprintInfo, onConfirm: () => void]
}>()

// 表单状态
const host = ref('')
const port = ref(22)
const username = ref('root')
const authMode = ref<SshAuthMode>('password')
const password = ref('')
const keyPath = ref('')
const passphrase = ref('')
const profileName = ref('')

// UI 状态
const connecting = ref(false)
const error = ref('')
const savedProfiles = ref<SshProfile[]>([])
const showSaveForm = ref(false)

// 加载已保存的连接配置
const loadProfiles = async () => {
  try {
    savedProfiles.value = await invoke<SshProfile[]>('ssh_load_profiles')
  } catch (e) {
    console.error('加载配置失败:', e)
  }
}

// 选择已保存的配置
const selectProfile = (profile: SshProfile) => {
  host.value = profile.host
  port.value = profile.port
  username.value = profile.username
  authMode.value = profile.authMode
  password.value = profile.password || ''
  keyPath.value = profile.keyPath || ''
  profileName.value = profile.name
  error.value = ''
}

// 删除已保存的配置
const deleteProfile = async (id: string) => {
  try {
    await invoke('ssh_delete_profile', { id })
    await loadProfiles()
  } catch (e) {
    console.error('删除配置失败:', e)
  }
}

// 选择私钥文件
const selectKeyFile = async () => {
  const selected = await open({
    title: '选择 SSH 私钥文件',
    filters: [{ name: '所有文件', extensions: ['*'] }]
  })
  if (selected && typeof selected === 'string') {
    keyPath.value = selected
  }
}

// 保存当前连接配置
const saveCurrentProfile = async () => {
  if (!profileName.value.trim()) {
    error.value = '请输入配置名称'
    return
  }
  try {
    const profile: SshProfile = {
      id: crypto.randomUUID(),
      name: profileName.value.trim(),
      host: host.value.trim(),
      port: port.value,
      username: username.value.trim(),
      authMode: authMode.value,
      password: authMode.value === 'password' ? password.value : undefined,
      keyPath: authMode.value === 'privateKey' ? keyPath.value : undefined,
    }
    await invoke('ssh_save_profile', { profile })
    await loadProfiles()
    showSaveForm.value = false
  } catch (e) {
    error.value = `保存失败: ${e}`
  }
}

// 连接 SSH 服务器
const connect = async () => {
  if (!host.value.trim() || !username.value.trim()) {
    error.value = '请填写主机地址和用户名'
    return
  }

  connecting.value = true
  error.value = ''

  try {
    // 步骤1：建立 TCP 连接并获取指纹
    const fingerprint = await invoke<FingerprintInfo>('ssh_connect', {
      host: host.value.trim(),
      port: port.value,
      username: username.value.trim(),
    })

    // 步骤2：通过事件让父组件显示指纹确认弹窗
    emit('fingerprint', fingerprint, async () => {
      try {
        // 步骤3：用户确认指纹后执行认证
        if (authMode.value === 'password') {
          await invoke('ssh_auth_password', { password: password.value })
        } else {
          if (!keyPath.value.trim()) {
            error.value = '请选择私钥文件'
            connecting.value = false
            return
          }
          await invoke('ssh_auth_key', {
            keyPath: keyPath.value.trim(),
            passphrase: passphrase.value || null,
          })
        }
        emit('connected')
      } catch (e) {
        error.value = `认证失败: ${e}`
        connecting.value = false
      }
    })
  } catch (e) {
    error.value = `连接失败: ${e}`
  } finally {
    connecting.value = false
  }
}

onMounted(loadProfiles)
</script>

<template>
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="emit('close')">
    <Card class="w-full max-w-lg p-6 m-4 max-h-[85vh] overflow-auto">
      <div class="flex items-center justify-between mb-4">
        <h3 class="font-semibold text-lg flex items-center gap-2">
          <Monitor class="w-5 h-5 text-blue-600" />
          SSH 远程连接
        </h3>
        <Button variant="ghost" size="sm" @click="emit('close')" class="h-8 w-8 p-0">
          <X class="w-4 h-4" />
        </Button>
      </div>

      <!-- 已保存的连接配置 -->
      <div v-if="savedProfiles.length > 0" class="mb-4">
        <p class="text-xs text-muted-foreground mb-2">已保存的连接</p>
        <div class="space-y-1">
          <div
            v-for="profile in savedProfiles"
            :key="profile.id"
            class="flex items-center gap-2 p-2 rounded-lg border hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer group"
            @click="selectProfile(profile)"
          >
            <Star class="w-3.5 h-3.5 text-amber-500 flex-shrink-0" />
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium truncate">{{ profile.name }}</div>
              <div class="text-xs text-muted-foreground truncate">
                {{ profile.username }}@{{ profile.host }}:{{ profile.port }}
                · {{ profile.authMode === 'password' ? '密码' : '私钥' }}
              </div>
            </div>
            <Button
              variant="ghost" size="sm"
              @click.stop="deleteProfile(profile.id)"
              class="h-6 w-6 p-0 opacity-0 group-hover:opacity-100 text-destructive"
            >
              <Trash2 class="w-3 h-3" />
            </Button>
          </div>
        </div>
      </div>

      <!-- 连接表单 -->
      <div class="space-y-3">
        <div class="grid grid-cols-3 gap-2">
          <div class="col-span-2">
            <Label class="text-xs mb-1 block">主机地址</Label>
            <Input v-model="host" placeholder="192.168.1.100" :disabled="connecting" />
          </div>
          <div>
            <Label class="text-xs mb-1 block">端口</Label>
            <Input v-model.number="port" type="number" placeholder="22" :disabled="connecting" />
          </div>
        </div>

        <div>
          <Label class="text-xs mb-1 block">用户名</Label>
          <Input v-model="username" placeholder="root" :disabled="connecting" />
        </div>

        <!-- 认证方式切换 -->
        <div>
          <Label class="text-xs mb-1 block">认证方式</Label>
          <div class="flex gap-2">
            <Button
              :variant="authMode === 'password' ? 'default' : 'outline'"
              size="sm" class="flex-1"
              @click="authMode = 'password'"
              :disabled="connecting"
            >
              <Lock class="w-3.5 h-3.5" />
              密码
            </Button>
            <Button
              :variant="authMode === 'privateKey' ? 'default' : 'outline'"
              size="sm" class="flex-1"
              @click="authMode = 'privateKey'"
              :disabled="connecting"
            >
              <Key class="w-3.5 h-3.5" />
              私钥
            </Button>
          </div>
        </div>

        <!-- 密码输入 -->
        <div v-if="authMode === 'password'">
          <Label class="text-xs mb-1 block">密码</Label>
          <Input v-model="password" type="password" placeholder="输入密码" :disabled="connecting" @keyup.enter="connect" />
        </div>

        <!-- 私钥选择 -->
        <template v-if="authMode === 'privateKey'">
          <div>
            <Label class="text-xs mb-1 block">私钥文件</Label>
            <div class="flex gap-2">
              <Input v-model="keyPath" placeholder="~/.ssh/id_rsa" :disabled="connecting" class="flex-1" />
              <Button variant="outline" size="sm" @click="selectKeyFile" :disabled="connecting">
                选择
              </Button>
            </div>
          </div>
          <div>
            <Label class="text-xs mb-1 block">
              私钥密码 <span class="text-muted-foreground">(可选)</span>
            </Label>
            <Input v-model="passphrase" type="password" placeholder="私钥密码" :disabled="connecting" />
          </div>
        </template>

        <!-- 错误提示 -->
        <div v-if="error" class="text-sm text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-950 p-2 rounded">
          {{ error }}
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="flex items-center justify-between mt-5">
        <Button
          variant="ghost" size="sm"
          @click="showSaveForm = !showSaveForm"
          :disabled="connecting || !host.trim()"
        >
          <Plus class="w-3.5 h-3.5" />
          保存连接
        </Button>
        <div class="flex gap-2">
          <Button variant="ghost" @click="emit('close')">取消</Button>
          <Button @click="connect" :disabled="connecting">
            <Loader2 v-if="connecting" class="w-4 h-4 animate-spin" />
            <Wifi v-else class="w-4 h-4" />
            {{ connecting ? '连接中...' : '连接' }}
          </Button>
        </div>
      </div>

      <!-- 保存配置表单 -->
      <div v-if="showSaveForm" class="mt-3 p-3 border rounded-lg bg-gray-50 dark:bg-gray-800">
        <div class="flex gap-2">
          <Input v-model="profileName" placeholder="配置名称" class="flex-1" />
          <Button size="sm" @click="saveCurrentProfile">保存</Button>
        </div>
      </div>
    </Card>
  </div>
</template>
