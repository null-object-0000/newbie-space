<template>
  <div class="tools-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />

    <main class="tools-main">
      <section class="tools-header">
        <h1>工具中心</h1>
        <p>常用的小工具尽量本地运行，降低依赖，也避免把文件上传到第三方服务。</p>
      </section>

      <!-- 搜索 -->
      <section class="search-bar">
        <Search :size="16" class="search-icon" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="搜索工具名称、描述、标签…"
          class="search-input"
        />
        <button v-if="searchQuery" class="search-clear" @click="searchQuery = ''">
          <X :size="14" />
        </button>
      </section>

      <!-- 流转筛选 -->
      <section class="filter-bar">
        <div class="filter-group">
          <span class="filter-label">我有</span>
          <div class="segmented">
            <button :class="{ active: inputFilter === null }" @click="inputFilter = null">全部</button>
            <button
              v-for="dt in dataTypes"
              :key="dt.key"
              :class="{ active: inputFilter === dt.key }"
              @click="inputFilter = dt.key"
            >{{ dt.label }}</button>
          </div>
        </div>

        <ArrowRight :size="16" class="filter-arrow" />

        <div class="filter-group">
          <span class="filter-label">想要</span>
          <div class="segmented">
            <button :class="{ active: outputFilter === null }" @click="outputFilter = null">全部</button>
            <button
              v-for="dt in dataTypes"
              :key="dt.key"
              :class="{ active: outputFilter === dt.key }"
              @click="outputFilter = dt.key"
            >{{ dt.label }}</button>
          </div>
        </div>

        <span v-if="inputFilter || outputFilter" class="filter-count">
          匹配 {{ filteredTools.length }} 个工具
          <button class="clear-filter" @click="clearFilters">清除</button>
        </span>
      </section>

      <!-- 卡片列表（按类型分组） -->
      <div v-if="!filteredTools.length" class="empty-tip">
        <p>没有匹配的工具</p>
        <button class="clear-filter" @click="clearFilters">清除筛选</button>
      </div>

      <section v-for="group in groupedTools" :key="group.category" class="tool-group">
        <div class="group-title">
          <span class="group-dot"></span>
          <h3>{{ group.category }}</h3>
          <span class="group-count">{{ group.tools.length }}</span>
        </div>
        <div class="tools-grid">
          <router-link v-for="tool in group.tools" :key="tool.id" class="tool-card" :to="tool.path">
            <div class="card-head">
              <div class="tool-icon" :style="iconStyle(tool.color)">
                <component :is="tool.icon" :size="18" />
              </div>
              <h2>{{ tool.name }}</h2>
            </div>
            <p class="card-desc">{{ tool.desc }}</p>
            <div class="tool-tags">
              <span v-if="tool.inputType.length" class="io-badge input-badge">
                <ArrowDownRight :size="10" />{{ typeLabel(tool.inputType) }}
              </span>
              <span class="io-badge output-badge">
                <ArrowUpRight :size="10" />{{ typeLabel(tool.outputType) }}
              </span>
            </div>
          </router-link>
        </div>
      </section>
    </main>

    <AppFooter />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { ArrowDownRight, ArrowRight, ArrowUpRight, Search, X } from 'lucide-vue-next'
import { useTheme } from '@/composables/useTheme'
import { tools, type DataType } from '@/data/tools'

const { isDark } = useTheme()

const dataTypes: { key: DataType; label: string }[] = [
  { key: 'text', label: '文本' },
  { key: 'image', label: '图片' }
]

const inputFilter = ref<DataType | null>(null)
const outputFilter = ref<DataType | null>(null)
const searchQuery = ref('')

const filteredTools = computed(() => {
  const q = searchQuery.value.trim().toLowerCase()
  return tools.filter(t => {
    if (inputFilter.value && !t.inputType.includes(inputFilter.value)) return false
    if (outputFilter.value && t.outputType !== outputFilter.value) return false
    if (q) {
      const matchName = t.name.toLowerCase().includes(q)
      const matchDesc = t.desc.toLowerCase().includes(q)
      const matchTags = t.tags.some(tag => tag.toLowerCase().includes(q))
      if (!matchName && !matchDesc && !matchTags) return false
    }
    return true
  })
})

const groupedTools = computed(() => {
  const groups: Record<string, typeof tools> = {}
  for (const t of filteredTools.value) {
    const cat = t.tags[0] || '其他'
    if (!groups[cat]) groups[cat] = []
    groups[cat].push(t)
  }
  return Object.entries(groups).map(([category, tools]) => ({ category, tools }))
})

function clearFilters() {
  inputFilter.value = null
  outputFilter.value = null
  searchQuery.value = ''
}

function typeLabel(types: DataType | DataType[]): string {
  const arr = Array.isArray(types) ? types : [types]
  return arr.map(t => t === 'text' ? '文本' : '图片').join('、')
}

const DEFAULT_ICON_COLOR = '#10b981'

function iconStyle(color?: string) {
  const c = color || DEFAULT_ICON_COLOR
  return {
    color: c,
    background: `color-mix(in srgb, ${c} 14%, transparent)`
  }
}
</script>

<style scoped>
.tools-page {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--bg-main);
  color: var(--text-primary);
}

.tools-main {
  flex: 1;
  width: 100%;
  max-width: 72rem;
  margin: 0 auto;
  padding: 5rem 1rem 2.5rem;
}
@media (min-width: 640px) { .tools-main { padding: 5.5rem 1.5rem 3rem; } }
@media (min-width: 1024px) { .tools-main { padding: 5.5rem 2rem 3rem; } }

.tools-header {
  margin-bottom: 0.75rem;
}
.tools-header h1 {
  font-size: 1.25rem; line-height: 1.2; margin: 0 0 0.25rem;
}
.tools-header p {
  color: var(--text-secondary); font-size: 0.75rem; margin: 0;
}

/* ====== 搜索条 ====== */
.search-bar {
  display: flex; align-items: center; gap: 0.5rem;
  margin-bottom: 0.625rem;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  background: var(--bg-surface);
  transition: border-color 0.15s;
}
.search-bar:focus-within {
  border-color: var(--brand-500);
}
.search-icon {
  color: var(--text-secondary); flex-shrink: 0;
}
.search-input {
  flex: 1; min-width: 0;
  border: 0; outline: 0; background: transparent;
  color: var(--text-primary);
  font-size: 0.8125rem; font-family: inherit;
}
.search-input::placeholder {
  color: var(--text-secondary); opacity: 0.6;
}
.search-clear {
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0; width: 1.5rem; height: 1.5rem;
  border: 0; border-radius: 0.375rem;
  background: transparent; color: var(--text-secondary);
  cursor: pointer;
}
.search-clear:hover { color: var(--text-primary); background: var(--bg-elevated); }

/* ====== 筛选条 ====== */
.filter-bar {
  display: flex; align-items: center; flex-wrap: wrap;
  gap: 0.5rem; margin-bottom: 1rem;
  padding: 0.625rem 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  background: var(--bg-surface);
}

.filter-group {
  display: flex; align-items: center; gap: 0.375rem;
}
.filter-label {
  font-size: 0.6875rem; font-weight: 700;
  color: var(--text-secondary);
  white-space: nowrap;
}

.segmented {
  display: flex;
  padding: 0.1875rem; border-radius: 0.375rem;
  background: var(--bg-elevated);
}
.segmented button {
  min-height: 1.75rem; padding: 0 0.5rem;
  border: 0; border-radius: 0.25rem;
  background: transparent; color: var(--text-secondary);
  font-size: 0.6875rem; font-weight: 600; cursor: pointer;
  transition: background 0.15s, color 0.15s;
  white-space: nowrap;
}
.segmented button.active {
  background: var(--bg-surface); color: var(--text-primary); box-shadow: var(--shadow-1);
}

.filter-arrow {
  color: var(--text-secondary); flex-shrink: 0;
}

.filter-count {
  font-size: 0.6875rem; color: var(--text-secondary);
  display: flex; align-items: center; gap: 0.375rem;
}
.clear-filter {
  border: 0; padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  background: var(--bg-elevated); color: var(--text-secondary);
  font-size: 0.625rem; cursor: pointer;
}
.clear-filter:hover { color: var(--brand-500); }

/* ====== 分组 ====== */
.tool-group {
  margin-bottom: 1.25rem;
}
.group-title {
  display: flex; align-items: center; gap: 0.5rem;
  margin-bottom: 0.5rem; padding-left: 0.125rem;
}
.group-dot {
  width: 0.5rem; height: 0.5rem; border-radius: 50%; flex-shrink: 0;
  background: var(--brand-500);
}
.group-title h3 {
  font-size: 0.8125rem; font-weight: 700; margin: 0;
}
.group-count {
  font-size: 0.6875rem; color: var(--text-secondary);
  padding: 0.0625rem 0.375rem; border-radius: 999px;
  background: var(--bg-elevated);
}

/* ====== 工具卡片 ====== */
.tools-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 0.5rem;
}
@media (min-width: 640px) { .tools-grid { grid-template-columns: repeat(2, 1fr); } }
@media (min-width: 1024px) { .tools-grid { grid-template-columns: repeat(3, 1fr); } }

.tool-card {
  display: flex; flex-direction: column; gap: 0.375rem;
  padding: 0.75rem;
  border: 1px solid var(--border-color); border-radius: 0.75rem;
  background: var(--bg-surface);
  color: inherit; text-decoration: none;
  transition: transform 0.15s, border-color 0.15s, box-shadow 0.15s;
}
.tool-card:hover {
  color: inherit; transform: translateY(-2px);
  border-color: var(--brand-500); box-shadow: var(--shadow-2);
}

.card-head {
  display: flex; align-items: center; gap: 0.5rem;
}
.tool-icon {
  width: 1.75rem; height: 1.75rem; flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
  border-radius: 0.4375rem;
}
.card-head h2 {
  font-size: 0.875rem; margin: 0;
}

.card-desc {
  color: var(--text-secondary); font-size: 0.75rem;
  line-height: 1.4; margin: 0;
}

.tool-tags {
  display: flex; flex-wrap: wrap; align-items: center; gap: 0.25rem;
}

.io-badge {
  display: inline-flex; align-items: center; gap: 0.125rem;
  padding: 0.1875rem 0.375rem; border-radius: 0.25rem;
  font-size: 0.625rem; font-weight: 700;
  border: 1px solid;
}
.input-badge {
  color: #b45309;
  background: color-mix(in srgb, #f59e0b 12%, transparent);
  border-color: color-mix(in srgb, #f59e0b 25%, transparent);
}
.output-badge {
  color: #047857;
  background: color-mix(in srgb, #10b981 12%, transparent);
  border-color: color-mix(in srgb, #10b981 25%, transparent);
}

.tag-chip {
  padding: 0.125rem 0.3125rem; border-radius: 0.25rem;
  color: var(--text-secondary); background: var(--bg-elevated);
  font-size: 0.625rem;
}

.empty-tip {
  grid-column: 1 / -1;
  text-align: center; padding: 2rem 1rem;
  color: var(--text-secondary); font-size: 0.875rem;
  display: flex; flex-direction: column; align-items: center; gap: 0.5rem;
}
.empty-tip p { margin: 0; }
</style>
