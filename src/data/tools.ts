import type { Component } from 'vue'
import { ImageOff } from 'lucide-vue-next'

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
  }
]
