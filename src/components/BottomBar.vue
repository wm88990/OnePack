<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  isInstalling: boolean
  selectedCount: number
  progressCurrent: number
  progressTotal: number
  progressName: string
  progressPercent: number
}>()

const emit = defineEmits<{
  start: []
  stop: []
}>()

const showProgress = computed(() => props.isInstalling)
const progressText = computed(() => {
  if (!props.isInstalling) return ''
  return `第 ${props.progressCurrent}/${props.progressTotal} 个 · ${props.progressName}`
})

const installDisabled = computed(() => props.isInstalling || props.selectedCount === 0)

function handleStart() {
  emit('start')
}

function handleStop() {
  emit('stop')
}
</script>

<template>
  <div :class="['bottombar', { installing: isInstalling }]">
    <!-- 左侧信息 -->
    <div class="bar-info">
      <span v-if="!isInstalling">已选 <b>{{ selectedCount }}</b> 款</span>
      <span v-else class="installing-label">
        <span class="dot-loading"><span></span><span></span><span></span></span>
        安装中
      </span>
    </div>

    <!-- 进度条 -->
    <div :class="['progress-section', { show: showProgress }]">
      <div class="progress-wrap">
        <div class="progress-bar" :style="{ width: progressPercent + '%' }"></div>
      </div>
      <div class="progress-text">{{ progressText }}</div>
    </div>

    <!-- 操作按钮 -->
    <div class="bar-actions">
      <button
        v-if="!isInstalling"
        class="btn-start"
        :disabled="installDisabled"
        @click="handleStart"
      >▶ 开始安装</button>
      <button v-else class="btn-stop" @click="handleStop">■ 停止</button>
    </div>
  </div>
</template>

<style scoped>
.bottombar {
  height: 56px;
  background: linear-gradient(135deg, #2b5ea7, #1e4f94);
  display: flex;
  align-items: center;
  padding: 0 16px;
  gap: 14px;
  flex-shrink: 0;
  color: #fff;
}

/* 左侧信息 */
.bar-info {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.85);
  min-width: 80px;
}
.bar-info b {
  color: #fff;
  font-size: 16px;
}

.installing-label {
  display: flex;
  align-items: center;
  gap: 6px;
}

/* 动画指示器 */
@keyframes pulse-dot {
  0%, 80%, 100% { opacity: 0.3; }
  40% { opacity: 1; }
}
.dot-loading span {
  width: 5px;
  height: 5px;
  background: #fff;
  border-radius: 50%;
  display: inline-block;
  animation: pulse-dot 1.2s infinite;
}
.dot-loading span:nth-child(2) { animation-delay: 0.2s; }
.dot-loading span:nth-child(3) { animation-delay: 0.4s; }

/* 进度条区域 */
.progress-section {
  flex: 1;
  display: none;
  align-items: center;
  gap: 10px;
}
.progress-section.show {
  display: flex;
}

.progress-wrap {
  flex: 1;
  height: 8px;
  background: rgba(255, 255, 255, 0.15);
  border-radius: 4px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, #4fc3f7, #29b6f6);
  border-radius: 4px;
  width: 0%;
  transition: width 0.4s ease;
}

.progress-text {
  color: rgba(255, 255, 255, 0.8);
  font-size: 11px;
  white-space: nowrap;
  min-width: 100px;
  text-align: right;
}

/* 操作按钮 */
.bar-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.btn-start {
  background: #fff;
  color: #2b5ea7;
  border: none;
  padding: 7px 26px;
  border-radius: 5px;
  font-size: 13px;
  font-weight: bold;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.15s;
}
.btn-start:hover { background: #e8f0fe; }
.btn-start:disabled {
  background: rgba(255, 255, 255, 0.15);
  color: rgba(255, 255, 255, 0.5);
  cursor: not-allowed;
}

.btn-stop {
  background: #e74c3c;
  color: #fff;
  border: none;
  padding: 7px 18px;
  border-radius: 5px;
  font-size: 13px;
  cursor: pointer;
  font-family: inherit;
  transition: background 0.15s;
}
.btn-stop:hover { background: #c0392b; }
</style>
