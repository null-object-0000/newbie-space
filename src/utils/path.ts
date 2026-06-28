/**
 * 路径工具函数
 * 桌面应用使用相对路径，不需要 GitHub Pages 子路径检测
 */

export function getBasePath(): string {
  return './'
}

export function getAssetPath(path: string): string {
  const normalizedPath = path.startsWith('/') ? '.' + path : './' + path
  return normalizedPath
}
