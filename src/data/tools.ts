import type { Component } from 'vue'
import { BarChart3, Braces, Clock, Container, Crop, FileImage, Fingerprint, GitCompare, Hash, Image, ImageOff, Link2, Network, Palette, QrCode, ScanLine, Send, Shuffle } from 'lucide-vue-next'

/** 工具间流转的数据类型 */
export type DataType = 'text' | 'image'

export type ToolCategory = '图片处理' | '生成器' | '文本处理' | '编码转换' | '网络开发'

export interface ToolItem {
  id: string
  name: string
  desc: string
  path: string
  icon: Component
  tags: string[]
  /** 工具分类（用于首页 Tab 筛选），未设置时取 tags[0] */
  category?: ToolCategory
  /** 该工具接受的输入类型（可为多种） */
  inputType: DataType[]
  /** 该工具产生的输出类型 */
  outputType: DataType
  /** 图标颜色 */
  color?: string
  /** 输出类型显示标签（覆盖默认映射），比如文本统计工具输出更适合叫"统计结果"而非"文本" */
  outputLabel?: string
}

export const tools: ToolItem[] = [
  {
    id: 'http-client',
    name: 'HTTP 请求',
    desc: '轻量级 API 调试工具，支持 GET / POST / PUT / PATCH / DELETE、查询参数、请求头和 JSON 请求体。',
    path: '/tools/http-client',
    icon: Send,
    tags: ['网络开发', 'API', 'HTTP'],
    inputType: ['text'],
    outputType: 'text',
    color: '#f97316'
  },
  {
    id: 'ip-lookup',
    name: 'IP 信息解析',
    desc: '聚合多个公开 IP 数据服务，查询归属地、ASN、网络运营商、时区和经纬度等信息。',
    path: '/tools/ip-lookup',
    icon: Network,
    tags: ['网络开发', 'IP', 'API'],
    inputType: ['text'],
    outputType: 'text',
    color: '#06b6d4'
  },
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
    category: '编码转换',
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
    category: '编码转换',
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
    category: '编码转换',
    inputType: ['text'],
    outputType: 'text',
    color: '#3b82f6'
  },
  {
    id: 'random-port',
    name: '随机端口生成',
    desc: '在合法端口范围内随机生成端口号，支持按范围筛选（全部、非特权、注册、动态、自定义）。',
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
    outputLabel: '分析结果',
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
    outputLabel: '分析结果',
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
  },
  {
    id: 'image-to-base64',
    name: '图片转 Base64',
    desc: '将图片编码为 Base64 data URL，支持粘贴、拖拽上传，输出可直接用于 img 标签或 CSS。',
    path: '/tools/image-to-base64',
    icon: Image,
    tags: ['图片处理', 'Canvas', '本地运行'],
    inputType: ['image'],
    outputType: 'text',
    color: '#10b981'
  },
  {
    id: 'base64-to-image',
    name: 'Base64 转图片',
    desc: '将 Base64 编码的图片数据还原为图片，支持 data URL 或纯 Base64 字符串，可预览和下载。',
    path: '/tools/base64-to-image',
    icon: FileImage,
    tags: ['图片处理', '本地运行'],
    inputType: ['text'],
    outputType: 'image',
    color: '#10b981'
  },
  {
    id: 'uuid-generator',
    name: 'UUID 生成器',
    desc: '生成 UUID v4 / v7 或自定义长度随机字符串，支持批量生成，纯本地 crypto API 处理。',
    path: '/tools/uuid-generator',
    icon: Hash,
    tags: ['生成器', '本地运行'],
    inputType: [],
    outputType: 'text',
    color: '#8b5cf6'
  },
  {
    id: 'json-formatter',
    name: 'JSON 格式化',
    desc: 'JSON 美化、压缩和语法验证，自动检测错误位置，显示键数量和嵌套深度。',
    path: '/tools/json-formatter',
    icon: Braces,
    tags: ['文本处理', '本地运行'],
    inputType: ['text'],
    outputType: 'text',
    color: '#3b82f6'
  }
]
