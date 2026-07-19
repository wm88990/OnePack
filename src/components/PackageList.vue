<script setup lang="ts">
import { ref, computed } from 'vue'
import type { PackageItem, CategoryItem } from '../types'

const props = defineProps<{
  packages: PackageItem[]
  categories: CategoryItem[]
  selectedIds: Set<string>
  installedIds?: Set<string>
  checkingStatus?: boolean
}>()

const emit = defineEmits<{
  'toggle-package': [id: string, checked: boolean]
  'toggle-check-all': [checked: boolean]
  'toggle-enabled': [id: string, enabled: boolean]
  'edit-package': [pkg: PackageItem]
}>()

// 视图模式: list | compact | grid
const viewMode = ref<'list' | 'compact' | 'grid'>('list')

function setViewMode(mode: 'list' | 'compact' | 'grid') {
  viewMode.value = mode
}

function isChecked(id: string) {
  return props.selectedIds.has(id)
}

function togglePkg(id: string) {
  emit('toggle-package', id, !isChecked(id))
}

// 全选状态
const allChecked = computed(() => {
  return props.packages.length > 0 && props.packages.every(p => props.selectedIds.has(p.id))
})

function toggleCheckAll(e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  emit('toggle-check-all', checked)
}

// 是否已安装
function isInstalled(id: string) {
  return props.installedIds?.has(id) || false
}

function editPkg(pkg: PackageItem) {
  emit('edit-package', pkg)
}

// 获取分类图标
function getCategoryIcon(catId: string) {
  const cat = props.categories.find(c => c.id === catId)
  return cat?.icon || '📦'
}
</script>

<template>
  <div class="package-list">
    <!-- 视图切换按钮 -->
    <div class="view-switch">
      <button
        :class="['vs-btn', { active: viewMode === 'list' }]"
        title="标准列表"
        @click="setViewMode('list')"
      >☰</button>
      <button
        :class="['vs-btn', { active: viewMode === 'compact' }]"
        title="紧凑列表"
        @click="setViewMode('compact')"
      >≡</button>
      <button
        :class="['vs-btn', { active: viewMode === 'grid' }]"
        title="网格卡片"
        @click="setViewMode('grid')"
      >⊞</button>
    </div>

    <!-- 全选 -->
    <div class="check-all">
      <input type="checkbox" :checked="allChecked" @change="toggleCheckAll" />
      <label>全选当前列表 ({{ packages.length }} 款)</label>
    </div>

    <!-- 空状态 -->
    <div v-if="packages.length === 0" class="empty-state">
      <div class="empty-icon">📭</div>
      <div>当前分类下暂无软件</div>
    </div>

    <!-- ===== 标准列表视图 ===== -->
    <div v-else-if="viewMode === 'list'" class="view-list">
      <div
        v-for="(pkg, idx) in packages"
        :key="pkg.id"
        :class="['pkg-row', { even: idx % 2 === 1, disabled: !pkg.enabled }]"
        @click="togglePkg(pkg.id)"
      >
        <input type="checkbox" :checked="isChecked(pkg.id)" @click.stop />
        <div class="pkg-icon">{{ pkg.icon }}</div>
        <div class="pkg-info">
          <div class="pkg-name">{{ pkg.name }}</div>
          <div class="pkg-desc">{{ pkg.description }}</div>
        </div>
        <div class="pkg-meta">
          <span class="pkg-ver">{{ pkg.version }}</span>
          <span class="pkg-size">{{ pkg.size }}</span>
          <span v-if="isInstalled(pkg.id)" class="installed-badge" title="已安装">✓ 已安装</span>
        </div>
        <label class="enable-switch" @click.stop>
          <input type="checkbox" :checked="pkg.enabled" @change="emit('toggle-enabled', pkg.id, ($event.target as HTMLInputElement).checked)" />
          <span class="switch-slider"></span>
        </label>
        <button class="edit-btn" @click.stop="editPkg(pkg)" title="编辑配置">✏️</button>
      </div>
    </div>

    <!-- ===== 紧凑列表视图 ===== -->
    <div v-else-if="viewMode === 'compact'" class="view-compact">
      <div
        v-for="pkg in packages"
        :key="pkg.id"
        :class="['pkg-row-compact', { disabled: !pkg.enabled }]"
        @click="togglePkg(pkg.id)"
      >
        <input type="checkbox" :checked="isChecked(pkg.id)" @click.stop />
        <div class="pkg-icon-sm">{{ pkg.icon }}</div>
        <div class="pkg-name-compact">{{ pkg.name }}</div>
        <div class="pkg-tag">
          <span class="pkg-ver-tag">{{ pkg.version }}</span>
          <span class="pkg-size-tag">{{ pkg.size }}</span>
          <span v-if="isInstalled(pkg.id)" class="installed-badge-sm" title="已安装">✓</span>
        </div>
        <label class="enable-switch enable-switch-sm" @click.stop>
          <input type="checkbox" :checked="pkg.enabled" @change="emit('toggle-enabled', pkg.id, ($event.target as HTMLInputElement).checked)" />
          <span class="switch-slider"></span>
        </label>
        <button class="edit-btn-sm" @click.stop="editPkg(pkg)" title="编辑配置">✏️</button>
      </div>
    </div>

    <!-- ===== 网格卡片视图 ===== -->
    <div v-else class="view-grid">
      <div
        v-for="pkg in packages"
        :key="pkg.id"
        :class="['pkg-card', { checked: isChecked(pkg.id), disabled: !pkg.enabled }]"
        @click="togglePkg(pkg.id)"
      >
        <div class="card-icon">{{ pkg.icon }}</div>
        <div class="card-name">{{ pkg.name }}</div>
        <div class="card-ver">{{ pkg.version }} · {{ pkg.size }}</div>
        <div v-if="isInstalled(pkg.id)" class="card-installed" title="已安装">✓ 已安装</div>
        <div class="card-check">
          <input type="checkbox" :checked="isChecked(pkg.id)" @click.stop />
        </div>
        <label class="enable-switch enable-switch-sm" @click.stop>
          <input type="checkbox" :checked="pkg.enabled" @change="emit('toggle-enabled', pkg.id, ($event.target as HTMLInputElement).checked)" />
          <span class="switch-slider"></span>
        </label>
        <button class="card-edit-btn" @click.stop="editPkg(pkg)" title="编辑配置">✏️</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.package-list {
  flex: 1;
  overflow-y: auto;
  position: relative;
}

/* ===== 视图切换按钮 ===== */
.view-switch {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 2px;
  z-index: 10;
  background: #f0f0f0;
  border-radius: 4px;
  padding: 2px;
}
.vs-btn {
  width: 28px;
  height: 26px;
  border: none;
  background: transparent;
  border-radius: 3px;
  cursor: pointer;
  font-size: 15px;
  color: #888;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  font-family: inherit;
}
.vs-btn:hover { background: #e0e0e0; }
.vs-btn.active { background: #fff; color: #2b5ea7; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }

/* ===== 全选 ===== */
.check-all {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid #eee;
  font-size: 12px;
  color: #666;
  background: #fafafa;
}
.check-all input[type="checkbox"] { accent-color: #2b5ea7; }

/* ===== 空状态 ===== */
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: #bbb;
  font-size: 14px;
  padding: 40px;
}
.empty-icon { font-size: 40px; }

/* ===== 标准列表视图 ===== */
.view-list {}
.pkg-row {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  cursor: pointer;
  transition: background 0.12s;
  gap: 10px;
}
.pkg-row:hover { background: #e3ecf7; }
.pkg-row.even { background: #f7f9fc; }
.pkg-row.even:hover { background: #dde8f8; }
.pkg-row input[type="checkbox"] { width: 15px; height: 15px; accent-color: #2b5ea7; flex-shrink: 0; }
.pkg-icon {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  flex-shrink: 0;
  background: #e8f0fe;
}
.pkg-info { flex: 1; min-width: 0; }
.pkg-name { font-weight: 600; font-size: 13px; color: #222; }
.pkg-desc { font-size: 11px; color: #888; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin-top: 1px; }
.pkg-meta { display: flex; gap: 12px; flex-shrink: 0; font-size: 11px; color: #999; }
.pkg-ver { color: #2b5ea7; background: #e8f0fe; padding: 1px 7px; border-radius: 3px; }
.pkg-size { color: #888; }
.installed-badge {
  color: #389e0d;
  background: #f6ffed;
  border: 1px solid #b7eb8f;
  padding: 1px 7px;
  border-radius: 3px;
  font-size: 11px;
}
.edit-btn {
  width: 26px;
  height: 26px;
  border: 1px solid #e0e0e0;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s;
}
.pkg-row:hover .edit-btn { opacity: 1; }
.edit-btn:hover { background: #e3ecf7; border-color: #2b5ea7; }
.edit-btn-sm {
  width: 22px;
  height: 22px;
  border: none;
  background: transparent;
  border-radius: 3px;
  cursor: pointer;
  font-size: 11px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s;
}
.pkg-row-compact:hover .edit-btn-sm { opacity: 1; }
.edit-btn-sm:hover { background: #e3ecf7; }
.card-edit-btn {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 22px;
  height: 22px;
  border: none;
  background: transparent;
  border-radius: 3px;
  cursor: pointer;
  font-size: 11px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.15s;
}
.pkg-card:hover .card-edit-btn { opacity: 1; }
.card-edit-btn:hover { background: rgba(43,94,167,0.1); }

/* ===== 紧凑列表视图 ===== */
.view-compact {}
.pkg-row-compact {
  display: flex;
  align-items: center;
  padding: 5px 12px;
  cursor: pointer;
  transition: background 0.12s;
  gap: 8px;
}
.pkg-row-compact:hover { background: #e3ecf7; }
.pkg-row-compact input[type="checkbox"] { width: 14px; height: 14px; accent-color: #2b5ea7; flex-shrink: 0; }
.pkg-icon-sm {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  flex-shrink: 0;
}
.pkg-name-compact { flex: 1; font-size: 12.5px; color: #333; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.pkg-tag {
  display: none; /* hidden by default, show on hover */
  font-size: 10px;
  color: #888;
  gap: 6px;
  flex-shrink: 0;
}
.pkg-row-compact:hover .pkg-tag { display: flex; }
.pkg-ver-tag { color: #2b5ea7; background: #e8f0fe; padding: 1px 5px; border-radius: 2px; }
.pkg-size-tag { color: #888; }
.installed-badge-sm {
  color: #389e0d;
  background: #f6ffed;
  padding: 1px 4px;
  border-radius: 2px;
  font-size: 10px;
}

/* ===== 网格卡片视图 ===== */
.view-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
  gap: 10px;
  padding: 8px 12px;
}
.pkg-card {
  background: #f7f9fc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 14px 8px 10px;
  text-align: center;
  cursor: pointer;
  transition: all 0.15s;
  position: relative;
}
.pkg-card:hover {
  border-color: #2b5ea7;
  background: #e8f0fe;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(43, 94, 167, 0.12);
}
.pkg-card.checked {
  border-color: #2b5ea7;
  background: #dde8f8;
}
.card-icon {
  width: 42px;
  height: 42px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
  margin: 0 auto 6px;
  background: #fff;
  border: 1px solid #e8e8e8;
}
.card-name { font-size: 12px; font-weight: 600; color: #333; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin-bottom: 2px; }
.card-ver { font-size: 10px; color: #999; }
.card-check { margin-top: 6px; }
.card-check input[type="checkbox"] { accent-color: #2b5ea7; }
.card-installed {
 font-size: 10px;
 color: #389e0d;
 background: #f6ffed;
 border: 1px solid #b7eb8f;
 border-radius: 3px;
 padding: 1px 6px;
 margin-top: 4px;
}

/* ===== 禁用状态 ===== */
.pkg-row.disabled,
.pkg-row-compact.disabled,
.pkg-card.disabled {
  opacity: 0.45;
  pointer-events: none;
}
.pkg-row.disabled .enable-switch,
.pkg-row-compact.disabled .enable-switch,
.pkg-card.disabled .enable-switch {
  pointer-events: auto;
}
.pkg-card.disabled {
  filter: grayscale(0.3);
}

/* ===== 启用/禁用开关 ===== */
.enable-switch {
  position: relative;
  display: inline-block;
  width: 36px;
  height: 20px;
  flex-shrink: 0;
  cursor: pointer;
}
.enable-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}
.switch-slider {
  position: absolute;
  inset: 0;
  background: #ccc;
  border-radius: 20px;
  transition: background 0.2s;
}
.switch-slider::before {
  content: '';
  position: absolute;
  width: 16px;
  height: 16px;
  left: 2px;
  bottom: 2px;
  background: #fff;
  border-radius: 50%;
  transition: transform 0.2s;
}
.enable-switch input:checked + .switch-slider {
  background: #2b5ea7;
}
.enable-switch input:checked + .switch-slider::before {
  transform: translateX(16px);
}
.enable-switch-sm {
  width: 30px;
  height: 16px;
}
.enable-switch-sm .switch-slider::before {
  width: 12px;
  height: 12px;
}
.enable-switch-sm input:checked + .switch-slider::before {
  transform: translateX(14px);
}
</style>
