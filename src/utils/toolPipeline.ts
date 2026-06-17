import { tools, type DataType, type ToolItem } from '@/data/tools'

// ============================================================
// 图片流转存储
// ============================================================

const IMAGE_KEY = 'tool-pipeline-image'
const IMAGE_NAME_KEY = 'tool-pipeline-filename'
const IMAGE_FROM_KEY = 'tool-pipeline-from'
const IMAGE_WIDTH_KEY = 'tool-pipeline-width'
const IMAGE_HEIGHT_KEY = 'tool-pipeline-height'
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

  const sizeMB = (image.dataUrl.length * 0.75) / (1024 * 1024)
  if (sizeMB > MAX_SIZE_MB) {
    console.warn(`[toolPipeline] 图片过大 (${sizeMB.toFixed(1)} MB)，不存入流转`)
    return false
  }

  try {
    sessionStorage.setItem(IMAGE_KEY, image.dataUrl)
    sessionStorage.setItem(IMAGE_NAME_KEY, image.fileName)
    sessionStorage.setItem(IMAGE_FROM_KEY, image.fromTool)
    sessionStorage.setItem(IMAGE_WIDTH_KEY, String(image.width))
    sessionStorage.setItem(IMAGE_HEIGHT_KEY, String(image.height))
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

  const dataUrl = sessionStorage.getItem(IMAGE_KEY)
  if (!dataUrl) return null

  const fileName = sessionStorage.getItem(IMAGE_NAME_KEY) || 'pipeline-image.png'
  const fromTool = sessionStorage.getItem(IMAGE_FROM_KEY) || '未知工具'
  const width = parseInt(sessionStorage.getItem(IMAGE_WIDTH_KEY) || '0', 10)
  const height = parseInt(sessionStorage.getItem(IMAGE_HEIGHT_KEY) || '0', 10)

  clearPipelineImage()

  return { dataUrl, fileName, fromTool, width, height }
}

/** 检查是否有待消费的流转图片 */
export function hasPipelineImage(): boolean {
  if (!canUseStorage()) return false
  return !!sessionStorage.getItem(IMAGE_KEY)
}

function clearPipelineImage() {
  sessionStorage.removeItem(IMAGE_KEY)
  sessionStorage.removeItem(IMAGE_NAME_KEY)
  sessionStorage.removeItem(IMAGE_FROM_KEY)
  sessionStorage.removeItem(IMAGE_WIDTH_KEY)
  sessionStorage.removeItem(IMAGE_HEIGHT_KEY)
}

// ============================================================
// 文本流转存储
// ============================================================

const TEXT_KEY = 'tool-pipeline-text'
const TEXT_FROM_KEY = 'tool-pipeline-text-from'
const MAX_TEXT_KB = 64

export interface PipelineText {
  text: string
  fromTool: string
}

export function storePipelineText(data: PipelineText): boolean {
  if (!canUseStorage()) return false

  const sizeKB = new Blob([data.text]).size / 1024
  if (sizeKB > MAX_TEXT_KB) {
    console.warn(`[toolPipeline] 文本过大 (${sizeKB.toFixed(1)} KB)，不存入流转`)
    return false
  }

  try {
    sessionStorage.setItem(TEXT_KEY, data.text)
    sessionStorage.setItem(TEXT_FROM_KEY, data.fromTool)
    return true
  } catch (e) {
    console.warn('[toolPipeline] sessionStorage 写入失败', e)
    return false
  }
}

export function consumePipelineText(): PipelineText | null {
  if (!canUseStorage()) return null

  const text = sessionStorage.getItem(TEXT_KEY)
  if (!text) return null

  const fromTool = sessionStorage.getItem(TEXT_FROM_KEY) || '未知工具'

  clearPipelineText()

  return { text, fromTool }
}

export function hasPipelineText(): boolean {
  if (!canUseStorage()) return false
  return !!sessionStorage.getItem(TEXT_KEY)
}

function clearPipelineText() {
  sessionStorage.removeItem(TEXT_KEY)
  sessionStorage.removeItem(TEXT_FROM_KEY)
}

// ============================================================
// 通用流转 API
// ============================================================

/** 按类型存储流转数据 */
export function storePipelineByType(
  type: DataType,
  data: { fromTool: string } & Record<string, unknown>
): boolean {
  if (type === 'image') {
    const img = data as unknown as PipelineImage
    return storePipelineImage(img)
  }
  if (type === 'text') {
    const txt = data as unknown as PipelineText
    return storePipelineText(txt)
  }
  return false
}

/** 按类型检查是否有待消费数据 */
export function hasPipelineByType(type: DataType): boolean {
  if (type === 'image') return hasPipelineImage()
  if (type === 'text') return hasPipelineText()
  return false
}

// ============================================================
// 工具匹配
// ============================================================

/** 查找工具对象 */
export function findTool(toolId: string): ToolItem | undefined {
  return tools.find(t => t.id === toolId)
}

/** 检查两个工具是否可以流转（from 的输出是否匹配 to 的输入） */
export function canPipeline(from: ToolItem, to: ToolItem): boolean {
  return to.inputType.includes(from.outputType)
}

/**
 * 获取下游工具列表：可以接收当前工具输出的工具。
 * 按匹配的 inputType 排序（精确匹配优先）。
 */
export function getDownstreamTools(currentToolId: string): ToolItem[] {
  const current = findTool(currentToolId)
  if (!current) return []

  return tools
    .filter(t => t.id !== currentToolId && canPipeline(current, t))
}

/**
 * 获取上游工具列表：其输出可以被当前工具接收的工具。
 */
export function getUpstreamTools(currentToolId: string): ToolItem[] {
  const current = findTool(currentToolId)
  if (!current) return []

  return tools
    .filter(t => t.id !== currentToolId && canPipeline(t, current))
}

// ============================================================
// 工具函数
// ============================================================

function canUseStorage(): boolean {
  try {
    return typeof sessionStorage !== 'undefined'
  } catch {
    return false
  }
}
