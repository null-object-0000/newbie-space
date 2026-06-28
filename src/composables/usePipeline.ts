import { onMounted, ref, computed, type Ref, type ComputedRef } from 'vue'
import { useRouter } from 'vue-router'
import { type ToolItem } from '@/data/tools'
import {
  findTool,
  getDownstreamTools,
  consumePipelineImage,
  consumePipelineText,
  storePipelineImage,
  storePipelineText,
  type PipelineImage,
  type PipelineText
} from '@/utils/toolPipeline'

export interface PipelineIncomingImage {
  type: 'image'
  data: PipelineImage
}

export interface PipelineIncomingText {
  type: 'text'
  data: PipelineText
}

export type PipelineIncoming = PipelineIncomingImage | PipelineIncomingText

export interface UsePipelineOptions {
  /** 当前工具 ID */
  toolId: string
  /** 收到流转数据时的回调，返回 false 表示未能消费 */
  onIncoming?: (incoming: PipelineIncoming) => Promise<boolean> | boolean
}

export interface UsePipelineReturn {
  /** 流转来源工具名称 */
  pipelineFrom: Ref<string>
  /** 是否有待消费数据被成功加载 */
  hasIncoming: Ref<boolean>
  /** 下游工具列表 */
  downstreamTools: ComputedRef<ToolItem[]>
  /**
   * 将当前输出存储为图片，并跳转到目标工具。
   * 返回 false 表示存储失败（如数据过大）。
   */
  sendImageTo: (targetTool: ToolItem, image: Omit<PipelineImage, 'fromTool'>) => { ok: boolean; message: string }
  /**
   * 将当前输出存储为文本，并跳转到目标工具。
   * 返回 false 表示存储失败。
   */
  sendTextTo: (targetTool: ToolItem, text: string) => { ok: boolean; message: string }
}

/**
 * 工具流转 composable。
 *
 * 自动处理：
 * - 入站：检测上游传来的数据并消费
 * - 出站：存储当前输出并跳转到下游工具
 * - 匹配：计算可流转的下游工具列表
 */
export function usePipeline(options: UsePipelineOptions): UsePipelineReturn {
  const { toolId, onIncoming } = options

  const currentTool = findTool(toolId)
  const pipelineFrom = ref('')
  const hasIncoming = ref(false)

  const downstreamTools = computed(() => getDownstreamTools(toolId))

  const router = useRouter()

  // --- 入站检测（onMounted 时自动执行）---
  onMounted(async () => {
    const incoming = checkIncoming()
    if (!incoming) return

    if (onIncoming) {
      try {
        const ok = await onIncoming(incoming)
        if (ok) {
          hasIncoming.value = true
          pipelineFrom.value = incoming.data.fromTool
        }
      } catch {
        // 消费失败，静默处理
      }
    }
  })

  function checkIncoming(): PipelineIncoming | null {
    if (!currentTool) return null

    for (const inputType of currentTool.inputType) {
      if (inputType === 'image') {
        const img = consumePipelineImage()
        if (img) return { type: 'image', data: img }
      }
      if (inputType === 'text') {
        const txt = consumePipelineText()
        if (txt) return { type: 'text', data: txt }
      }
    }

    return null
  }

  // --- 出站 ---
  function sendImageTo(
    targetTool: ToolItem,
    image: Omit<PipelineImage, 'fromTool'>
  ): { ok: boolean; message: string } {
    const current = findTool(toolId)
    const ok = storePipelineImage({
      ...image,
      fromTool: current?.name ?? toolId
    })

    if (ok) {
      setTimeout(() => {
        router.push(targetTool.path)
      }, 300)
      return { ok: true, message: `已发送至「${targetTool.name}」，即将跳转` }
    }
    return { ok: false, message: '数据过大，无法流转（请下载后重新上传）' }
  }

  function sendTextTo(
    targetTool: ToolItem,
    text: string
  ): { ok: boolean; message: string } {
    const current = findTool(toolId)
    const ok = storePipelineText({
      text,
      fromTool: current?.name ?? toolId
    })

    if (ok) {
      setTimeout(() => {
        router.push(targetTool.path)
      }, 300)
      return { ok: true, message: `已发送至「${targetTool.name}」，即将跳转` }
    }
    return { ok: false, message: '文本过长，无法流转' }
  }

  return {
    pipelineFrom,
    hasIncoming,
    downstreamTools,
    sendImageTo,
    sendTextTo
  }
}
