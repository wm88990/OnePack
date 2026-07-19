<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import Sidebar from './components/Sidebar.vue'
import PackageList from './components/PackageList.vue'
import BottomBar from './components/BottomBar.vue'
import ToastContainer from './components/ToastContainer.vue'
import type { ConfigFile, InstallProgressEvent } from './types'
import { useToast } from './composables/useToast'

const config = ref<ConfigFile | null>(null)
const loading = ref(true)
const loadError = ref('')
const selectedIds = ref<Set<string>>(new Set())
const installProgress = ref({ current: 0, total: 0, name: '', percent: 0 })
const isInstalling = ref(false)
const currentCategory = ref('all')
const searchKeyword = ref('')
const { toasts, addToast, removeToast } = useToast()

let unlisten: UnlistenFn | null = null

// 计算属性：已勾选的包数量
const selectedCount = computed(() => selectedIds.value.size)

// 加载配置
async function loadPackages() {
  loading.value = true
  loadError.value = ''
  try {
    config.value = await invoke<ConfigFile>('load_packages')
  } catch (e) {
    loadError.value = String(e)
    addToast(`加载配置失败: ${e}`, 'error')
  } finally {
    loading.value = false
  }
}

// 开始安装
async function startInstall() {
  if (selectedIds.value.size === 0) {
    addToast('请先勾选要安装的软件', 'info')
    return
  }
  isInstalling.value = true
  try {
    await invoke('start_install', { packageIds: Array.from(selectedIds.value) })
  } catch (e) {
    addToast(`安装失败: ${e}`, 'error')
  }
}

// 停止安装
async function stopInstall() {
  try {
    await invoke('stop_install_cmd')
    addToast('正在停止安装...', 'info')
  } catch (e) {
    addToast(`停止失败: ${e}`, 'error')
  }
}

// 导出配置
async function exportConfig() {
  if (!config.value) return
  try {
    await invoke('export_config', { configData: JSON.parse(JSON.stringify(config.value)) })
    addToast('配置已导出', 'success')
  } catch (e) {
    addToast(`导出失败: ${e}`, 'error')
  }
}

// 切换分类
function setCategory(cat: string) {
  currentCategory.value = cat
}

// 设置搜索
function setSearch(keyword: string) {
  searchKeyword.value = keyword
}

// 全选/取消全选/反选
function selectAllVisible() {
  if (!config.value) return
  const filtered = filteredPackages.value
  filtered.forEach(pkg => selectedIds.value.add(pkg.id))
}

function deselectAllVisible() {
  if (!config.value) return
  const filtered = filteredPackages.value
  filtered.forEach(pkg => selectedIds.value.delete(pkg.id))
}

function invertSelection() {
  if (!config.value) return
  const filtered = filteredPackages.value
  filtered.forEach(pkg => {
    if (selectedIds.value.has(pkg.id)) {
      selectedIds.value.delete(pkg.id)
    } else {
      selectedIds.value.add(pkg.id)
    }
  })
}

// 计算属性：过滤后的包列表
const filteredPackages = computed(() => {
  if (!config.value) return []
  const keyword = searchKeyword.value.trim().toLowerCase()
  return config.value.packages.filter(pkg => {
    if (currentCategory.value !== 'all') {
      if (pkg.category !== currentCategory.value) return false
    }
    if (keyword) {
      const searchStr = `${pkg.name} ${pkg.description} ${pkg.id}`.toLowerCase()
      if (!searchStr.includes(keyword)) return false
    }
    return true
  })
})

// 切换单个包的选中状态
function togglePackage(id: string, checked: boolean) {
  if (checked) {
    selectedIds.value.add(id)
  } else {
    selectedIds.value.delete(id)
  }
}

// 切换全选
function toggleCheckAll(checked: boolean) {
  if (!config.value) return
  const filtered = filteredPackages.value
  if (checked) {
    filtered.forEach(pkg => selectedIds.value.add(pkg.id))
  } else {
    filtered.forEach(pkg => selectedIds.value.delete(pkg.id))
  }
}

// 切换软件启用/禁用
function toggleEnabled(id: string, enabled: boolean) {
  if (!config.value) return
  const pkg = config.value.packages.find(p => p.id === id)
  if (pkg) {
    pkg.enabled = enabled
  }
}

onMounted(async () => {
  await loadPackages()

  // 监听安装进度事件
  unlisten = await listen<InstallProgressEvent>('install-progress', (event) => {
    const data = event.payload
    if (data.type === 'start') {
      installProgress.value.current = data.current || 0
      installProgress.value.total = data.total || 0
      installProgress.value.name = data.package_name
      installProgress.value.percent = data.total ? Math.round((((data.current || 0) - 1) / (data.total || 1)) * 100) : 0
    } else if (data.type === 'progress') {
      installProgress.value.percent = data.progress || 0
    } else if (data.type === 'success') {
      installProgress.value.current = (data.current || installProgress.value.current) + 1
      installProgress.value.percent = installProgress.value.total
        ? Math.round((installProgress.value.current / installProgress.value.total) * 100) : 0
      selectedIds.value.delete(data.package_id)
      addToast(`${data.package_name} 安装成功`, 'success')
    } else if (data.type === 'error') {
      selectedIds.value.delete(data.package_id)
      addToast(`${data.package_name}: ${data.message}`, 'error')
    } else if (data.type === 'complete') {
      isInstalling.value = false
      installProgress.value.percent = 100
      addToast(data.message || '安装完成', (data.failed ?? 0) > 0 ? 'error' : 'success')
    }
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
})
</script>

<template>
  <div class="app-container">
    <!-- 顶栏 -->
    <header class="topbar">
      <div class="logo">📦 OnePack <span>v1.0.0</span></div>
      <div class="spacer"></div>
      <button class="topbar-btn" @click="exportConfig">📤 导出配置</button>
      <button class="topbar-btn" @click="loadPackages">🔄 刷新</button>
    </header>

    <!-- 主体 -->
    <div class="main-body">
      <!-- 左侧分类栏 -->
      <Sidebar
        :categories="config?.categories || []"
        :packages="config?.packages || []"
        :current="currentCategory"
        :selected-count="selectedCount"
        @select-category="setCategory"
      />

      <!-- 右侧内容区 -->
      <div class="content-area">
        <!-- 工具栏 -->
        <div class="toolbar">
          <div class="search-box">
            <span class="search-icon">🔍</span>
            <input
              type="text"
              placeholder="搜索软件名称..."
              :value="searchKeyword"
              @input="setSearch(($event.target as HTMLInputElement).value)"
            />
          </div>
          <button class="tb-btn" @click="selectAllVisible">全选当前</button>
          <button class="tb-btn" @click="deselectAllVisible">取消当前</button>
          <button class="tb-btn" @click="invertSelection">反选</button>
          <div class="tb-info">
            共 <b>{{ config?.packages.length || 0 }}</b> 款，
            已选 <b>{{ selectedCount }}</b> 款
          </div>
        </div>

        <!-- 加载状态 -->
        <div v-if="loading" class="loading-state">
          <div class="loading-spinner">⏳</div>
          <div>正在加载配置...</div>
        </div>

        <!-- 错误状态 -->
        <div v-else-if="loadError" class="error-state">
          <div class="error-icon">⚠️</div>
          <div>{{ loadError }}</div>
          <button class="retry-btn" @click="loadPackages">重试</button>
        </div>

        <!-- 软件列表 -->
        <PackageList
          v-else-if="config"
          :packages="filteredPackages"
          :categories="config.categories"
          :selected-ids="selectedIds"
          @toggle-package="togglePackage"
          @toggle-check-all="toggleCheckAll"
          @toggle-enabled="toggleEnabled"
        />

        <!-- 底栏 -->
        <BottomBar
          :is-installing="isInstalling"
          :selected-count="selectedCount"
          :progress-current="installProgress.current"
          :progress-total="installProgress.total"
          :progress-name="installProgress.name"
          :progress-percent="installProgress.percent"
          @start="startInstall"
          @stop="stopInstall"
        />
      </div>
    </div>

    <!-- Toast 通知 -->
    <ToastContainer :toasts="toasts" @remove="removeToast" />
  </div>
</template>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

/* 顶栏 */
.topbar {
  height: 42px;
  background: #2b5ea7;
  display: flex;
  align-items: center;
  padding: 0 16px;
  gap: 10px;
  flex-shrink: 0;
}
.logo {
  font-size: 15px;
  font-weight: bold;
  color: #fff;
}
.logo span {
  color: #8cb8e8;
  font-weight: normal;
  font-size: 12px;
  margin-left: 8px;
}
.spacer { flex: 1; }
.topbar-btn {
  background: rgba(255,255,255,0.12);
  border: 1px solid rgba(255,255,255,0.2);
  color: #fff;
  padding: 4px 14px;
  border-radius: 3px;
  cursor: pointer;
  font-size: 12px;
  transition: background 0.15s;
}
.topbar-btn:hover { background: rgba(255,255,255,0.22); }

/* 主体 */
.main-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

/* 内容区 */
.content-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 工具栏 */
.toolbar {
  padding: 8px 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  border-bottom: 1px solid #ddd;
  background: #fafafa;
  flex-shrink: 0;
}
.search-box {
  display: flex;
  align-items: center;
  background: #fff;
  border: 1px solid #ccc;
  border-radius: 3px;
  padding: 0 8px;
  height: 30px;
  width: 220px;
}
.search-box input {
  border: none;
  outline: none;
  font-size: 12px;
  flex: 1;
  margin-left: 4px;
  background: transparent;
  color: #333;
  font-family: inherit;
}
.search-icon { color: #aaa; font-size: 13px; }
.tb-btn {
  padding: 4px 10px;
  border: 1px solid #bbb;
  background: #fff;
  border-radius: 3px;
  cursor: pointer;
  font-size: 12px;
  color: #444;
  font-family: inherit;
}
.tb-btn:hover { background: #e8e8e8; }
.tb-info {
  margin-left: auto;
  font-size: 12px;
  color: #777;
}
.tb-info b { color: #2b5ea7; }

/* 加载/错误状态 */
.loading-state,
.error-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: #999;
  font-size: 14px;
}
.loading-spinner,
.error-icon { font-size: 36px; }
.retry-btn {
  margin-top: 8px;
  padding: 6px 20px;
  background: #2b5ea7;
  border: none;
  color: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}
</style>
