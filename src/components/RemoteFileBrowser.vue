<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import Button from './ui/Button.vue'
import Input from './ui/Input.vue'
import Card from './ui/Card.vue'
import {
  Search, Folder, FileJson, ChevronRight, Loader2, X, FolderSearch, ArrowLeft
} from 'lucide-vue-next'
import type { RemoteFileEntry, ConfigSearchResult } from '@/types/config'

const emit = defineEmits<{
  close: []
  select: [path: string]
}>()

// 状态
const searching = ref(false)
const browsing = ref(false)
const searchResults = ref<ConfigSearchResult[]>([])
const currentPath = ref('/')
const dirEntries = ref<RemoteFileEntry[]>([])
const pathHistory = ref<string[]>([])
const customPath = ref('')
const activeTab = ref<'search' | 'browse'>('search')

// 面包屑路径
const breadcrumbs = computed(() => {
  const parts = currentPath.value.split('/').filter(Boolean)
  return parts.map((part, index) => ({
    name: part,
    path: '/' + parts.slice(0, index + 1).join('/')
  }))
})

// 自动搜索配置文件
const searchConfig = async () => {
  searching.value = true
  try {
    searchResults.value = await invoke<ConfigSearchResult[]>('ssh_search_config')
  } catch (e) {
    console.error('搜索失败:', e)
  } finally {
    searching.value = false
  }
}

// 浏览目录（pushHistory 控制是否记录历史）
const browseDir = async (path: string, pushHistory = true) => {
  browsing.value = true
  try {
    dirEntries.value = await invoke<RemoteFileEntry[]>('ssh_list_dir', { path })
    if (pushHistory && currentPath.value !== path) {
      pathHistory.value.push(currentPath.value)
    }
    currentPath.value = path
  } catch (e) {
    console.error('浏览目录失败:', e)
  } finally {
    browsing.value = false
  }
}

// 返回上级目录
const goBack = () => {
  const prev = pathHistory.value.pop()
  if (prev !== undefined) {
    browseDir(prev, false)
  }
}

// 跳转到自定义路径
const goToCustomPath = () => {
  if (customPath.value.trim()) {
    browseDir(customPath.value.trim())
  }
}

// 点击文件条目
const handleEntryClick = (entry: RemoteFileEntry) => {
  if (entry.isDir) {
    browseDir(entry.path)
  } else if (entry.name.endsWith('.json')) {
    emit('select', entry.path)
  }
}

// 选择搜索结果
const selectResult = (result: ConfigSearchResult) => {
  emit('select', result.path)
}

// 格式化文件大小
const formatSize = (bytes: number): string => {
  if (bytes === 0) return '-'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

onMounted(searchConfig)
</script>

<template>
  <div class="oc-modal-overlay" @click.self="emit('close')">
    <Card class="oc-modal-card w-full max-w-2xl flex max-h-[80vh] flex-col">
      <div class="flex items-center justify-between border-b p-4" style="border-color: var(--oc-divider-soft);">
        <h3 class="flex items-center gap-2 text-lg font-semibold" style="color: var(--oc-text-primary);">
          <FolderSearch class="w-5 h-5" style="color: var(--oc-accent);" />
          远程配置文件
        </h3>
        <Button variant="ghost" size="sm" @click="emit('close')" class="h-8 w-8 p-0">
          <X class="w-4 h-4" />
        </Button>
      </div>

      <div class="flex border-b px-4" style="border-color: var(--oc-divider-soft);">
        <button
          class="px-4 py-2 text-sm font-medium border-b-2 transition-colors"
          :class="activeTab === 'search'
            ? 'border-[var(--oc-accent)] text-[var(--oc-accent)]'
            : 'border-transparent text-[var(--oc-text-muted)] hover:text-[var(--oc-text-primary)]'"
          @click="activeTab = 'search'"
        >
          <Search class="w-3.5 h-3.5 inline mr-1" />
          自动搜索
        </button>
        <button
          class="px-4 py-2 text-sm font-medium border-b-2 transition-colors"
          :class="activeTab === 'browse'
            ? 'border-[var(--oc-accent)] text-[var(--oc-accent)]'
            : 'border-transparent text-[var(--oc-text-muted)] hover:text-[var(--oc-text-primary)]'"
          @click="activeTab = 'browse'; if (dirEntries.length === 0) browseDir('/')"
        >
          <Folder class="w-3.5 h-3.5 inline mr-1" />
          浏览目录
        </button>
      </div>

      <div v-if="activeTab === 'search'" class="flex-1 overflow-auto p-4">
        <div v-if="searching" class="flex items-center justify-center py-12" style="color: var(--oc-text-muted);">
          <Loader2 class="w-5 h-5 animate-spin mr-2" />
          正在搜索配置文件...
        </div>

        <div v-else-if="searchResults.length === 0" class="py-12 text-center" style="color: var(--oc-text-muted);">
          <FolderSearch class="w-10 h-10 mx-auto mb-2 opacity-30" />
          <p class="text-sm">未找到配置文件</p>
          <p class="text-xs mt-1">尝试切换到"浏览目录"手动查找</p>
          <Button variant="outline" size="sm" class="mt-3" @click="searchConfig">
            <Search class="w-3.5 h-3.5" />
            重新搜索
          </Button>
        </div>

        <div v-else class="space-y-2">
          <p class="mb-3 text-xs" style="color: var(--oc-text-muted);">
            找到 {{ searchResults.length }} 个配置文件，点击加载
          </p>
          <div
            v-for="result in searchResults"
            :key="result.path"
            class="flex cursor-pointer items-center gap-3 rounded-[11px] border p-3 transition-colors hover:opacity-90"
            style="border-color: var(--oc-card-border); background: var(--oc-card);"
            @click="selectResult(result)"
          >
            <FileJson class="w-5 h-5 flex-shrink-0" style="color: var(--oc-accent);" />
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium" style="color: var(--oc-text-primary);">{{ result.fileName }}</div>
              <div class="truncate text-xs" style="color: var(--oc-text-muted);">{{ result.dirPath }}</div>
            </div>
            <ChevronRight class="w-4 h-4 flex-shrink-0" style="color: var(--oc-text-muted);" />
          </div>
        </div>
      </div>

      <div v-if="activeTab === 'browse'" class="flex-1 overflow-visible flex flex-col">
        <div class="space-y-2 border-b px-4 py-2" style="border-color: var(--oc-divider-soft);">
          <div class="flex items-center gap-1 text-sm overflow-x-auto">
            <Button variant="ghost" size="sm" class="h-6 px-1" @click="goBack" :disabled="pathHistory.length === 0">
              <ArrowLeft class="w-3.5 h-3.5" />
            </Button>
            <button class="text-xs hover:underline" style="color: var(--oc-accent);" @click="browseDir('/')">
              /
            </button>
            <template v-for="crumb in breadcrumbs" :key="crumb.path">
              <ChevronRight class="w-3 h-3 flex-shrink-0" style="color: var(--oc-text-muted);" />
              <button class="truncate text-xs hover:underline max-w-[120px]" style="color: var(--oc-accent);" @click="browseDir(crumb.path)">
                {{ crumb.name }}
              </button>
            </template>
          </div>
          <div class="flex gap-2">
            <Input v-model="customPath" placeholder="输入路径跳转" class="text-xs h-7" @keyup.enter="goToCustomPath" />
            <Button variant="outline" size="sm" class="h-7 text-xs" @click="goToCustomPath">跳转</Button>
          </div>
        </div>

        <div class="flex-1 overflow-auto p-2">
          <div v-if="browsing" class="flex items-center justify-center py-8" style="color: var(--oc-text-muted);">
            <Loader2 class="w-5 h-5 animate-spin mr-2" />
            加载中...
          </div>

          <div v-else-if="dirEntries.length === 0" class="py-8 text-center" style="color: var(--oc-text-muted);">
            <Folder class="w-8 h-8 mx-auto mb-2 opacity-30" />
            <p class="text-sm">空目录</p>
          </div>

          <div v-else class="space-y-0.5">
            <div
              v-for="entry in dirEntries"
              :key="entry.path"
              class="flex cursor-pointer items-center gap-2 rounded-[9px] px-3 py-1.5 text-sm transition-colors hover:opacity-90"
              :style="{ background: entry.name.endsWith('.json') ? 'var(--oc-accent-soft)' : 'transparent', color: entry.name.endsWith('.json') ? 'var(--oc-accent)' : 'var(--oc-text-secondary)' }"
              @click="handleEntryClick(entry)"
            >
              <Folder v-if="entry.isDir" class="w-4 h-4 flex-shrink-0" style="color: var(--oc-warning);" />
              <FileJson v-else-if="entry.name.endsWith('.json')" class="w-4 h-4 flex-shrink-0" style="color: var(--oc-accent);" />
              <div v-else class="w-4 h-4 flex-shrink-0" />
              <span class="flex-1 truncate">{{ entry.name }}</span>
              <span v-if="!entry.isDir" class="text-xs" style="color: var(--oc-text-muted);">{{ formatSize(entry.size) }}</span>
              <ChevronRight v-if="entry.isDir" class="w-3.5 h-3.5" style="color: var(--oc-text-muted);" />
            </div>
          </div>
        </div>
      </div>
    </Card>
  </div>
</template>
