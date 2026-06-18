<template>
  <div class="card-link-wrapper" 
    @mouseenter="!isMobile && handleCardHover($event)" 
    @mouseleave="!isMobile && handleCardLeave($event)">
    <a :href="link.link" target="_blank" rel="noopener noreferrer" class="card-link"
      @click.prevent="handleLinkClick"
      @mouseleave="!isMobile && handleCardLeave($event)">
      <div class="card-header">
        <div class="card-title-row">
          <div class="card-icon-wrapper">
            <!-- 如果是 lucide 图标名称，使用 lucide 组件 -->
            <component v-if="isLucideIcon(link.icon)" :is="getIconComponent(link.icon)" :size="20"
              class="card-icon-lucide" />
            <!-- 如果是图片路径或 URL，使用图片 -->
            <img v-else-if="getIconUrl(link)" :src="getIconUrl(link)" :alt="link.name" class="card-icon"
              @error="handleIconError" />
            <!-- 默认回退图标 -->
            <Globe v-else :size="20" class="card-icon-lucide" />
          </div>
          <h3 class="card-title">{{ link.name }}</h3>
        </div>
        <ExternalLink v-if="!isInternalLink" :size="14" class="card-external-icon" />
      </div>
      <div class="card-content">
        <p class="card-desc" :class="{ 'card-desc-empty': !link.desc }">{{ link.desc || "No description available." }}</p>
      </div>
    </a>
    <!-- 移动端展开按钮 -->
    <button v-if="isMobile && hasSublinks" 
      class="card-expand-btn" 
      @click.stop="toggleSublinks"
      :title="isExpanded ? '收起' : '展开'">
      <component :is="isExpanded ? ChevronUp : ChevronDown" :size="18" />
    </button>
    <!-- 桌面端：子链接弹出菜单（悬浮显示） -->
    <div v-if="!isMobile && hasSublinks && isHovered" 
      class="card-sublinks-menu" :style="sublinksMenuStyle"
      @mouseenter="handleMenuHover" @mouseleave="handleMenuLeave">
      <div class="sublinks-menu-header">
        <span class="sublinks-menu-title">{{ link.name }}</span>
      </div>
      <div class="sublinks-menu-list">
        <a v-for="(subLink, subIdx) in link.subLinks" :key="subIdx" 
          :href="subLink.link" target="_blank" rel="noopener noreferrer" 
          class="sublink-item" @click.stop.prevent="handleSubLinkClick(subLink)">
          <div class="sublink-icon-wrapper">
            <component v-if="isLucideIcon(subLink.icon)" :is="getIconComponent(subLink.icon)" :size="16"
              class="sublink-icon-lucide" />
            <img v-else-if="getIconUrl(subLink)" :src="getIconUrl(subLink)" :alt="subLink.name" 
              class="sublink-icon" @error="handleIconError" />
            <Globe v-else :size="16" class="sublink-icon-lucide" />
          </div>
          <div class="sublink-content">
            <span class="sublink-name">{{ subLink.name }}</span>
          </div>
          <ExternalLink :size="12" class="sublink-external-icon" />
        </a>
      </div>
    </div>
    <!-- 移动端：子链接内联展开 -->
    <div v-if="isMobile && hasSublinks && isExpanded" 
      class="card-sublinks-inline">
      <div class="sublinks-inline-list">
        <a v-for="(subLink, subIdx) in link.subLinks" :key="subIdx" 
          :href="subLink.link" target="_blank" rel="noopener noreferrer" 
          class="sublink-item-inline" @click.stop.prevent="handleSubLinkClick(subLink)">
          <div class="sublink-icon-wrapper">
            <component v-if="isLucideIcon(subLink.icon)" :is="getIconComponent(subLink.icon)" :size="16"
              class="sublink-icon-lucide" />
            <img v-else-if="getIconUrl(subLink)" :src="getIconUrl(subLink)" :alt="subLink.name" 
              class="sublink-icon" @error="handleIconError" />
            <Globe v-else :size="16" class="sublink-icon-lucide" />
          </div>
          <div class="sublink-content">
            <span class="sublink-name">{{ subLink.name }}</span>
          </div>
          <ExternalLink :size="12" class="sublink-external-icon" />
        </a>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, h, createApp } from 'vue'
import { useData } from '@/composables/useTheme'
import { getAssetPath, getBasePath } from '@/utils/path'
import * as lucideIcons from 'lucide-vue-next'
import { Globe, ExternalLink, ChevronDown, ChevronUp } from 'lucide-vue-next'

interface SubLink {
  name: string
  link: string
  desc?: string
  icon?: string
}

interface Link {
  name: string
  link: string
  desc?: string
  icon?: string
  subLinks?: SubLink[]
}

interface Props {
  link: Link
  showSublinks?: boolean // 是否展示子链接卡片
}

const props = withDefaults(defineProps<Props>(), {
  showSublinks: true // 默认展示子链接
})
const emit = defineEmits<{
  click: [linkUrl: string, linkData?: any]
}>()

// 处理主链接点击
const handleLinkClick = () => {
  emit('click', props.link.link, props.link)
}

// 处理子链接点击
const handleSubLinkClick = (subLink: SubLink) => {
  emit('click', subLink.link, subLink)
}

const { site } = useData()
const base = computed(() => site.value.base || getBasePath())

// 状态管理
const isHovered = ref(false)
const isExpanded = ref(false)
const isMobile = ref(false)
const sublinksMenuStyle = ref<{ top?: string; left?: string; right?: string; bottom?: string }>({})
let hideMenuTimer: ReturnType<typeof setTimeout> | null = null
let isMenuHovered = false

const hasSublinks = computed(() => {
  return props.showSublinks && props.link.subLinks && props.link.subLinks.length > 0
})

// 内部链接（站内路由）不显示外链图标
const isInternalLink = computed(() => {
  return props.link.link.startsWith('/')
})

// 检测是否为移动端
const checkIsMobile = () => {
  if (typeof window === 'undefined') return
  isMobile.value = window.innerWidth < 768 || 'ontouchstart' in window
}

// 处理卡片悬浮（桌面端）
const handleCardHover = (event: MouseEvent) => {
  if (!hasSublinks.value) return
  
  // 清除之前的隐藏定时器
  if (hideMenuTimer) {
    clearTimeout(hideMenuTimer)
    hideMenuTimer = null
  }
  
  isHovered.value = true
  const wrapperElement = (event.currentTarget as HTMLElement)
  // 获取实际的 card-link 元素位置
  const cardElement = wrapperElement.querySelector('.card-link') as HTMLElement
  if (!cardElement) return
  const rect = cardElement.getBoundingClientRect()
  const menuWidth = 280 // 菜单宽度
  const menuHeight = Math.min((props.link.subLinks?.length || 0) * 48 + 40, 400) // 菜单高度
  
  // 计算菜单位置，优先显示在右侧，如果空间不足则显示在左侧
  const spaceRight = window.innerWidth - rect.right
  const spaceLeft = rect.left
  const spaceBottom = window.innerHeight - rect.bottom
  
  if (spaceRight >= menuWidth) {
    // 右侧有足够空间
    sublinksMenuStyle.value = {
      top: `${rect.top + window.scrollY}px`,
      left: `${rect.right + 4}px`
    }
  } else if (spaceLeft >= menuWidth) {
    // 左侧有足够空间
    sublinksMenuStyle.value = {
      top: `${rect.top + window.scrollY}px`,
      right: `${window.innerWidth - rect.left + 4}px`
    }
  } else {
    // 两侧空间都不足，显示在下方
    if (spaceBottom >= menuHeight) {
      sublinksMenuStyle.value = {
        top: `${rect.bottom + 4}px`,
        left: `${rect.left + window.scrollX}px`
      }
    } else {
      sublinksMenuStyle.value = {
        bottom: `${window.innerHeight - rect.top + 4}px`,
        left: `${rect.left + window.scrollX}px`
      }
    }
  }
}

// 处理卡片离开（桌面端）
const handleCardLeave = (event?: MouseEvent) => {
  // 如果菜单正在被悬浮，不隐藏
  if (isMenuHovered) return
  
  // 检查鼠标是否移动到了子菜单
  if (event?.relatedTarget) {
    const relatedTarget = event.relatedTarget as HTMLElement
    const menuElement = relatedTarget.closest?.('.card-sublinks-menu')
    if (menuElement) {
      // 鼠标移动到了菜单，不隐藏
      return
    }
  }
  
  // 延迟隐藏，以便用户能够移动到子菜单
  hideMenuTimer = setTimeout(() => {
    if (!isMenuHovered) {
      isHovered.value = false
    }
  }, 150)
}

// 处理菜单悬浮
const handleMenuHover = () => {
  isMenuHovered = true
  if (hideMenuTimer) {
    clearTimeout(hideMenuTimer)
    hideMenuTimer = null
  }
}

// 处理菜单离开
const handleMenuLeave = () => {
  isMenuHovered = false
  isHovered.value = false
}

// 切换子链接展开/收起（移动端）
const toggleSublinks = () => {
  isExpanded.value = !isExpanded.value
}

// 获取域名
const getDomain = (link: string) => {
  try {
    return new URL(link).hostname
  } catch (e) {
    return ''
  }
}

// 判断是否是 lucide 图标名称
const isLucideIcon = (icon: any): boolean => {
  if (!icon || typeof icon !== 'string') return false
  // 如果是路径（以 / 开头）或 URL（以 http:// 或 https:// 开头），不是 lucide 图标
  if (icon.startsWith('/') || icon.startsWith('http://') || icon.startsWith('https://')) {
    return false
  }
  // 其他情况认为是 lucide 图标名称
  return true
}

// 获取图标 URL
const getIconUrl = (link: Link | SubLink) => {
  // 如果是 lucide 图标，返回 null
  if (isLucideIcon(link.icon)) {
    return null
  }
  
  // 如果 link.icon 存在且是路径（以 / 开头），使用本地图标
  if (link.icon && typeof link.icon === 'string') {
    if (link.icon.startsWith('/')) {
      // 使用 getAssetPath 处理路径，自动适配不同部署环境
      return getAssetPath(link.icon)
    } else if (link.icon.startsWith('http://') || link.icon.startsWith('https://')) {
      // 如果是完整的 URL，直接使用
      return link.icon
    }
  }
  // 如果没有本地图标，回退到 Google Favicon 服务
  return `https://www.google.com/s2/favicons?domain=${getDomain(link.link)}&sz=64`
}

// 将配置中的图标名称转换为 Lucide 组件名（PascalCase）
const toPascalCase = (str: string): string => {
  return str
    .split(/[-_\s]+/)
    .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join('')
}

// 常见的图标名称映射
const iconNameMap: Record<string, string> = {
  'message': 'MessageSquare',
  'grid': 'LayoutGrid'
}

// 从配置中动态获取图标组件
const getIconComponent = (iconName: string) => {
  if (!iconName) {
    return lucideIcons.LayoutGrid // 默认图标
  }

  // 先检查是否有映射
  const mappedName = iconNameMap[iconName.toLowerCase()]
  const finalName = mappedName || iconName

  // 转换为 PascalCase 组件名
  const componentName = toPascalCase(finalName)

  // 从 lucideIcons 中获取对应的组件
  const IconComponent = (lucideIcons as any)[componentName]

  // 如果找不到，尝试使用默认图标
  return IconComponent || lucideIcons.LayoutGrid
}

const handleIconError = (e: Event) => {
  const target = e.target as HTMLImageElement
  if (target && target.parentElement) {
    // 如果本地图标加载失败，尝试使用 Google Favicon 作为回退
    const link = target.closest('.card-link, .sublink-item, .sublink-item-inline')
    if (link) {
      const linkElement = link as HTMLElement
      const linkUrl = linkElement.getAttribute('href')
      
      if (linkUrl) {
        // 尝试找到对应的 link 数据
        let linkData: Link | SubLink | null = null
        if (props.link.link === linkUrl) {
          linkData = props.link
        } else if (props.link.subLinks) {
          linkData = props.link.subLinks.find(sl => sl.link === linkUrl) || null
        }
        
        if (linkData && linkData.icon && linkData.icon.startsWith('/')) {
          // 如果当前使用的是本地图标但加载失败，回退到 Google Favicon
          target.src = `https://www.google.com/s2/favicons?domain=${getDomain(linkUrl)}&sz=64`
          return
        }
      }
    }

    // 如果所有回退都失败，显示默认图标（使用 lucide Globe 图标）
    target.style.display = 'none'
    const fallback = document.createElement('div')
    fallback.className = 'card-icon-fallback'
    // 创建临时 Vue 应用来渲染 Globe 组件
    const app = createApp({
      render: () => h(Globe, { size: 20, class: 'card-icon-lucide' })
    })
    app.mount(fallback)
    target.parentNode?.replaceChild(fallback, target)
  }
}

// 监听窗口大小变化
const handleResize = () => {
  checkIsMobile()
  // 如果是桌面端，清除展开状态
  if (!isMobile.value) {
    isExpanded.value = false
  }
}

// 全局鼠标移动监听，用于检测鼠标是否离开了卡片和菜单区域
let globalMouseMoveHandler: ((event: MouseEvent) => void) | null = null

const setupGlobalMouseListener = () => {
  if (typeof window === 'undefined') return
  if (isMobile.value) return
  
  if (!globalMouseMoveHandler) {
    globalMouseMoveHandler = (event: MouseEvent) => {
      if (!isHovered.value) return
      
      const target = event.target as HTMLElement
      if (!target) return
      
      // 检查鼠标是否在卡片或菜单区域内
      const isInCard = target.closest('.card-link-wrapper') !== null
      const isInMenu = target.closest('.card-sublinks-menu') !== null
      
      if (!isInCard && !isInMenu) {
        // 鼠标不在卡片和菜单区域内，隐藏菜单
        if (hideMenuTimer) {
          clearTimeout(hideMenuTimer)
        }
        isHovered.value = false
        isMenuHovered = false
      }
    }
    document.addEventListener('mousemove', globalMouseMoveHandler)
  }
}

const removeGlobalMouseListener = () => {
  if (globalMouseMoveHandler) {
    document.removeEventListener('mousemove', globalMouseMoveHandler)
    globalMouseMoveHandler = null
  }
}

// 监听 isHovered 变化，动态添加/移除全局监听器
watch(isHovered, (newVal) => {
  if (newVal && !isMobile.value) {
    setupGlobalMouseListener()
  } else {
    removeGlobalMouseListener()
  }
})

onMounted(() => {
  checkIsMobile()
  if (typeof window !== 'undefined') {
    window.addEventListener('resize', handleResize)
  }
})

onUnmounted(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('resize', handleResize)
  }
  removeGlobalMouseListener()
  if (hideMenuTimer) {
    clearTimeout(hideMenuTimer)
  }
})
</script>

<style>
.card-link-wrapper {
  position: relative;
  width: 100%;
  max-width: 100%;
  box-sizing: border-box;
  min-width: 0;
}

.card-link {
  position: relative;
  display: flex;
  flex-direction: column;
  padding: 12px;
  border-radius: 12px;
  border: 1px solid;
  transition: all 0.3s;
  text-decoration: none;
  color: inherit;
  flex: 1;
  box-sizing: border-box;
  width: 100%;
  max-width: 100%;
  min-width: 0;
  overflow: hidden;
  height: 100%;
}

@media (min-width: 375px) {
  .card-link {
    padding: 14px;
    border-radius: 14px;
  }
}

@media (min-width: 640px) {
  .card-link {
    padding: 16px;
    border-radius: 16px;
  }
}

.card-link:hover {
  transform: translateY(-4px);
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
}

.dark-mode .card-link {
  background-color: rgba(30, 41, 59, 0.4);
  border-color: rgba(71, 85, 105, 0.5);
}

.dark-mode .card-link:hover {
  background-color: rgba(30, 41, 59, 0.8);
  border-color: rgba(59, 130, 246, 0.3);
}

.light-mode .card-link {
  background-color: rgba(255, 255, 255, 0.6);
  border-color: rgba(226, 232, 240, 0.6);
  box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
}

.light-mode .card-link:hover {
  background-color: white;
  border-color: rgba(147, 197, 253, 0.5);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
  gap: 8px;
}

@media (min-width: 375px) {
  .card-header {
    margin-bottom: 8px;
  }
}

.card-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
  overflow: hidden;
}

@media (min-width: 375px) {
  .card-title-row {
    gap: 10px;
  }
}

.card-icon-wrapper {
  padding: 5px;
  border-radius: 6px;
  flex-shrink: 0;
}

@media (min-width: 375px) {
  .card-icon-wrapper {
    padding: 6px;
    border-radius: 8px;
  }
}

.dark-mode .card-icon-wrapper {
  background-color: rgba(15, 23, 42, 0.5);
}

.light-mode .card-icon-wrapper {
  background-color: #f1f5f9;
}

.card-icon {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  object-fit: contain;
  display: block;
}

@media (min-width: 375px) {
  .card-icon {
    width: 20px;
    height: 20px;
  }
}

.card-icon-lucide {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  color: inherit;
}

@media (min-width: 375px) {
  .card-icon-lucide {
    width: 20px;
    height: 20px;
  }
}

.card-icon-fallback {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.5;
  color: inherit;
}

.card-external-icon {
  opacity: 0;
  transition: opacity 0.2s;
  color: inherit;
  flex-shrink: 0;
}

.card-link:hover .card-external-icon {
  opacity: 0.5;
}

.card-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.card-title {
  font-weight: 600;
  font-size: 14px;
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.5;
  flex: 1;
  min-width: 0;
}

@media (min-width: 375px) {
  .card-title {
    font-size: 15px;
  }
}

@media (min-width: 640px) {
  .card-title {
    font-size: 16px;
  }
}

.card-desc {
  font-size: 11px;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  margin: 0;
  word-wrap: break-word;
  overflow-wrap: break-word;
  min-height: calc(1.5em * 2);
  flex-shrink: 0;
  width: 100%;
  box-sizing: border-box;
}

.card-desc-empty {
  opacity: 0.5;
}

@media (min-width: 375px) {
  .card-desc {
    font-size: 12px;
    min-height: calc(1.5em * 2);
  }
}

.dark-mode .card-desc {
  color: #94a3b8;
}

.light-mode .card-desc {
  color: #64748b;
}

/* 移动端展开按钮 */
.card-expand-btn {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  transition: all 0.2s;
  z-index: 5;
}

@media (min-width: 375px) {
  .card-expand-btn {
    top: 14px;
    right: 14px;
    width: 32px;
    height: 32px;
    border-radius: 8px;
  }
}

.dark-mode .card-expand-btn {
  background-color: rgba(30, 41, 59, 0.6);
  color: #94a3b8;
}

.dark-mode .card-expand-btn:active {
  background-color: rgba(30, 41, 59, 0.8);
  transform: scale(0.95);
}

.light-mode .card-expand-btn {
  background-color: rgba(255, 255, 255, 0.8);
  color: #64748b;
}

.light-mode .card-expand-btn:active {
  background-color: rgba(255, 255, 255, 1);
  transform: scale(0.95);
}

@media (min-width: 768px) {
  .card-expand-btn {
    display: none;
  }
}

/* 桌面端子链接菜单 */
.card-sublinks-menu {
  position: fixed;
  z-index: 100;
  width: 280px;
  max-height: 400px;
  border-radius: 12px;
  backdrop-filter: blur(12px);
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  border: 1px solid;
  animation: fadeInUp 0.2s ease-out;
  overflow: hidden;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.dark-mode .card-sublinks-menu {
  background-color: rgba(30, 41, 59, 0.95);
  border-color: rgba(71, 85, 105, 0.5);
}

.light-mode .card-sublinks-menu {
  background-color: rgba(255, 255, 255, 0.95);
  border-color: rgba(226, 232, 240, 0.8);
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
}

.sublinks-menu-header {
  padding: 12px 16px;
  border-bottom: 1px solid;
}

.dark-mode .sublinks-menu-header {
  border-bottom-color: rgba(71, 85, 105, 0.5);
}

.light-mode .sublinks-menu-header {
  border-bottom-color: rgba(226, 232, 240, 0.8);
}

.sublinks-menu-title {
  font-size: 14px;
  font-weight: 600;
  color: inherit;
}

.sublinks-menu-list {
  max-height: 360px;
  overflow-y: auto;
  padding: 4px 8px 4px 4px;
}

.sublinks-menu-list::-webkit-scrollbar {
  width: 6px;
}

.sublinks-menu-list::-webkit-scrollbar-track {
  background: transparent;
}

.dark-mode .sublinks-menu-list::-webkit-scrollbar-thumb {
  background-color: rgba(71, 85, 105, 0.5);
  border-radius: 3px;
}

.light-mode .sublinks-menu-list::-webkit-scrollbar-thumb {
  background-color: rgba(226, 232, 240, 0.8);
  border-radius: 3px;
}

.sublink-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 8px;
  text-decoration: none;
  color: inherit;
  transition: all 0.2s;
  position: relative;
}

.sublink-item:hover {
  transform: translateX(4px);
}

.dark-mode .sublink-item:hover {
  background-color: rgba(59, 130, 246, 0.15);
}

.light-mode .sublink-item:hover {
  background-color: rgba(59, 130, 246, 0.1);
}

.sublink-icon-wrapper {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
}

.dark-mode .sublink-icon-wrapper {
  background-color: rgba(15, 23, 42, 0.5);
}

.light-mode .sublink-icon-wrapper {
  background-color: #f1f5f9;
}

.sublink-icon {
  width: 16px;
  height: 16px;
  border-radius: 4px;
  object-fit: contain;
  display: block;
}

.sublink-icon-lucide {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
  color: inherit;
}

.sublink-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.sublink-name {
  font-size: 14px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sublink-external-icon {
  flex-shrink: 0;
  opacity: 0.4;
  transition: opacity 0.2s;
}

.sublink-item:hover .sublink-external-icon {
  opacity: 0.7;
}

/* 移动端内联子链接 */
.card-sublinks-inline {
  margin-top: 8px;
  border-radius: 12px;
  border: 1px solid;
  overflow: hidden;
  animation: slideDown 0.2s ease-out;
}

@keyframes slideDown {
  from {
    opacity: 0;
    max-height: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    max-height: 1000px;
    transform: translateY(0);
  }
}

.dark-mode .card-sublinks-inline {
  background-color: rgba(30, 41, 59, 0.4);
  border-color: rgba(71, 85, 105, 0.5);
}

.light-mode .card-sublinks-inline {
  background-color: rgba(255, 255, 255, 0.6);
  border-color: rgba(226, 232, 240, 0.6);
}

.sublinks-inline-list {
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sublink-item-inline {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 8px;
  text-decoration: none;
  color: inherit;
  transition: all 0.2s;
}

.sublink-item-inline:active {
  transform: scale(0.98);
}

.dark-mode .sublink-item-inline {
  background-color: rgba(15, 23, 42, 0.3);
}

.dark-mode .sublink-item-inline:active {
  background-color: rgba(59, 130, 246, 0.15);
}

.light-mode .sublink-item-inline {
  background-color: rgba(255, 255, 255, 0.4);
}

.light-mode .sublink-item-inline:active {
  background-color: rgba(59, 130, 246, 0.1);
}

/* 移动端隐藏桌面端悬浮菜单 */
@media (max-width: 767px) {
  .card-sublinks-menu {
    display: none;
  }
}
</style>
