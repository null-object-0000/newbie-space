import type { RouteRecordRaw } from 'vue-router'
import { posts } from '@/data/posts'
import { isDesktopApp } from '@/utils/runtime'

export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/views/Home.vue')
  },
  {
    path: '/nav',
    redirect: '/nav/'
  },
  {
    path: '/nav/',
    name: 'nav',
    component: () => import('@/views/Nav.vue')
  },
  {
    path: '/posts',
    name: 'blog',
    component: () => import('@/views/Blog.vue')
  },
  {
    path: '/posts/:slug',
    name: 'post',
    component: () => import('@/views/Post.vue')
  },
  {
    path: '/projects',
    name: 'projects',
    component: () => import('@/views/Projects.vue')
  },
  {
    path: '/tools',
    name: 'tools',
    component: () => import('@/views/Tools.vue')
  },
  {
    path: '/tools/background-transparent',
    name: 'background-transparent',
    component: () => import('@/views/BackgroundTransparent.vue')
  },
  {
    path: '/tools/image-resize',
    name: 'image-resize',
    component: () => import('@/views/ImageResize.vue')
  },
  {
    path: '/tools/qrcode-gen',
    name: 'qrcode-gen',
    component: () => import('@/views/QRCodeGen.vue')
  },
  {
    path: '/tools/qrcode-decode',
    name: 'qrcode-decode',
    component: () => import('@/views/QRCodeDecode.vue')
  },
  {
    path: '/tools/url-encoder',
    name: 'url-encoder',
    component: () => import('@/views/UrlEncoder.vue')
  },
  {
    path: '/tools/hash-text',
    name: 'hash-text',
    component: () => import('@/views/HashText.vue')
  },
  {
    path: '/tools/format-convert',
    name: 'format-convert',
    component: () => import('@/views/FormatConvert.vue')
  },
  {
    path: '/tools/color-converter',
    name: 'color-converter',
    component: () => import('@/views/ColorConverter.vue')
  },
  {
    path: '/tools/random-port',
    name: 'random-port',
    component: () => import('@/views/RandomPort.vue')
  },
  {
    path: '/tools/date-converter',
    name: 'date-converter',
    component: () => import('@/views/DateConverter.vue')
  },
  {
    path: '/tools/text-statistics',
    name: 'text-statistics',
    component: () => import('@/views/TextStatistics.vue')
  },
  {
    path: '/tools/text-diff',
    name: 'text-diff',
    component: () => import('@/views/TextDiff.vue')
  },
  {
    path: '/tools/docker-compose',
    name: 'docker-compose',
    component: () => import('@/views/DockerCompose.vue')
  },
  {
    path: '/tools/http-client',
    name: 'http-client',
    component: () => import('@/views/HttpClient.vue')
  },
  {
    path: '/tools/ip-lookup',
    name: 'ip-lookup',
    component: () => import('@/views/IpLookup.vue')
  },
  {
    path: '/tools/image-to-base64',
    name: 'image-to-base64',
    component: () => import('@/views/ImageToBase64.vue')
  },
  {
    path: '/tools/base64-to-image',
    name: 'base64-to-image',
    component: () => import('@/views/Base64ToImage.vue')
  },
  {
    path: '/tools/uuid-generator',
    name: 'uuid-generator',
    component: () => import('@/views/UuidGenerator.vue')
  },
  {
    path: '/tools/json-formatter',
    name: 'json-formatter',
    component: () => import('@/views/JsonFormatter.vue')
  },
  ...(isDesktopApp() ? [
    {
      path: '/tools/port-check',
      name: 'port-check',
      component: () => import('@/views/PortCheck.vue')
    },
    {
      path: '/tools/file-usage-check',
      name: 'file-usage-check',
      component: () => import('@/views/FileUsageCheck.vue')
    },
    {
      path: '/tools/process-manager',
      name: 'process-manager',
      component: () => import('@/views/ProcessManager.vue')
    },
    {
      path: '/tools/llm-gateway',
      name: 'llm-gateway',
      component: () => import('@/views/LlmGateway.vue')
    }
  ] : [])
]

// 为 SSG 生成所有博客文章的静态路由
export function getStaticRoutes(): string[] {
  const staticRoutes = [
    '/',
    '/nav/',
    '/posts',
    '/projects',
    '/tools',
    '/tools/background-transparent',
    '/tools/image-resize',
    '/tools/qrcode-gen',
    '/tools/qrcode-decode',
    '/tools/url-encoder',
    '/tools/hash-text',
    '/tools/format-convert',
    '/tools/color-converter',
    '/tools/random-port',
    '/tools/date-converter',
    '/tools/text-statistics',
    '/tools/text-diff',
    '/tools/docker-compose',
    '/tools/http-client',
    '/tools/ip-lookup',
    '/tools/image-to-base64',
    '/tools/base64-to-image',
    '/tools/uuid-generator',
    '/tools/json-formatter'
  ]
  
  // 添加所有博客文章路由
  for (const post of posts) {
    staticRoutes.push(`/posts/${post.slug}`)
  }
  
  return staticRoutes
}
