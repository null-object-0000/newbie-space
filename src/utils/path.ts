/**
 * 路径工具函数
 * 用于处理不同部署环境下的路径问题
 */

export function getBasePath(): string {
  if (import.meta.env.MODE === 'desktop') {
    return './'
  }

  if (typeof window === 'undefined') {
    return import.meta.env.BASE_URL || '/'
  }

  const pathname = window.location.pathname
  if (pathname.includes('/newbie-space/')) {
    return '/newbie-space/'
  }

  return '/'
}

export function getAssetPath(path: string): string {
  if (import.meta.env.MODE === 'desktop') {
    return path.startsWith('/') ? '.' + path : './' + path
  }

  const base = getBasePath()
  const normalizedPath = path.startsWith('/') ? path : '/' + path
  return base.endsWith('/') ? base + normalizedPath.substring(1) : base + normalizedPath
}
