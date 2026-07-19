<script setup lang="ts">
import type { ToastItem } from '../composables/useToast'

defineProps<{
  toasts: ToastItem[]
}>()

const emit = defineEmits<{
  remove: [id: number]
}>()

function typeClass(type: ToastItem['type']) {
  return {
    success: 'toast-success',
    error: 'toast-error',
    info: 'toast-info',
  }[type]
}

function typeIcon(type: ToastItem['type']) {
  return {
    success: '✓',
    error: '✕',
    info: 'ℹ',
  }[type]
}
</script>

<template>
  <div class="toast-container">
    <TransitionGroup name="toast">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        :class="['toast-item', typeClass(toast.type)]"
        @click="emit('remove', toast.id)"
      >
        <span class="toast-icon">{{ typeIcon(toast.type) }}</span>
        <span class="toast-msg">{{ toast.message }}</span>
        <span class="toast-close">×</span>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-container {
  position: fixed;
  top: 52px;
  right: 16px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}

.toast-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border-radius: 6px;
  font-size: 13px;
  color: #fff;
  cursor: pointer;
  pointer-events: auto;
  box-shadow: 0 3px 10px rgba(0, 0, 0, 0.18);
  max-width: 360px;
  animation: toast-in 0.25s ease;
}

.toast-item:hover { filter: brightness(1.08); }

.toast-success { background: #27ae60; }
.toast-error { background: #e74c3c; }
.toast-info { background: #2b5ea7; }

.toast-icon {
  font-size: 15px;
  font-weight: bold;
  flex-shrink: 0;
  width: 18px;
  text-align: center;
}

.toast-msg {
  flex: 1;
  line-height: 1.4;
}

.toast-close {
  font-size: 16px;
  opacity: 0.6;
  flex-shrink: 0;
}
.toast-close:hover { opacity: 1; }

/* TransitionGroup */
.toast-enter-active { animation: toast-in 0.25s ease; }
.toast-leave-active { animation: toast-out 0.2s ease; }

@keyframes toast-in {
  from { opacity: 0; transform: translateX(40px); }
  to { opacity: 1; transform: translateX(0); }
}
@keyframes toast-out {
  from { opacity: 1; transform: translateX(0); }
  to { opacity: 0; transform: translateX(40px); }
}
</style>
