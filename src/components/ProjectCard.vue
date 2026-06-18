<template>
  <div class="project-card">
    <div class="project-image">
      <div class="project-image-placeholder">
        {{ project.name }}
      </div>
      <div class="project-actions">
        <a 
          v-if="project.demoUrl"
          :href="project.demoUrl" 
          target="_blank"
          class="demo-btn"
          @click.stop
        >
          <Play :size="12" />
          Live Demo
        </a>
      </div>
    </div>
    <div class="project-content">
      <div class="project-header">
        <h3 class="project-name">{{ project.name }}</h3>
        <div class="project-stats">
          <span class="stat-item">
            <Star :size="16" />
            {{ project.stars }}
          </span>
        </div>
      </div>
      <p class="project-desc">
        {{ project.desc }}
      </p>
      <div class="project-footer">
        <div class="project-stack">
          <span 
            v-for="tech in project.stack" 
            :key="tech"
            class="tech-dot" 
            :class="getTechColor(tech)"
            :title="tech"
          ></span>
        </div>
        <a 
          v-if="project.url"
          :href="project.url" 
          target="_blank"
          class="project-link"
          @click.stop
        >
          查看文档
          <ArrowRight :size="16" />
        </a>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Project } from '@/data/projects'
import { Star, Play, ArrowRight } from 'lucide-vue-next'

defineProps<{
  project: Project
}>()

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
/* ========== 项目卡片 ========== */
.project-card {
  background: var(--bg-surface);
  border-radius: 1rem;
  border: 1px solid var(--border-color);
  overflow: hidden;
  transition: all 0.3s;
  display: flex;
  flex-direction: column;
}

.project-card:hover {
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  transform: translateY(-2px);
}

.dark-mode .project-card:hover {
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.3), 0 10px 10px -5px rgba(0, 0, 0, 0.2);
}

/* ========== 项目图片区域 ========== */
.project-image {
  height: 12rem;
  background: var(--bg-elevated);
  position: relative;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.project-image-placeholder {
  font-size: 2.25rem;
  font-weight: 700;
  opacity: 0.2;
  transform: scale(1);
  transition: transform 0.5s;
}

.project-card:hover .project-image-placeholder {
  transform: scale(1.1);
}

.project-actions {
  position: absolute;
  bottom: 1rem;
  right: 1rem;
  display: flex;
  gap: 0.5rem;
  opacity: 0;
  transform: translateY(0.5rem);
  transition: opacity 0.3s, transform 0.3s;
}

.project-card:hover .project-actions {
  opacity: 1;
  transform: translateY(0);
}

.demo-btn {
  background: var(--bg-surface);
  color: var(--text-primary);
  padding: 0.25rem 0.75rem;
  border-radius: 9999px;
  font-size: 0.875rem;
  font-weight: 700;
  text-decoration: none;
  box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  display: flex;
  align-items: center;
  gap: 0.25rem;
  transition: all 0.2s;
}

.demo-btn:hover {
  transform: scale(1.05);
}

/* ========== 项目内容 ========== */
.project-content {
  padding: 1.5rem;
  flex: 1;
  display: flex;
  flex-direction: column;
}

.project-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 0.5rem;
}

.project-name {
  font-size: 1.25rem;
  font-weight: 700;
  margin: 0;
}

.project-stats {
  display: flex;
  gap: 0.75rem;
  color: var(--text-muted);
  font-size: 0.875rem;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.project-desc {
  color: var(--text-secondary);
  font-size: 0.875rem;
  margin: 0 0 1rem;
  flex: 1;
  line-height: 1.5;
}

/* ========== 项目底部 ========== */
.project-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 1rem;
  border-top: 1px solid var(--border-color);
}

.project-stack {
  display: flex;
  gap: 0.5rem;
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

.project-link {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--brand-500);
  text-decoration: none;
  display: flex;
  align-items: center;
  gap: 0.25rem;
  transition: color 0.2s;
}

.project-link:hover {
  color: var(--brand-600);
}
</style>
