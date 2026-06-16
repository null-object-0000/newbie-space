<template>
  <div class="tools-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />

    <main class="tools-main">
      <section class="tools-header">
        <h1>工具中心</h1>
        <p>常用的小工具尽量本地运行，降低依赖，也避免把文件上传到第三方服务。</p>
      </section>

      <section class="tools-grid" aria-label="工具列表">
        <router-link v-for="tool in tools" :key="tool.id" class="tool-card" :to="tool.path">
          <div class="tool-icon">
            <component :is="tool.icon" :size="22" />
          </div>
          <div class="tool-content">
            <h2>{{ tool.name }}</h2>
            <p>{{ tool.desc }}</p>
          </div>
          <div class="tool-tags">
            <span v-for="tag in tool.tags" :key="tag">{{ tag }}</span>
          </div>
        </router-link>
      </section>
    </main>

    <AppFooter />
  </div>
</template>

<script setup lang="ts">
import { useTheme } from '@/composables/useTheme'
import { tools } from '@/data/tools'

const { isDark } = useTheme()
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
  padding: 6rem 1rem 4rem;
}

@media (min-width: 640px) {
  .tools-main {
    padding-left: 1.5rem;
    padding-right: 1.5rem;
  }
}

@media (min-width: 1024px) {
  .tools-main {
    padding-left: 2rem;
    padding-right: 2rem;
  }
}

.tools-header {
  max-width: 42rem;
  margin-bottom: 2rem;
}

.tools-header h1 {
  font-size: 2rem;
  line-height: 1.2;
  margin: 0 0 0.75rem;
}

.tools-header p {
  color: var(--text-secondary);
  margin: 0;
}

.tools-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 1rem;
}

@media (min-width: 768px) {
  .tools-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

.tool-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-height: 12rem;
  padding: 1.25rem;
  border: 1px solid var(--border-color);
  border-radius: 1rem;
  background: var(--bg-surface);
  color: inherit;
  text-decoration: none;
  transition: transform 0.2s, border-color 0.2s, box-shadow 0.2s;
}

.tool-card:hover {
  color: inherit;
  transform: translateY(-4px);
  border-color: var(--brand-500);
  box-shadow: var(--shadow-2);
}

.tool-icon {
  width: 2.75rem;
  height: 2.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 0.75rem;
  color: #10b981;
  background: color-mix(in srgb, #10b981 14%, transparent);
}

.tool-content {
  flex: 1;
}

.tool-content h2 {
  font-size: 1.25rem;
  margin: 0 0 0.5rem;
}

.tool-content p {
  color: var(--text-secondary);
  font-size: 0.9rem;
  line-height: 1.7;
  margin: 0;
}

.tool-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.tool-tags span {
  padding: 0.25rem 0.5rem;
  border-radius: 0.375rem;
  color: var(--text-secondary);
  background: var(--bg-elevated);
  font-size: 0.75rem;
}
</style>
