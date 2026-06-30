import { createApp as createVueApp, type App as VueApp } from 'vue'
import { createRouter, createWebHashHistory } from 'vue-router'
import { ViteSSG } from 'vite-ssg'
import App from './App.vue'
import { routes, getStaticRoutes } from './router'
import { isDesktopApp } from './utils/runtime'

// 本地字体（避免 Google Fonts CDN 在中国大陆的访问问题）
import '@fontsource/outfit/400.css'
import '@fontsource/outfit/500.css'
import '@fontsource/outfit/600.css'
import '@fontsource/outfit/700.css'
import '@fontsource/outfit/800.css'
import '@fontsource/noto-sans-sc/400.css'
import '@fontsource/noto-sans-sc/500.css'
import '@fontsource/noto-sans-sc/600.css'
import '@fontsource/noto-sans-sc/700.css'

import './styles/variables.css'
import './styles/global.css'

// 导入全局组件
import AppHeader from './components/AppHeader.vue'
import AppFooter from './components/AppFooter.vue'
import CardLink from './components/nav/CardLink.vue'

function registerGlobalComponents(app: VueApp) {
  app.component('AppHeader', AppHeader)
  app.component('AppFooter', AppFooter)
  app.component('CardLink', CardLink)
}

if (isDesktopApp()) {
  const router = createRouter({
    history: createWebHashHistory(),
    routes,
  })

  const app = createVueApp(App)
  app.use(router)
  registerGlobalComponents(app)
  app.mount('#app')
}

export const createApp = ViteSSG(
  App,
  {
    routes,
    base: import.meta.env.BASE_URL || '/',
    // @ts-ignore vite-ssg supports getRoutes for static route generation.
    getRoutes: getStaticRoutes
  },
  ({ app }) => {
    registerGlobalComponents(app)
  }
)
