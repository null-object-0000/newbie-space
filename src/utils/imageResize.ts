export interface ResizeOptions {
  width: number
  height: number
  format: 'png' | 'jpeg' | 'webp'
  quality: number // 0–1, only for jpeg / webp
}

export interface ResizeResult {
  blob: Blob
  dataUrl: string
  width: number
  height: number
  originalWidth: number
  originalHeight: number
  format: string
  sizeBytes: number
}

/**
 * 使用 Canvas 将图片缩放至目标尺寸，支持指定输出格式与质量。
 * 全部在浏览器本地完成，不会上传图片。
 */
export function resizeImage(
  source: HTMLImageElement | ImageData,
  options: ResizeOptions
): ResizeResult {
  const canvas = document.createElement('canvas')
  canvas.width = options.width
  canvas.height = options.height

  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('当前浏览器不支持 Canvas')

  // 高质量缩放
  ctx.imageSmoothingEnabled = true
  ctx.imageSmoothingQuality = 'high'

  if (source instanceof ImageData) {
    const tempCanvas = document.createElement('canvas')
    tempCanvas.width = source.width
    tempCanvas.height = source.height
    const tempCtx = tempCanvas.getContext('2d')
    if (!tempCtx) throw new Error('当前浏览器不支持 Canvas')
    tempCtx.putImageData(source, 0, 0)
    ctx.drawImage(tempCanvas, 0, 0, options.width, options.height)
  } else {
    ctx.drawImage(source, 0, 0, options.width, options.height)
  }

  const mimeType = `image/${options.format}` as const
  const quality = options.format === 'png' ? undefined : options.quality
  const dataUrl = canvas.toDataURL(mimeType, quality)

  // 通过 dataUrl 转 Blob
  const parts = dataUrl.split(',')
  const mime = parts[0].match(/:(.*?);/)?.[1] || mimeType
  const bytes = atob(parts[1])
  const buffer = new Uint8Array(bytes.length)
  for (let i = 0; i < bytes.length; i++) {
    buffer[i] = bytes.charCodeAt(i)
  }
  const blob = new Blob([buffer], { type: mime })

  return {
    blob,
    dataUrl,
    width: options.width,
    height: options.height,
    originalWidth: source instanceof ImageData ? source.width : source.naturalWidth,
    originalHeight: source instanceof ImageData ? source.height : source.naturalHeight,
    format: options.format,
    sizeBytes: blob.size
  }
}

/**
 * 根据目标宽度和原始宽高比计算等比例高度。
 */
export function calcAspectHeight(originalWidth: number, originalHeight: number, targetWidth: number): number {
  if (originalWidth <= 0) return targetWidth
  return Math.round((originalHeight / originalWidth) * targetWidth)
}

/**
 * 根据目标高度和原始宽高比计算等比例宽度。
 */
export function calcAspectWidth(originalWidth: number, originalHeight: number, targetHeight: number): number {
  if (originalHeight <= 0) return targetHeight
  return Math.round((originalWidth / originalHeight) * targetHeight)
}

/** 格式化文件大小 */
export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}
