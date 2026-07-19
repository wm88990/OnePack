<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { CategoryItem } from '../types'
import { useToast } from '../composables/useToast'

const props = defineProps<{
  selectedIds: string[]
  categories: CategoryItem[]
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'updated'): void
}>()

const { addToast } = useToast()

// 操作模式
const mode = ref<'install-dir' | 'category' | 'delete' | 'export-selected'>('install-dir')

// 批量安装目录
const batchDir = ref('')
const batchDirFormat = ref('/D=')

// 批量分类
const batchCategory = ref('system')

// 处理中
const processing = ref(false)

const count = computed(() => props.selectedIds.length)

async function executeBatch() {
  if (count.value === 0) return
  processing.value = true

  try {
    switch (mode.value) {
      case 'install-dir': {
        if (!batchDir.value.trim()) {
          addToast('请输入安装目录', 'info')
          return
        }
        const msg = await invoke<string>('batch_update_install_dir', {
          packageIds: props.selectedIds,
          installDir: batchDir.value.trim(),
          dirFormat: batchDirFormat.value.trim()
        })
        addToast(msg, 'success')
        break
      }
      case 'category': {
        const msg = await invoke<string>('batch_update_category', {
          packageIds: props.selectedIds,
          category: batchCategory.value
        })
        addToast(msg, 'success')
        break
      }
      case 'delete': {
        const msg = await invoke<string>('delete_packages', {
          packageIds: props.selectedIds
        })
        addToast(msg, 'success')
        break
      }
      case 'export-selected': {
        // 导出选中软件的配置
        const msg = await invoke<string>('export_selected_packages', {
          packageIds: props.selectedIds
        })
        addToast(msg, 'success')
        break
      }
    }
    emit('updated')
    emit('close')
  } catch (e) {
    addToast(`操作失败: ${e}`, 'error')
  } finally {
    processing.value = false
  }
}

function close() {
  emit('close')
}
</script>

<template>
  <div class="dialog-overlay" @click.self="close">
    <div class="dialog">
      <div class="dialog-header">
        <span class="dialog-title">⚡ 批量操作 ({{ count }} 款软件)</span>
        <button class="close-btn" @click="close">&times;</button>
      </div>

      <div class="dialog-body">
        <!-- 操作选择 -->
        <div class="mode-tabs">
          <button :class="['mode-tab', { active: mode === 'install-dir' }]" @click="mode = 'install-dir'">📁 安装目录</button>
          <button :class="['mode-tab', { active: mode === 'category' }]" @click="mode = 'category'">🏷️ 分类</button>
          <button :class="['mode-tab', { active: mode === 'delete' }]" @click="mode = 'delete'">🗑️ 删除</button>
        </div>

        <!-- 安装目录 -->
        <div v-if="mode === 'install-dir'" class="mode-content">
          <div class="form-row">
            <label class="form-label">统一安装目录</label>
            <input class="form-input" v-model="batchDir" placeholder="如 D:\Software 或 D:\Software\%NAME%" />
            <span class="form-hint">所有选中的 exe/msi 软件将安装到此目录</span>
          </div>
          <div class="form-row">
            <label class="form-label">目录参数格式</label>
            <select class="form-select" v-model="batchDirFormat">
              <option value="/D=">/D= (NSIS)</option>
              <option value="/DIR=">/DIR= (Inno Setup)</option>
              <option value="-d">-d (通用)</option>
              <option value="--install-dir=">--install-dir= (通用)</option>
            </select>
          </div>
          <div class="warn-box">
            ⚠️ 此操作将覆盖选中软件已有的安装目录配置
          </div>
        </div>

        <!-- 分类 -->
        <div v-if="mode === 'category'" class="mode-content">
          <div class="form-row">
            <label class="form-label">统一分类</label>
            <select class="form-select" v-model="batchCategory">
              <option v-for="cat in categories" :key="cat.id" :value="cat.id">{{ cat.icon }} {{ cat.name }}</option>
            </select>
          </div>
        </div>

        <!-- 删除 -->
        <div v-if="mode === 'delete'" class="mode-content">
          <div class="danger-box">
            <div class="danger-title">确认删除 {{ count }} 个软件？</div>
            <div class="danger-desc">此操作将从配置文件中移除这些软件条目，不会删除安装包文件。</div>
            <div class="danger-list">
              <span v-for="id in selectedIds" :key="id" class="danger-tag">{{ id }}</span>
            </div>
          </div>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn-cancel" @click="close">取消</button>
        <button
          :class="['btn-confirm', { 'btn-danger': mode === 'delete' }]"
          @click="executeBatch"
          :disabled="processing || count === 0"
        >
          {{ processing ? '处理中...' : (mode === 'delete' ? '确认删除' : '执行') }}
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
  width: 480px;
  max-height: 70vh;
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
.dialog-title { font-size: 15px; font-weight: 600; color: #333; }
.close-btn { background: none; border: none; font-size: 20px; cursor: pointer; color: #999; padding: 0 4px; }
.close-btn:hover { color: #333; }

.dialog-body { flex: 1; overflow-y: auto; padding: 16px 18px; }

.mode-tabs {
  display: flex;
  gap: 6px;
  margin-bottom: 16px;
}
.mode-tab {
  padding: 6px 14px;
  border: 1px solid #d9d9d9;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  color: #555;
  font-family: inherit;
}
.mode-tab:hover { border-color: #2b5ea7; color: #2b5ea7; }
.mode-tab.active { background: #2b5ea7; color: #fff; border-color: #2b5ea7; }

.mode-content { }

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
.form-select {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
  color: #333;
  background: #fff;
  font-family: inherit;
}
.form-hint { font-size: 11px; color: #999; margin-top: 4px; display: block; }

.warn-box {
  background: #fffbe6;
  border: 1px solid #ffe58f;
  border-radius: 4px;
  padding: 8px 12px;
  font-size: 12px;
  color: #ad6800;
  margin-top: 8px;
}

.danger-box {
  background: #fff2f0;
  border: 1px solid #ffccc7;
  border-radius: 6px;
  padding: 14px;
}
.danger-title { font-size: 14px; font-weight: 600; color: #cf1322; margin-bottom: 6px; }
.danger-desc { font-size: 12px; color: #a8071a; margin-bottom: 10px; }
.danger-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}
.danger-tag {
  font-size: 11px;
  padding: 2px 8px;
  background: #ffa39e;
  color: #820014;
  border-radius: 3px;
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
.btn-danger { background: #cf1322; }
.btn-danger:hover { background: #a8071a; }
</style>
