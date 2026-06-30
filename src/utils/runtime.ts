export type RuntimeTarget = 'web' | 'desktop' | 'both'

export function isDesktopApp(): boolean {
  return import.meta.env.MODE === 'desktop'
}

export function supportsRuntime(target: RuntimeTarget = 'both'): boolean {
  if (target === 'both') return true
  return target === 'desktop' ? isDesktopApp() : !isDesktopApp()
}
