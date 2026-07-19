<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { PackageItem } from '../types'

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'added', packages: PackageItem[]): void
}>()

const isDragging = ref(false)
const scanning = ref(false)
const scannedFiles = ref<ScannedFile[]>([])
const selectedFiles = ref<Set<string>>(new Set())
const errorMessage = ref('')

interface ScannedFile {
  id: string
  name: string
  icon: string
  category: string
  description: string
  version: string
  size: string
  enabled: boolean
  source: {
    type: string
    path: string
    hash_sha256: string
  }
  install: {
    type: string
    silent_args: string
    main_exe?: string
    extract_subdir?: string
    create_shortcut: boolean
  }
}

const selectedCount = computed(() => selectedFiles.value.size)

// 拖拽处理
function onDragOver(e: DragEvent) {
  e.preventDefault()
  isDragging.value = true
}

function onDragLeave() {
  isDragging.value = false
}

async function onDrop(e: DragEvent) {
  e.preventDefault()
  isDragging.value = false
  // Tauri 的拖拽会提供文件路径，触发扫描
  await doScan()
}

// 选择文件按钮 - 触发扫描 packages 目录
async function selectFiles() {
  await doScan()
}

// 扫描 packages 目录
async function doScan() {
  scanning.value = true
  errorMessage.value = ''
  try {
    const result = await invoke<ScannedFile[]>('scan_packages')
    scannedFiles.value = result
    // 默认全选扫描到的新文件
    selectedFiles.value = new Set(result.map(f => f.id))
    if (result.length === 0) {
      errorMessage.value = '未在 packages/ 目录中发现新的安装包文件'
    }
  } catch (e) {
    errorMessage.value = String(e)
  } finally {
    scanning.value = false
  }
}

// 全选/取消全选
function toggleSelectAll(checked: boolean) {
  if (checked) {
    selectedFiles.value = new Set(scannedFiles.value.map(f => f.id))
  } else {
    selectedFiles.value.clear()
  }
}

// 切换单个选择
function toggleFile(id: string, checked: boolean) {
  if (checked) {
    selectedFiles.value.add(id)
  } else {
    selectedFiles.value.delete(id)
  }
}

// 确认添加
async function confirmAdd() {
  if (selectedFiles.value.size === 0) {
    errorMessage.value = '请至少选择一个文件'
    return
  }

  const selected = scannedFiles.value.filter(f => selectedFiles.value.has(f.id))
  try {
    await invoke('add_packages', { packages: selected })
    emit('added', selected as unknown as PackageItem[])
    emit('close')
  } catch (e) {
    errorMessage.value = String(e)
  }
}

// 关闭
function close() {
  emit('close')
}

// 根据安装类型显示图标
function typeIcon(type: string) {
  switch (type) {
    case 'exe': return '📝'
    case 'msi': return '📦'
    case 'green': return '🍃'
    default: return '📄'
  }
}

function typeLabel(type: string) {
  switch (type) {
    case 'exe': return '静默安装'
    case 'msi': return 'MSI安装'
    case 'green': return '绿色解压'
    default: return type
  }
}
</script>

<template>
  <div class="dialog-overlay" @click.self="close">
    <div class="dialog">
      <div class="dialog-header">
        <span class="dialog-title">📥 添加软件包</span>
        <button class="close-btn" @click="close">&times;</button>
      </div>

      <div class="dialog-body">
        <!-- 拖拽/选择区域 -->
        <div
          class="drop-zone"
          :class="{ dragging: isDragging }"
          @dragover="onDragOver"
          @dragleave="onDragLeave"
          @drop="onDrop"
        >
          <div class="drop-icon">{{ isDragging ? '📥' : '📂' }}</div>
          <div class="drop-text">
            {{ isDragging ? '释放文件到此处' : '将安装包拖放到此处，或点击下方按钮扫描 packages/ 目录' }}
          </div>
          <button class="scan-btn" @click="selectFiles" :disabled="scanning">
            {{ scanning ? '⏳ 扫描中...' : '🔍 扫描 packages/ 目录' }}
          </button>
        </div>

        <!-- 错误消息 -->
        <div v-if="errorMessage" class="error-msg">{{ errorMessage }}</div>

        <!-- 扫描结果 -->
        <div v-if="scannedFiles.length > 0" class="scan-results">
          <div class="results-header">
            <label class="select-all-label">
              <input type="checkbox" :checked="selectedCount === scannedFiles.length" @change="toggleSelectAll(($event.target as HTMLInputElement).checked)" />
              <span>全选</span>
            </label>
            <span class="results-count">找到 {{ scannedFiles.length }} 个安装包，已选 {{ selectedCount }} 个</span>
          </div>

          <div class="file-list">
            <div
              v-for="file in scannedFiles"
              :key="file.id"
              class="file-item"
              :class="{ selected: selectedFiles.has(file.id) }"
            >
              <label class="file-check">
                <input type="checkbox" :checked="selectedFiles.has(file.id)" @change="toggleFile(file.id, ($event.target as HTMLInputElement).checked)" />
              </label>
              <span class="file-icon">{{ file.icon }}</span>
              <div class="file-info">
                <div class="file-name">{{ file.name }}</div>
                <div class="file-meta">
                  <span class="type-badge" :class="file.install.type">
                    {{ typeIcon(file.install.type) }} {{ typeLabel(file.install.type) }}
                  </span>
                  <span class="file-size">{{ file.size }}</span>
                </div>
              </div>
              <div class="file-actions">
                <span class="file-path">{{ file.source.path }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn-cancel" @click="close">取消</button>
        <button class="btn-confirm" @click="confirmAdd" :disabled="selectedCount === 0">
          添加 ({{ selectedCount }})
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}
.dialog {
  background: #fff;
  border-radius: 8px;
  width: 620px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
}
.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid #e5e5e5;
}
.dialog-title {
  font-size: 15px;
  font-weight: 600;
  color: #333;
}
.close-btn {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: #999;
  padding: 0 4px;
  line-height: 1;
}
.close-btn:hover { color: #333; }

.dialog-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 18px;
}

/* 拖拽区域 */
.drop-zone {
  border: 2px dashed #ccc;
  border-radius: 8px;
  padding: 28px 16px;
  text-align: center;
  transition: all 0.2s;
  background: #fafafa;
}
.drop-zone.dragging {
  border-color: #2b5ea7;
  background: #e8f0fe;
}
.drop-icon { font-size: 32px; margin-bottom: 8px; }
.drop-text {
  font-size: 13px;
  color: #777;
  margin-bottom: 12px;
  line-height: 1.5;
}
.scan-btn {
  padding: 7px 20px;
  background: #2b5ea7;
  color: #fff;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  font-family: inherit;
}
.scan-btn:hover { background: #1e4c8a; }
.scan-btn:disabled { background: #aaa; cursor: not-allowed; }

.error-msg {
  margin-top: 10px;
  padding: 8px 12px;
  background: #fff2f0;
  border: 1px solid #ffccc7;
  border-radius: 4px;
  color: #cf1322;
  font-size: 12px;
}

/* 扫描结果 */
.scan-results {
  margin-top: 14px;
}
.results-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}
.select-all-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #555;
  cursor: pointer;
}
.results-count {
  font-size: 12px;
  color: #999;
}

.file-list {
  border: 1px solid #e5e5e5;
  border-radius: 4px;
  max-height: 240px;
  overflow-y: auto;
}
.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-bottom: 1px solid #f0f0f0;
  transition: background 0.1s;
}
.file-item:last-child { border-bottom: none; }
.file-item:hover { background: #f7f9fc; }
.file-item.selected { background: #e8f0fe; }
.file-check input[type="checkbox"] {
  cursor: pointer;
  accent-color: #2b5ea7;
}
.file-icon { font-size: 18px; }
.file-info { flex: 1; min-width: 0; }
.file-name {
  font-size: 13px;
  font-weight: 500;
  color: #333;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.file-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 2px;
}
.type-badge {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 3px;
  background: #f0f0f0;
  color: #666;
}
.type-badge.green { background: #f0fff4; color: #389e0d; }
.type-badge.msi { background: #fff7e6; color: #d48806; }
.file-size { font-size: 11px; color: #999; }
.file-actions { flex-shrink: 0; }
.file-path {
  font-size: 11px;
  color: #bbb;
  font-family: monospace;
  max-width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 12px 18px;
  border-top: 1px solid #e5e5e5;
}
.btn-cancel {
  padding: 7px 18px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  font-family: inherit;
  color: #555;
}
.btn-cancel:hover { background: #f5f5f5; }
.btn-confirm {
  padding: 7px 22px;
  background: #2b5ea7;
  color: #fff;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  font-family: inherit;
}
.btn-confirm:hover { background: #1e4c8a; }
.btn-confirm:disabled { background: #aaa; cursor: not-allowed; }
</style>
