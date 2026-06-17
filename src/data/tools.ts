import type { Component } from 'vue'
import { Crop, Fingerprint, ImageOff, Link2, QrCode, ScanLine, Shuffle } from 'lucide-vue-next'

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
    outputType: 'image'
  },
  {
    id: 'image-resize',
    name: '图片尺寸调整',
    desc: '自由调整图片宽度和高度，支持锁定宽高比、百分比缩放，输出 PNG / JPEG / WebP。',
    path: '/tools/image-resize',
    icon: Crop,
    tags: ['图片处理', 'Canvas', '本地运行'],
    inputType: ['image'],
    outputType: 'image'
  },
  {
    id: 'qrcode-gen',
    name: '二维码生成',
    desc: '输入文字或链接生成二维码，支持自定义颜色、尺寸和纠错等级，纯本地处理。',
    path: '/tools/qrcode-gen',
    icon: QrCode,
    tags: ['生成器', 'Canvas', '本地运行'],
    inputType: ['text'],
    outputType: 'image'
  },
  {
    id: 'url-encoder',
    name: 'URL 编解码',
    desc: '对文本进行 URL 编码（encodeURIComponent / encodeURI）或解码，纯本地处理。',
    path: '/tools/url-encoder',
    icon: Link2,
    tags: ['文本处理', '本地运行'],
    inputType: ['text'],
    outputType: 'text'
  },
  {
    id: 'qrcode-decode',
    name: '二维码反解析',
    desc: '上传二维码图片，解析其中的文字或链接，支持反色识别。',
    path: '/tools/qrcode-decode',
    icon: ScanLine,
    tags: ['图片处理', 'Canvas', '本地运行'],
    inputType: ['image'],
    outputType: 'text'
  },
  {
    id: 'hash-text',
    name: '文本哈希',
    desc: '计算文本的 MD5 / SHA-1 / SHA-224 / SHA-256 / SHA-384 / SHA-512 哈希值，支持 Hex / Base64 / Base64url / Bin 编码。',
    path: '/tools/hash-text',
    icon: Fingerprint,
    tags: ['文本处理', '本地运行', '加密'],
    inputType: ['text'],
    outputType: 'text'
  },
  {
    id: 'format-convert',
    name: '图片格式转换',
    desc: '将图片转换为 PNG / JPEG / WebP 格式，不改变尺寸，纯本地处理。',
    path: '/tools/format-convert',
    icon: Shuffle,
    tags: ['图片处理', 'Canvas', '本地运行'],
    inputType: ['image'],
    outputType: 'image'
  }
]
