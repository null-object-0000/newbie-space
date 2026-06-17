import type { Component } from 'vue'
import { BarChart3, Clock, Container, Crop, Fingerprint, GitCompare, ImageOff, Link2, Network, Palette, QrCode, ScanLine, Shuffle } from 'lucide-vue-next'

/** 工具间流转的数据类型 */
export type DataType = 'text' | 'image'

export interface ToolItem {
  id: string
  name: string
  desc: string
  path: string
  icon: Component
  tags: string[]
  /** 该工具接受的输入类型（可为多种） */
  inputType: DataType[]
  /** 该工具产生的输出类型 */
  outputType: DataType
  /** 图标颜色 */
  color?: string
}

export const tools: ToolItem[] = [
  {
    id: 'background-transparent',
    name: '背景透明化',
    desc: '将白色或指定颜色背景转换为透明 PNG，支持仅背景和全局替换两种模式。',
    path: '/tools/background-transparent',
    icon: ImageOff,
    tags: ['图片处理', 'Canvas', '本地运行'],
    inputType: ['image'],
    outputType: 'image',
    color: '#10b981'
  },
  {
    id: 'image-resize',
    name: '图片尺寸调整',
    desc: '自由调整图片宽度和高度，支持锁定宽高比、百分比缩放，输出 PNG / JPEG / WebP。',
    path: '/tools/image-resize',
    icon: Crop,
    tags: ['图片处理', 'Canvas', '本地运行'],
    inputType: ['image'],
    outputType: 'image',
    color: '#10b981'
  },
  {
    id: 'qrcode-gen',
    name: '二维码生成',
    desc: '输入文字或链接生成二维码，支持自定义颜色、尺寸和纠错等级，纯本地处理。',
    path: '/tools/qrcode-gen',
    icon: QrCode,
    tags: ['生成器', 'Canvas', '本地运行'],
    inputType: ['text'],
    outputType: 'image',
    color: '#8b5cf6'
  },
  {
    id: 'url-encoder',
    name: 'URL 编解码',
    desc: '对文本进行 URL 编码（encodeURIComponent / encodeURI）或解码，纯本地处理。',
    path: '/tools/url-encoder',
    icon: Link2,
    tags: ['文本处理', '本地运行'],
    inputType: ['text'],
    outputType: 'text',
    color: '#3b82f6'
  },
  {
    id: 'qrcode-decode',
    name: '二维码反解析',
    desc: '上传二维码图片，解析其中的文字或链接，支持反色识别。',
    path: '/tools/qrcode-decode',
    icon: ScanLine,
    tags: ['图片处理', 'Canvas', '本地运行'],
    inputType: ['image'],
    outputType: 'text',
    color: '#10b981'
  },
  {
    id: 'hash-text',
    name: '文本哈希',
    desc: '计算文本的 MD5 / SHA-1 / SHA-224 / SHA-256 / SHA-384 / SHA-512 哈希值，支持 Hex / Base64 / Base64url / Bin 编码。',
    path: '/tools/hash-text',
    icon: Fingerprint,
    tags: ['文本处理', '本地运行', '加密'],
    inputType: ['text'],
    outputType: 'text',
    color: '#ef4444'
  },
  {
    id: 'format-convert',
    name: '图片格式转换',
    desc: '将图片转换为 PNG / JPEG / WebP 格式，不改变尺寸，纯本地处理。',
    path: '/tools/format-convert',
    icon: Shuffle,
    tags: ['图片处理', 'Canvas', '本地运行'],
    inputType: ['image'],
    outputType: 'image',
    color: '#10b981'
  },
  {
    id: 'date-converter',
    name: '时间戳转换',
    desc: '日期字符串与 Unix 时间戳互转，支持 ISO 8601 / 本地时间 / UTC / 相对时间等格式，自动识别输入。',
    path: '/tools/date-converter',
    icon: Clock,
    tags: ['文本处理', '本地运行'],
    inputType: ['text'],
    outputType: 'text',
    color: '#3b82f6'
  },
  {
    id: 'color-converter',
    name: '颜色转换',
    desc: '输入任意格式颜色值，自动转换为 HEX / RGB / HSL / HSV / LCH / CMYK 等 10 种格式。',
    path: '/tools/color-converter',
    icon: Palette,
    tags: ['文本处理', '本地运行'],
    inputType: ['text'],
    outputType: 'text',
    color: '#3b82f6'
  },
  {
    id: 'random-port',
    name: '随机端口生成',
    desc: '在合法端口范围内随机生成端口号，支持按范围筛选（非特权、注册、动态、自定义）。',
    path: '/tools/random-port',
    icon: Network,
    tags: ['生成器', '本地运行'],
    inputType: [],
    outputType: 'text',
    color: '#8b5cf6'
  },
  {
    id: 'text-statistics',
    name: '文本统计',
    desc: '统计文本的字符数、单词数、行数、段落数、句子数和字节大小，纯本地处理。',
    path: '/tools/text-statistics',
    icon: BarChart3,
    tags: ['文本处理', '本地运行'],
    inputType: ['text'],
    outputType: 'text',
    color: '#3b82f6'
  },
  {
    id: 'text-diff',
    name: '文本差异对比',
    desc: '对比两段文本的差异，高亮显示新增和删除的行，纯本地处理。',
    path: '/tools/text-diff',
    icon: GitCompare,
    tags: ['文本处理', '本地运行'],
    inputType: ['text'],
    outputType: 'text',
    color: '#3b82f6'
  },
  {
    id: 'docker-compose',
    name: 'Docker Run 转换',
    desc: '将 docker run 命令转换为 docker-compose.yml 格式，支持常用参数解析。',
    path: '/tools/docker-compose',
    icon: Container,
    tags: ['文本处理', '本地运行'],
    inputType: ['text'],
    outputType: 'text',
    color: '#3b82f6'
  }
]
