<template>
  <div class="blog-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <!-- 公共导航栏 -->
    <AppHeader />

    <!-- 主内容区域 -->
    <main class="blog-main">
      <div class="blog-container animate-slide-up">
        <!-- 标题区域 -->
        <div class="blog-header">
          <div>
            <h2 class="blog-title">技术专栏</h2>
            <p class="blog-subtitle">分享代码、架构设计与生活随笔</p>
          </div>
          <div class="blog-filters">
            <span class="filter-tag" :class="{ active: currentSeries === 'all' }" @click="currentSeries = 'all'">
              全部
            </span>
            <span class="filter-tag" :class="{ active: currentSeries === '每月故障' }"
              @click="currentSeries = '每月故障'">
              每月故障
            </span>
            <span class="filter-tag" :class="{ active: currentSeries === '每日一题' }"
              @click="currentSeries = '每日一题'">
              每日一题
            </span>
          </div>
        </div>

        <!-- 文章列表 -->
        <div class="posts-list">
          <article v-for="post in filteredPosts" :key="post.slug" class="post-card" @click="goToPost(post.slug)">
            <!-- 内容区域 -->
            <div class="post-content">
              <div class="post-meta-info">
                <time class="post-date">{{ post.date }}</time>
                <span class="meta-dot"></span>
                <span class="post-read-time">{{ post.readTime || 5 }} min read</span>
              </div>
              <h3 class="post-title">{{ post.title }}</h3>
              <p class="post-desc">{{ post.excerpt || '暂无描述' }}</p>
              <div class="post-tags" v-if="post.tags && post.tags.length > 0">
                <span v-for="tag in post.tags" :key="tag" class="post-tag">
                  #{{ tag }}
                </span>
              </div>
            </div>

            <!-- 封面图片区域 -->
            <div v-if="post.cover" class="post-cover">
              <img :src="post.cover" :alt="post.title" />
            </div>
          </article>
        </div>
      </div>
    </main>

    <!-- 页脚 -->
    <AppFooter />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useTheme } from '@/composables/useTheme'
import { getAllPosts } from '@/data/posts'
import AppHeader from '@/components/AppHeader.vue'
import AppFooter from '@/components/AppFooter.vue'

const router = useRouter()
const { isDark } = useTheme()
const currentSeries = ref<string>('all')

const allPosts = getAllPosts()

const filteredPosts = computed(() => {
  if (currentSeries.value === 'all') {
    return allPosts
  }
  return allPosts.filter(p => p.series === currentSeries.value)
})

const goToPost = (slug: string) => {
  router.push(`/posts/${slug}`)
}
</script>

<style scoped>
/* ========== 基础样式 ========== */
.blog-page {
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
.blog-main {
  flex: 1;
  padding-top: 6rem;
  padding-bottom: 4rem;
  padding-left: 1rem;
  padding-right: 1rem;
}

@media (min-width: 640px) {
  .blog-main {
    padding-left: 1.5rem;
    padding-right: 1.5rem;
  }
}

@media (min-width: 1024px) {
  .blog-main {
    padding-left: 2rem;
    padding-right: 2rem;
  }
}

.blog-container {
  max-width: 48rem;
  margin: 0 auto;
  width: 100%;
}

/* ========== 标题区域 ========== */
.blog-header {
  margin-bottom: 2rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 1rem;
}

.blog-title {
  font-size: 1.875rem;
  font-weight: 700;
  margin: 0 0 0.5rem;
}

.blog-subtitle {
  font-size: 0.875rem;
  color: var(--text-muted);
  margin: 0;
}

.blog-filters {
  display: flex;
  gap: 0.5rem;
  font-size: 0.875rem;
}

.filter-tag {
  padding: 0.25rem 0.75rem;
  border-radius: 9999px;
  cursor: pointer;
  transition: all 0.2s;
  color: var(--text-muted);
}

.filter-tag:hover {
  background: var(--bg-elevated);
}

.filter-tag.active {
  background: var(--brand-500);
  color: white;
}

/* ========== 文章列表 ========== */
.posts-list {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.post-card {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  padding: 1rem;
  margin: 0 -1rem;
  border-radius: 1rem;
  cursor: pointer;
  transition: all 0.3s;
}

@media (min-width: 640px) {
  .post-card {
    flex-direction: row;
  }
}

.post-card:hover {
  background: var(--bg-surface);
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
}

.dark-mode .post-card:hover {
  box-shadow: none;
}

/* 内容区域 */
.post-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-width: 0;
}

.post-meta-info {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  font-size: 0.75rem;
  color: var(--text-muted);
  margin-bottom: 0.5rem;
}

.post-date {
  font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, 'Liberation Mono', monospace;
}

.meta-dot {
  width: 0.25rem;
  height: 0.25rem;
  border-radius: 50%;
  background: var(--text-muted);
}

.post-read-time {
  font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, 'Liberation Mono', monospace;
}

.post-title {
  font-size: 1.25rem;
  font-weight: 700;
  margin: 0 0 0.5rem;
  transition: color 0.2s;
}

.post-card:hover .post-title {
  color: var(--brand-500);
}

.post-desc {
  font-size: 0.875rem;
  color: var(--text-secondary);
  line-height: 1.5;
  margin: 0 0 0.75rem;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.post-tags {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.post-tag {
  font-size: 0.75rem;
  color: var(--brand-600);
  background: rgba(59, 130, 246, 0.1);
  padding: 0.125rem 0.5rem;
  border-radius: 0.25rem;
}

.dark-mode .post-tag {
  color: var(--brand-400, #60a5fa);
  background: rgba(59, 130, 246, 0.2);
}

/* 封面图片区域 */
.post-cover {
  width: 100%;
  height: 8rem;
  border-radius: 0.5rem;
  flex-shrink: 0;
  overflow: hidden;
  order: -1;
  background: var(--bg-elevated);
}

@media (min-width: 640px) {
  .post-cover {
    width: 12rem;
    order: 1;
  }
}

.post-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.5s;
}

.post-card:hover .post-cover img {
  transform: scale(1.05);
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
  background: var(--brand-500);
  color: white;
}
</style>
