<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits<{
  close: []
  saved: []
}>()

const yamlContent = ref('')
const originalContent = ref('')
const loading = ref(false)
const saving = ref(false)
const error = ref('')
const dirty = ref(false)
const lineHeight = ref(18)
const editorRef = ref<HTMLTextAreaElement | null>(null)

async function loadConfig() {
  loading.value = true
  error.value = ''
  try {
    const content = await invoke<string>('get_config_yaml')
    yamlContent.value = content
    originalContent.value = content
    dirty.value = false
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

function onInput() {
  dirty.value = yamlContent.value !== originalContent.value
}

async function saveConfig() {
  saving.value = true
  error.value = ''
  try {
    await invoke('save_config_yaml', { yamlContent: yamlContent.value })
    originalContent.value = yamlContent.value
    dirty.value = false
    emit('saved')
  } catch (e) {
    error.value = String(e)
  } finally {
    saving.value = false
  }
}

// Tab 键支持缩进
function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Tab') {
    e.preventDefault()
    const ta = editorRef.value
    if (!ta) return
    const start = ta.selectionStart
    const end = ta.selectionEnd
    const val = ta.value
    yamlContent.value = val.substring(0, start) + '  ' + val.substring(end)
    requestAnimationFrame(() => {
      ta.selectionStart = ta.selectionEnd = start + 2
    })
    onInput()
  }
  // Ctrl+S 保存
  if ((e.ctrlKey || e.metaKey) && e.key === 's') {
    e.preventDefault()
    if (dirty.value) saveConfig()
  }
}

function formatYaml() {
  // 简单格式化：去除多余空行（连续3+空行合并为2行）
  const lines = yamlContent.value.split('\n')
  const result: string[] = []
  let emptyCount = 0
  for (const line of lines) {
    if (line.trim() === '') {
      emptyCount++
      if (emptyCount <= 2) result.push('')
    } else {
      emptyCount = 0
      result.push(line)
    }
  }
  // 移除末尾多余空行
  while (result.length > 0 && result[result.length - 1].trim() === '') {
    result.pop()
  }
  result.push('')
  yamlContent.value = result.join('\n')
  onInput()
}

function resetContent() {
  yamlContent.value = originalContent.value
  dirty.value = false
}

function lineCount(text: string): number {
  if (!text) return 1
  return text.split('\n').length
}

onMounted(() => {
  loadConfig()
})
</script>

<template>
  <div class="config-editor-overlay" @click.self="$emit('close')">
    <div class="config-editor-dialog">
      <!-- 标题栏 -->
      <div class="ce-header">
        <div class="ce-title">
          <span class="ce-icon">📝</span>
          <span>编辑配置文件 (config.yaml)</span>
        </div>
        <div class="ce-dirty" v-if="dirty">● 已修改</div>
        <div class="ce-actions">
          <button class="ce-btn ce-btn-secondary" @click="formatYaml" :disabled="saving" title="格式化（去除多余空行）">格式化</button>
          <button class="ce-btn ce-btn-secondary" @click="resetContent" :disabled="saving || !dirty" title="撤销所有修改">重置</button>
          <button class="ce-btn ce-btn-save" @click="saveConfig" :disabled="saving || !dirty">
            {{ saving ? '保存中...' : '保存 (Ctrl+S)' }}
          </button>
          <button class="ce-btn ce-btn-close" @click="$emit('close')">✕</button>
        </div>
      </div>

      <!-- 加载状态 -->
      <div v-if="loading" class="ce-loading">
        <span class="ce-loading-icon">⏳</span> 正在加载配置文件...
      </div>

      <!-- 编辑器主体 -->
      <div v-else class="ce-body">
        <!-- 错误提示 -->
        <div v-if="error" class="ce-error">
          <span>⚠️</span> {{ error }}
          <button class="ce-error-close" @click="error = ''">✕</button>
        </div>

        <!-- 编辑区域 -->
        <div class="ce-editor-wrap">
          <!-- 行号 -->
          <div class="ce-linenumbers">
            <div
              v-for="n in lineCount(yamlContent)"
              :key="n"
              class="ce-line-no"
              :style="{ height: lineHeight + 'px', lineHeight: lineHeight + 'px' }"
            >{{ n }}</div>
          </div>
          <!-- 文本编辑器 -->
          <textarea
            ref="editorRef"
            class="ce-textarea"
            v-model="yamlContent"
            @input="onInput"
            @keydown="onKeydown"
            spellcheck="false"
            :style="{ lineHeight: lineHeight + 'px' }"
            placeholder="配置文件为空..."
          ></textarea>
        </div>

        <!-- 状态栏 -->
        <div class="ce-statusbar">
          <span>{{ lineCount(yamlContent) }} 行</span>
          <span class="ce-sep">|</span>
          <span>{{ yamlContent.length }} 字符</span>
          <span class="ce-sep">|</span>
          <span :class="dirty ? 'ce-dirty-text' : 'ce-clean-text'">{{ dirty ? '已修改' : '未修改' }}</span>
          <span class="ce-sep">|</span>
          <span>路径: config.yaml</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.config-editor-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.config-editor-dialog {
  width: 820px;
  max-width: 94vw;
  height: 580px;
  max-height: 88vh;
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 8px 40px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 标题栏 */
.ce-header {
  display: flex;
  align-items: center;
  padding: 0 16px;
  height: 44px;
  background: #2b5ea7;
  color: #fff;
  flex-shrink: 0;
  gap: 12px;
}
.ce-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  font-weight: 600;
}
.ce-icon { font-size: 16px; }
.ce-dirty {
  font-size: 12px;
  color: #ffd166;
  font-weight: normal;
}
.ce-actions {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 6px;
}
.ce-btn {
  padding: 4px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  border: none;
  transition: background 0.15s, opacity 0.15s;
  font-family: inherit;
}
.ce-btn:disabled { opacity: 0.4; cursor: not-allowed; }
.ce-btn-secondary {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
  border: 1px solid rgba(255, 255, 255, 0.25);
}
.ce-btn-secondary:hover:not(:disabled) { background: rgba(255, 255, 255, 0.25); }
.ce-btn-save {
  background: #4caf50;
  color: #fff;
}
.ce-btn-save:hover:not(:disabled) { background: #43a047; }
.ce-btn-close {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
  width: 28px;
  padding: 4px;
  text-align: center;
}
.ce-btn-close:hover { background: rgba(255, 255, 255, 0.2); }

/* 加载状态 */
.ce-loading {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #888;
  font-size: 14px;
  gap: 8px;
}
.ce-loading-icon { font-size: 20px; }

/* 编辑器主体 */
.ce-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 错误提示 */
.ce-error {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: #fff3cd;
  color: #856404;
  font-size: 12px;
  flex-shrink: 0;
}
.ce-error-close {
  margin-left: auto;
  background: none;
  border: none;
  cursor: pointer;
  color: #856404;
  font-size: 14px;
}

/* 编辑器容器 */
.ce-editor-wrap {
  flex: 1;
  display: flex;
  overflow: hidden;
  border-top: 1px solid #ddd;
}

/* 行号 */
.ce-linenumbers {
  width: 48px;
  background: #f5f7fa;
  border-right: 1px solid #ddd;
  overflow: hidden;
  padding-top: 8px;
  flex-shrink: 0;
  user-select: none;
}
.ce-line-no {
  text-align: right;
  padding-right: 10px;
  font-size: 12px;
  color: #999;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

/* 文本编辑区域 */
.ce-textarea {
  flex: 1;
  border: none;
  outline: none;
  resize: none;
  padding: 8px 12px;
  font-size: 13px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  color: #333;
  background: #fff;
  line-height: 18px;
  tab-size: 2;
  white-space: pre;
  overflow: auto;
}
.ce-textarea::selection {
  background: #b3d7ff;
}
.ce-textarea::placeholder {
  color: #ccc;
}

/* 状态栏 */
.ce-statusbar {
  display: flex;
  align-items: center;
  padding: 4px 12px;
  height: 24px;
  background: #f0f2f5;
  border-top: 1px solid #ddd;
  font-size: 11px;
  color: #888;
  gap: 8px;
  flex-shrink: 0;
}
.ce-sep { color: #ccc; }
.ce-dirty-text { color: #e65100; font-weight: 500; }
.ce-clean-text { color: #4caf50; }
</style>
