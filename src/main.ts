import { createApp } from 'vue'
import { createRouter, createWebHashHistory } from 'vue-router'
import App from './App.vue'
import { routes } from './router'

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

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

const app = createApp(App)
app.use(router)

// 注册全局组件
app.component('AppHeader', AppHeader)
app.component('AppFooter', AppFooter)
app.component('CardLink', CardLink)

app.mount('#app')
