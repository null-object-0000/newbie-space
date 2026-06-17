const STORAGE_KEY = 'tool-pipeline-image'
const STORAGE_KEY_NAME = 'tool-pipeline-filename'
const STORAGE_KEY_FROM = 'tool-pipeline-from'
const MAX_SIZE_MB = 4

export interface PipelineImage {
  /** 图片 data URL (PNG) */
  dataUrl: string
  /** 原始文件名 */
  fileName: string
  /** 来源工具名称 */
  fromTool: string
  /** 图片宽度 */
  width: number
  /** 图片高度 */
  height: number
}

/**
 * 将处理结果存入 sessionStorage，供下一个工具读取。
 * 超过 MAX_SIZE_MB 时返回 false。
 */
export function storePipelineImage(image: PipelineImage): boolean {
  if (!canUseStorage()) return false

  const sizeMB = (image.dataUrl.length * 0.75) / (1024 * 1024) // 粗略估算 base64 原始字节
  if (sizeMB > MAX_SIZE_MB) {
    console.warn(`[toolPipeline] 图片过大 (${sizeMB.toFixed(1)} MB)，不存入流转`)
    return false
  }

  try {
    sessionStorage.setItem(STORAGE_KEY, image.dataUrl)
    sessionStorage.setItem(STORAGE_KEY_NAME, image.fileName)
    sessionStorage.setItem(STORAGE_KEY_FROM, image.fromTool)
    sessionStorage.setItem('tool-pipeline-width', String(image.width))
    sessionStorage.setItem('tool-pipeline-height', String(image.height))
    return true
  } catch (e) {
    console.warn('[toolPipeline] sessionStorage 写入失败', e)
    return false
  }
}

/**
 * 读取上一个工具传来的图片。读取后自动清除。
 * 如果没有则返回 null。
 */
export function consumePipelineImage(): PipelineImage | null {
  if (!canUseStorage()) return null

  const dataUrl = sessionStorage.getItem(STORAGE_KEY)
  if (!dataUrl) return null

  const fileName = sessionStorage.getItem(STORAGE_KEY_NAME) || 'pipeline-image.png'
  const fromTool = sessionStorage.getItem(STORAGE_KEY_FROM) || '未知工具'
  const width = parseInt(sessionStorage.getItem('tool-pipeline-width') || '0', 10)
  const height = parseInt(sessionStorage.getItem('tool-pipeline-height') || '0', 10)

  // 读取后立即清除，避免重复消费
  clearPipelineImage()

  return { dataUrl, fileName, fromTool, width, height }
}

/** 检查是否有待消费的流转图片 */
export function hasPipelineImage(): boolean {
  if (!canUseStorage()) return false
  return !!sessionStorage.getItem(STORAGE_KEY)
}

function clearPipelineImage() {
  sessionStorage.removeItem(STORAGE_KEY)
  sessionStorage.removeItem(STORAGE_KEY_NAME)
  sessionStorage.removeItem(STORAGE_KEY_FROM)
  sessionStorage.removeItem('tool-pipeline-width')
  sessionStorage.removeItem('tool-pipeline-height')
}

function canUseStorage(): boolean {
  try {
    return typeof sessionStorage !== 'undefined'
  } catch {
    return false
  }
}
