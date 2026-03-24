/**
 * 路径工具函数
 * 用于处理不同部署环境下的路径问题
 */

/**
 * 动态计算 base 路径
 * 支持：
 * - https://null-object-0000.github.io/newbie-space/ (GitHub Pages 子路径)
 * - https://snewbie.site (自定义域名根路径)
 */
export function getBasePath(): string {
  if (typeof window === 'undefined') {
    // SSR 环境，使用环境变量
    return import.meta.env.BASE_URL || '/'
  }

  // 客户端环境，动态检测
  const pathname = window.location.pathname
  
  // 如果路径包含 /newbie-space/，说明是 GitHub Pages 子路径部署
  if (pathname.includes('/newbie-space/')) {
    return '/newbie-space/'
  }
  
  // 否则使用根路径
  return '/'
}

/**
 * 获取资源路径（相对于根路径）
 * @param path 资源路径，如 '/logo.png' 或 '/icons/xxx.ico'
 * @returns 完整的资源路径
 */
export function getAssetPath(path: string): string {
  const base = getBasePath()
  
  // 确保 path 以 / 开头
  const normalizedPath = path.startsWith('/') ? path : '/' + path
  
  // 拼接 base 和 path
  if (base.endsWith('/')) {
    return base + normalizedPath.substring(1)
  } else {
    return base + normalizedPath
  }
}
