<template>
  <div class="home-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <!-- 公共导航栏 -->
    <AppHeader />

    <!-- 主内容区域 -->
    <main class="main-content">
      <!-- Hero 区域 -->
      <section class="hero-section">
        <h1 class="hero-title">
          构建代码，记录思考。
        </h1>
        <p class="hero-desc">
          欢迎来到我的数字花园。这里有我的技术文章、精选的开发工具导航，以及我正在折腾的开源项目。
        </p>
      </section>

      <!-- 仪表板卡片网格 - 完全按照 test.html 布局 -->
      <div class="dashboard-grid">
        <!-- 第一行: 最新文章卡片 (2列) + 关于我卡片 (1列) -->

        <!-- 最新文章卡片 -->
        <div class="card card-article" @click="goToLatestPost">
          <div class="card-badge">
            <PenTool :size="14" />
            <span>最新发布</span>
          </div>
          <h3 class="card-title">{{ latestPost?.title || '暂无文章' }}</h3>
          <p class="card-desc">{{ latestPost?.excerpt || '敬请期待更多内容...' }}</p>
          <div class="card-tags" v-if="latestPost && latestPost.tags && latestPost.tags.length > 0">
            <span class="tag" v-for="tag in latestPost.tags" :key="tag">#{{ tag }}</span>
          </div>
        </div>

        <!-- 关于我卡片 -->
        <div class="card card-about">
          <div class="about-avatar">
            <User :size="24" />
          </div>
          <h3 class="about-title">About Me</h3>
          <p class="about-desc">全栈开发者，热爱开源，目前专注于 AI 和可视化工具链领域。</p>
          <a href="https://github.com/null-object-0000" target="_blank" class="about-link">
            了解更多 →
          </a>
        </div>

        <!-- 第二行: 开源项目卡片 (1列) + 常用导航卡片 (2列) -->

        <!-- 开源项目卡片 -->
        <div @click="goToProject(featuredProject)" class="card card-project" v-if="featuredProject">
          <div class="project-badge">
            <Box :size="14" />
            <span>精选项目</span>
          </div>
          <div class="project-image">
            <div class="project-image-placeholder">
              {{ featuredProject.name }}
            </div>
          </div>
          <h3 class="project-title">{{ featuredProject.name }}</h3>
          <p class="project-desc">{{ featuredProject.desc }}</p>
          <div class="project-footer">
            <div class="project-stats">
              <span class="stat-item">
                <Star :size="14" />
                <span>{{ featuredProject.stars }}</span>
              </span>
            </div>
            <div class="project-stack">
              <span v-for="tech in featuredProject.stack" :key="tech" class="tech-dot" :class="getTechColor(tech)"
                :title="tech"></span>
            </div>
          </div>
        </div>

        <!-- 常用导航卡片 -->
        <div class="card card-nav">
          <div class="nav-header">
            <div class="nav-badge">
              <Compass :size="14" />
              <span>常用导航</span>
            </div>
            <router-link to="/nav/" class="view-all">查看全部 →</router-link>
          </div>
          <div class="quick-links">
            <CardLink v-for="link in featuredLinks" :key="link.name" :link="link" :show-sublinks="false" @click="handleLinkClick" />
          </div>
        </div>
      </div>
    </main>

    <!-- 页脚 -->
    <AppFooter />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useTheme } from '@/composables/useTheme'
import { useProjects } from '@/composables/useProjects'
import { getAllPosts } from '@/data/posts'
import { Project } from '@/data/projects'
import navData from '@/data/nav-data.json'
import {
  PenTool, User, Compass,
  Box, Star
} from 'lucide-vue-next'

const router = useRouter()
const { isDark } = useTheme()
const { projects } = useProjects()

// 获取最新文章
const posts = getAllPosts()
const latestPost = posts[0]

// 获取 stars 最多的项目
const featuredProject = computed(() => {
  if (projects.value.length === 0) return null

  // 将 stars 转换为数字进行比较，字符串（如 "New"）视为 0
  const getStarsValue = (stars: number | string): number => {
    if (typeof stars === 'number') return stars
    if (typeof stars === 'string') {
      const num = parseInt(stars, 10)
      return isNaN(num) ? 0 : num
    }
    return 0
  }

  return projects.value.reduce((max, project) => {
    const maxStars = getStarsValue(max.stars)
    const currentStars = getStarsValue(project.stars)
    return currentStars > maxStars ? project : max
  })
})

// 近期使用 - 从 localStorage 读取点击记录
const STORAGE_KEY_CLICKS = 'nav-click-counts'
const clickCounts = ref<Record<string, number>>({})

const loadClickCounts = () => {
  if (typeof localStorage === 'undefined') return
  try {
    const stored = localStorage.getItem(STORAGE_KEY_CLICKS)
    if (stored) {
      clickCounts.value = JSON.parse(stored)
    }
  } catch (e) {
    console.error('Failed to load click counts:', e)
    clickCounts.value = {}
  }
}

onMounted(() => {
  loadClickCounts()
})

// 获取所有链接的扁平列表
const getAllLinks = () => {
  return navData.flatMap((cat: any) =>
    cat.links.map((link: any) => ({
      ...link,
      categoryName: cat.name
    }))
  )
}

// 默认常用工具列表（当近期使用不足时用于填充）
const defaultCategories = ['AI 对话', '常用工具', 'AI 编码', 'AI 代理']

const getDefaultLinks = () => {
  const links: any[] = []
  for (const catName of defaultCategories) {
    const category = navData.find((c: any) => c.name === catName)
    if (category && category.links && category.links.length > 0) {
      links.push(category.links[0])
    }
  }
  return links.slice(0, 4)
}

// 精选导航链接 - 优先使用近期使用，不足时用默认工具填充
const featuredLinks = computed(() => {
  const allLinks = getAllLinks()
  const MAX_LINKS = 4

  // 1. 获取近期使用的链接（按点击次数排序）
  const recentLinks = allLinks
    .map(link => ({
      ...link,
      clickCount: clickCounts.value[link.link] || 0
    }))
    .filter(link => link.clickCount > 0)
    .sort((a, b) => b.clickCount - a.clickCount)
    .slice(0, MAX_LINKS)
    .map(({ clickCount, categoryName, ...link }) => link)

  // 2. 如果近期使用的链接足够，直接返回
  if (recentLinks.length >= MAX_LINKS) {
    return recentLinks
  }

  // 3. 不足时用默认工具填充
  const defaultLinks = getDefaultLinks()
  const recentUrls = new Set(recentLinks.map(l => l.link))

  // 过滤掉已在近期使用中的默认链接
  const fillLinks = defaultLinks.filter(l => !recentUrls.has(l.link))

  // 合并并限制数量
  return [...recentLinks, ...fillLinks].slice(0, MAX_LINKS)
})

const goToLatestPost = () => {
  if (latestPost) {
    router.push(`/posts/${latestPost.slug}`)
  }
}

const goToProject = (project: Project) => {
  window.open(project.url, '_blank', 'noopener,noreferrer')
}

// 处理链接点击（记录点击次数并打开链接）
const handleLinkClick = (linkUrl: string) => {
  // 记录点击次数
  clickCounts.value[linkUrl] = (clickCounts.value[linkUrl] || 0) + 1
  if (typeof localStorage !== 'undefined') {
    try {
      localStorage.setItem(STORAGE_KEY_CLICKS, JSON.stringify(clickCounts.value))
    } catch (e) {
      console.error('Failed to save click counts:', e)
    }
  }

  // 打开链接
  window.open(linkUrl, '_blank', 'noopener,noreferrer')
}

// 获取技术栈颜色类
const getTechColor = (tech: string) => {
  const map: Record<string, string> = {
    vue: 'tech-vue',
    react: 'tech-react',
    ts: 'tech-ts',
    vite: 'tech-vite',
    node: 'tech-node',
    less: 'tech-less',
    java: 'tech-java'
  }
  return map[tech] || 'tech-default'
}
</script>

<style scoped>
/* ========== 基础样式 ========== */
.home-page {
  min-height: 100vh;
  min-height: 100dvh;
  /* 动态视口高度 */
  display: flex;
  flex-direction: column;
  font-family: 'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  transition: background-color 0.3s, color 0.3s;
  -webkit-font-smoothing: antialiased;
  overflow-x: hidden;
  max-width: 100%;
}

/* ========== 深色主题 (完全匹配 test.html) ========== */
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
.main-content {
  flex: 1;
  padding-top: 4rem;
  padding-bottom: 1.5rem;
  padding-left: 12px;
  padding-right: 12px;
  max-width: 72rem;
  margin-left: auto;
  margin-right: auto;
  box-sizing: border-box;
}

@media (min-width: 375px) {
  .main-content {
    padding-top: 4.5rem;
    padding-left: 16px;
    padding-right: 16px;
  }
}

@media (min-width: 640px) {
  .main-content {
    padding-top: 6rem;
    padding-bottom: 4rem;
    padding-left: 24px;
    padding-right: 24px;
  }
}

@media (min-width: 1024px) {
  .main-content {
    padding-left: 32px;
    padding-right: 32px;
  }
}

/* 支持安全区域的设备 */
@supports (padding: env(safe-area-inset-left)) {
  .main-content {
    padding-left: calc(12px + env(safe-area-inset-left, 0px));
    padding-right: calc(12px + env(safe-area-inset-right, 0px));
  }

  @media (min-width: 375px) {
    .main-content {
      padding-left: calc(16px + env(safe-area-inset-left, 0px));
      padding-right: calc(16px + env(safe-area-inset-right, 0px));
    }
  }

  @media (min-width: 640px) {
    .main-content {
      padding-left: calc(24px + env(safe-area-inset-left, 0px));
      padding-right: calc(24px + env(safe-area-inset-right, 0px));
    }
  }
}

/* ========== Hero 区域 ========== */
.hero-section {
  text-align: center;
  padding: 1rem 0;
  animation: slideUp 0.5s ease-out;
}

@media (min-width: 375px) {
  .hero-section {
    padding: 1.25rem 0;
  }
}

@media (min-width: 640px) {
  .hero-section {
    padding: 2rem 0;
  }
}

@media (min-width: 768px) {
  .hero-section {
    padding: 2.5rem 0;
  }
}

.hero-title {
  font-size: 1.5rem;
  font-weight: 700;
  margin: 0 0 0.5rem;
  line-height: 1.2;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

@media (min-width: 375px) {
  .hero-title {
    font-size: 1.75rem;
    margin: 0 0 0.75rem;
  }
}

@media (min-width: 640px) {
  .hero-title {
    font-size: 2.25rem;
    margin: 0 0 1rem;
  }
}

@media (min-width: 768px) {
  .hero-title {
    font-size: 3.75rem;
  }
}

.light-mode .hero-title {
  background: linear-gradient(to right, #111827, #4b5563);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.dark-mode .hero-title {
  background: linear-gradient(to right, #ffffff, #a1a1aa);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.hero-desc {
  font-size: 0.875rem;
  color: var(--text-secondary);
  max-width: 42rem;
  margin: 0 auto;
  line-height: 1.5;
  padding: 0 0.25rem;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

@media (min-width: 375px) {
  .hero-desc {
    font-size: 0.9375rem;
    padding: 0 0.5rem;
    line-height: 1.6;
  }
}

@media (min-width: 640px) {
  .hero-desc {
    font-size: 1rem;
    line-height: 1.7;
    padding: 0;
  }
}

@media (min-width: 768px) {
  .hero-desc {
    font-size: 1.125rem;
    line-height: 1.75;
  }
}

/* ========== 仪表板卡片网格 ========== */
.dashboard-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 0.75rem;
  animation: slideUp 0.5s ease-out;
  width: 100%;
}

@media (min-width: 375px) {
  .dashboard-grid {
    gap: 1rem;
  }
}

@media (min-width: 640px) {
  .dashboard-grid {
    gap: 1.25rem;
  }
}

@media (min-width: 768px) {
  .dashboard-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 1.5rem;
  }
}

/* ========== 通用卡片样式 ========== */
.card {
  border-radius: 0.75rem;
  padding: 0.875rem;
  border: 1px solid var(--border-color);
  transition: all 0.2s;
  cursor: pointer;
  touch-action: manipulation;
  -webkit-tap-highlight-color: transparent;
  box-sizing: border-box;
  word-wrap: break-word;
  overflow-wrap: break-word;
  min-width: 0;
  overflow: hidden;
}

@media (min-width: 375px) {
  .card {
    padding: 1rem;
  }
}

@media (min-width: 640px) {
  .card {
    border-radius: 1rem;
    padding: 1.25rem;
  }
}

@media (min-width: 768px) {
  .card {
    padding: 1.5rem;
  }
}

.card:active {
  transform: scale(0.98);
}

.light-mode .card {
  background: var(--bg-surface);
}

.dark-mode .card {
  background: var(--bg-surface);
}

.card:hover {
  border-color: var(--brand-500);
}

/* ========== 文章卡片 (占2列) ========== */
.card-article {
  grid-column: 1;
}

@media (min-width: 768px) {
  .card-article {
    grid-column: span 2;
  }
}

.card-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--brand-500);
  font-size: 0.625rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 0.625rem;
}

@media (min-width: 375px) {
  .card-badge {
    font-size: 0.6875rem;
    margin-bottom: 0.75rem;
  }
}

@media (min-width: 640px) {
  .card-badge {
    font-size: 0.75rem;
    margin-bottom: 1rem;
  }
}

.card-title {
  font-size: 1rem;
  font-weight: 700;
  margin: 0 0 0.375rem;
  line-height: 1.3;
  transition: color 0.2s;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

@media (min-width: 375px) {
  .card-title {
    font-size: 1.125rem;
    margin: 0 0 0.5rem;
  }
}

@media (min-width: 640px) {
  .card-title {
    font-size: 1.25rem;
  }
}

@media (min-width: 768px) {
  .card-title {
    font-size: 1.5rem;
  }
}

.card:hover .card-title {
  color: var(--brand-500);
}

.card-desc {
  color: var(--text-muted);
  line-height: 1.5;
  margin: 0;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  font-size: 0.75rem;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

@media (min-width: 375px) {
  .card-desc {
    font-size: 0.8125rem;
  }
}

@media (min-width: 640px) {
  .card-desc {
    font-size: 0.875rem;
    line-height: 1.625;
  }
}

.card-tags {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.75rem;
  flex-wrap: wrap;
}

@media (min-width: 640px) {
  .card-tags {
    margin-top: 1rem;
  }
}

.tag {
  padding: 0.25rem 0.5rem;
  border-radius: 0.25rem;
  font-size: 0.6875rem;
  color: var(--text-muted);
}

@media (min-width: 640px) {
  .tag {
    font-size: 0.75rem;
  }
}

.light-mode .tag {
  background: #f3f4f6;
}

.dark-mode .tag {
  background: rgba(39, 39, 42, 1);
}

/* ========== 关于我卡片 ========== */
.card-about {
  background: linear-gradient(to bottom right, var(--brand-500), var(--accent-600)) !important;
  color: white;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  border: none !important;
}

.card-about:hover {
  border: none !important;
}

.about-avatar {
  width: 2.25rem;
  height: 2.25rem;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 9999px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 0.625rem;
}

@media (min-width: 375px) {
  .about-avatar {
    width: 2.5rem;
    height: 2.5rem;
    margin-bottom: 0.75rem;
  }
}

@media (min-width: 640px) {
  .about-avatar {
    width: 3rem;
    height: 3rem;
    margin-bottom: 1rem;
  }
}

.about-title {
  font-size: 1rem;
  font-weight: 700;
  margin: 0;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

@media (min-width: 375px) {
  .about-title {
    font-size: 1.125rem;
  }
}

@media (min-width: 640px) {
  .about-title {
    font-size: 1.25rem;
  }
}

.about-desc {
  font-size: 0.75rem;
  opacity: 0.8;
  margin: 0.375rem 0 0;
  line-height: 1.5;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

@media (min-width: 375px) {
  .about-desc {
    font-size: 0.8125rem;
    margin: 0.5rem 0 0;
  }
}

@media (min-width: 640px) {
  .about-desc {
    font-size: 0.875rem;
  }
}

.about-link {
  font-size: 0.8125rem;
  font-weight: 500;
  color: white;
  text-decoration: none;
  margin-top: 0.75rem;
  align-self: flex-start;
  padding: 0.25rem 0;
  min-height: 2rem;
  display: inline-flex;
  align-items: center;
  touch-action: manipulation;
}

@media (min-width: 640px) {
  .about-link {
    font-size: 0.875rem;
    margin-top: 1rem;
  }
}

.about-link:hover {
  text-decoration: underline;
}

/* ========== 开源项目卡片 ========== */
.card-project {
  text-decoration: none;
  color: inherit;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  transition: transform 0.2s, border-color 0.2s;
}

.card-project:hover {
  transform: translateY(-4px);
}

.project-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--accent-500);
  font-size: 0.625rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 0.625rem;
}

@media (min-width: 375px) {
  .project-badge {
    font-size: 0.6875rem;
    margin-bottom: 0.75rem;
  }
}

@media (min-width: 640px) {
  .project-badge {
    font-size: 0.75rem;
    margin-bottom: 1rem;
  }
}

.project-image {
  width: 100%;
  height: 4rem;
  background: var(--bg-elevated);
  border-radius: 0.5rem;
  margin-bottom: 0.625rem;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  flex-shrink: 0;
}

@media (min-width: 375px) {
  .project-image {
    height: 4.5rem;
    margin-bottom: 0.75rem;
  }
}

@media (min-width: 640px) {
  .project-image {
    height: 5rem;
    margin-bottom: 1rem;
  }
}

.project-image-placeholder {
  font-size: 1.5rem;
  font-weight: 700;
  opacity: 0.2;
  transform: scale(1);
  transition: transform 0.5s;
  word-wrap: break-word;
  overflow-wrap: break-word;
  text-align: center;
  padding: 0 0.5rem;
}

.card-project:hover .project-image-placeholder {
  transform: scale(1.05);
}

@media (min-width: 375px) {
  .project-image-placeholder {
    font-size: 1.75rem;
  }
}

@media (min-width: 640px) {
  .project-image-placeholder {
    font-size: 2rem;
  }
}

.project-title {
  font-size: 1rem;
  font-weight: 700;
  margin: 0 0 0.375rem;
  line-height: 1.3;
  transition: color 0.2s;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

@media (min-width: 375px) {
  .project-title {
    font-size: 1.125rem;
    margin: 0 0 0.5rem;
  }
}

@media (min-width: 640px) {
  .project-title {
    font-size: 1.25rem;
  }
}

.card-project:hover .project-title {
  color: var(--accent-500);
}

.project-desc {
  color: var(--text-muted);
  line-height: 1.5;
  margin: 0.375rem 0 0;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  font-size: 0.75rem;
  word-wrap: break-word;
  overflow-wrap: break-word;
  flex: 1;
}

@media (min-width: 375px) {
  .project-desc {
    font-size: 0.8125rem;
    margin: 0.5rem 0 0;
  }
}

@media (min-width: 640px) {
  .project-desc {
    font-size: 0.875rem;
    line-height: 1.625;
  }
}

.project-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 0.75rem;
  gap: 0.5rem;
}

@media (min-width: 640px) {
  .project-footer {
    margin-top: 1rem;
  }
}

.project-stats {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  font-size: 0.75rem;
  color: var(--text-muted);
  flex-wrap: wrap;
}

@media (min-width: 375px) {
  .project-stats {
    gap: 0.75rem;
    font-size: 0.8125rem;
  }
}

@media (min-width: 640px) {
  .project-stats {
    gap: 1rem;
    font-size: 0.875rem;
  }
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.project-stack {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.tech-dot {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 9999px;
}

.tech-vue {
  background: #10b981;
}

.tech-react {
  background: #3b82f6;
}

.tech-ts {
  background: #2563eb;
}

.tech-vite {
  background: #06b6d4;
}

.tech-node {
  background: #16a34a;
}

.tech-less {
  background: #fb923c;
}

.tech-java {
  background: #f59e0b;
}

.tech-default {
  background: #9ca3af;
}

/* ========== 常用导航卡片 (占2列，无外边框) ========== */
.card-nav {
  grid-column: 1;
  background: transparent !important;
  border: none !important;
  padding: 0.125rem;
  cursor: default;
}

@media (min-width: 375px) {
  .card-nav {
    padding: 0.25rem;
  }
}

.card-nav:hover {
  border: none !important;
  transform: none;
}

@media (min-width: 640px) {
  .card-nav {
    padding: 0.5rem;
  }
}

@media (min-width: 768px) {
  .card-nav {
    grid-column: span 2;
  }
}

.nav-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.75rem;
  flex-wrap: wrap;
  gap: 0.5rem;
}

@media (min-width: 375px) {
  .nav-header {
    margin-bottom: 1rem;
  }
}

@media (min-width: 640px) {
  .nav-header {
    margin-bottom: 1.25rem;
  }
}

@media (min-width: 768px) {
  .nav-header {
    margin-bottom: 1.5rem;
  }
}

.nav-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  color: #22c55e;
  font-size: 0.625rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

@media (min-width: 375px) {
  .nav-badge {
    font-size: 0.6875rem;
  }
}

@media (min-width: 640px) {
  .nav-badge {
    font-size: 0.75rem;
  }
}

.view-all {
  font-size: 0.6875rem;
  color: var(--text-muted);
  text-decoration: none;
  transition: color 0.2s;
  padding: 0.25rem 0;
  min-height: 2rem;
  display: inline-flex;
  align-items: center;
  touch-action: manipulation;
}

@media (min-width: 640px) {
  .view-all {
    font-size: 0.75rem;
  }
}

.view-all:hover,
.view-all:active {
  color: var(--brand-500);
}

.quick-links {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 0.5rem;
  width: 100%;
}

@media (min-width: 375px) {
  .quick-links {
    gap: 0.625rem;
  }
}

@media (min-width: 640px) {
  .quick-links {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.75rem;
  }
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
  background: var(--brand-500);
  color: white;
}
</style>
