<template>
  <div>
    <!-- 毛玻璃导航栏 -->
    <nav class="glass-nav">
      <div class="nav-container">
        <div class="logo-area" @click="$router.push('/')">
          <img src="/logo.png" alt="Newbie Space" class="logo-image" />
          <span class="site-title">Newbie Space</span>
        </div>

        <div class="nav-links">
          <router-link to="/" class="nav-item" :class="{ active: $route.path === '/' }">
            首页
            <span class="nav-indicator" v-if="$route.path === '/'"></span>
          </router-link>
          <router-link to="/nav/" class="nav-item" :class="{ active: $route.path === '/nav/' }">
            导航
            <span class="nav-indicator" v-if="$route.path === '/nav/'"></span>
          </router-link>
          <router-link to="/posts" class="nav-item" :class="{ active: $route.path.startsWith('/posts') }">
            文章
            <span class="nav-indicator" v-if="$route.path.startsWith('/posts')"></span>
          </router-link>
          <router-link to="/projects" class="nav-item" :class="{ active: $route.path === '/projects' }">
            项目
            <span class="nav-indicator" v-if="$route.path === '/projects'"></span>
          </router-link>
          <router-link to="/tools" class="nav-item" :class="{ active: $route.path.startsWith('/tools') }">
            工具
            <span class="nav-indicator" v-if="$route.path.startsWith('/tools')"></span>
          </router-link>
        </div>

        <div class="nav-actions">
          <a href="https://github.com/null-object-0000" target="_blank" class="action-btn">
            <Github :size="20" />
          </a>
          <button @click="toggleDark()" class="action-btn">
            <Sun v-if="isDark" :size="20" />
            <Moon v-else :size="20" />
          </button>
          <button class="mobile-menu-btn" @click="mobileMenuOpen = !mobileMenuOpen">
            <Menu :size="24" />
          </button>
        </div>
      </div>
    </nav>

    <!-- 移动端菜单遮罩层 -->
    <Transition name="fade">
      <div v-if="mobileMenuOpen" class="mobile-menu-overlay" @click="mobileMenuOpen = false"></div>
    </Transition>

    <!-- 移动端菜单 -->
    <Transition name="slide">
      <div v-if="mobileMenuOpen" class="mobile-menu">
        <router-link to="/" class="mobile-nav-item" :class="{ active: $route.path === '/' }" @click="mobileMenuOpen = false">
          首页
        </router-link>
        <router-link to="/nav/" class="mobile-nav-item" :class="{ active: $route.path === '/nav/' }" @click="mobileMenuOpen = false">
          导航
        </router-link>
        <router-link to="/posts" class="mobile-nav-item" :class="{ active: $route.path.startsWith('/posts') }" @click="mobileMenuOpen = false">
          文章
        </router-link>
        <router-link to="/projects" class="mobile-nav-item" :class="{ active: $route.path === '/projects' }" @click="mobileMenuOpen = false">
          项目
        </router-link>
        <router-link to="/tools" class="mobile-nav-item" :class="{ active: $route.path.startsWith('/tools') }" @click="mobileMenuOpen = false">
          工具
        </router-link>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { Sun, Moon, Github, Menu } from 'lucide-vue-next'

const { isDark, toggleDark } = useTheme()
const mobileMenuOpen = ref(false)
</script>

<style scoped>
/* ========== 毛玻璃导航栏 ========== */
.glass-nav {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 50;
  border-bottom: 1px solid var(--border-color);
  transition: all 0.3s;
}
</style>

<style>
/* 全局样式，用于响应父组件的主题类 */
.light-mode .glass-nav {
  background: rgba(255, 255, 255, 0.7);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border-color: rgba(229, 231, 235, 1);
}

.dark-mode .glass-nav {
  background: rgba(24, 24, 27, 0.6);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border-color: rgba(255, 255, 255, 0.05);
}

.light-mode .action-btn:hover {
  background: rgba(229, 231, 235, 1);
}

.dark-mode .action-btn:hover {
  background: rgba(39, 39, 42, 1);
}

.light-mode .mobile-menu {
  background: #ffffff;
  border-right: 1px solid var(--border-color, rgba(229, 231, 235, 1));
}

.dark-mode .mobile-menu {
  background: var(--bg-main, #09090b);
  border-right: 1px solid var(--border-color, rgba(255, 255, 255, 0.05));
}
</style>

<style scoped>

.nav-container {
  margin: 0 auto;
  padding: 0 12px;
  height: 3.5rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  box-sizing: border-box;
}

@media (min-width: 375px) {
  .nav-container {
    padding: 0 16px;
    height: 4rem;
  }
}

@media (min-width: 640px) {
  .nav-container {
    padding: 0 24px;
  }
}

@media (min-width: 1024px) {
  .nav-container {
    padding: 0 32px;
    max-width: 72rem;
  }
}

.logo-area {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  flex-shrink: 0;
}

@media (min-width: 375px) {
  .logo-area {
    gap: 0.625rem;
  }
}

@media (min-width: 640px) {
  .logo-area {
    gap: 0.75rem;
  }
}

.logo-image {
  width: 2rem;
  height: 2rem;
  border-radius: 0.5rem;
  object-fit: cover;
  flex-shrink: 0;
  transition: transform 0.2s;
}

@media (min-width: 375px) {
  .logo-image {
    width: 2.25rem;
    height: 2.25rem;
    border-radius: 0.5rem;
  }
}

@media (min-width: 640px) {
  .logo-image {
    width: 2.5rem;
    height: 2.5rem;
    border-radius: 0.625rem;
  }
}

.logo-area:hover .logo-image {
  transform: scale(1.05);
}

.site-title {
  font-size: 0.875rem;
  font-weight: 700;
  letter-spacing: -0.025em;
  white-space: nowrap;
}

@media (min-width: 375px) {
  .site-title {
    font-size: 1rem;
  }
}

@media (min-width: 640px) {
  .site-title {
    font-size: 1.125rem;
  }
}

.nav-links {
  display: none;
  align-items: center;
  gap: 2rem;
}

@media (min-width: 768px) {
  .nav-links {
    display: flex;
  }
}

.nav-item {
  position: relative;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-muted, #71717a);
  text-decoration: none;
  padding: 0.25rem 0;
  transition: color 0.2s;
}

.nav-item:hover {
  color: var(--brand-500, #3b82f6);
}

.nav-item.active {
  color: var(--brand-500, #3b82f6);
}

.nav-indicator {
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  height: 2px;
  background: var(--brand-500, #3b82f6);
  border-radius: 9999px;
}

.nav-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-shrink: 0;
}

@media (min-width: 375px) {
  .nav-actions {
    gap: 0.75rem;
  }
}

@media (min-width: 640px) {
  .nav-actions {
    gap: 1rem;
  }
}

.action-btn {
  padding: 0.375rem;
  border-radius: 9999px;
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
  transition: background-color 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 2.25rem;
  min-height: 2.25rem;
  touch-action: manipulation;
}

@media (min-width: 375px) {
  .action-btn {
    padding: 0.5rem;
    min-width: 2.5rem;
    min-height: 2.5rem;
  }
}

@media (min-width: 768px) {
  .action-btn {
    min-width: auto;
    min-height: auto;
  }
}

.mobile-menu-btn {
  display: block;
  padding: 0.375rem;
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
  min-width: 2.25rem;
  min-height: 2.25rem;
  touch-action: manipulation;
}

@media (min-width: 375px) {
  .mobile-menu-btn {
    padding: 0.5rem;
    min-width: 2.5rem;
    min-height: 2.5rem;
  }
}

@media (min-width: 768px) {
  .mobile-menu-btn {
    display: none;
  }
}

/* ========== 移动端菜单遮罩层 ========== */
.mobile-menu-overlay {
  position: fixed;
  inset: 0;
  z-index: 45;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
}

@media (min-width: 768px) {
  .mobile-menu-overlay {
    display: none;
  }
}

/* ========== 移动端菜单 ========== */
.mobile-menu {
  position: fixed;
  top: 4rem;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 50;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
}

@media (min-width: 768px) {
  .mobile-menu {
    display: none;
  }
}

.mobile-nav-item {
  font-size: 1.125rem;
  font-weight: 500;
  color: var(--text-muted, #71717a);
  text-decoration: none;
  text-align: left;
  padding: 0.875rem 1rem;
  border-radius: 0.5rem;
  transition: all 0.2s;
  min-height: 3rem;
  display: flex;
  align-items: center;
  touch-action: manipulation;
}

.mobile-nav-item:hover,
.mobile-nav-item:active {
  color: var(--brand-500, #3b82f6);
  background: var(--bg-soft, rgba(0, 0, 0, 0.05));
}

.mobile-nav-item.active {
  color: var(--brand-500, #3b82f6);
  background: var(--bg-soft, rgba(0, 0, 0, 0.05));
}

/* ========== 动画 ========== */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.slide-enter-active {
  transition: transform 0.3s ease, opacity 0.3s ease;
}

.slide-leave-active {
  transition: transform 0.3s ease, opacity 0.3s ease;
}

.slide-enter-from {
  transform: translateX(-100%);
  opacity: 0;
}

.slide-leave-to {
  transform: translateX(-100%);
  opacity: 0;
}
</style>
