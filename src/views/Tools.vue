<template>
  <div class="tools-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />

    <main class="tools-main">
      <!-- 标题 + 搜索横向布局 -->
      <section class="tools-header">
        <div class="header-text">
          <h1>工具中心</h1>
          <p>精选实用工具，提升效率，解决开发与日常工作中的小问题。</p>
        </div>
        <div class="search-bar">
          <Search :size="16" class="search-icon" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索工具名称、功能或关键词，如：二维码、图片、哈希..."
            class="search-input"
          />
          <button v-if="searchQuery" class="search-clear" @click="searchQuery = ''">
            <X :size="14" />
          </button>
        </div>
      </section>

      <!-- 分类 Tab -->
      <nav class="category-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          :class="{ active: activeTab === tab.key }"
          @click="activeTab = tab.key"
        >
          {{ tab.label }}
          <span class="tab-count">{{ tab.count }}</span>
        </button>
      </nav>

      <!-- 空状态 -->
      <div v-if="!filteredTools.length" class="empty-tip">
        <p>没有匹配的工具</p>
        <button class="clear-filter" @click="clearFilters">清除筛选</button>
      </div>

      <!-- 三列卡片网格 -->
      <div class="tools-grid">
        <router-link v-for="tool in filteredTools" :key="tool.id" class="tool-card" :to="tool.path">
          <ArrowUpRight :size="16" class="card-arrow" />
          <div class="card-head">
            <div class="tool-icon" :style="iconStyle(tool.color)">
              <component :is="tool.icon" :size="18" />
            </div>
            <h2>{{ tool.name }}</h2>
          </div>
          <p class="card-desc">{{ tool.desc }}</p>
          <div class="tool-tags">
            <span v-if="tool.inputType.length" class="io-badge">
              <ArrowDownRight :size="10" />输入 {{ typeLabel(tool.inputType) }}
            </span>
            <span class="io-badge">
              <ArrowUpRight :size="10" />输出 {{ tool.outputLabel || typeLabel(tool.outputType) }}
            </span>
          </div>
        </router-link>
      </div>

      <!-- 底部提示 -->
      <p class="tools-footer-tip">工具持续增加中，欢迎提交建议或贡献工具</p>
    </main>

    <AppFooter />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { ArrowDownRight, ArrowUpRight, Search, X } from 'lucide-vue-next'
import { useTheme } from '@/composables/useTheme'
import { tools, type DataType } from '@/data/tools'

const { isDark } = useTheme()

const activeTab = ref<string>('all')
const searchQuery = ref('')

// 分类 Tab（搜索时计数也实时变动）
const tabs = computed(() => {
  // 先按搜索词筛选
  const q = searchQuery.value.trim().toLowerCase()
  let list = tools
  if (q) {
    list = list.filter(t =>
      t.name.toLowerCase().includes(q) ||
      t.desc.toLowerCase().includes(q) ||
      t.tags.some(tag => tag.toLowerCase().includes(q))
    )
  }
  const cats: Record<string, number> = {}
  for (const t of list) {
    const c = t.category || t.tags[0] || '其他'
    cats[c] = (cats[c] || 0) + 1
  }
  return [
    { key: 'all', label: '全部', count: list.length },
    { key: '图片处理', label: '图片处理', count: cats['图片处理'] || 0 },
    { key: '生成器', label: '生成器', count: cats['生成器'] || 0 },
    { key: '文本处理', label: '文本处理', count: cats['文本处理'] || 0 },
    { key: '编码转换', label: '编码转换', count: cats['编码转换'] || 0 },
    { key: '网络开发', label: '网络开发', count: cats['网络开发'] || 0 },
  ]
})

const filteredTools = computed(() => {
  let list = tools
  // Tab 筛选
  if (activeTab.value !== 'all') {
    list = list.filter(t => (t.category || t.tags[0]) === activeTab.value)
  }
  // 搜索
  const q = searchQuery.value.trim().toLowerCase()
  if (q) {
    list = list.filter(t =>
      t.name.toLowerCase().includes(q) ||
      t.desc.toLowerCase().includes(q) ||
      t.tags.some(tag => tag.toLowerCase().includes(q))
    )
  }
  return list
})

function clearFilters() {
  activeTab.value = 'all'
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
  background: var(--page-bg);
  color: var(--text-primary);
}

.tools-main {
  flex: 1;
  width: 100%;
  max-width: 1180px;
  margin: 0 auto;
  padding: 5rem 1rem 2.5rem;
}
@media (min-width: 640px) { .tools-main { padding: 5.5rem 1.5rem 3rem; } }
@media (min-width: 1024px) { .tools-main { padding: 5.5rem 2rem 3rem; } }

/* ====== 标题 + 搜索横向布局 ====== */
.tools-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1.5rem;
  margin-bottom: 1.25rem;
}
.header-text h1 {
  font-size: 1.5rem; line-height: 1.3; margin: 0 0 0.375rem;
}
.header-text p {
  color: var(--text-secondary); font-size: 0.875rem; margin: 0; line-height: 1.5;
}

@media (max-width: 767px) {
  .tools-header { flex-direction: column; align-items: stretch; gap: 0.75rem; }
}

/* ====== 搜索框 ====== */
.search-bar {
  display: flex; align-items: center; gap: 0.5rem;
  width: clamp(320px, 40vw, 620px);
  flex-shrink: 0;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  background: var(--bg-surface);
  transition: border-color 0.15s;
}
.search-bar:focus-within { border-color: var(--brand-500); }
@media (max-width: 767px) { .search-bar { width: 100%; } }

.search-icon { color: var(--text-secondary); flex-shrink: 0; }
.search-input {
  flex: 1; min-width: 0;
  border: 0; outline: 0; background: transparent;
  color: var(--text-primary);
  font-size: 0.875rem; font-family: inherit;
}
.search-input::placeholder { color: var(--text-muted); }
.search-clear {
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0; width: 1.5rem; height: 1.5rem;
  border: 0; border-radius: 0.375rem;
  background: transparent; color: var(--text-secondary);
  cursor: pointer;
}
.search-clear:hover { color: var(--text-primary); background: var(--bg-elevated); }

/* ====== 分类 Tab ====== */
.category-tabs {
  display: flex; gap: 0.25rem;
  padding: 0.25rem;
  margin-bottom: 1rem;
  border-radius: 0.75rem;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
}
.category-tabs button {
  display: inline-flex; align-items: center; gap: 0.375rem;
  min-height: 2.25rem; padding: 0 0.875rem;
  border: 0; border-radius: 0.5rem;
  background: transparent; color: var(--text-secondary);
  font-size: 0.875rem; font-weight: 600; cursor: pointer;
  transition: background 0.15s, color 0.15s;
  white-space: nowrap;
}
.category-tabs button:hover { color: var(--text-primary); }
.category-tabs button.active {
  background: var(--brand-500); color: #fff;
}
.tab-count {
  font-size: 0.875rem; font-weight: 700;
  padding: 0.0625rem 0.375rem; border-radius: 999px;
  background: color-mix(in srgb, currentColor 12%, transparent);
}

/* ====== 卡片网格 ====== */
.tools-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 1rem;
}
@media (min-width: 640px) { .tools-grid { grid-template-columns: repeat(2, 1fr); } }
@media (min-width: 1024px) { .tools-grid { grid-template-columns: repeat(3, 1fr); } }

/* ====== 工具卡片 ====== */
.tool-card {
  position: relative;
  display: flex; flex-direction: column; gap: 0.5rem;
  padding: 1.25rem;
  min-height: 140px;
  border: 1px solid var(--border-color); border-radius: 1rem;
  background: var(--bg-surface);
  color: inherit; text-decoration: none;
  transition: transform 0.15s, border-color 0.15s, box-shadow 0.15s;
}
.tool-card:hover {
  color: inherit; transform: translateY(-2px);
  border-color: var(--brand-500);
  box-shadow: 0 8px 24px rgba(15, 23, 42, 0.06);
}

/* 右上角箭头 */
.card-arrow {
  position: absolute;
  top: 1.25rem; right: 1.25rem;
  color: var(--text-muted);
  opacity: 0;
  transition: opacity 0.15s, transform 0.15s;
}
.tool-card:hover .card-arrow { opacity: 1; transform: translate(2px, -2px); }

/* 卡片头部 */
.card-head {
  display: flex; align-items: center; gap: 0.625rem;
}
.tool-icon {
  width: 2rem; height: 2rem; flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
  border-radius: 0.5rem;
}
.card-head h2 {
  font-size: 1rem; margin: 0;
}

/* 描述 */
.card-desc {
  color: var(--text-secondary); font-size: 0.875rem;
  line-height: 1.5; margin: 0;
  flex: 1;
}

/* 输入/输出标签 */
.tool-tags {
  display: flex; flex-wrap: wrap; align-items: center; gap: 0.375rem;
  margin-top: auto;
}
.io-badge {
  display: inline-flex; align-items: center; gap: 0.125rem;
  padding: 0.125rem 0.375rem; border-radius: 0.25rem;
  font-size: 0.6875rem; font-weight: 600;
  background: var(--badge-bg);
  color: var(--badge-text);
}

/* ====== 空状态 ====== */
.empty-tip {
  text-align: center; padding: 3rem 1rem;
  color: var(--text-secondary); font-size: 0.875rem;
  display: flex; flex-direction: column; align-items: center; gap: 0.5rem;
}
.empty-tip p { margin: 0; }
.clear-filter {
  border: 0; padding: 0.25rem 0.75rem;
  border-radius: 0.375rem;
  background: var(--bg-elevated); color: var(--text-secondary);
  font-size: 0.875rem; cursor: pointer;
}
.clear-filter:hover { color: var(--brand-500); }

/* ====== 底部提示 ====== */
.tools-footer-tip {
  text-align: center;
  color: var(--text-muted);
  font-size: 0.8125rem;
  padding: 2.5rem 0 1rem;
}
</style>
