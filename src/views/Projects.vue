<template>
  <div class="projects-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <!-- 公共导航栏 -->
    <AppHeader />

    <!-- 主内容区域 -->
    <main class="main-content">
      <div class="projects-header">
        <h2 class="projects-title">开源工坊</h2>
        <p class="projects-desc">造轮子，不仅是为了使用，更是为了理解原理。</p>
      </div>

      <div class="projects-grid">
        <ProjectCard 
          v-for="project in projects" 
          :key="project.id" 
          :project="project"
        />
      </div>
    </main>

    <!-- 页脚 -->
    <AppFooter />
  </div>
</template>

<script setup lang="ts">
import { useTheme } from '@/composables/useTheme'
import { useProjects } from '@/composables/useProjects'
import ProjectCard from '@/components/ProjectCard.vue'

const { isDark } = useTheme()
const { projects } = useProjects()
</script>

<style scoped>
/* ========== 基础样式 ========== */
.projects-page {
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
.main-content {
  flex: 1;
  padding-top: 6rem;
  padding-bottom: 4rem;
  padding-left: 1rem;
  padding-right: 1rem;
  max-width: 72rem;
  margin: 0 auto;
  width: 100%;
  animation: slideUp 0.5s ease-out;
}

@media (min-width: 640px) {
  .main-content {
    padding-left: 1.5rem;
    padding-right: 1.5rem;
  }
}

@media (min-width: 1024px) {
  .main-content {
    padding-left: 2rem;
    padding-right: 2rem;
  }
}

/* ========== 项目页头部 ========== */
.projects-header {
  margin-bottom: 2.5rem;
}

.projects-title {
  font-size: 1.875rem;
  font-weight: 700;
  margin: 0 0 1rem;
}

.projects-desc {
  color: var(--text-secondary);
  margin: 0;
}

/* ========== 项目网格 ========== */
.projects-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 2rem;
}

@media (min-width: 768px) {
  .projects-grid {
    grid-template-columns: repeat(2, 1fr);
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
