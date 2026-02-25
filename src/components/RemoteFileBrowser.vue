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
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="emit('close')">
    <Card class="w-full max-w-2xl m-4 flex flex-col max-h-[80vh]">
      <!-- 标题栏 -->
      <div class="flex items-center justify-between p-4 border-b">
        <h3 class="font-semibold text-lg flex items-center gap-2">
          <FolderSearch class="w-5 h-5 text-blue-600" />
          远程配置文件
        </h3>
        <Button variant="ghost" size="sm" @click="emit('close')" class="h-8 w-8 p-0">
          <X class="w-4 h-4" />
        </Button>
      </div>

      <!-- Tab 切换 -->
      <div class="flex border-b px-4">
        <button
          class="px-4 py-2 text-sm font-medium border-b-2 transition-colors"
          :class="activeTab === 'search'
            ? 'border-blue-600 text-blue-600'
            : 'border-transparent text-muted-foreground hover:text-foreground'"
          @click="activeTab = 'search'"
        >
          <Search class="w-3.5 h-3.5 inline mr-1" />
          自动搜索
        </button>
        <button
          class="px-4 py-2 text-sm font-medium border-b-2 transition-colors"
          :class="activeTab === 'browse'
            ? 'border-blue-600 text-blue-600'
            : 'border-transparent text-muted-foreground hover:text-foreground'"
          @click="activeTab = 'browse'; if (dirEntries.length === 0) browseDir('/')"
        >
          <Folder class="w-3.5 h-3.5 inline mr-1" />
          浏览目录
        </button>
      </div>

      <!-- 自动搜索 Tab -->
      <div v-if="activeTab === 'search'" class="flex-1 overflow-auto p-4">
        <div v-if="searching" class="flex items-center justify-center py-12 text-muted-foreground">
          <Loader2 class="w-5 h-5 animate-spin mr-2" />
          正在搜索配置文件...
        </div>

        <div v-else-if="searchResults.length === 0" class="text-center py-12 text-muted-foreground">
          <FolderSearch class="w-10 h-10 mx-auto mb-2 opacity-30" />
          <p class="text-sm">未找到配置文件</p>
          <p class="text-xs mt-1">尝试切换到"浏览目录"手动查找</p>
          <Button variant="outline" size="sm" class="mt-3" @click="searchConfig">
            <Search class="w-3.5 h-3.5" />
            重新搜索
          </Button>
        </div>

        <div v-else class="space-y-2">
          <p class="text-xs text-muted-foreground mb-3">
            找到 {{ searchResults.length }} 个配置文件，点击加载
          </p>
          <div
            v-for="result in searchResults"
            :key="result.path"
            class="flex items-center gap-3 p-3 rounded-lg border hover:bg-blue-50 dark:hover:bg-blue-950 cursor-pointer transition-colors"
            @click="selectResult(result)"
          >
            <FileJson class="w-5 h-5 text-blue-600 flex-shrink-0" />
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium">{{ result.fileName }}</div>
              <div class="text-xs text-muted-foreground truncate">{{ result.dirPath }}</div>
            </div>
            <ChevronRight class="w-4 h-4 text-muted-foreground flex-shrink-0" />
          </div>
        </div>
      </div>

      <!-- 浏览目录 Tab -->
      <div v-if="activeTab === 'browse'" class="flex-1 overflow-hidden flex flex-col">
        <!-- 路径导航 -->
        <div class="px-4 py-2 border-b space-y-2">
          <div class="flex items-center gap-1 text-sm overflow-x-auto">
            <Button variant="ghost" size="sm" class="h-6 px-1" @click="goBack" :disabled="pathHistory.length === 0">
              <ArrowLeft class="w-3.5 h-3.5" />
            </Button>
            <button class="text-blue-600 hover:underline text-xs" @click="browseDir('/')">
              /
            </button>
            <template v-for="crumb in breadcrumbs" :key="crumb.path">
              <ChevronRight class="w-3 h-3 text-muted-foreground flex-shrink-0" />
              <button class="text-blue-600 hover:underline text-xs truncate max-w-[120px]" @click="browseDir(crumb.path)">
                {{ crumb.name }}
              </button>
            </template>
          </div>
          <div class="flex gap-2">
            <Input v-model="customPath" placeholder="输入路径跳转" class="text-xs h-7" @keyup.enter="goToCustomPath" />
            <Button variant="outline" size="sm" class="h-7 text-xs" @click="goToCustomPath">跳转</Button>
          </div>
        </div>

        <!-- 文件列表 -->
        <div class="flex-1 overflow-auto p-2">
          <div v-if="browsing" class="flex items-center justify-center py-8 text-muted-foreground">
            <Loader2 class="w-5 h-5 animate-spin mr-2" />
            加载中...
          </div>

          <div v-else-if="dirEntries.length === 0" class="text-center py-8 text-muted-foreground">
            <Folder class="w-8 h-8 mx-auto mb-2 opacity-30" />
            <p class="text-sm">空目录</p>
          </div>

          <div v-else class="space-y-0.5">
            <div
              v-for="entry in dirEntries"
              :key="entry.path"
              class="flex items-center gap-2 px-3 py-1.5 rounded hover:bg-gray-100 dark:hover:bg-gray-800 cursor-pointer text-sm"
              :class="{ 'text-blue-600 font-medium': entry.name.endsWith('.json') }"
              @click="handleEntryClick(entry)"
            >
              <Folder v-if="entry.isDir" class="w-4 h-4 text-amber-500 flex-shrink-0" />
              <FileJson v-else-if="entry.name.endsWith('.json')" class="w-4 h-4 text-blue-500 flex-shrink-0" />
              <div v-else class="w-4 h-4 flex-shrink-0" />
              <span class="flex-1 truncate">{{ entry.name }}</span>
              <span v-if="!entry.isDir" class="text-xs text-muted-foreground">{{ formatSize(entry.size) }}</span>
              <ChevronRight v-if="entry.isDir" class="w-3.5 h-3.5 text-muted-foreground" />
            </div>
          </div>
        </div>
      </div>
    </Card>
  </div>
</template>
