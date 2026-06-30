<template>
  <div class="tool-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />

    <main class="tool-main">
      <div class="tool-topbar">
        <router-link to="/tools" class="back-link">
          <ArrowLeft :size="16" />
          <span>工具中心</span>
        </router-link>
      </div>

      <section class="tool-header">
        <div class="tool-heading">
          <div class="heading-icon">
            <FileImage :size="22" />
          </div>
          <div>
            <h1>{{ tool?.name }}</h1>
            <p>{{ tool?.desc }}</p>
          </div>
        </div>
      </section>

      <div v-if="pipelineFrom" class="pipeline-banner">
        <ArrowRightLeft :size="14" />
        <span>来自「{{ pipelineFrom }}」的传递数据</span>
      </div>

      <div class="workspace">
        <!-- 左侧：Base64 输入 -->
        <div class="panel panel-left">
          <textarea
            v-model="inputBase64"
            placeholder="粘贴 Base64 图片编码（支持 data:image/...;base64,... 格式）…"
            rows="10"
            @input="scheduleDecode"
          ></textarea>

          <div v-if="parseError" class="parse-error">
            <AlertCircle :size="14" />
            <span>{{ parseError }}</span>
          </div>

          <div v-if="parsedInfo && !parseError" class="info-badge">
            <span>{{ parsedInfo.format.toUpperCase() }} · {{ parsedInfo.width }}×{{ parsedInfo.height }} · {{ formatSize(parsedInfo.sizeBytes) }}</span>
          </div>
        </div>

        <!-- 右侧：预览 + 操作 -->
        <div class="panel panel-right">
          <div class="preview-area" :class="{ empty: !decodedDataUrl }">
            <img v-if="decodedDataUrl" :src="decodedDataUrl" alt="解码后的图片" />
            <div v-else class="preview-empty">
              <FileImage :size="32" />
              <span>解码后将在此预览</span>
            </div>
          </div>

          <div class="actions">
            <button class="btn secondary" :disabled="!decodedDataUrl" @click="downloadImage">
              <Download :size="16" />下载图片
            </button>
            <button class="btn secondary" :disabled="!inputBase64" @click="clearAll">
              <Trash2 :size="16" />清空
            </button>
            <PipelineSend
              :tools="downstreamTools"
              :disabled="!decodedDataUrl"
              @send="sendToTool"
            />
          </div>
        </div>
      </div>
    </main>

    <Transition name="toast">
      <div v-if="toastMessage" class="toast" :class="toastType">{{ toastMessage }}</div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { usePipeline } from '@/composables/usePipeline'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import { findTool } from '@/utils/toolPipeline'
import {
  AlertCircle,
  ArrowLeft,
  ArrowRightLeft,
  Download,
  FileImage,
  Trash2
} from 'lucide-vue-next'

const tool = findTool('base64-to-image')
const { isDark } = useTheme()

// --- 流转 ---
const { pipelineFrom, downstreamTools, sendImageTo } = usePipeline({
  toolId: 'base64-to-image',
  onIncoming: async (incoming) => {
    if (incoming.type !== 'text') return false
    inputBase64.value = incoming.data.text
    scheduleDecode()
    return true
  }
})

function sendToTool(target: typeof downstreamTools.value[number]) {
  if (!decodedDataUrl.value || !parsedInfo.value) return
  const result = sendImageTo(target, {
    dataUrl: decodedDataUrl.value,
    fileName: `decoded.${parsedInfo.value.format}`,
    width: parsedInfo.value.width,
    height: parsedInfo.value.height
  })
  showToast(result.message, result.ok ? 'success' : 'error')
}

// --- 状态 ---
const inputBase64 = ref('')
const decodedDataUrl = ref('')
const parseError = ref('')
const parsedInfo = ref<{
  format: string
  width: number
  height: number
  sizeBytes: number
} | null>(null)
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')

let decodeTimer: ReturnType<typeof setTimeout> | null = null
let toastTimer: ReturnType<typeof setTimeout> | null = null
let objectUrl: string | null = null

// --- 解码 ---
function scheduleDecode() {
  if (decodeTimer) clearTimeout(decodeTimer)
  decodeTimer = setTimeout(() => {
    doDecode()
  }, 150)
}

function doDecode() {
  const raw = inputBase64.value.trim()
  if (!raw) {
    decodedDataUrl.value = ''
    parseError.value = ''
    parsedInfo.value = null
    return
  }

  let dataUrl: string

  // 已经是 data URL
  if (raw.startsWith('data:image/')) {
    // 检查 base64 部分是否有效
    const commaIdx = raw.indexOf(',')
    if (commaIdx < 0) {
      parseError.value = '无效的 data URL 格式'
      decodedDataUrl.value = ''
      parsedInfo.value = null
      return
    }
    dataUrl = raw
  } else {
    // 纯 base64 字符串，尝试推断格式
    let prefix = 'data:image/png;base64,'
    // 尝试从 base64 头部字节推断图片类型
    let testStr = raw
    // 移除可能的前缀 "data:image/xxx;base64," 如果存在
    const prefixMatch = testStr.match(/^data:image\/(\w+);base64,/)
    if (prefixMatch) {
      prefix = `data:image/${prefixMatch[1]};base64,`
      testStr = testStr.slice(prefixMatch[0].length)
    }
    dataUrl = prefix + testStr
  }

  if (!isValidBase64(dataUrl)) {
    parseError.value = 'Base64 解码失败，请检查输入'
    decodedDataUrl.value = ''
    parsedInfo.value = null
    return
  }

  parseError.value = ''

  // 加载图片以获取尺寸
  const img = new Image()
  img.onload = () => {
    decodedDataUrl.value = dataUrl
    parsedInfo.value = {
      format: guessFormat(dataUrl),
      width: img.naturalWidth,
      height: img.naturalHeight,
      sizeBytes: estimateSize(dataUrl)
    }
  }
  img.onerror = () => {
    parseError.value = '无法解码为图片，请检查 Base64 内容'
    decodedDataUrl.value = ''
    parsedInfo.value = null
  }
  img.src = dataUrl
}

function isValidBase64(dataUrl: string): boolean {
  const parts = dataUrl.split(',')
  if (parts.length < 2) return false
  try {
    atob(parts[1])
    return true
  } catch {
    return false
  }
}

function guessFormat(dataUrl: string): string {
  const match = dataUrl.match(/data:image\/(\w+);/)
  return match?.[1] || 'png'
}

function estimateSize(dataUrl: string): number {
  const parts = dataUrl.split(',')
  if (parts.length < 2) return 0
  try {
    return atob(parts[1]).length
  } catch {
    return 0
  }
}

// --- 操作 ---
function downloadImage() {
  if (!decodedDataUrl.value) return
  const ext = parsedInfo.value?.format || 'png'
  const a = document.createElement('a')
  a.href = decodedDataUrl.value
  a.download = `decoded.${ext}`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  showToast('下载已开始', 'success')
}

function clearAll() {
  inputBase64.value = ''
  decodedDataUrl.value = ''
  parseError.value = ''
  parsedInfo.value = null
  clearObjectUrl()
}

function clearObjectUrl() {
  if (objectUrl) {
    URL.revokeObjectURL(objectUrl)
    objectUrl = null
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

function showToast(message: string, type: 'success' | 'error') {
  if (toastTimer) clearTimeout(toastTimer)
  toastMessage.value = message
  toastType.value = type
  toastTimer = setTimeout(() => {
    toastMessage.value = ''
  }, 2200)
}

// 监听输入变化（处理粘贴等场景）
watch(inputBase64, () => {
  if (inputBase64.value.trim()) scheduleDecode()
})

// --- 生命周期 ---
onUnmounted(() => {
  clearObjectUrl()
  if (decodeTimer) clearTimeout(decodeTimer)
  if (toastTimer) clearTimeout(toastTimer)
})
</script>

<style scoped>
.heading-icon { --tool-color: #10b981; }

/* --- 左侧 textarea --- */
.panel-left textarea {
  width: 100%;
  padding: 0.625rem 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 0.75rem;
  font-family: var(--font-family-mono, monospace);
  resize: vertical;
  outline: none;
  box-sizing: border-box;
  line-height: 1.5;
  word-break: break-all;
}
.panel-left textarea:focus { border-color: var(--brand-500); }

/* --- 解析错误 --- */
.parse-error {
  display: flex; align-items: center; gap: 0.375rem;
  padding: 0.375rem 0.625rem; border-radius: 0.375rem;
  background: color-mix(in srgb, #ef4444 10%, transparent);
  color: #b91c1c; font-size: 0.8125rem; font-weight: 600;
}

/* --- 信息 --- */
.info-badge {
  padding: 0.375rem 0.625rem; border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-secondary); font-size: 0.8125rem;
  font-family: var(--font-family-mono, monospace);
}

/* --- 预览 --- */
.preview-area {
  flex: 1; min-height: 14rem;
  display: flex; align-items: center; justify-content: center;
  border-radius: 0.375rem; background: #fff; overflow: hidden;
}
.preview-area.empty { background: var(--bg-elevated); }
.preview-area img { max-width: 100%; max-height: 300px; object-fit: contain; }
.preview-empty {
  display: flex; flex-direction: column; align-items: center; gap: 0.5rem;
  color: var(--text-secondary); font-size: 0.875rem;
}
</style>
