<script setup lang="ts">
import { ref, computed } from 'vue'
import type { CategoryItem, PackageItem } from '../types'

const props = defineProps<{
  categories: CategoryItem[]
  packages: PackageItem[]
  current: string
  selectedCount: number
}>()

const emit = defineEmits<{
  'select-category': [id: string]
}>()

// 每个分类的折叠状态
const collapsedMap = ref<Record<string, boolean>>({})

function toggleCollapse(id: string) {
  collapsedMap.value[id] = !collapsedMap.value[id]
}

function isCollapsed(id: string) {
  return !!collapsedMap.value[id]
}

// 每个分类下的软件数量
const categoryCount = computed(() => {
  const map: Record<string, number> = {}
  for (const pkg of props.packages) {
    map[pkg.category] = (map[pkg.category] || 0) + 1
  }
  return map
})

// 已选总数
const totalPackages = computed(() => props.packages.length)

function selectCategory(id: string) {
  emit('select-category', id)
}
</script>

<template>
  <aside class="sidebar">
    <!-- 全部分类 -->
    <div
      :class="['sidebar-item', 'sidebar-all', { active: current === 'all' }]"
      @click="selectCategory('all')"
    >
      <span class="si-icon">📋</span>
      <span class="si-label">全部软件</span>
      <span class="si-badge">{{ totalPackages }}</span>
    </div>

    <div class="sidebar-divider"></div>

    <!-- 分类列表 -->
    <div v-for="cat in categories" :key="cat.id" class="sidebar-group">
      <div class="sg-header" @click="toggleCollapse(cat.id)">
        <span
          :class="['sidebar-item', { active: current === cat.id }]"
          @click.stop="selectCategory(cat.id)"
        >
          <span class="si-icon">{{ cat.icon }}</span>
          <span class="si-label">{{ cat.name }}</span>
        </span>
        <span class="sg-count">{{ categoryCount[cat.id] || 0 }}</span>
        <span :class="['sg-arrow', { collapsed: isCollapsed(cat.id) }]">▾</span>
      </div>
    </div>

    <!-- 底部统计 -->
    <div class="sidebar-footer">
      <div class="sf-total">共 <b>{{ totalPackages }}</b> 款软件</div>
      <div class="sf-selected">已选 <b>{{ selectedCount }}</b> 款</div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 200px;
  background: #fff;
  border-right: 1px solid #e0e0e0;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  overflow-y: auto;
}

/* 分类项 */
.sidebar-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 14px;
  cursor: pointer;
  transition: background 0.12s;
  font-size: 13px;
  color: #444;
  border-radius: 0 4px 4px 0;
  margin-right: 6px;
}
.sidebar-item:hover { background: #e8f0fe; }
.sidebar-item.active {
  background: #dde8f8;
  color: #2b5ea7;
  font-weight: 600;
}
.sidebar-all { margin-top: 4px; }

.si-icon { font-size: 16px; flex-shrink: 0; width: 20px; text-align: center; }
.si-label { flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.si-badge {
  font-size: 10px;
  background: #e0e0e0;
  color: #666;
  padding: 1px 7px;
  border-radius: 8px;
  flex-shrink: 0;
}
.sidebar-item.active .si-badge {
  background: #2b5ea7;
  color: #fff;
}

/* 分隔线 */
.sidebar-divider {
  height: 1px;
  background: #e8e8e8;
  margin: 6px 14px;
}

/* 分组头 */
.sidebar-group {}
.sg-header {
  display: flex;
  align-items: center;
  padding-right: 10px;
}
.sg-header > .sidebar-item {
  flex: 1;
  min-width: 0;
}
.sg-count {
  font-size: 10px;
  color: #999;
  flex-shrink: 0;
  margin-right: 2px;
}
.sg-arrow {
  font-size: 11px;
  color: #aaa;
  flex-shrink: 0;
  transition: transform 0.2s;
  width: 14px;
  text-align: center;
}
.sg-arrow.collapsed { transform: rotate(-90deg); }

/* 底部统计 */
.sidebar-footer {
  margin-top: auto;
  padding: 12px 14px;
  border-top: 1px solid #e8e8e8;
  font-size: 12px;
  color: #888;
  line-height: 1.8;
}
.sf-total, .sf-selected { display: flex; align-items: center; gap: 4px; }
.sf-total b, .sf-selected b { color: #2b5ea7; font-size: 14px; }
</style>
