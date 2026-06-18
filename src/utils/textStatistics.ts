/**
 * 文本统计工具函数
 * 参考 it-tools 的 text-statistics 实现
 */

export interface TextStats {
  /** 字符数（含空格） */
  charCount: number
  /** 字符数（不含空格） */
  charCountNoSpaces: number
  /** 单词数（英文单词，以空格分隔） */
  wordCount: number
  /** 中文字数 */
  chineseCount: number
  /** 行数 */
  lineCount: number
  /** 段落数（以空行分隔） */
  paragraphCount: number
  /** 句子数（以 .!?。！？ 分隔） */
  sentenceCount: number
  /** UTF-8 字节大小 (number) */
  byteSizeRaw: number
  /** 格式化后的字节大小 */
  byteSize: string
}

/** 计算字符串的 UTF-8 字节大小 */
export function getStringSizeInBytes(text: string): number {
  return new TextEncoder().encode(text).byteLength
}

/** 格式化字节数 */
export function formatBytes(bytes: number): string {
  if (bytes <= 0) return '0 B'

  const units = ['B', 'KB', 'MB', 'GB']
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1)
  const val = bytes / Math.pow(1024, i)
  // 整数就省略小数
  const formatted = val % 1 === 0 ? val.toFixed(0) : val.toFixed(2)
  return `${formatted} ${units[i]}`
}

/** 统计中文字符数量 */
function countChinese(text: string): number {
  // 匹配 CJK 统一汉字范围（基本区 + 扩展A-G）
  const matches = text.match(/[一-鿿㐀-䶿豈-﫿]/g)
  return matches ? matches.length : 0
}

/** 统计文本各项指标 */
export function getTextStats(text: string): TextStats {
  const trimmed = text.trim()
  const byteSizeRaw = getStringSizeInBytes(text)

  return {
    charCount: text.length,
    charCountNoSpaces: text.replace(/\s/g, '').length,
    wordCount: trimmed === '' ? 0 : trimmed.split(/\s+/).length,
    chineseCount: countChinese(text),
    lineCount: text === '' ? 0 : text.split(/\r\n|\r|\n/).length,
    paragraphCount: text === '' ? 0 : text.split(/\n\s*\n/).filter(p => p.trim() !== '').length,
    sentenceCount: trimmed === '' ? 0 : trimmed.split(/[.!?。！？]+/).filter(s => s.trim() !== '').length,
    byteSizeRaw,
    byteSize: formatBytes(byteSizeRaw)
  }
}
