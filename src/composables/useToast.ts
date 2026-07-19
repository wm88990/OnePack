import { ref } from 'vue'

export interface ToastItem {
  id: number
  message: string
  type: 'success' | 'error' | 'info'
}

let toastId = 0

export function useToast() {
  const toasts = ref<ToastItem[]>([])

  function addToast(message: string, type: ToastItem['type'] = 'info', duration = 3500) {
    const id = ++toastId
    toasts.value.push({ id, message, type })
    setTimeout(() => {
      removeToast(id)
    }, duration)
  }

  function removeToast(id: number) {
    const idx = toasts.value.findIndex(t => t.id === id)
    if (idx !== -1) {
      toasts.value.splice(idx, 1)
    }
  }

  return { toasts, addToast, removeToast }
}
