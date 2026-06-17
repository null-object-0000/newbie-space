import type { RouteRecordRaw } from 'vue-router'
import { posts } from '@/data/posts'

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
  }
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
    '/tools/image-resize'
  ]
  
  // 添加所有博客文章路由
  for (const post of posts) {
    staticRoutes.push(`/posts/${post.slug}`)
  }
  
  return staticRoutes
}
