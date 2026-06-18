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
            <Shuffle :size="22" />
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
        <!-- 左侧：上传 + 设置 -->
        <div class="panel panel-left">
          <label
            class="upload-zone"
            :class="{ 'drag-over': isDragging, 'has-image': !!originalDataUrl }"
            @dragenter.prevent="isDragging = true"
            @dragover.prevent="isDragging = true"
            @dragleave.prevent="isDragging = false"
            @drop.prevent="handleDrop"
          >
            <input ref="fileInputRef" type="file" accept="image/*" @change="handleFileChange">
            <UploadCloud :size="28" />
            <strong v-if="!originalDataUrl">点击上传图片或拖拽到此处</strong>
            <strong v-else>{{ originalFileName }}</strong>
          </label>

          <div class="format-bar">
            <span class="format-label">转为</span>
            <div class="segmented">
              <button
                v-for="fmt in formats"
                :key="fmt.key"
                :class="{ active: outputFormat === fmt.key }"
                @click="outputFormat = fmt.key"
              >{{ fmt.label }}</button>
            </div>
          </div>

          <div class="quality-bar" v-if="outputFormat !== 'png'">
            <label for="quality">画质 <strong>{{ quality }}%</strong></label>
            <input
              id="quality"
              v-model.number="quality"
              type="range"
              min="10"
              max="100"
              step="1"
            >
          </div>

          <div v-if="originalSizeBytes" class="size-badge">
            <span>原始 {{ formatSize(originalSizeBytes) }} · {{ originalWidth }}×{{ originalHeight }} · {{ originalFormat.toUpperCase() }}</span>
          </div>
        </div>

        <!-- 右侧：预览 + 下载 -->
        <div class="panel panel-right">
          <div class="preview-area" :class="{ empty: !convertedDataUrl }">
            <img v-if="convertedDataUrl" :src="convertedDataUrl" alt="转换后">
            <div v-else class="preview-empty">
              <Shuffle :size="32" />
              <span>转换后将在此预览</span>
            </div>
          </div>

          <div class="compare-bar" v-if="convertedDataUrl && originalSizeBytes">
            <span class="cmp-item">
              <span class="cmp-label">原始</span>
              <strong>{{ formatSize(originalSizeBytes) }}</strong>
            </span>
            <ArrowRight :size="14" class="cmp-arrow" />
            <span class="cmp-item converted">
              <span class="cmp-label">{{ outputFormat.toUpperCase() }}</span>
              <strong>{{ formatSize(convertedSizeBytes) }}</strong>
            </span>
            <span v-if="sizeRatio !== null" class="cmp-ratio" :class="sizeRatio < 1 ? 'smaller' : 'larger'">
              {{ sizeRatio < 1 ? '减小' : '增大' }} {{ Math.round(Math.abs(sizeRatio - 1) * 100) }}%
            </span>
          </div>

          <div class="actions">
            <button class="btn primary" :disabled="!convertedDataUrl" @click="convertImage">预览转换</button>
            <button class="btn secondary" :disabled="!convertedDataUrl" @click="downloadImage">
              <Download :size="16" />下载
            </button>
            <PipelineSend
              :tools="downstreamTools"
              :disabled="!convertedDataUrl"
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
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { usePipeline } from '@/composables/usePipeline'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import { findTool } from '@/utils/toolPipeline'
import {
  ArrowLeft,
  ArrowRight,
  ArrowRightLeft,
  Download,
  Shuffle,
  UploadCloud
} from 'lucide-vue-next'

const tool = findTool('format-convert')
const { isDark } = useTheme()

// --- 流转 ---
const { pipelineFrom, downstreamTools, sendImageTo } = usePipeline({
  toolId: 'format-convert',
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

// --- 格式 ---
interface FormatDef {
  key: 'png' | 'jpeg' | 'webp'
  label: string
}

const formats: FormatDef[] = [
  { key: 'png', label: 'PNG' },
  { key: 'jpeg', label: 'JPEG' },
  { key: 'webp', label: 'WebP' }
]

// --- 状态 ---
const fileInputRef = ref<HTMLInputElement | null>(null)
const originalImage = ref<HTMLImageElement | null>(null)
const originalDataUrl = ref('')
const originalFileName = ref('')
const originalFormat = ref('')
const originalWidth = ref(0)
const originalHeight = ref(0)
const originalSizeBytes = ref(0)
const isDragging = ref(false)

const outputFormat = ref<'png' | 'jpeg' | 'webp'>('webp')
const quality = ref(85)

const convertedDataUrl = ref('')
const convertedSizeBytes = ref(0)

const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')

let objectUrl: string | null = null
let toastTimer: ReturnType<typeof setTimeout> | null = null

const sizeRatio = computed(() => {
  if (!originalSizeBytes.value || !convertedSizeBytes.value) return null
  return convertedSizeBytes.value / originalSizeBytes.value
})

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
  originalFileName.value = file.name
  originalSizeBytes.value = file.size
  originalFormat.value = file.type.split('/')[1] || 'png'

  try {
    const img = await loadImage(objectUrl)
    originalImage.value = img
    originalWidth.value = img.naturalWidth
    originalHeight.value = img.naturalHeight

    const canvas = document.createElement('canvas')
    canvas.width = img.naturalWidth
    canvas.height = img.naturalHeight
    const ctx = canvas.getContext('2d')
    if (ctx) {
      ctx.drawImage(img, 0, 0)
      originalDataUrl.value = canvas.toDataURL('image/png')
    }

    convertedDataUrl.value = ''
    convertedSizeBytes.value = 0
  } catch {
    showToast('图片加载失败', 'error')
  }
}

// --- 转换 ---
function convertImage() {
  if (!originalImage.value) {
    showToast('请先上传图片', 'error')
    return
  }

  try {
    const canvas = document.createElement('canvas')
    canvas.width = originalWidth.value
    canvas.height = originalHeight.value
    const ctx = canvas.getContext('2d')
    if (!ctx) {
      showToast('当前浏览器不支持', 'error')
      return
    }
    ctx.drawImage(originalImage.value, 0, 0)

    const mime = `image/${outputFormat.value}`
    if (outputFormat.value === 'png') {
      convertedDataUrl.value = canvas.toDataURL('image/png')
    } else {
      convertedDataUrl.value = canvas.toDataURL(mime, quality.value / 100)
    }

    // 计算大小
    const parts = convertedDataUrl.value.split(',')
    const bytes = atob(parts[1])
    convertedSizeBytes.value = bytes.length
    showToast('转换完成', 'success')
  } catch {
    showToast('转换失败', 'error')
  }
}

// 格式 / 画质变化时自动重新转换
watch([outputFormat, quality], () => {
  if (originalImage.value) convertImage()
})

// --- 下载 ---
function downloadImage() {
  if (!convertedDataUrl.value) return

  const ext = outputFormat.value === 'jpeg' ? 'jpg' : outputFormat.value
  const base = originalFileName.value.replace(/\.[^.]+$/, '') || 'converted'
  const a = document.createElement('a')
  a.href = convertedDataUrl.value
  a.download = `${base}.${ext}`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  showToast('下载已开始', 'success')
}

// --- 流转 ---
function sendToTool(target: typeof downstreamTools.value[number]) {
  if (!convertedDataUrl.value) return
  const result = sendImageTo(target, {
    dataUrl: convertedDataUrl.value,
    fileName: `${originalFileName.value.replace(/\.[^.]+$/, '') || 'converted'}.${outputFormat.value === 'jpeg' ? 'jpg' : outputFormat.value}`,
    width: originalWidth.value,
    height: originalHeight.value
  })
  showToast(result.message, result.ok ? 'success' : 'error')
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
.upload-zone.has-image { border-style: solid; border-color: #06b6d4; }

.format-bar {
  display: flex; align-items: center; gap: 0.5rem;
}
.format-label {
  font-size: 0.875rem; color: var(--text-secondary); white-space: nowrap;
}

.quality-bar {
  display: flex; flex-direction: column; gap: 0.25rem;
}
.quality-bar label {
  font-size: 0.875rem; color: var(--text-secondary);
}
.quality-bar input { width: 100%; accent-color: var(--brand-500); }

.size-badge {
  padding: 0.375rem 0.625rem; border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-secondary); font-size: 0.8125rem;
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

/* --- 对比条 --- */
.compare-bar {
  display: flex; align-items: center; gap: 0.5rem;
  padding: 0.5rem 0.625rem; border-radius: 0.5rem;
  background: var(--bg-elevated);
}
.cmp-item { display: flex; flex-direction: column; gap: 0.125rem; }
.cmp-label { font-size: 0.625rem; color: var(--text-secondary); }
.cmp-item strong { font-size: 0.8125rem; }
.cmp-item.converted strong { color: #06b6d4; }
.cmp-arrow { color: var(--text-secondary); flex-shrink: 0; }
.cmp-ratio {
  margin-left: auto; font-size: 0.8125rem; font-weight: 700;
  padding: 0.125rem 0.375rem; border-radius: 0.25rem;
}
.cmp-ratio.smaller { color: #047857; background: color-mix(in srgb, #10b981 14%, transparent); }
.cmp-ratio.larger { color: #b45309; background: color-mix(in srgb, #f59e0b 14%, transparent); }
</style>
