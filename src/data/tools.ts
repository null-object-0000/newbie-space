import type { Component } from 'vue'
import { Crop, ImageOff } from 'lucide-vue-next'

export interface ToolItem {
  id: string
  name: string
  desc: string
  path: string
  icon: Component
  tags: string[]
}

export const tools: ToolItem[] = [
  {
    id: 'background-transparent',
    name: '背景透明化',
    desc: '将白色或指定颜色背景转换为透明 PNG，支持仅背景和全局替换两种模式。',
    path: '/tools/background-transparent',
    icon: ImageOff,
    tags: ['图片处理', 'Canvas', '本地运行']
  },
  {
    id: 'image-resize',
    name: '图片尺寸调整',
    desc: '自由调整图片宽度和高度，支持锁定宽高比、百分比缩放，输出 PNG / JPEG / WebP。',
    path: '/tools/image-resize',
    icon: Crop,
    tags: ['图片处理', 'Canvas', '本地运行']
  }
]
