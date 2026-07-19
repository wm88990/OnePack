<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { PackageItem, CategoryItem } from '../types'
import { useToast } from '../composables/useToast'

const props = defineProps<{
  pkg: PackageItem
  categories: CategoryItem[]
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'saved', pkg: PackageItem): void
}>()

const { addToast } = useToast()

// 深拷贝编辑数据
const form = ref<PackageItem>(JSON.parse(JSON.stringify(props.pkg)))

// 标签页
const activeTab = ref<'basic' | 'install' | 'shortcut'>('basic')

// 保存中
const saving = ref(false)

// 监听外部 pkg 变化
watch(() => props.pkg, (val) => {
  form.value = JSON.parse(JSON.stringify(val))
}, { deep: true })

async function save() {
  saving.value = true
  try {
    await invoke('update_package', { packageData: form.value })
    addToast(`已保存: ${form.value.name}`, 'success')
    emit('saved', JSON.parse(JSON.stringify(form.value)))
    emit('close')
  } catch (e) {
    addToast(`保存失败: ${e}`, 'error')
  } finally {
    saving.value = false
  }
}

function close() {
  emit('close')
}

// 安装类型变化时的提示
function onInstallTypeChange(type: string) {
  if (type === 'green' && !form.value.install.main_exe) {
    form.value.install.silent_args = ''
    form.value.install.main_exe = `${form.value.id}.exe`
    form.value.install.extract_subdir = form.value.name
  } else if (type === 'exe' && form.value.install.silent_args === '') {
    form.value.install.silent_args = '/S'
    form.value.install.main_exe = undefined
    form.value.install.extract_subdir = undefined
  }
}
</script>

<template>
  <div class="dialog-overlay" @click.self="close">
    <div class="dialog dialog-lg">
      <div class="dialog-header">
        <span class="dialog-title">✏️ 编辑: {{ pkg.name }}</span>
        <button class="close-btn" @click="close">&times;</button>
      </div>

      <!-- 标签页 -->
      <div class="tabs">
        <button :class="['tab', { active: activeTab === 'basic' }]" @click="activeTab = 'basic'">基本信息</button>
        <button :class="['tab', { active: activeTab === 'install' }]" @click="activeTab = 'install'">安装配置</button>
        <button :class="['tab', { active: activeTab === 'shortcut' }]" @click="activeTab = 'shortcut'">快捷方式</button>
      </div>

      <div class="dialog-body">
        <!-- 基本信息 -->
        <div v-if="activeTab === 'basic'" class="tab-content">
          <div class="form-row">
            <label class="form-label">ID</label>
            <input class="form-input" v-model="form.id" disabled />
            <span class="form-hint">不可修改，唯一标识</span>
          </div>
          <div class="form-row">
            <label class="form-label">名称</label>
            <input class="form-input" v-model="form.name" />
          </div>
          <div class="form-row">
            <label class="form-label">图标</label>
            <input class="form-input input-sm" v-model="form.icon" style="width:60px" />
            <span class="form-preview">{{ form.icon }}</span>
          </div>
          <div class="form-row">
            <label class="form-label">分类</label>
            <select class="form-select" v-model="form.category">
              <option v-for="cat in categories" :key="cat.id" :value="cat.id">{{ cat.icon }} {{ cat.name }}</option>
            </select>
          </div>
          <div class="form-row">
            <label class="form-label">描述</label>
            <input class="form-input" v-model="form.description" />
          </div>
          <div class="form-row">
            <label class="form-label">版本</label>
            <input class="form-input" v-model="form.version" />
          </div>
          <div class="form-row">
            <label class="form-label">大小</label>
            <input class="form-input" v-model="form.size" />
          </div>
          <div class="form-row">
            <label class="form-label">启用</label>
            <label class="toggle-label">
              <input type="checkbox" v-model="form.enabled" />
              <span>{{ form.enabled ? '已启用' : '已禁用' }}</span>
            </label>
          </div>
        </div>

        <!-- 安装配置 -->
        <div v-if="activeTab === 'install'" class="tab-content">
          <div class="form-row">
            <label class="form-label">安装类型</label>
            <select class="form-select" v-model="form.install.type" @change="onInstallTypeChange(form.install.type)">
              <option value="exe">静默安装 (exe)</option>
              <option value="msi">MSI 安装</option>
              <option value="green">绿色解压 (zip)</option>
            </select>
          </div>
          <div class="form-row">
            <label class="form-label">安装包路径</label>
            <input class="form-input" v-model="form.source.path" placeholder="packages/ 下的相对路径" />
          </div>
          <div v-if="form.install.type !== 'green'" class="form-row">
            <label class="form-label">静默参数</label>
            <input class="form-input" v-model="form.install.silent_args" placeholder="/S /VERYSILENT 等" />
          </div>
          <div v-if="form.install.type === 'green'" class="form-row">
            <label class="form-label">主程序</label>
            <input class="form-input" v-model="form.install.main_exe" placeholder="主程序 exe 文件名" />
          </div>
          <div v-if="form.install.type === 'green'" class="form-row">
            <label class="form-label">解压子目录</label>
            <input class="form-input" v-model="form.install.extract_subdir" placeholder="解压到 apps/ 下的子目录名" />
          </div>
          <div v-if="form.install.type !== 'green'" class="form-row">
            <label class="form-label">安装目录</label>
            <input class="form-input" v-model="form.install.install_dir" placeholder="如 C:\Program Files\MyApp" />
          </div>
          <div v-if="form.install.type !== 'green'" class="form-row">
            <label class="form-label">目录参数格式</label>
            <input class="form-input" v-model="form.install.dir_format" placeholder="如 /D= 或 /DIR=" />
          </div>
          <div class="form-row">
            <label class="form-label">SHA256</label>
            <input class="form-input font-mono" v-model="form.source.hash_sha256" placeholder="校验哈希（可选）" />
          </div>
        </div>

        <!-- 快捷方式 -->
        <div v-if="activeTab === 'shortcut'" class="tab-content">
          <div class="form-row">
            <label class="form-label">创建桌面快捷方式</label>
            <label class="toggle-label">
              <input type="checkbox" v-model="form.install.create_shortcut" />
              <span>{{ form.install.create_shortcut ? '是' : '否' }}</span>
            </label>
          </div>
          <div v-if="form.install.create_shortcut" class="form-row">
            <label class="form-label">快捷方式名称</label>
            <input class="form-input" v-model="form.install.shortcut_name" :placeholder="form.name" />
            <span class="form-hint">留空则使用软件名称</span>
          </div>
          <div class="form-info-box">
            <div class="info-title">快捷方式说明</div>
            <ul>
              <li><b>绿色软件</b>：指向 apps/ 目录下的主程序</li>
              <li><b>exe/msi</b>：指向配置的安装目录中的可执行文件</li>
              <li>快捷方式创建在公共桌面（所有用户可见）</li>
            </ul>
          </div>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn-cancel" @click="close">取消</button>
        <button class="btn-confirm" @click="save" :disabled="saving">
          {{ saving ? '保存中...' : '保存修改' }}
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
  width: 520px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
}
.dialog-lg { width: 560px; }
.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid #e5e5e5;
}
.dialog-title { font-size: 15px; font-weight: 600; color: #333; }
.close-btn {
  background: none; border: none; font-size: 20px;
  cursor: pointer; color: #999; padding: 0 4px; line-height: 1;
}
.close-btn:hover { color: #333; }

/* 标签页 */
.tabs {
  display: flex;
  border-bottom: 1px solid #e5e5e5;
  background: #fafafa;
}
.tab {
  padding: 8px 18px;
  border: none;
  background: none;
  font-size: 13px;
  color: #666;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  font-family: inherit;
}
.tab:hover { color: #2b5ea7; }
.tab.active {
  color: #2b5ea7;
  border-bottom-color: #2b5ea7;
  font-weight: 600;
}

.dialog-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 18px;
}

/* 表单 */
.form-row {
  margin-bottom: 14px;
}
.form-label {
  display: block;
  font-size: 12px;
  font-weight: 600;
  color: #555;
  margin-bottom: 4px;
}
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
.form-input:focus { border-color: #2b5ea7; outline: none; box-shadow: 0 0 0 2px rgba(43,94,167,0.15); }
.form-input:disabled { background: #f5f5f5; color: #999; }
.form-input-sm { width: 60px; }
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
.form-select:focus { border-color: #2b5ea7; outline: none; }
.form-hint { font-size: 11px; color: #999; margin-left: 4px; }
.form-preview { font-size: 18px; margin-left: 8px; }
.font-mono { font-family: monospace; font-size: 12px; }
.toggle-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #333;
  cursor: pointer;
}
.toggle-label input[type="checkbox"] { accent-color: #2b5ea7; width: 16px; height: 16px; }

.form-info-box {
  background: #f0f7ff;
  border: 1px solid #d4e8f7;
  border-radius: 6px;
  padding: 12px 14px;
  margin-top: 8px;
}
.info-title { font-size: 12px; font-weight: 600; color: #2b5ea7; margin-bottom: 6px; }
.form-info-box ul {
  margin: 0;
  padding-left: 18px;
  font-size: 12px;
  color: #555;
  line-height: 1.8;
}
.form-info-box li b { color: #2b5ea7; }

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
