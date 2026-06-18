<template>
  <div class="nav-dashboard" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <!-- 背景光效 -->
    <div class="bg-effects">
      <div class="bg-effect bg-effect-1"></div>
      <div class="bg-effect bg-effect-2"></div>
    </div>

    <!-- 移动端菜单遮罩 -->
    <div v-if="isMobileMenuOpen" class="mobile-overlay" @click="isMobileMenuOpen = false"></div>

    <!-- 左侧边栏 -->
    <aside class="sidebar" :class="{ 'sidebar-open': isMobileMenuOpen }">
      <!-- Sidebar Header (Logo / Brand) -->
      <div class="sidebar-header">
        <div class="sidebar-brand" @click="goHome" style="cursor: pointer;">
          <img :src="getAssetPath('/logo.png')" alt="Newbie Space" class="sidebar-logo" />
          <h1 class="sidebar-title">Newbie Space</h1>
        </div>
      </div>

      <!-- Navigation Links -->
      <nav class="sidebar-nav">
        <button v-for="cat in displayData" :key="cat.name" class="nav-button"
          :class="{ 'nav-button-active': activeCategory === cat.name }" @click="scrollToCategory(cat.name)">
          <component :is="getIconComponent(cat.iconName)" :size="18" class="nav-icon" />
          <span>{{ cat.name }}</span>
        </button>
      </nav>

      <!-- Sidebar Footer (Home & Recent Toggle) -->
      <div class="sidebar-footer">
        <button class="home-button" @click="goHome">
          <Home :size="16" class="theme-icon" />
          <span class="theme-text">返回首页</span>
        </button>
        <button v-if="hasClickRecords && !isMobile" class="recent-toggle" @click="toggleRecentUsed">
          <component :is="showRecentUsed ? EyeOff : Eye" :size="16" class="theme-icon" />
          <span class="theme-text">{{ showRecentUsed ? '隐藏近期使用' : '显示近期使用' }}</span>
        </button>
      </div>
    </aside>

    <!-- 右侧主内容区域 -->
    <main class="main-content">
      <!-- 右上角主题切换按钮（桌面端） -->
      <button class="desktop-theme-toggle" @click="toggleDark()" title="切换主题">
        <component :is="isDark ? Sun : Moon" :size="20" />
      </button>

      <!-- 移动端入口按钮（头部隐藏时显示） -->
      <Transition name="mobile-entry-fade">
        <div v-if="!showMobileHeader && !showCompactHeader && !isMobileMenuOpen" class="mobile-entry-buttons">
          <button class="mobile-entry-menu-btn" @click="isMobileMenuOpen = true" title="打开菜单">
            <Menu :size="20" />
          </button>
          <button class="mobile-entry-theme-btn" @click="toggleDark()" title="切换主题">
            <component :is="isDark ? Sun : Moon" :size="20" />
          </button>
        </div>
      </Transition>

      <!-- 顶部移动端 Header -->
      <Transition name="mobile-header-fade">
        <div v-if="showMobileHeader && !isMobileMenuOpen" class="mobile-header">
          <button class="mobile-menu-btn" @click="isMobileMenuOpen = true">
            <Menu :size="20" />
          </button>
          <span class="mobile-title">{{ activeCategory }}</span>
          <div class="mobile-header-actions">
            <button class="mobile-theme-toggle" @click="toggleDark()" title="切换主题">
              <component :is="isDark ? Sun : Moon" :size="20" />
            </button>
          </div>
        </div>
      </Transition>

      <!-- 移动端紧凑头部（滚动时显示） -->
      <Transition name="compact-header-slide">
        <div v-if="showCompactHeader && !isMobileMenuOpen" class="mobile-compact-header">
          <button class="compact-menu-btn" @click="isMobileMenuOpen = true">
            <Menu :size="18" />
          </button>
          <div class="compact-header-content">
            <div class="compact-time">{{ formattedTime }}</div>
            <div class="compact-location" v-if="ipData">
              <MapPin :size="10" />
              <span>{{ ipData.city || 'Unknown' }}</span>
            </div>
          </div>
          <button class="compact-theme-toggle" @click="toggleDark()" title="切换主题">
            <component :is="isDark ? Sun : Moon" :size="18" />
          </button>
        </div>
      </Transition>

      <div class="content-wrapper" style="position: relative;">
        <!-- Header Area: Time & Search -->
        <div ref="headerAreaRef" class="header-area">
          <!-- Time -->
          <div class="time-display">
            <h1 class="time-text">{{ formattedTime }}</h1>
            <p class="date-text">{{ formattedDate }}</p>
          </div>

          <!-- IP Location Widget -->
          <div ref="ipLocationWidgetRef" class="ip-location-widget">
            <MapPin :size="12" class="ip-location-icon" />
            <Transition name="location-fade" mode="out-in">
              <div v-if="ipData" key="content" class="ip-location-content">
                <span v-if="ipData.city === (ipData.country_name || ipData.country)">{{ ipData.city || 'Unknown'
                  }}</span>
                <span v-else>{{ ipData.city || 'Unknown' }}, {{ ipData.country_name || ipData.country || 'Unknown'
                  }}</span>
                <span class="ip-location-divider"></span>
                <span class="ip-location-ip">{{ ipData.ip }}</span>
              </div>
              <span v-else key="loading" class="ip-location-loading">定位中...</span>
            </Transition>
          </div>

          <!-- Large Search Bar -->
          <div class="search-container">
            <div class="search-bg"></div>
            <form class="search-form" @submit.prevent="handleSearch">
              <select v-model="searchEngine" class="search-select">
                <option value="google">Google</option>
                <option value="baidu">Baidu</option>
                <option value="bing">Bing</option>
              </select>
              <div class="search-divider"></div>
              <input v-model="searchQuery" type="text" placeholder="Search..." class="search-input" />
              <button type="submit" class="search-button">
                <Search :size="20" />
              </button>
            </form>
          </div>
        </div>

        <!-- Categories Sections -->
        <div class="categories-container">
          <section v-for="category in displayData" :key="category.name" :ref="el => setCategoryRef(category.name, el)"
            class="category-section">
            <!-- Section Title -->
            <div class="section-title">
              <div class="section-icon-wrapper">
                <component :is="getIconComponent(category.iconName)" :size="20" class="section-icon" />
              </div>
              <h2 class="section-name">{{ category.name }}</h2>
              <span class="section-count">{{ category.links.length }}</span>
            </div>

            <!-- Cards Grid -->
            <div class="cards-grid">
              <div v-for="(link, idx) in category.links" :key="idx" class="card-wrapper" :class="{
                'recent-card': category.name === '近期使用'
              }">
                <div class="card-link-container">
                  <CardLink :link="link" @click="handleLinkClick" />
                </div>
                <button v-if="category.name === '近期使用'" class="card-delete-btn"
                  @click.stop="removeFromRecent(link.link)" :title="'删除 ' + link.name">
                  <Trash2 :size="14" />
                </button>
              </div>
            </div>
          </section>
        </div>
        <!-- 页脚 -->
        <AppFooter />
      </div>
    </main>

    <!-- 非大陆网络提醒对话框 -->
    <Dialog v-model="showNetworkWarning" title="需要非大陆网络" message="检测到您当前位于中国大陆，该网站需要非大陆网络才能正常访问。"
      sub-message="如果您已配置代理或 VPN，可以继续访问；否则可能无法正常使用。" :icon="AlertTriangle" cancel-text="取消" confirm-text="继续访问"
      @confirm="confirmNavigation" @cancel="cancelNavigation" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick, h, createApp } from 'vue'
import { useRouter } from 'vue-router'
import { useTheme, useData } from '@/composables/useTheme'
import { getAssetPath, getBasePath } from '@/utils/path'
import navData from '@/data/nav-data.json'
import * as lucideIcons from 'lucide-vue-next'
import { Sun, Moon, MapPin, Eye, EyeOff, Trash2, Globe, Menu, Search, Home, AlertTriangle } from 'lucide-vue-next'
import Dialog from '@/components/Dialog.vue'

// 处理数据，从配置中读取图标名称
const rawData = navData.map((cat: any) => {
  const iconName = cat.icon || 'layout-grid' // 从配置中读取 icon，如果没有则使用默认值 'layout-grid'
  return {
    ...cat,
    iconName
  }
})

// 自有工具完整列表（用于动态填充子菜单）
const selfToolsList = [
  { name: '背景透明化', icon: 'image-off', link: '/tools/background-transparent', desc: '将白色或指定颜色背景转换为透明 PNG' },
  { name: '图片尺寸调整', icon: 'crop', link: '/tools/image-resize', desc: '自由调整图片宽度和高度' },
  { name: '二维码生成', icon: 'qr-code', link: '/tools/qrcode-gen', desc: '输入文字或链接生成二维码' },
  { name: '二维码反解析', icon: 'scan-line', link: '/tools/qrcode-decode', desc: '上传二维码图片，解析其中内容' },
  { name: 'URL 编解码', icon: 'link-2', link: '/tools/url-encoder', desc: '对文本进行 URL 编码或解码' },
  { name: '文本哈希', icon: 'fingerprint', link: '/tools/hash-text', desc: '计算 MD5/SHA 等哈希值' },
  { name: '图片格式转换', icon: 'shuffle', link: '/tools/format-convert', desc: '将图片转换为 PNG/JPEG/WebP' },
  { name: '时间戳转换', icon: 'clock', link: '/tools/date-converter', desc: '日期字符串与 Unix 时间戳互转' },
  { name: '颜色转换', icon: 'palette', link: '/tools/color-converter', desc: '颜色值多格式互转' },
  { name: '随机端口生成', icon: 'network', link: '/tools/random-port', desc: '随机生成合法端口号' },
  { name: '文本统计', icon: 'bar-chart-3', link: '/tools/text-statistics', desc: '统计字符数、单词数、行数等' },
  { name: '文本差异对比', icon: 'git-compare', link: '/tools/text-diff', desc: '对比两段文本的差异' },
  { name: 'Docker Run 转换', icon: 'container', link: '/tools/docker-compose', desc: 'docker run 转 docker-compose.yml' }
]

// 使用自定义主题管理
const { isDark, toggleDark } = useTheme()
const { site } = useData()
const base = computed(() => site.value.base || getBasePath())
const router = useRouter()

// 本地存储键名
const STORAGE_KEY_CLICKS = 'nav-click-counts'
const STORAGE_KEY_SHOW_RECENT = 'nav-show-recent'
const TOP_N = 4 // 显示 TOP N 个链接

// 状态
const activeCategory = ref(rawData[0].name)
const searchEngine = ref('google')
const searchQuery = ref('')
const currentTime = ref(new Date())
const isMobileMenuOpen = ref(false)
const categoryRefs = ref<Record<string, HTMLElement>>({})
const ipLocationWidgetRef = ref<HTMLElement | null>(null)
const headerAreaRef = ref<HTMLElement | null>(null)
const ipData = ref<any>(null)
const showRecentUsed = ref(true) // 默认显示近期使用分类
const clickCounts = ref<Record<string, number>>({}) // 链接点击次数统计
const showMobileHeader = ref(false) // 移动端头部显示状态（默认隐藏）
const showCompactHeader = ref(false) // 紧凑头部显示状态
const scrollY = ref(0)
const isMobile = ref(false) // 是否为移动端
const showNetworkWarning = ref(false) // 是否显示网络警告对话框
const pendingLinkUrl = ref<string | null>(null) // 待跳转的链接
const pendingLinkName = ref<string>('') // 待跳转的链接名称

// 本地存储相关函数
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

const saveClickCounts = () => {
  if (typeof localStorage === 'undefined') return
  try {
    localStorage.setItem(STORAGE_KEY_CLICKS, JSON.stringify(clickCounts.value))
  } catch (e) {
    console.error('Failed to save click counts:', e)
  }
}

// 检查是否在中国大陆
const isInMainlandChina = (): boolean => {
  if (!ipData.value) {
    // 如果 IP 数据还未加载，默认不阻止（避免误判）
    return false
  }

  // 检查国家代码
  const countryCode = ipData.value.country_code || ipData.value.countryCode || ''
  const countryName = ipData.value.country_name || ipData.value.country || ''

  // 检查是否是中国（CN）或中国大陆
  if (countryCode === 'CN' || countryCode === 'cn') {
    return true
  }

  // 也检查国家名称（作为备用）
  if (countryName && (
    countryName.toLowerCase().includes('china') ||
    countryName.toLowerCase().includes('中国') ||
    countryName === 'CN'
  )) {
    return true
  }

  return false
}

// 处理链接点击
const handleLinkClick = (linkUrl: string, linkData?: any) => {
  // 内部路由导航（自有工具页面）
  if (linkUrl.startsWith('/')) {
    incrementClickCount(linkUrl)
    router.push(linkUrl)
    return
  }

  // 查找链接数据
  let link: any = linkData
  if (!link) {
    // 从所有分类中查找链接
    for (const cat of rawData) {
      const found = cat.links.find((l: any) => l.link === linkUrl)
      if (found) {
        link = found
        break
      }
      // 也检查子链接
      for (const l of cat.links) {
        if (l.subLinks) {
          const subFound = l.subLinks.find((sl: any) => sl.link === linkUrl)
          if (subFound) {
            link = subFound
            break
          }
        }
      }
      if (link) break
    }
  }

  // 检查是否需要非大陆网络
  if (link && link.requiresNonMainlandNetwork) {
    // 检查是否在中国大陆
    if (isInMainlandChina()) {
      // 显示警告对话框
      pendingLinkUrl.value = linkUrl
      pendingLinkName.value = link.name || '该网站'
      showNetworkWarning.value = true
      return // 阻止默认跳转
    }
  }

  // 正常跳转
  incrementClickCount(linkUrl)
  window.open(linkUrl, '_blank', 'noopener,noreferrer')
}

// 确认导航（继续访问）
const confirmNavigation = () => {
  if (pendingLinkUrl.value) {
    incrementClickCount(pendingLinkUrl.value)
    window.open(pendingLinkUrl.value, '_blank', 'noopener,noreferrer')
  }
  pendingLinkUrl.value = null
  pendingLinkName.value = ''
}

// 取消导航
const cancelNavigation = () => {
  pendingLinkUrl.value = null
  pendingLinkName.value = ''
}

const incrementClickCount = (linkUrl: string) => {
  clickCounts.value[linkUrl] = (clickCounts.value[linkUrl] || 0) + 1
  saveClickCounts()
}

const removeFromRecent = (linkUrl: string) => {
  if (clickCounts.value[linkUrl]) {
    delete clickCounts.value[linkUrl]
    saveClickCounts()
  }
}


const loadShowRecentSetting = () => {
  if (typeof localStorage === 'undefined') return
  try {
    const stored = localStorage.getItem(STORAGE_KEY_SHOW_RECENT)
    if (stored !== null) {
      showRecentUsed.value = JSON.parse(stored)
    }
  } catch (e) {
    console.error('Failed to load show recent setting:', e)
  }
}

const saveShowRecentSetting = () => {
  if (typeof localStorage === 'undefined') return
  try {
    localStorage.setItem(STORAGE_KEY_SHOW_RECENT, JSON.stringify(showRecentUsed.value))
  } catch (e) {
    console.error('Failed to save show recent setting:', e)
  }
}

// 获取所有链接的扁平列表
const getAllLinks = () => {
  return rawData.flatMap(cat =>
    cat.links.map(link => ({
      ...link,
      categoryName: cat.name,
      categoryIcon: cat.iconName
    }))
  )
}

// 计算"近期使用"分类
const recentUsedCategory = computed(() => {
  // 移动端不显示近期使用
  if (isMobile.value) return null
  if (!showRecentUsed.value) return null

  const allLinks = getAllLinks()
  const linksWithCounts = allLinks
    .map(link => ({
      ...link,
      clickCount: clickCounts.value[link.link] || 0
    }))
    .filter(link => link.clickCount > 0) // 只显示被点击过的链接
    .sort((a, b) => b.clickCount - a.clickCount) // 按点击次数降序排序
    .slice(0, TOP_N) // 只取 TOP N

  if (linksWithCounts.length === 0) return null

  return {
    name: '近期使用',
    title: 'Recent Used',
    icon: 'clock',
    iconName: 'clock',
    links: linksWithCounts.map(({ categoryName, categoryIcon, clickCount, ...link }) => link)
  }
})

// 检查是否有任何点击记录
const hasClickRecords = computed(() => {
  return Object.keys(clickCounts.value).length > 0 &&
    Object.values(clickCounts.value).some(count => count > 0)
})

// 自有工具的常用子菜单（按点击次数排序，最多 8 个）
const recentSelfToolsSubLinks = computed(() => {
  const recent = selfToolsList
    .map(t => ({ ...t, clickCount: clickCounts.value[t.link] || 0 }))
    .filter(t => t.clickCount > 0)
    .sort((a, b) => b.clickCount - a.clickCount)
    .slice(0, 8)
    .map(({ clickCount, ...t }) => t)
  return recent.length > 0 ? recent : undefined
})

// 为"自有工具箱"卡片注入动态子菜单
const injectSelfToolsSubLinks = (link: any) => {
  if (link.name === '自有工具箱') {
    return { ...link, subLinks: recentSelfToolsSubLinks.value }
  }
  return link
}

// 合并后的分类数据（包含近期使用 + 动态子菜单）
const displayData = computed(() => {
  const categories = []
  if (recentUsedCategory.value) {
    const links = recentUsedCategory.value.links.map(injectSelfToolsSubLinks)
    categories.push({ ...recentUsedCategory.value, links })
  }

  for (const cat of rawData) {
    if (cat.name === '常用工具') {
      const links = cat.links.map(injectSelfToolsSubLinks)
      categories.push({ ...cat, links })
    } else {
      categories.push(cat)
    }
  }

  return categories
})

// 计算属性
const formattedTime = computed(() => {
  const date = currentTime.value
  const hours = String(date.getHours()).padStart(2, '0')
  const minutes = String(date.getMinutes()).padStart(2, '0')
  const seconds = String(date.getSeconds()).padStart(2, '0')
  return `${hours}:${minutes}:${seconds}`
})

const formattedDate = computed(() => {
  const date = currentTime.value
  const weekdays = ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六']
  const weekday = weekdays[date.getDay()]
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${weekday} ${year}-${month}-${day}`
})

// 方法
const setCategoryRef = (name: string, el: any) => {
  if (el && el instanceof HTMLElement) {
    categoryRefs.value[name] = el
  }
}

const goHome = () => {
  router.push('/')
}

const scrollToCategory = (name: string) => {
  activeCategory.value = name
  isMobileMenuOpen.value = false
  const element = categoryRefs.value[name]
  if (element) {
    // 计算头部高度偏移量（移动端需要考虑头部占位）
    const headerOffset = isMobile.value ? 76 : 0 // 移动端头部高度约 56px，加上一些间距
    const elementPosition = element.getBoundingClientRect().top + window.pageYOffset
    const offsetPosition = elementPosition - headerOffset

    window.scrollTo({
      top: offsetPosition,
      behavior: 'smooth'
    })

    // 如果是在 main-content 内滚动，需要调整
    const mainContent = document.querySelector('.main-content') as HTMLElement
    if (mainContent && isMobile.value) {
      const scrollTop = mainContent.scrollTop
      const elementRelativeTop = element.getBoundingClientRect().top - mainContent.getBoundingClientRect().top
      const targetScrollTop = scrollTop + elementRelativeTop - headerOffset

      mainContent.scrollTo({
        top: Math.max(0, targetScrollTop),
        behavior: 'smooth'
      })
    } else {
      element.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }
  }
}

const toggleRecentUsed = () => {
  showRecentUsed.value = !showRecentUsed.value
  saveShowRecentSetting()
  // 如果隐藏了近期使用分类，切换到第一个普通分类
  if (!showRecentUsed.value && activeCategory.value === '近期使用') {
    activeCategory.value = rawData[0].name
  }
}

const handleSearch = () => {
  if (!searchQuery.value.trim()) return
  let url = ''
  switch (searchEngine.value) {
    case 'google':
      url = `https://www.google.com/search?q=${encodeURIComponent(searchQuery.value)}`
      break
    case 'baidu':
      url = `https://www.baidu.com/s?wd=${encodeURIComponent(searchQuery.value)}`
      break
    case 'bing':
      url = `https://www.bing.com/search?q=${encodeURIComponent(searchQuery.value)}`
      break
    default:
      url = `https://www.google.com/search?q=${encodeURIComponent(searchQuery.value)}`
  }
  window.open(url, '_blank')
}

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
const getIconUrl = (link: any) => {
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
// 例如: "wrench" -> "Wrench", "message-square" -> "MessageSquare", "message" -> "MessageSquare"
const toPascalCase = (str: string): string => {
  return str
    .split(/[-_\s]+/)
    .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join('')
}

// 常见的图标名称映射（处理特殊情况，如 message -> MessageSquare）
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
    const link = target.closest('.card-link')
    if (link) {
      const linkElement = link as HTMLElement
      const linkData = rawData
        .flatMap(cat => cat.links)
        .find(l => l.link === linkElement.getAttribute('href'))

      if (linkData) {
        // 如果当前使用的是本地图标但加载失败，回退到 Google Favicon
        if (linkData.icon && linkData.icon.startsWith('/')) {
          target.src = `https://www.google.com/s2/favicons?domain=${getDomain(linkData.link)}&sz=64`
          return
        }
      }
    }

    // 如果所有回退都失败，显示默认图标（使用 lucide Globe 图标）
    // 使用 Vue 的 createApp 来创建并挂载 Globe 组件
    target.style.display = 'none'
    const fallback = document.createElement('div')
    fallback.className = 'card-icon-fallback'
    // 创建临时 Vue 应用来渲染 Globe 组件
    const app = createApp({
      render: () => h(Globe, { size: 24, class: 'card-icon-lucide' })
    })
    app.mount(fallback)
    target.parentNode?.replaceChild(fallback, target)
  }
}

// 时间更新
let timer: ReturnType<typeof setInterval> | null = null

// IP 获取 - 并发查询，使用最先响应的结果
const fetchIpData = async () => {
  const apis = [
    'https://ipwhois.app/json/',
    'https://get.geojs.io/v1/ip/geo.json',
    'https://ipinfo.io/json'
  ]

  // 创建一个包装函数，将每个API请求转换为 Promise
  // 无论成功还是失败都 resolve，以便使用 Promise.race
  const createRequest = async (api: string): Promise<{ success: boolean; data?: any; error?: any; api: string }> => {
    try {
      const response = await fetch(api)
      if (response.ok) {
        const data = await response.json()
        return { success: true, data, api }
      } else {
        return { success: false, error: new Error(`HTTP ${response.status}`), api }
      }
    } catch (error) {
      return { success: false, error, api }
    }
  }

  // 并发发起所有请求
  const requests = apis.map(api => createRequest(api))

  // 使用 Promise.race 循环获取最先完成的请求
  // 如果最先完成的是成功的，就使用它；否则继续等待下一个
  const pendingIndices = new Set(requests.map((_, index) => index))

  while (pendingIndices.size > 0) {
    // 获取所有待处理的请求
    const pendingRequests = Array.from(pendingIndices).map(index =>
      requests[index].then(result => ({ ...result, index }))
    )

    try {
      // 等待最先完成的请求
      const result = await Promise.race(pendingRequests)

      if (result.success) {
        // 找到最先成功的响应，使用它
        ipData.value = result.data
        console.log(`IP data fetched from ${result.api}`)
        return
      } else {
        // 这个请求失败了，从待处理列表中移除，继续等待其他请求
        pendingIndices.delete(result.index)
      }
    } catch (error) {
      // 如果出现错误，继续处理剩余的请求
      // 由于我们的包装函数不会 reject，这里理论上不会执行
      break
    }
  }

  // 如果所有请求都失败
  console.error('All IP APIs failed')
}

// 检查位置信息是否在视野中
const checkHeaderVisibility = () => {
  if (!ipLocationWidgetRef.value) return

  const locationWidget = ipLocationWidgetRef.value
  const rect = locationWidget.getBoundingClientRect()
  const mainContent = locationWidget.closest('.main-content') as HTMLElement

  if (!mainContent) return

  // 检查位置信息是否完全滚出视野
  const isLocationOutOfView = rect.bottom < 0

  // 检查是否在顶部（滚动位置很小）
  const isAtTop = mainContent.scrollTop < 50

  // 移动端才应用这个逻辑
  if (typeof window !== 'undefined' && window.innerWidth <= 768) {
    if (isAtTop) {
      // 在顶部时，隐藏所有头部（默认状态）
      showMobileHeader.value = false
      showCompactHeader.value = false
    } else if (isLocationOutOfView) {
      // 位置信息滚出视野时，隐藏普通头部，显示紧凑头部
      showMobileHeader.value = false
      showCompactHeader.value = true
    } else {
      // 位置信息还在视野中时，隐藏所有头部
      showMobileHeader.value = false
      showCompactHeader.value = false
    }
  } else {
    // 桌面端始终显示普通头部
    showMobileHeader.value = true
    showCompactHeader.value = false
  }
}

// 滚动事件处理
const handleScroll = () => {
  checkHeaderVisibility()
}

// 窗口大小变化处理
const handleResize = () => {
  checkHeaderVisibility()
}

let mainContentElement: HTMLElement | null = null

// 检测是否为移动端
const checkIsMobile = () => {
  if (typeof window === 'undefined') return
  isMobile.value = window.innerWidth <= 768
}

onMounted(() => {
  // 设置 body overflow 为 hidden，让 NavDashboard 内部的滚动容器处理滚动
  document.body.style.overflow = 'hidden'

  // 检测是否为移动端
  checkIsMobile()

  // 加载本地存储的数据
  loadClickCounts()
  loadShowRecentSetting()

  // 初始化 activeCategory（移动端不显示近期使用）
  if (!isMobile.value && showRecentUsed.value && recentUsedCategory.value) {
    activeCategory.value = '近期使用'
  } else {
    activeCategory.value = rawData[0].name
  }

  timer = setInterval(() => {
    currentTime.value = new Date()
  }, 1000)
  fetchIpData()

  // 添加滚动监听（移动端）
  mainContentElement = document.querySelector('.main-content') as HTMLElement
  if (mainContentElement) {
    mainContentElement.addEventListener('scroll', handleScroll, { passive: true })
    // 初始检查
    nextTick(() => {
      checkHeaderVisibility()
    })
  }

  // 监听窗口大小变化
  window.addEventListener('resize', () => {
    checkIsMobile()
    handleResize()
  }, { passive: true })
})

// 监听 ipData 变化，实现外框宽度平滑过渡
watch(ipData, async (newData, oldData) => {
  if (!ipLocationWidgetRef.value) return

  // 如果从无数据到有数据，或从有数据到无数据，触发宽度过渡
  if ((!oldData && newData) || (oldData && !newData)) {
    // 先获取当前宽度
    const currentWidth = ipLocationWidgetRef.value.offsetWidth

    // 等待 DOM 更新
    await nextTick()

    // 获取新内容的宽度
    const newWidth = ipLocationWidgetRef.value.scrollWidth

    // 如果宽度不同，设置过渡
    if (Math.abs(newWidth - currentWidth) > 1) {
      // 临时设置固定宽度以实现过渡
      ipLocationWidgetRef.value.style.width = `${currentWidth}px`

      // 强制重排
      ipLocationWidgetRef.value.offsetHeight

      // 设置新宽度，触发过渡
      requestAnimationFrame(() => {
        if (ipLocationWidgetRef.value) {
          ipLocationWidgetRef.value.style.width = `${newWidth}px`

          // 过渡完成后，恢复自动宽度
          setTimeout(() => {
            if (ipLocationWidgetRef.value) {
              ipLocationWidgetRef.value.style.width = ''
            }
          }, 500)
        }
      })
    }
  }
}, { flush: 'post' })

onUnmounted(() => {
  // 恢复 body overflow
  if (typeof document !== 'undefined') {
    document.body.style.overflow = ''
  }

  if (timer) {
    clearInterval(timer)
  }

  // 移除滚动监听
  if (mainContentElement) {
    mainContentElement.removeEventListener('scroll', handleScroll)
  }

  // 移除窗口大小监听
  if (typeof window !== 'undefined') {
    window.removeEventListener('resize', handleResize)
  }
})
</script>

<style scoped>
.nav-dashboard {
  display: flex;
  height: 100vh;
  overflow: hidden;
  transition: background-color 0.5s, color 0.5s;
}

.dark-mode {
  background-color: #0f172a;
  color: #e2e8f0;
}

.light-mode {
  background-color: #f8fafc;
  color: #1e293b;
}

/* 背景光效 */
.bg-effects {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
}

.bg-effect {
  position: absolute;
  border-radius: 50%;
  mix-blend-mode: screen;
  filter: blur(120px);
  opacity: 0.1;
}

.bg-effect-1 {
  top: 0;
  left: 0;
  width: 800px;
  height: 800px;
}

.dark-mode .bg-effect-1 {
  background-color: #2563eb;
}

.light-mode .bg-effect-1 {
  background-color: #60a5fa;
}

.bg-effect-2 {
  bottom: 0;
  right: 0;
  width: 600px;
  height: 600px;
  filter: blur(100px);
}

.dark-mode .bg-effect-2 {
  background-color: #9333ea;
}

.light-mode .bg-effect-2 {
  background-color: #a78bfa;
}

/* 移动端菜单遮罩 */
.mobile-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.5);
  z-index: 40;
  backdrop-filter: blur(4px);
}

@media (min-width: 768px) {
  .mobile-overlay {
    display: none;
  }
}

/* 左侧边栏 */
.sidebar {
  position: fixed;
  z-index: 50;
  width: 256px;
  height: 100%;
  display: flex;
  flex-direction: column;
  border-right: 1px solid;
  backdrop-filter: blur(12px);
  transition: transform 0.3s ease-in-out;
  transform: translateX(-100%);
}

.dark-mode .sidebar {
  background-color: rgba(15, 23, 42, 0.8);
  border-right-color: #1e293b;
}

.light-mode .sidebar {
  background-color: rgba(255, 255, 255, 0.8);
  border-right-color: #e2e8f0;
}

@media (min-width: 768px) {
  .sidebar {
    position: relative;
    transform: translateX(0);
  }
}

.sidebar-open {
  transform: translateX(0);
}

.sidebar-header {
  padding: 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 12px;
}

.sidebar-logo {
  width: 32px;
  height: 32px;
  object-fit: contain;
  flex-shrink: 0;
}

.sidebar-title {
  font-size: 24px;
  font-weight: 700;
  letter-spacing: -0.025em;
  background: linear-gradient(to right, #60a5fa, #a78bfa);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin: 0;
}

.sidebar-nav {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sidebar-nav::-webkit-scrollbar {
  display: none;
}

.sidebar-nav {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

.nav-button {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 12px;
  transition: all 0.2s;
  font-size: 14px;
  font-weight: 500;
  background: none;
  border: none;
  cursor: pointer;
  color: inherit;
  opacity: 0.7;
}

.nav-button:hover {
  opacity: 1;
}

.dark-mode .nav-button:hover {
  background-color: rgba(148, 163, 184, 0.1);
}

.light-mode .nav-button:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

.nav-button-active {
  opacity: 1;
}

.dark-mode .nav-button-active {
  background-color: rgba(37, 99, 235, 0.2);
  color: #93c5fd;
}

.light-mode .nav-button-active {
  background-color: #dbeafe;
  color: #1e40af;
}

.nav-icon {
  display: flex;
  align-items: center;
  width: 18px;
  height: 18px;
  flex-shrink: 0;
}

.sidebar-footer {
  padding: 16px;
  border-top: 1px solid;
}

.dark-mode .sidebar-footer {
  border-top-color: rgba(148, 163, 184, 0.1);
}

.light-mode .sidebar-footer {
  border-top-color: rgba(0, 0, 0, 0.1);
}

.theme-toggle {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 8px;
  border-radius: 8px;
  transition: background-color 0.2s;
  background: none;
  border: none;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  color: inherit;
}

.dark-mode .theme-toggle {
  background-color: #1e293b;
}

.dark-mode .theme-toggle:hover {
  background-color: #334155;
}

.light-mode .theme-toggle {
  background-color: #e2e8f0;
}

.light-mode .theme-toggle:hover {
  background-color: #cbd5e1;
}

.home-button {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 8px;
  border-radius: 8px;
  transition: background-color 0.2s;
  background: none;
  border: none;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  color: inherit;
  margin-bottom: 8px;
}

.dark-mode .home-button {
  background-color: #1e293b;
}

.dark-mode .home-button:hover {
  background-color: #334155;
}

.light-mode .home-button {
  background-color: #e2e8f0;
}

.light-mode .home-button:hover {
  background-color: #cbd5e1;
}

.recent-toggle {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 8px;
  border-radius: 8px;
  transition: background-color 0.2s;
  background: none;
  border: none;
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  color: inherit;
  margin-bottom: 8px;
}

.dark-mode .recent-toggle {
  background-color: #1e293b;
}

.dark-mode .recent-toggle:hover {
  background-color: #334155;
}

.light-mode .recent-toggle {
  background-color: #e2e8f0;
}

.light-mode .recent-toggle:hover {
  background-color: #cbd5e1;
}

.theme-icon {
  display: flex;
  align-items: center;
  width: 16px;
  height: 16px;
}

.theme-text {
  font-size: 12px;
}

/* 右侧主内容区域 */
.main-content {
  flex: 1;
  position: relative;
  overflow-y: auto;
  scroll-behavior: smooth;
}

/* 桌面端右上角主题切换按钮 */
.desktop-theme-toggle {
  position: fixed;
  top: 24px;
  right: 24px;
  z-index: 100;
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  border: none;
  cursor: pointer;
  transition: all 0.2s;
  backdrop-filter: blur(12px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.dark-mode .desktop-theme-toggle {
  background-color: rgba(30, 41, 59, 0.8);
  color: #e2e8f0;
  border: 1px solid rgba(148, 163, 184, 0.2);
}

.dark-mode .desktop-theme-toggle:hover {
  background-color: rgba(30, 41, 59, 0.95);
  transform: scale(1.05);
}

.light-mode .desktop-theme-toggle {
  background-color: rgba(255, 255, 255, 0.8);
  color: #1e293b;
  border: 1px solid rgba(226, 232, 240, 0.6);
}

.light-mode .desktop-theme-toggle:hover {
  background-color: rgba(255, 255, 255, 0.95);
  transform: scale(1.05);
}

@media (max-width: 767px) {
  .desktop-theme-toggle {
    display: none;
  }
}

.mobile-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  position: sticky;
  top: 0;
  z-index: 30;
  backdrop-filter: blur(12px);
  border-bottom: 1px solid;
  min-height: 56px;
}

.dark-mode .mobile-header {
  background-color: rgba(15, 23, 42, 0.8);
  border-bottom-color: rgba(148, 163, 184, 0.1);
}

.light-mode .mobile-header {
  background-color: rgba(255, 255, 255, 0.8);
  border-bottom-color: rgba(0, 0, 0, 0.1);
}

@media (min-width: 768px) {
  .mobile-header {
    display: none;
  }
}

/* 移动端入口按钮（头部隐藏时显示） */
.mobile-entry-buttons {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  z-index: 10;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  pointer-events: none;
}

@media (min-width: 768px) {
  .mobile-entry-buttons {
    display: none;
  }
}

.mobile-entry-menu-btn,
.mobile-entry-theme-btn {
  width: 40px;
  height: 40px;
  padding: 0;
  background-color: rgba(148, 163, 184, 0.15);
  backdrop-filter: blur(12px);
  border-radius: 10px;
  border: none;
  cursor: pointer;
  color: inherit;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  pointer-events: auto;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.dark-mode .mobile-entry-menu-btn,
.dark-mode .mobile-entry-theme-btn {
  background-color: rgba(30, 41, 59, 0.8);
  border: 1px solid rgba(148, 163, 184, 0.2);
}

.light-mode .mobile-entry-menu-btn,
.light-mode .mobile-entry-theme-btn {
  background-color: rgba(255, 255, 255, 0.8);
  border: 1px solid rgba(226, 232, 240, 0.6);
}

.mobile-entry-menu-btn:hover,
.mobile-entry-theme-btn:hover {
  transform: scale(1.05);
  background-color: rgba(148, 163, 184, 0.25);
}

.dark-mode .mobile-entry-menu-btn:hover,
.dark-mode .mobile-entry-theme-btn:hover {
  background-color: rgba(30, 41, 59, 0.95);
}

.light-mode .mobile-entry-menu-btn:hover,
.light-mode .mobile-entry-theme-btn:hover {
  background-color: rgba(255, 255, 255, 0.95);
}

/* 入口按钮过渡动画 */
.mobile-entry-fade-enter-active,
.mobile-entry-fade-leave-active {
  transition: opacity 0.3s ease;
}

.mobile-entry-fade-enter-from,
.mobile-entry-fade-leave-to {
  opacity: 0;
}

/* 移动端紧凑头部（滚动时显示） */
.mobile-compact-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 100;
  backdrop-filter: blur(12px);
  border-bottom: 1px solid;
  min-height: 56px;
}

.dark-mode .mobile-compact-header {
  background-color: rgba(15, 23, 42, 0.95);
  border-bottom-color: rgba(148, 163, 184, 0.1);
}

.light-mode .mobile-compact-header {
  background-color: rgba(255, 255, 255, 0.95);
  border-bottom-color: rgba(0, 0, 0, 0.1);
}

@media (min-width: 768px) {
  .mobile-compact-header {
    display: none;
  }
}

.compact-menu-btn,
.compact-theme-toggle {
  width: 40px;
  height: 40px;
  padding: 0;
  background-color: rgba(148, 163, 184, 0.15);
  backdrop-filter: blur(12px);
  border-radius: 10px;
  border: none;
  cursor: pointer;
  color: inherit;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.dark-mode .compact-menu-btn,
.dark-mode .compact-theme-toggle {
  background-color: rgba(30, 41, 59, 0.8);
  border: 1px solid rgba(148, 163, 184, 0.2);
}

.light-mode .compact-menu-btn,
.light-mode .compact-theme-toggle {
  background-color: rgba(255, 255, 255, 0.8);
  border: 1px solid rgba(226, 232, 240, 0.6);
}

.compact-menu-btn:hover,
.compact-theme-toggle:hover {
  transform: scale(1.05);
  background-color: rgba(148, 163, 184, 0.25);
}

.dark-mode .compact-menu-btn:hover,
.dark-mode .compact-theme-toggle:hover {
  background-color: rgba(30, 41, 59, 0.95);
}

.light-mode .compact-menu-btn:hover,
.light-mode .compact-theme-toggle:hover {
  background-color: rgba(255, 255, 255, 0.95);
}

.compact-header-content {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
  justify-content: center;
}

.compact-time {
  font-size: 16px;
  font-weight: 600;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  white-space: nowrap;
}

.compact-location {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  opacity: 0.7;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 120px;
}

/* 头部过渡动画 */
.mobile-header-fade-enter-active,
.mobile-header-fade-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.mobile-header-fade-enter-from {
  opacity: 0;
  transform: translateY(-100%);
}

.mobile-header-fade-leave-to {
  opacity: 0;
  transform: translateY(-100%);
}

.compact-header-slide-enter-active,
.compact-header-slide-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.compact-header-slide-enter-from {
  opacity: 0;
  transform: translateY(-100%);
}

.compact-header-slide-leave-to {
  opacity: 0;
  transform: translateY(-100%);
}

.mobile-menu-btn {
  width: 40px;
  height: 40px;
  padding: 0;
  background-color: rgba(148, 163, 184, 0.15);
  backdrop-filter: blur(12px);
  border-radius: 10px;
  border: none;
  cursor: pointer;
  color: inherit;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.dark-mode .mobile-menu-btn {
  background-color: rgba(30, 41, 59, 0.8);
  border: 1px solid rgba(148, 163, 184, 0.2);
}

.light-mode .mobile-menu-btn {
  background-color: rgba(255, 255, 255, 0.8);
  border: 1px solid rgba(226, 232, 240, 0.6);
}

.mobile-menu-btn:hover {
  transform: scale(1.05);
  background-color: rgba(148, 163, 184, 0.25);
}

.dark-mode .mobile-menu-btn:hover {
  background-color: rgba(30, 41, 59, 0.95);
}

.light-mode .mobile-menu-btn:hover {
  background-color: rgba(255, 255, 255, 0.95);
}

.mobile-title {
  font-size: 16px;
  font-weight: 700;
  flex: 1;
  text-align: center;
}

.mobile-header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.mobile-theme-toggle {
  width: 40px;
  height: 40px;
  padding: 0;
  background-color: rgba(148, 163, 184, 0.15);
  backdrop-filter: blur(12px);
  border-radius: 10px;
  border: none;
  cursor: pointer;
  color: inherit;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.dark-mode .mobile-theme-toggle {
  background-color: rgba(30, 41, 59, 0.8);
  border: 1px solid rgba(148, 163, 184, 0.2);
}

.light-mode .mobile-theme-toggle {
  background-color: rgba(255, 255, 255, 0.8);
  border: 1px solid rgba(226, 232, 240, 0.6);
}

.mobile-theme-toggle:hover {
  transform: scale(1.05);
  background-color: rgba(148, 163, 184, 0.25);
}

.dark-mode .mobile-theme-toggle:hover {
  background-color: rgba(30, 41, 59, 0.95);
}

.light-mode .mobile-theme-toggle:hover {
  background-color: rgba(255, 255, 255, 0.95);
}

.content-wrapper {
  max-width: 1280px;
  margin: 0 auto;
  padding: 32px 16px;
}

@media (min-width: 640px) {
  .content-wrapper {
    padding: 48px 32px;
  }
}

/* Header Area */
.header-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  margin-bottom: 64px;
  gap: 32px;
  animation: fade-in 0.6s ease-out forwards;
}

@keyframes fade-in {
  from {
    opacity: 0;
    transform: translateY(10px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.time-display {
  text-align: center;
}

.time-text {
  font-size: 48px;
  font-weight: 300;
  letter-spacing: -0.025em;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  margin-bottom: 8px;
  line-height: 1;
}

@media (min-width: 768px) {
  .time-text {
    font-size: 80px;
  }
}

.date-text {
  font-size: 12px;
  opacity: 0.6;
  text-transform: uppercase;
  letter-spacing: 0.2em;
  font-weight: 400;
}

@media (min-width: 768px) {
  .date-text {
    font-size: 14px;
  }
}

/* IP Location Widget */
.ip-location-widget {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 6px 16px;
  border-radius: 9999px;
  font-size: 12px;
  font-weight: 500;
  backdrop-filter: blur(12px);
  border: 1px solid;
  box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  transition: width 0.5s cubic-bezier(0.4, 0, 0.2, 1),
    max-width 0.5s cubic-bezier(0.4, 0, 0.2, 1),
    transform 0.3s,
    background-color 0.3s,
    border-color 0.3s;
  user-select: none;
  max-width: 100%;
  overflow: hidden;
  width: auto;
  min-width: 0;
  will-change: width;
}

.ip-location-widget:hover {
  transform: scale(1.05);
}

.dark-mode .ip-location-widget {
  background-color: rgba(30, 41, 59, 0.4);
  border-color: rgba(71, 85, 105, 0.5);
  color: #94a3b8;
}

.dark-mode .ip-location-widget:hover {
  background-color: rgba(30, 41, 59, 0.6);
}

.light-mode .ip-location-widget {
  background-color: rgba(255, 255, 255, 0.6);
  border-color: rgba(226, 232, 240, 0.6);
  color: #64748b;
}

.light-mode .ip-location-widget:hover {
  background-color: rgba(255, 255, 255, 0.8);
}

.ip-location-icon {
  flex-shrink: 0;
}

.dark-mode .ip-location-icon {
  color: #fb7185;
}

.light-mode .ip-location-icon {
  color: #ef4444;
}

.ip-location-content {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  flex: 1;
  overflow: hidden;
  white-space: nowrap;
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.ip-location-divider {
  width: 1px;
  height: 12px;
}

.dark-mode .ip-location-divider {
  background-color: rgba(255, 255, 255, 0.1);
}

.light-mode .ip-location-divider {
  background-color: rgba(0, 0, 0, 0.1);
}

.ip-location-ip {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
  opacity: 0.8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex-shrink: 1;
  min-width: 0;
}

.ip-location-loading {
  opacity: 0.7;
  white-space: nowrap;
  transition: opacity 0.3s ease, transform 0.3s ease;
  display: inline-block;
}

/* 定位信息切换过渡动画 */
.location-fade-enter-active {
  transition: opacity 0.4s ease 0.1s, transform 0.4s ease 0.1s;
}

.location-fade-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.location-fade-enter-from {
  opacity: 0;
  transform: translateX(-8px) scale(0.95);
}

.location-fade-leave-to {
  opacity: 0;
  transform: translateX(8px) scale(0.95);
}

.location-fade-enter-to,
.location-fade-leave-from {
  opacity: 1;
  transform: translateX(0) scale(1);
}

/* 移动端 IP Location Widget 响应式调整 */
@media (max-width: 768px) {
  .ip-location-widget {
    padding: 6px 12px;
    font-size: 11px;
    max-width: calc(100vw - 32px);
  }

  .ip-location-content {
    gap: 6px;
  }

  /* 在小屏幕上，如果空间不足，隐藏IP地址 */
  .ip-location-ip {
    display: none;
  }

  .ip-location-divider {
    display: none;
  }
}

/* 在中等屏幕上，如果空间仍然不足，也隐藏IP地址 */
@media (max-width: 480px) {
  .ip-location-widget {
    padding: 6px 10px;
    font-size: 10px;
  }
}

/* Search Bar */
.search-container {
  width: 100%;
  max-width: 640px;
  position: relative;
  transition: transform 0.3s;
}

.search-container:focus-within {
  transform: scale(1.02);
}

.search-bg {
  position: absolute;
  inset: 0;
  border-radius: 16px;
  filter: blur(8px);
  opacity: 0.2;
  transition: opacity 0.3s;
}

.search-container:hover .search-bg {
  opacity: 0.4;
}

.search-container:focus-within .search-bg {
  opacity: 0.5;
}

.dark-mode .search-bg {
  background: linear-gradient(to right, #3b82f6, #9333ea);
}

.light-mode .search-bg {
  background: linear-gradient(to right, #60a5fa, #a78bfa);
}

.search-form {
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;
  padding: 6px;
  border-radius: 16px;
  border: 1px solid;
  backdrop-filter: blur(12px);
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
}

.dark-mode .search-form {
  background-color: rgba(30, 41, 59, 0.8);
  border-color: #475569;
  color: white;
}

.light-mode .search-form {
  background-color: rgba(255, 255, 255, 0.9);
  border-color: #e2e8f0;
  color: #1e293b;
}

.search-select {
  background: transparent;
  font-size: 14px;
  font-weight: 500;
  padding: 12px 8px 12px 16px;
  outline: none;
  border: none;
  cursor: pointer;
  color: inherit;
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
}

.dark-mode .search-select {
  color: #e2e8f0 !important;
  background-color: transparent;
}

.dark-mode .search-select option {
  background-color: #1e293b !important;
  color: #e2e8f0 !important;
}

.light-mode .search-select {
  color: #1e293b !important;
  background-color: transparent;
}

.light-mode .search-select option {
  background-color: #ffffff !important;
  color: #1e293b !important;
}

.search-divider {
  width: 1px;
  height: 24px;
  margin: 0 8px;
}

.dark-mode .search-divider {
  background-color: rgba(148, 163, 184, 0.2);
}

.light-mode .search-divider {
  background-color: rgba(0, 0, 0, 0.1);
}

.search-input {
  flex: 1;
  background: transparent;
  padding: 8px;
  outline: none;
  border: none;
  font-size: 18px;
  color: inherit;
}

.search-input::placeholder {
  color: rgba(148, 163, 184, 0.5);
}

.search-button {
  padding: 12px;
  border-radius: 12px;
  color: white;
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
  transition: transform 0.2s, background-color 0.2s;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  min-width: 44px;
}

.search-button:active {
  transform: scale(0.95);
}

.dark-mode .search-button {
  background-color: #2563eb;
}

.dark-mode .search-button:hover {
  background-color: #3b82f6;
}

.light-mode .search-button {
  background-color: #3b82f6;
}

.light-mode .search-button:hover {
  background-color: #60a5fa;
}

/* 移动端响应式调整 */
@media (max-width: 768px) {
  .search-form {
    padding: 4px;
  }

  .search-select {
    padding: 8px 4px 8px 12px;
    font-size: 12px;
  }

  .search-input {
    padding: 6px 4px;
    font-size: 16px;
    min-width: 0;
  }

  .search-button {
    padding: 8px;
    min-width: 40px;
  }

  .search-divider {
    height: 20px;
    margin: 0 4px;
  }
}

/* Categories */
.categories-container {
  display: flex;
  flex-direction: column;
  gap: 48px;
  padding-bottom: 80px;
}

.category-section {
  scroll-margin-top: 32px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 24px;
  padding-left: 8px;
  border-left: 4px solid;
}

.dark-mode .section-title {
  border-left-color: rgba(59, 130, 246, 0.5);
}

.light-mode .section-title {
  border-left-color: rgba(59, 130, 246, 0.5);
}

.section-icon-wrapper {
  padding: 8px;
  border-radius: 8px;
  box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
}

.dark-mode .section-icon-wrapper {
  background-color: #1e293b;
  color: #60a5fa;
}

.light-mode .section-icon-wrapper {
  background-color: white;
  color: #2563eb;
}

.section-icon {
  display: flex;
  align-items: center;
  width: 20px;
  height: 20px;
}

.section-name {
  font-size: 24px;
  font-weight: 700;
  letter-spacing: -0.025em;
  opacity: 0.9;
}

.section-count {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 9999px;
  font-family: ui-monospace, monospace;
  opacity: 0.6;
}

.dark-mode .section-count {
  background-color: rgba(148, 163, 184, 0.1);
}

.light-mode .section-count {
  background-color: rgba(0, 0, 0, 0.05);
}

.cards-grid {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr));
  gap: 16px;
  align-items: stretch;
}

@media (min-width: 640px) {
  .cards-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (min-width: 1024px) {
  .cards-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}

@media (min-width: 1280px) {
  .cards-grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
}

.card-wrapper {
  position: relative;
  display: flex;
  width: 100%;
  height: 100%;
  min-width: 0;
}

.recent-card {
  position: relative;
}

.card-link-container {
  position: relative;
  display: flex;
  align-items: flex-start;
  width: 100%;
  height: 100%;
}

.card-delete-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  opacity: 0;
  transition: all 0.2s;
  z-index: 10;
  backdrop-filter: blur(8px);
}

.recent-card:hover .card-delete-btn {
  opacity: 1;
}

.dark-mode .card-delete-btn {
  background-color: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
}

.dark-mode .card-delete-btn:hover {
  background-color: rgba(239, 68, 68, 0.4);
  color: #f87171;
  transform: scale(1.1);
}

.light-mode .card-delete-btn {
  background-color: rgba(239, 68, 68, 0.15);
  color: #dc2626;
}

.light-mode .card-delete-btn:hover {
  background-color: rgba(239, 68, 68, 0.3);
  color: #b91c1c;
  transform: scale(1.1);
}
</style>
