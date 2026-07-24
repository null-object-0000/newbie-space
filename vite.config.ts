import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { VitePWA } from 'vite-plugin-pwa'
import { resolve } from 'path'

// 从环境变量或命令行参数获取 base 路径，默认为 '/'
// 构建时可以通过 --base 参数设置，Vite 会自动处理
// 也可以通过 VITE_BASE_URL 或 BASE_URL 环境变量设置
const getBase = () => {
  // 优先使用命令行参数（通过 --base 传递）
  const baseArg = process.argv.find(arg => arg.startsWith('--base='))
  if (baseArg) {
    return baseArg.split('=')[1]
  }
  // 其次使用环境变量
  return process.env.VITE_BASE_URL || process.env.BASE_URL || '/'
}

export default defineConfig({
  base: getBase(),
  plugins: [
    vue(),
    VitePWA({
      registerType: 'autoUpdate',
      // 基础路径：与部署根路径一致（OSS 根目录）
      includeAssets: [
        'pwa-192x192.png',
        'pwa-512x512.png',
        'maskable-512x512.png',
        'apple-touch-icon.png',
        'logo.png',
      ],
      manifest: {
        name: 'Newbie Space',
        short_name: 'Newbie Space',
        description: '我的个人导航与博客，收集灵感，记录成长，分享技术',
        lang: 'zh-CN',
        theme_color: '#646cff',
        background_color: '#ffffff',
        display: 'standalone',
        orientation: 'portrait',
        start_url: '/',
        scope: '/',
        id: '/',
        categories: ['navigation', 'blog', 'tools'],
        icons: [
          { src: 'pwa-192x192.png', sizes: '192x192', type: 'image/png' },
          { src: 'pwa-512x512.png', sizes: '512x512', type: 'image/png' },
          {
            src: 'maskable-512x512.png',
            sizes: '512x512',
            type: 'image/png',
            purpose: 'maskable',
          },
        ],
      },
      workbox: {
        // 预缓存所有静态资源（JS/CSS/HTML/图片/字体）
        // 上限设为 3 MiB：icons/ 下有少量第三方 favicon 略超默认 2 MiB
        // （如 skillhub_tencent_com.png 2.56 MB），这些图标对导航页二次加载有缓存价值
        maximumFileSizeToCacheInBytes: 3 * 1024 * 1024,
        globPatterns: [
          '**/*.{js,css,html,svg,png,ico,webmanifest,woff,woff2,ttf}',
        ],
        // 导航请求：网络优先，3s 超时后回退缓存
        // → 首次访问走网络并缓存；二次访问若网络慢于 3s 直接命中缓存
        runtimeCaching: [
          {
            urlPattern: ({ request }) => request.mode === 'navigate',
            handler: 'NetworkFirst',
            options: {
              cacheName: 'html-cache',
              networkTimeoutSeconds: 3,
              expiration: {
                maxEntries: 50,
                maxAgeSeconds: 60 * 60 * 24 * 7, // 7 天
              },
              cacheableResponse: { statuses: [200] },
            },
          },
          {
            // 同域静态资源（带 hash 的 JS/CSS/图片）：缓存优先
            urlPattern: ({ url }) => url.origin === self.location.origin,
            handler: 'CacheFirst',
            options: {
              cacheName: 'asset-cache',
              expiration: {
                maxEntries: 200,
                maxAgeSeconds: 60 * 60 * 24 * 30, // 30 天
              },
              cacheableResponse: { statuses: [200] },
            },
          },
          {
            // 字体（Google Fonts 等第三方）：缓存优先，长过期
            urlPattern: ({ request }) => request.destination === 'font',
            handler: 'CacheFirst',
            options: {
              cacheName: 'font-cache',
              expiration: {
                maxEntries: 30,
                maxAgeSeconds: 60 * 60 * 24 * 365, // 1 年
              },
              cacheableResponse: { statuses: [200] },
            },
          },
        ],
        cleanupOutdatedCaches: true,
        navigateFallback: 'index.html',
      },
      // 开发模式不启用 PWA，避免缓存干扰调试
      devOptions: {
        enabled: false,
      },
    }),
  ],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  },
  ssgOptions: {
    script: 'async',
    formatting: 'minify',
    crittersOptions: {
      preload: 'swap'
    }
  }
})
