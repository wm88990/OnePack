<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ConfigFile } from '../types'
import { useToast } from '../composables/useToast'

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'imported', config: ConfigFile): void
}>()

const { addToast } = useToast()

// 标签页模式
const activeTab = ref<'import' | 'compare'>('import')

// 导入
const importPath = ref('')
const importing = ref(false)

// 对比
const comparePath = ref('')
const comparing = ref(false)
const compareResult = ref<CompareResult | null>(null)

interface DiffItem {
  id: string
  name: string
  current: any
  other: any
  changes: { field: string; current: any; other: any }[]
}

interface CompareResult {
  current_name: string
  other_name: string
  current_count: number
  other_count: number
  only_in_current: any[]
  only_in_other: any[]
  different: DiffItem[]
}

async function doImport() {
  if (!importPath.value.trim()) {
    addToast('请输入配置文件路径', 'info')
    return
  }
  importing.value = true
  try {
    const config = await invoke<ConfigFile>('import_config', { importPath: importPath.value.trim() })
    addToast('配置导入成功', 'success')
    emit('imported', config)
    emit('close')
  } catch (e) {
    addToast(`导入失败: ${e}`, 'error')
  } finally {
    importing.value = false
  }
}

async function doCompare() {
  if (!comparePath.value.trim()) {
    addToast('请输入对比配置文件路径', 'info')
    return
  }
  comparing.value = true
  compareResult.value = null
  try {
    const result = await invoke<CompareResult>('compare_configs', { comparePath: comparePath.value.trim() })
    compareResult.value = result
  } catch (e) {
    addToast(`对比失败: ${e}`, 'error')
  } finally {
    comparing.value = false
  }
}

function close() {
  emit('close')
}

function getFieldLabel(field: string) {
  const map: Record<string, string> = {
    name: '名称', version: '版本', category: '分类', description: '描述',
    enabled: '启用状态', install_type: '安装类型', silent_args: '静默参数',
    install_dir: '安装目录', create_shortcut: '快捷方式'
  }
  return map[field] || field
}
</script>

<template>
  <div class="dialog-overlay" @click.self="close">
    <div class="dialog dialog-lg">
      <div class="dialog-header">
        <span class="dialog-title">📂 导入 / 对比配置</span>
        <button class="close-btn" @click="close">&times;</button>
      </div>

      <!-- 标签页 -->
      <div class="tabs">
        <button :class="['tab', { active: activeTab === 'import' }]" @click="activeTab = 'import'">📥 导入配置</button>
        <button :class="['tab', { active: activeTab === 'compare' }]" @click="activeTab = 'compare'">📊 对比配置</button>
      </div>

      <div class="dialog-body">
        <!-- 导入 -->
        <div v-if="activeTab === 'import'" class="tab-content">
          <div class="info-box">
            <b>导入说明</b>：将外部 config.yaml 的软件包合并到当前配置中。已有软件按 ID 更新，新软件追加。
          </div>
          <div class="form-row">
            <label class="form-label">配置文件路径</label>
            <input class="form-input font-mono" v-model="importPath" placeholder="config.yaml 的完整路径" />
          </div>
        </div>

        <!-- 对比 -->
        <div v-if="activeTab === 'compare'" class="tab-content">
          <div class="form-row">
            <label class="form-label">对比配置文件路径</label>
            <input class="form-input font-mono" v-model="comparePath" placeholder="要对比的 config.yaml 路径" />
          </div>
          <button class="btn-do-compare" @click="doCompare" :disabled="comparing">
            {{ comparing ? '对比中...' : '🔍 开始对比' }}
          </button>

          <!-- 对比结果 -->
          <div v-if="compareResult" class="compare-result">
            <div class="compare-summary">
              <div class="summary-item">
                <span class="summary-label">当前配置</span>
                <span class="summary-name">{{ compareResult.current_name }}</span>
                <span class="summary-count">{{ compareResult.current_count }} 款</span>
              </div>
              <div class="summary-vs">VS</div>
              <div class="summary-item">
                <span class="summary-label">对比配置</span>
                <span class="summary-name">{{ compareResult.other_name }}</span>
                <span class="summary-count">{{ compareResult.other_count }} 款</span>
              </div>
            </div>

            <!-- 差异统计 -->
            <div class="diff-stats">
              <div class="stat-item stat-added">
                仅在对比配置中: <b>{{ compareResult.only_in_other.length }}</b> 款
              </div>
              <div class="stat-item stat-removed">
                仅在当前配置中: <b>{{ compareResult.only_in_current.length }}</b> 款
              </div>
              <div class="stat-item stat-changed">
                存在差异: <b>{{ compareResult.different.length }}</b> 款
              </div>
            </div>

            <!-- 差异详情 -->
            <div v-if="compareResult.different.length > 0" class="diff-list">
              <div class="diff-section-title">差异详情</div>
              <div v-for="diff in compareResult.different" :key="diff.id" class="diff-item">
                <div class="diff-pkg-name">{{ diff.name }} <span class="diff-pkg-id">({{ diff.id }})</span></div>
                <div class="diff-changes">
                  <div v-for="change in diff.changes" :key="change.field" class="diff-change-row">
                    <span class="diff-field">{{ getFieldLabel(change.field) }}</span>
                    <span class="diff-arrow">&rarr;</span>
                    <span class="diff-current">{{ change.current }}</span>
                    <span class="diff-arrow">&rarr;</span>
                    <span class="diff-other">{{ change.other }}</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- 仅在当前/对比中的列表 -->
            <div v-if="compareResult.only_in_other.length > 0" class="diff-list">
              <div class="diff-section-title added">仅在对比配置中（新增）</div>
              <div v-for="pkg in compareResult.only_in_other" :key="pkg.id" class="diff-item-sm">
                {{ pkg.icon }} {{ pkg.name }} <span class="diff-pkg-id">({{ pkg.id }})</span>
              </div>
            </div>

            <div v-if="compareResult.only_in_current.length > 0" class="diff-list">
              <div class="diff-section-title removed">仅在当前配置中（将移除）</div>
              <div v-for="pkg in compareResult.only_in_current" :key="pkg.id" class="diff-item-sm">
                {{ pkg.icon }} {{ pkg.name }} <span class="diff-pkg-id">({{ pkg.id }})</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn-cancel" @click="close">取消</button>
        <button v-if="activeTab === 'import'" class="btn-confirm" @click="doImport" :disabled="importing">
          {{ importing ? '导入中...' : '导入并合并' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}
.dialog {
  background: #fff;
  border-radius: 8px;
  width: 560px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
}
.dialog-lg { max-height: 85vh; }
.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid #e5e5e5;
}
.dialog-title { font-size: 15px; font-weight: 600; color: #333; }
.close-btn { background: none; border: none; font-size: 20px; cursor: pointer; color: #999; padding: 0 4px; }
.close-btn:hover { color: #333; }

.tabs {
  display: flex;
  border-bottom: 1px solid #e5e5e5;
  background: #fafafa;
}
.tab {
  padding: 8px 18px;
  border: none; background: none;
  font-size: 13px; color: #666;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  font-family: inherit;
}
.tab:hover { color: #2b5ea7; }
.tab.active { color: #2b5ea7; border-bottom-color: #2b5ea7; font-weight: 600; }

.dialog-body { flex: 1; overflow-y: auto; padding: 16px 18px; }

.info-box {
  background: #f0f7ff;
  border: 1px solid #d4e8f7;
  border-radius: 6px;
  padding: 10px 14px;
  margin-bottom: 14px;
  font-size: 12px;
  color: #555;
  line-height: 1.6;
}

.form-row { margin-bottom: 14px; }
.form-label { display: block; font-size: 12px; font-weight: 600; color: #555; margin-bottom: 4px; }
.form-input {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
  color: #333;
  box-sizing: border-box;
  font-family: inherit;
}
.form-input:focus { border-color: #2b5ea7; outline: none; }
.font-mono { font-family: monospace; font-size: 12px; }

.btn-do-compare {
  width: 100%;
  padding: 8px;
  background: #2b5ea7;
  color: #fff;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  font-family: inherit;
  margin-bottom: 14px;
}
.btn-do-compare:hover { background: #1e4c8a; }
.btn-do-compare:disabled { background: #aaa; }

/* 对比结果 */
.compare-summary {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 12px;
  background: #fafafa;
  border-radius: 6px;
  margin-bottom: 12px;
}
.summary-item { text-align: center; }
.summary-label { display: block; font-size: 11px; color: #999; }
.summary-name { font-size: 14px; font-weight: 600; color: #333; }
.summary-count { font-size: 12px; color: #2b5ea7; }
.summary-vs { font-size: 13px; font-weight: 700; color: #999; }

.diff-stats {
  display: flex;
  gap: 12px;
  margin-bottom: 14px;
}
.stat-item {
  flex: 1;
  padding: 8px;
  border-radius: 4px;
  font-size: 12px;
  text-align: center;
}
.stat-added { background: #f6ffed; border: 1px solid #b7eb8f; color: #389e0d; }
.stat-removed { background: #fff2f0; border: 1px solid #ffccc7; color: #cf1322; }
.stat-changed { background: #fff7e6; border: 1px solid #ffe58f; color: #d48806; }

.diff-list { margin-top: 10px; }
.diff-section-title {
  font-size: 12px;
  font-weight: 600;
  color: #555;
  padding: 6px 0 4px;
  border-bottom: 1px solid #eee;
  margin-bottom: 4px;
}
.diff-section-title.added { color: #389e0d; }
.diff-section-title.removed { color: #cf1322; }

.diff-item {
  padding: 8px 10px;
  background: #fffbf0;
  border: 1px solid #ffe8c2;
  border-radius: 4px;
  margin-bottom: 6px;
}
.diff-pkg-name { font-size: 13px; font-weight: 600; color: #333; }
.diff-pkg-id { font-size: 11px; color: #999; font-weight: normal; }
.diff-changes { margin-top: 6px; }
.diff-change-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  padding: 2px 0;
}
.diff-field { color: #d48806; font-weight: 600; min-width: 60px; }
.diff-arrow { color: #ccc; }
.diff-current { color: #cf1322; text-decoration: line-through; background: #fff2f0; padding: 0 4px; border-radius: 2px; }
.diff-other { color: #389e0d; background: #f6ffed; padding: 0 4px; border-radius: 2px; }

.diff-item-sm {
  padding: 4px 10px;
  font-size: 12px;
  color: #555;
  border-bottom: 1px solid #f5f5f5;
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
