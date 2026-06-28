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
            <Image :size="22" />
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
        <!-- 左侧：上传 + 预览 -->
        <div class="panel panel-left">
          <label
            class="upload-zone"
            :class="{ 'drag-over': isDragging, 'has-image': !!sourceDataUrl }"
            @dragenter.prevent="isDragging = true"
            @dragover.prevent="isDragging = true"
            @dragleave.prevent="isDragging = false"
            @drop.prevent="handleDrop"
          >
            <input ref="fileInputRef" type="file" accept="image/*" @change="handleFileChange" />
            <UploadCloud :size="28" />
            <strong v-if="!sourceDataUrl">点击上传图片或拖拽到此处</strong>
            <strong v-else>{{ sourceFileName }}</strong>
          </label>

          <div v-if="sourceDataUrl" class="preview-box">
            <img :src="sourceDataUrl" alt="原始图片预览" />
          </div>

          <div v-if="sourceSizeBytes" class="info-badge">
            <span>{{ formatSize(sourceSizeBytes) }} · {{ sourceWidth }}×{{ sourceHeight }} · {{ sourceFormat.toUpperCase() }}</span>
          </div>
        </div>

        <!-- 右侧：Base64 输出 -->
        <div class="panel panel-right">
          <textarea
            v-model="base64Output"
            readonly
            rows="10"
            placeholder="Base64 编码结果将显示在这里…"
          ></textarea>

          <div class="meta" v-if="base64Output">
            <span>{{ base64Output.length.toLocaleString() }} 字符</span>
          </div>

          <div class="actions">
            <button class="btn primary" :disabled="!base64Output" @click="copyResult">
              <Copy :size="16" />{{ copyLabel }}
            </button>
            <button class="btn secondary" :disabled="!base64Output" @click="downloadTxt">
              <Download :size="16" />下载
            </button>
            <button class="btn secondary" :disabled="!sourceDataUrl" @click="clearAll">
              <Trash2 :size="16" />清空
            </button>
            <PipelineSend
              :tools="downstreamTools"
              :disabled="!base64Output"
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
import { onMounted, onUnmounted, ref } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { usePipeline } from '@/composables/usePipeline'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import { findTool } from '@/utils/toolPipeline'
import {
  ArrowLeft,
  ArrowRightLeft,
  Copy,
  Download,
  Image,
  Trash2,
  UploadCloud
} from 'lucide-vue-next'

const tool = findTool('image-to-base64')
const { isDark } = useTheme()

// --- 流转 ---
const { pipelineFrom, downstreamTools, sendTextTo } = usePipeline({
  toolId: 'image-to-base64',
  onIncoming: async (incoming) => {
    if (incoming.type !== 'image') return false
    try {
      const blob = dataUrlToBlob(incoming.data.dataUrl)
      const file = new File([blob], incoming.data.fileName, { type: 'image/png' })
      await loadFile(file)
      showToast(`已接收来自「${incoming.data.fromTool}」的图片`, 'success')
      return true
    } catch {
      showToast('读取传递数据失败', 'error')
      return false
    }
  }
})

function sendToTool(target: typeof downstreamTools.value[number]) {
  if (!base64Output.value) return
  const result = sendTextTo(target, base64Output.value)
  showToast(result.message, result.ok ? 'success' : 'error')
}

// --- 状态 ---
const fileInputRef = ref<HTMLInputElement | null>(null)
const sourceDataUrl = ref('')
const sourceFileName = ref('')
const sourceFormat = ref('')
const sourceWidth = ref(0)
const sourceHeight = ref(0)
const sourceSizeBytes = ref(0)
const base64Output = ref('')
const isDragging = ref(false)
const copyLabel = ref('复制')
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')

let objectUrl: string | null = null
let toastTimer: ReturnType<typeof setTimeout> | null = null

// --- 上传 ---
function handleFileChange(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (file) loadFile(file)
}

function handleDrop(event: DragEvent) {
  isDragging.value = false
  const file = event.dataTransfer?.files?.[0]
  if (file) loadFile(file)
}

function handlePaste(event: ClipboardEvent) {
  const items = Array.from(event.clipboardData?.items || [])
  const imageItem = items.find(item => item.type.startsWith('image/'))
  const file = imageItem?.getAsFile()
  if (file) {
    event.preventDefault()
    loadFile(file)
  }
}

async function loadFile(file: File) {
  if (!file.type.startsWith('image/')) {
    showToast('请选择图片文件', 'error')
    return
  }

  clearObjectUrl()
  objectUrl = URL.createObjectURL(file)
  sourceFileName.value = file.name
  sourceSizeBytes.value = file.size
  sourceFormat.value = file.type.split('/')[1] || 'png'

  try {
    const img = await loadImage(objectUrl)
    sourceWidth.value = img.naturalWidth
    sourceHeight.value = img.naturalHeight

    const canvas = document.createElement('canvas')
    canvas.width = img.naturalWidth
    canvas.height = img.naturalHeight
    const ctx = canvas.getContext('2d')
    if (ctx) {
      ctx.drawImage(img, 0, 0)
      sourceDataUrl.value = canvas.toDataURL('image/png')
      base64Output.value = sourceDataUrl.value
    }
  } catch {
    showToast('图片加载失败', 'error')
  }
}

// --- 操作 ---
async function copyResult() {
  if (!base64Output.value) return
  try {
    await navigator.clipboard.writeText(base64Output.value)
    copyLabel.value = '已复制'
    showToast('Base64 已复制到剪贴板', 'success')
    setTimeout(() => {
      copyLabel.value = '复制'
    }, 1500)
  } catch {
    showToast('复制失败', 'error')
  }
}

function downloadTxt() {
  if (!base64Output.value) return
  const base = sourceFileName.value.replace(/\.[^.]+$/, '') || 'base64'
  const blob = new Blob([base64Output.value], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${base}.txt`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
  showToast('下载已开始', 'success')
}

function clearAll() {
  sourceDataUrl.value = ''
  sourceFileName.value = ''
  sourceSizeBytes.value = 0
  base64Output.value = ''
  clearObjectUrl()
}

// --- 工具函数 ---
function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => resolve(img)
    img.onerror = () => reject(new Error('图片加载失败'))
    img.src = src
  })
}

function dataUrlToBlob(dataUrl: string): Blob {
  const parts = dataUrl.split(',')
  const mime = parts[0].match(/:(.*?);/)?.[1] || 'image/png'
  const bytes = atob(parts[1])
  const buffer = new Uint8Array(bytes.length)
  for (let i = 0; i < bytes.length; i++) {
    buffer[i] = bytes.charCodeAt(i)
  }
  return new Blob([buffer], { type: mime })
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

function clearObjectUrl() {
  if (objectUrl) {
    URL.revokeObjectURL(objectUrl)
    objectUrl = null
  }
}

// --- 生命周期 ---
onMounted(() => {
  window.addEventListener('paste', handlePaste)
})

onUnmounted(() => {
  window.removeEventListener('paste', handlePaste)
  clearObjectUrl()
  if (toastTimer) clearTimeout(toastTimer)
})
</script>

<style scoped>
.heading-icon { --tool-color: #10b981; }

/* --- 上传区 --- */
.upload-zone {
  min-height: 7rem;
  display: flex; flex-direction: column;
  align-items: center; justify-content: center;
  gap: 0.375rem; padding: 1.25rem 1rem;
  border: 2px dashed color-mix(in srgb, var(--text-secondary) 40%, transparent);
  border-radius: 0.5rem;
  background: var(--bg-elevated);
  text-align: center; cursor: pointer;
  transition: border-color 0.2s;
}
.upload-zone input { display: none; }
.upload-zone strong { color: var(--text-primary); font-size: 0.875rem; }
.upload-zone.drag-over,
.upload-zone:hover { border-color: var(--brand-500); }
.upload-zone.has-image { border-style: solid; border-color: #10b981; }

/* --- 预览 --- */
.preview-box {
  display: flex; align-items: center; justify-content: center;
  border-radius: 0.375rem; background: #fff; overflow: hidden;
  min-height: 8rem;
}
.preview-box img { max-width: 100%; max-height: 200px; object-fit: contain; }

/* --- 信息 --- */
.info-badge {
  padding: 0.375rem 0.625rem; border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-secondary); font-size: 0.8125rem;
}

/* --- 右侧 textarea --- */
.panel-right textarea {
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
  cursor: default;
  word-break: break-all;
}
.panel-right textarea:focus { border-color: var(--brand-500); }

/* --- 元信息 --- */
.meta {
  font-size: 0.875rem;
  color: var(--text-secondary);
  font-family: var(--font-family-mono, monospace);
}
</style>
