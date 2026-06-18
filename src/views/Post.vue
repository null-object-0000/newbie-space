<template>
  <div class="post-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <!-- 公共导航栏 -->
    <AppHeader />

    <!-- Main Content -->
    <main class="post-main">
      <div class="post-container animate-slide-up">
        <!-- 面包屑导航 -->
        <div class="post-breadcrumb">
          <button @click="goBack" class="back-button">
            <ArrowLeft :size="16" />
            <span>返回文章列表</span>
          </button>
        </div>

        <div class="post-grid">
          <!-- 左侧：主要内容 -->
          <div class="post-content-wrapper">
            <!-- 文章头部 -->
            <header class="post-header">
              <div class="post-tags" v-if="postMeta?.tags && postMeta.tags.length > 0">
                <span 
                  v-for="tag in postMeta.tags" 
                  :key="tag" 
                  class="post-tag"
                >
                  {{ tag }}
                </span>
              </div>
              <h1 class="post-title">{{ postMeta?.title || '加载中...' }}</h1>
              <div class="post-meta-info">
                <div class="meta-item">
                  <Calendar :size="16" />
                  <span>{{ postMeta?.date || '' }}</span>
                </div>
                <div class="meta-item" v-if="postMeta?.readTime">
                  <Clock :size="16" />
                  <span>{{ postMeta.readTime }} min read</span>
                </div>
                <div class="meta-item">
                  <User :size="16" />
                  <span>{{ postMeta?.author || 'Newbie Space' }}</span>
                </div>
              </div>
            </header>

            <!-- 文章封面（如果有） -->
            <div v-if="postMeta?.cover" class="post-cover">
              <img :src="postMeta.cover" :alt="postMeta.title" />
            </div>

            <!-- 文章正文内容 -->
            <div class="post-body">
              <MarkdownRenderer v-if="content" :content="content" />
              <div v-else class="loading">
                <p>加载中...</p>
              </div>
            </div>
            
            <!-- 文章底部 -->
            <div class="post-footer-actions">
              <div class="footer-meta">
                <span class="footer-text">最后编辑于 {{ formattedLastModified }}</span>
              </div>
            </div>

            <!-- 评论区 -->
            <GiscusComments />
          </div>

          <!-- 右侧：目录 (Sticky Sidebar) -->
          <aside class="post-sidebar">
            <div class="sidebar-sticky">
              <h3 class="sidebar-title">目录</h3>
              <ul class="toc-list">
                <!-- 目录项将从 Markdown 内容中提取 -->
                <li v-for="heading in tocItems" :key="heading.id" class="toc-item">
                  <a 
                    :href="`#${heading.id}`" 
                    class="toc-link"
                    :class="{ active: activeHeading === heading.id }"
                    @click.prevent="scrollToHeading(heading.id)"
                  >
                    {{ heading.text }}
                  </a>
                </li>
              </ul>
            </div>
          </aside>
        </div>
      </div>
    </main>

    <!-- 页脚 -->
    <AppFooter />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useTheme } from '@/composables/useTheme'
import { ArrowLeft, Calendar, Clock, User } from 'lucide-vue-next'
import MarkdownRenderer from '@/components/blog/MarkdownRenderer.vue'
import GiscusComments from '@/components/blog/GiscusComments.vue'
import { getPostBySlug } from '@/data/posts'
import AppHeader from '@/components/AppHeader.vue'
import AppFooter from '@/components/AppFooter.vue'

const route = useRoute()
const router = useRouter()
const { isDark } = useTheme()

const content = ref('')
const postMeta = computed(() => getPostBySlug(route.params.slug as string))
const tocItems = ref<Array<{ id: string; text: string; level: number }>>([])
const activeHeading = ref('')

// 格式化最后修改时间
const formattedLastModified = computed(() => {
  const meta = postMeta.value
  if (!meta) return ''
  
  // 优先使用 git 的最后修改时间
  if (meta.lastModified) {
    const date = new Date(meta.lastModified)
    return date.toLocaleDateString('zh-CN', { 
      year: 'numeric', 
      month: 'long', 
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  }
  
  // 回退到使用 frontmatter 的 date 字段
  return meta.date || ''
})

const goBack = () => {
  router.push('/posts')
}

// 提取目录
const extractToc = () => {
  const markdownBody = document.querySelector('.markdown-body')
  if (!markdownBody) return
  
  const headings = markdownBody.querySelectorAll('h1, h2, h3, h4, h5, h6')
  const items: Array<{ id: string; text: string; level: number }> = []
  const postTitle = postMeta.value?.title || ''
  
  headings.forEach((heading, index) => {
    const id = heading.id
    const text = heading.textContent || ''
    const level = parseInt(heading.tagName.charAt(1))
    
    // 跳过第一个 h1 标题（通常是文章标题，与页面标题重复）
    if (level === 1 && index === 0) {
      return
    }
    
    // 或者如果标题文本与文章标题相同，也跳过
    if (level === 1 && text === postTitle) {
      return
    }
    
    if (id && text) {
      items.push({
        id,
        text,
        level
      })
    }
  })
  
  tocItems.value = items
}

// 监听滚动，更新激活的标题
const handleScroll = () => {
  const headings = document.querySelectorAll('h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]')
  // 导航栏高度 + 额外偏移量
  const headerHeight = 64
  const offset = 20
  const scrollPosition = window.scrollY + headerHeight + offset
  
  for (let i = headings.length - 1; i >= 0; i--) {
    const heading = headings[i] as HTMLElement
    if (heading.offsetTop <= scrollPosition) {
      activeHeading.value = heading.id
      break
    }
  }
}

const scrollToHeading = (id: string) => {
  const element = document.getElementById(id)
  if (element) {
    // 获取导航栏高度（4rem = 64px）
    const headerHeight = 64
    const elementPosition = element.getBoundingClientRect().top + window.pageYOffset
    const offsetPosition = elementPosition - headerHeight - 20 // 额外 20px 的间距
    
    window.scrollTo({
      top: offsetPosition,
      behavior: 'smooth'
    })
    activeHeading.value = id
  }
}

// 动态加载文章内容
const loadPost = async (slug: string) => {
  try {
    // 使用 Vite 的 import.meta.glob 来加载 markdown 文件
    const modules = import.meta.glob('@/content/posts/*.md', { query: '?raw', import: 'default' })
    
    // 查找匹配的模块
    for (const [key, loader] of Object.entries(modules)) {
      if (key.includes(slug)) {
        const rawContent = await loader() as string
        content.value = rawContent
        
        // 等待 DOM 更新后提取目录
        await new Promise(resolve => setTimeout(resolve, 300))
        extractToc()
        return
      }
    }
    
    content.value = '# 文章未找到\n\n抱歉，您请求的文章不存在。'
  } catch (error) {
    console.error('Failed to load post:', error)
    content.value = '# 加载失败\n\n抱歉，文章加载失败，请稍后重试。'
  }
}

onMounted(() => {
  const slug = route.params.slug as string
  if (slug) {
    loadPost(slug)
  }
  window.addEventListener('scroll', handleScroll)
})

onBeforeUnmount(() => {
  window.removeEventListener('scroll', handleScroll)
})

watch(() => route.params.slug, (newSlug) => {
  if (newSlug) {
    loadPost(newSlug as string)
  }
})

// 监听 Markdown 渲染完成
watch(() => content.value, async () => {
  if (content.value) {
    await new Promise(resolve => setTimeout(resolve, 300))
    extractToc()
  }
})
</script>

<style scoped>
.post-page {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  font-family: 'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  transition: background-color 0.3s, color 0.3s;
  -webkit-font-smoothing: antialiased;
}

/* ========== 深色主题 ========== */
.dark-mode {
  background-color: var(--bg-main);
  color: var(--text-primary);
}

/* ========== 浅色主题 ========== */
.light-mode {
  background-color: var(--bg-main);
  color: var(--text-primary);
}

/* ========== 主内容区域 ========== */
.post-main {
  flex: 1;
  padding-top: 6rem;
  padding-bottom: 4rem;
  padding-left: 1rem;
  padding-right: 1rem;
}

@media (min-width: 640px) {
  .post-main {
    padding-left: 1.5rem;
    padding-right: 1.5rem;
  }
}

@media (min-width: 1024px) {
  .post-main {
    padding-left: 2rem;
    padding-right: 2rem;
  }
}

.post-container {
  max-width: 64rem;
  margin: 0 auto;
  width: 100%;
}

/* ========== 面包屑导航 ========== */
.post-breadcrumb {
  margin-bottom: 2rem;
}

.back-button {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
  color: var(--text-muted);
  background: none;
  border: none;
  cursor: pointer;
  padding: 0.5rem;
  border-radius: 0.25rem;
  transition: color 0.2s, background-color 0.2s;
}

.back-button:hover {
  color: var(--brand-500, #3b82f6);
  background-color: var(--bg-elevated);
}

/* ========== 文章网格布局 ========== */
.post-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 3rem;
}

@media (min-width: 1024px) {
  .post-grid {
    grid-template-columns: 3fr 1fr;
  }
}

/* ========== 文章内容区域 ========== */
.post-content-wrapper {
  min-width: 0;
}

/* ========== 文章头部 ========== */
.post-header {
  margin-bottom: 2rem;
  padding-bottom: 2rem;
  border-bottom: 1px solid var(--border-color);
}

.post-tags {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1rem;
  flex-wrap: wrap;
}

.post-tag {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--brand-600, #2563eb);
  background: rgba(59, 130, 246, 0.1);
  padding: 0.25rem 0.625rem;
  border-radius: 9999px;
}

.dark-mode .post-tag {
  color: var(--brand-400, #60a5fa);
  background: rgba(59, 130, 246, 0.2);
}

.post-title {
  font-size: 1.875rem;
  font-weight: 700;
  margin: 0 0 1rem;
  line-height: 1.2;
  color: var(--text-primary);
}

@media (min-width: 768px) {
  .post-title {
    font-size: 2.25rem;
  }
}

.post-meta-info {
  display: flex;
  align-items: center;
  gap: 1rem;
  font-size: 0.875rem;
  color: var(--text-muted);
  flex-wrap: wrap;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

/* ========== 文章封面 ========== */
.post-cover {
  margin-bottom: 2.5rem;
  border-radius: 1rem;
  overflow: hidden;
  aspect-ratio: 16 / 9;
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
  background: var(--bg-elevated);
}

.dark-mode .post-cover {
  box-shadow: none;
}

.post-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

/* ========== 文章正文 ========== */
.post-body {
  color: var(--text-secondary);
  line-height: 1.75;
}

.dark-mode .post-body {
  color: var(--text-primary);
}

.loading {
  text-align: center;
  padding: 3rem;
  color: var(--text-muted);
}

/* ========== 文章底部操作 ========== */
.post-footer-actions {
  margin-top: 3rem;
  padding-top: 2rem;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 1rem;
}

.footer-meta {
  font-size: 0.875rem;
  color: var(--text-muted);
}

.footer-buttons {
  display: flex;
  gap: 1rem;
}

.action-button {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.875rem;
  color: var(--text-muted);
  background: none;
  border: none;
  cursor: pointer;
  padding: 0.5rem;
  border-radius: 0.25rem;
  transition: color 0.2s, background-color 0.2s;
}

.action-button:hover {
  color: var(--brand-500, #3b82f6);
  background-color: var(--bg-elevated);
}

/* ========== 侧边栏（目录） ========== */
.post-sidebar {
  display: none;
}

@media (min-width: 1024px) {
  .post-sidebar {
    display: block;
  }
}

.sidebar-sticky {
  position: sticky;
  top: 7rem;
  height: fit-content;
}

.sidebar-title {
  font-size: 0.875rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
  margin: 0 0 1rem;
}

.toc-list {
  list-style: none;
  padding: 0;
  margin: 0;
  padding-left: 1rem;
  border-left: 1px solid var(--border-color);
}

.toc-item {
  margin-bottom: 0.75rem;
}

.toc-link {
  display: block;
  font-size: 0.875rem;
  color: var(--text-muted);
  text-decoration: none;
  transition: color 0.2s;
  padding-left: 1rem;
  margin-left: -1rem;
  border-left: 2px solid transparent;
}

.toc-link:hover {
  color: var(--text-primary);
}

.dark-mode .toc-link:hover {
  color: var(--text-primary);
}

.toc-link.active {
  color: var(--brand-500, #3b82f6);
  border-left-color: var(--brand-500, #3b82f6);
  font-weight: 500;
}

/* ========== 动画 ========== */
@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-slide-up {
  animation: slideUp 0.5s ease-out;
}

/* ========== 滚动条 ========== */
::-webkit-scrollbar {
  width: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background-color: #3f3f46;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background-color: #52525b;
}

/* ========== 文字选中效果 ========== */
::selection {
  background: var(--brand-500, #3b82f6);
  color: white;
}
</style>
