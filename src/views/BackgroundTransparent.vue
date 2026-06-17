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
            <ImageOff :size="26" />
          </div>
          <div>
            <p class="tool-kicker">Image Tool</p>
            <h1>背景透明化</h1>
          </div>
        </div>
        <p>将白色或指定颜色背景转换为透明 PNG。图片只在浏览器本地处理，不会上传。</p>
      </section>

      <section class="upload-card">
        <label
          class="upload-zone"
          :class="{ 'drag-over': isDragging, 'has-image': Boolean(originalImage) }"
          @dragenter.prevent="isDragging = true"
          @dragover.prevent="isDragging = true"
          @dragleave.prevent="isDragging = false"
          @drop.prevent="handleDrop"
        >
          <input ref="fileInputRef" type="file" accept="image/*" @change="handleFileChange">
          <UploadCloud :size="34" />
          <strong>{{ originalImage ? originalFileName : '点击上传图片或拖拽到此处' }}</strong>
          <span>支持 JPG、PNG、WebP、GIF，也可从剪贴板粘贴</span>
        </label>
      </section>

      <div v-if="pipelineFrom" class="pipeline-banner">
        <ArrowRightLeft :size="14" />
        <span>来自「{{ pipelineFrom }}」的流转图片</span>
      </div>

      <section v-if="originalImage" class="controls-card">
        <div class="control-group">
          <label for="targetColor">目标背景色</label>
          <div class="color-control">
            <input id="targetColor" v-model="targetColor" type="color" @input="scheduleProcess">
            <span>{{ targetColor.toUpperCase() }}</span>
          </div>
        </div>

        <div class="control-group tolerance-control">
          <label for="tolerance">容差范围: <strong>{{ tolerance }}</strong></label>
          <input id="tolerance" v-model.number="tolerance" type="range" min="0" max="100" step="1" @input="scheduleProcess">
        </div>

        <div class="control-group">
          <label>处理模式</label>
          <div class="mode-toggle" role="group" aria-label="处理模式">
            <button :class="{ active: mode === 'background' }" type="button" @click="setMode('background')">
              仅背景
            </button>
            <button :class="{ active: mode === 'global' }" type="button" @click="setMode('global')">
              全局替换
            </button>
          </div>
        </div>

        <button class="primary-button" type="button" :disabled="isProcessing" @click="processCurrentImage">
          <LoaderCircle v-if="isProcessing" :size="18" class="spin-icon" />
          <Sparkles v-else :size="18" />
          <span>{{ isProcessing ? '处理中' : '处理图片' }}</span>
        </button>
      </section>

      <section v-if="originalImage" class="preview-grid">
        <div class="preview-panel">
          <div class="panel-header">
            <span>原始图片</span>
            <span class="panel-badge">原图</span>
          </div>
          <div class="canvas-frame">
            <canvas ref="originalCanvasRef"></canvas>
          </div>
        </div>

        <div class="preview-panel">
          <div class="panel-header">
            <span>处理后</span>
            <span class="panel-badge success">透明背景</span>
          </div>
          <div class="canvas-frame">
            <canvas ref="processedCanvasRef"></canvas>
            <span v-if="!processedImageData" class="empty-preview">等待处理</span>
          </div>
        </div>
      </section>

      <section v-if="originalImage" class="action-card">
        <div class="stats-bar">
          <span v-if="stats">{{ stats.width }} x {{ stats.height }} px</span>
          <span v-if="stats">透明像素 {{ stats.transparentPixels.toLocaleString() }} / {{ stats.totalPixels.toLocaleString() }}</span>
          <span v-if="stats">{{ transparentPercent }}%</span>
        </div>
        <div class="action-buttons">
          <button class="secondary-button" type="button" @click="resetAll">
            <RotateCcw :size="16" />
            <span>重新上传</span>
          </button>
          <button
            class="pipeline-button"
            type="button"
            :disabled="!processedImageData"
            @click="sendToResize"
          >
            <ArrowRightLeft :size="16" />
            <span>发送到尺寸调整</span>
          </button>
          <button class="primary-button" type="button" :disabled="!processedImageData" @click="downloadImage">
            <Download :size="18" />
            <span>下载 PNG</span>
          </button>
        </div>
      </section>
    </main>

    <Transition name="toast">
      <div v-if="toastMessage" class="toast" :class="toastType">{{ toastMessage }}</div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import { useTheme } from '@/composables/useTheme'
import {
  ArrowLeft,
  ArrowRightLeft,
  Download,
  ImageOff,
  LoaderCircle,
  RotateCcw,
  Sparkles,
  UploadCloud
} from 'lucide-vue-next'
import {
  processImageData,
  type ProcessStats,
  type TransparentMode
} from '@/utils/backgroundTransparent'
import { storePipelineImage, consumePipelineImage } from '@/utils/toolPipeline'

const { isDark } = useTheme()

const fileInputRef = ref<HTMLInputElement | null>(null)
const originalCanvasRef = ref<HTMLCanvasElement | null>(null)
const processedCanvasRef = ref<HTMLCanvasElement | null>(null)
const originalImage = ref<HTMLImageElement | null>(null)
const originalImageData = ref<ImageData | null>(null)
const processedImageData = ref<ImageData | null>(null)
const stats = ref<ProcessStats | null>(null)
const originalFileName = ref('')
const targetColor = ref('#ffffff')
const tolerance = ref(30)
const mode = ref<TransparentMode>('background')
const isDragging = ref(false)
const isProcessing = ref(false)
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')

let objectUrl: string | null = null
let processTimer: ReturnType<typeof setTimeout> | null = null
let toastTimer: ReturnType<typeof setTimeout> | null = null

const pipelineFrom = ref('') // 如果从其他工具流转过来，显示来源

async function sendToResize() {
  if (!processedImageData.value) {
    showToast('请先处理图片', 'error')
    return
  }

  // 将处理后的图片导出为 data URL
  const canvas = document.createElement('canvas')
  canvas.width = processedImageData.value.width
  canvas.height = processedImageData.value.height
  const ctx = canvas.getContext('2d')
  if (!ctx) {
    showToast('当前浏览器不支持', 'error')
    return
  }
  ctx.putImageData(processedImageData.value, 0, 0)
  const dataUrl = canvas.toDataURL('image/png')

  const ok = storePipelineImage({
    dataUrl,
    fileName: originalFileName.value || 'transparent.png',
    fromTool: '背景透明化',
    width: processedImageData.value.width,
    height: processedImageData.value.height
  })

  if (ok) {
    showToast('已发送，即将跳转', 'success')
    setTimeout(() => {
      window.location.href = '/tools/image-resize'
    }, 300)
  } else {
    showToast('图片过大，无法流转（请下载后重新上传）', 'error')
  }
}

async function tryLoadPipelineImage() {
  const piped = consumePipelineImage()
  if (!piped) return

  pipelineFrom.value = piped.fromTool

  try {
    const img = await loadImage(piped.dataUrl)
    originalImage.value = img
    originalFileName.value = piped.fileName
    originalImageData.value = null
    processedImageData.value = null
    stats.value = null
    await nextTick()
    drawOriginalPreview()
    processCurrentImage()
    showToast(`已接收来自「${piped.fromTool}」的图片`, 'success')
  } catch {
    showToast('读取流转图片失败', 'error')
  }
}

const transparentPercent = computed(() => {
  if (!stats.value || stats.value.totalPixels === 0) return '0.0'
  return ((stats.value.transparentPixels / stats.value.totalPixels) * 100).toFixed(1)
})

onMounted(() => {
  window.addEventListener('paste', handlePaste)
  tryLoadPipelineImage()
})

onUnmounted(() => {
  window.removeEventListener('paste', handlePaste)
  clearObjectUrl()
  clearTimers()
})

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
  originalFileName.value = file.name || '剪贴板图片'

  try {
    const img = await loadImage(objectUrl)
    originalImage.value = img
    originalImageData.value = null
    processedImageData.value = null
    stats.value = null
    await nextTick()
    drawOriginalPreview()
    processCurrentImage()
  } catch (error) {
    console.error(error)
    showToast('图片加载失败', 'error')
  }
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => resolve(img)
    img.onerror = () => reject(new Error('图片加载失败'))
    img.src = src
  })
}

function setMode(nextMode: TransparentMode) {
  if (mode.value === nextMode) return
  mode.value = nextMode
  processCurrentImage()
}

function scheduleProcess() {
  if (processTimer) clearTimeout(processTimer)
  processTimer = setTimeout(() => {
    processCurrentImage()
  }, 160)
}

function processCurrentImage() {
  if (!originalImage.value || isProcessing.value) return

  isProcessing.value = true
  requestAnimationFrame(() => {
    try {
      const source = getOriginalImageData()
      const result = processImageData(source, {
        targetColor: targetColor.value,
        tolerance: tolerance.value,
        mode: mode.value
      })

      processedImageData.value = result.imageData
      stats.value = result.stats
      drawProcessedPreview()
      showToast('处理完成', 'success')
    } catch (error) {
      console.error(error)
      showToast(error instanceof Error ? error.message : '处理失败', 'error')
    } finally {
      isProcessing.value = false
    }
  })
}

function getOriginalImageData(): ImageData {
  if (originalImageData.value) return originalImageData.value
  if (!originalImage.value) throw new Error('请先上传图片')

  const canvas = document.createElement('canvas')
  canvas.width = originalImage.value.naturalWidth
  canvas.height = originalImage.value.naturalHeight
  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('当前浏览器不支持 Canvas')

  ctx.drawImage(originalImage.value, 0, 0)
  originalImageData.value = ctx.getImageData(0, 0, canvas.width, canvas.height)
  return originalImageData.value
}

function drawOriginalPreview() {
  if (!originalImage.value || !originalCanvasRef.value) return
  drawImageToCanvas(originalCanvasRef.value, originalImage.value)
}

function drawProcessedPreview() {
  if (!processedImageData.value || !processedCanvasRef.value) return
  drawImageDataToCanvas(processedCanvasRef.value, processedImageData.value)
}

function drawImageToCanvas(canvas: HTMLCanvasElement, image: HTMLImageElement) {
  const { width, height } = getPreviewSize(image.naturalWidth, image.naturalHeight)
  canvas.width = width
  canvas.height = height
  const ctx = canvas.getContext('2d')
  if (!ctx) return
  ctx.clearRect(0, 0, width, height)
  ctx.drawImage(image, 0, 0, width, height)
}

function drawImageDataToCanvas(canvas: HTMLCanvasElement, imageData: ImageData) {
  const { width, height } = getPreviewSize(imageData.width, imageData.height)
  canvas.width = width
  canvas.height = height
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  drawCheckerboard(ctx, width, height)
  const tempCanvas = document.createElement('canvas')
  tempCanvas.width = imageData.width
  tempCanvas.height = imageData.height
  const tempCtx = tempCanvas.getContext('2d')
  if (!tempCtx) return
  tempCtx.putImageData(imageData, 0, 0)
  ctx.drawImage(tempCanvas, 0, 0, width, height)
}

function getPreviewSize(sourceWidth: number, sourceHeight: number) {
  const maxDim = 520
  if (sourceWidth >= sourceHeight) {
    const width = Math.min(sourceWidth, maxDim)
    return { width: Math.round(width), height: Math.round((sourceHeight / sourceWidth) * width) }
  }

  const height = Math.min(sourceHeight, maxDim)
  return { width: Math.round((sourceWidth / sourceHeight) * height), height: Math.round(height) }
}

function drawCheckerboard(ctx: CanvasRenderingContext2D, width: number, height: number) {
  const squareSize = 12
  for (let y = 0; y < height; y += squareSize) {
    for (let x = 0; x < width; x += squareSize) {
      ctx.fillStyle = ((x / squareSize) + (y / squareSize)) % 2 === 0 ? '#f0f0f3' : '#dcdce0'
      ctx.fillRect(x, y, squareSize, squareSize)
    }
  }
}

function downloadImage() {
  if (!processedImageData.value) {
    showToast('请先处理图片', 'error')
    return
  }

  const canvas = document.createElement('canvas')
  canvas.width = processedImageData.value.width
  canvas.height = processedImageData.value.height
  const ctx = canvas.getContext('2d')
  if (!ctx) {
    showToast('当前浏览器不支持下载', 'error')
    return
  }

  ctx.putImageData(processedImageData.value, 0, 0)
  canvas.toBlob(blob => {
    if (!blob) {
      showToast('导出失败', 'error')
      return
    }

    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = buildDownloadName()
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
    showToast('下载已开始', 'success')
  }, 'image/png')
}

function buildDownloadName() {
  const baseName = originalFileName.value.replace(/\.[^.]+$/, '') || 'transparent-background'
  return `${baseName}-transparent.png`
}

function resetAll() {
  clearObjectUrl()
  originalImage.value = null
  originalImageData.value = null
  processedImageData.value = null
  stats.value = null
  originalFileName.value = ''
  targetColor.value = '#ffffff'
  tolerance.value = 30
  mode.value = 'background'
  if (fileInputRef.value) fileInputRef.value.value = ''
  showToast('已重置', 'success')
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

function clearTimers() {
  if (processTimer) clearTimeout(processTimer)
  if (toastTimer) clearTimeout(toastTimer)
}
</script>

<style scoped>
.tool-page {
  min-height: 100vh;
  background: var(--bg-main);
  color: var(--text-primary);
}

.tool-main {
  width: 100%;
  max-width: 72rem;
  margin: 0 auto;
  padding: 5.5rem 1rem 4rem;
}

@media (min-width: 640px) {
  .tool-main {
    padding: 6rem 1.5rem 4rem;
  }
}

@media (min-width: 1024px) {
  .tool-main {
    padding-left: 2rem;
    padding-right: 2rem;
  }
}

.tool-topbar {
  margin-bottom: 1rem;
}

.back-link {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  color: var(--text-secondary);
  font-size: 0.875rem;
}

.back-link:hover {
  color: var(--brand-500);
}

.tool-header {
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
  margin-bottom: 1.5rem;
}

.tool-heading {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.heading-icon {
  width: 3.25rem;
  height: 3.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 0.875rem;
  color: #10b981;
  background: color-mix(in srgb, #10b981 14%, transparent);
}

.tool-kicker {
  color: var(--brand-500);
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  margin-bottom: 0.25rem;
}

.tool-header h1 {
  font-size: 2rem;
  line-height: 1.1;
  margin: 0;
}

.tool-header p {
  max-width: 42rem;
  color: var(--text-secondary);
  margin: 0;
}

.upload-card,
.controls-card,
.action-card {
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 1rem;
  padding: 1rem;
  margin-bottom: 1rem;
}

.upload-zone {
  min-height: 12rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  padding: 2rem 1rem;
  border: 2px dashed color-mix(in srgb, var(--text-secondary) 45%, transparent);
  border-radius: 0.875rem;
  background: var(--bg-elevated);
  color: var(--text-secondary);
  text-align: center;
  cursor: pointer;
  transition: border-color 0.2s, background 0.2s, color 0.2s;
}

.upload-zone input {
  display: none;
}

.upload-zone strong {
  color: var(--text-primary);
  font-size: 1rem;
}

.upload-zone span {
  font-size: 0.875rem;
}

.upload-zone.drag-over,
.upload-zone:hover {
  border-color: var(--brand-500);
  color: var(--brand-500);
}

.upload-zone.has-image {
  border-style: solid;
  border-color: #10b981;
}

.controls-card {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 1rem;
  align-items: end;
}

@media (min-width: 768px) {
  .controls-card {
    grid-template-columns: 1fr 2fr 1.5fr auto;
  }
}

.control-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.control-group label {
  color: var(--text-secondary);
  font-size: 0.8125rem;
  font-weight: 700;
}

.color-control {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.color-control input {
  width: 2.5rem;
  height: 2.5rem;
  padding: 0.125rem;
  border: 1px solid var(--border-color);
  border-radius: 0.625rem;
  background: var(--bg-surface);
  cursor: pointer;
}

.color-control span {
  color: var(--text-secondary);
  font-family: var(--font-family-mono);
  font-size: 0.8125rem;
}

.tolerance-control input {
  width: 100%;
  accent-color: var(--brand-500);
}

.mode-toggle {
  display: grid;
  grid-template-columns: 1fr 1fr;
  padding: 0.25rem;
  border-radius: 0.75rem;
  background: var(--bg-elevated);
}

.mode-toggle button {
  min-height: 2.25rem;
  border: 0;
  border-radius: 0.55rem;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background 0.2s, color 0.2s;
}

.mode-toggle button.active {
  background: var(--bg-surface);
  color: var(--text-primary);
  box-shadow: var(--shadow-1);
}

.primary-button,
.secondary-button,
.pipeline-button {
  min-height: 2.5rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  border: 0;
  border-radius: 0.75rem;
  padding: 0 1rem;
  font-weight: 700;
  cursor: pointer;
  transition: transform 0.2s, opacity 0.2s, background 0.2s;
}

.primary-button {
  background: var(--brand-500);
  color: #fff;
}

.secondary-button {
  background: var(--bg-elevated);
  color: var(--text-primary);
}

.pipeline-button {
  background: color-mix(in srgb, #f59e0b 12%, transparent);
  color: #b45309;
  border: 1px solid color-mix(in srgb, #f59e0b 30%, transparent);
}

.pipeline-button:hover {
  background: color-mix(in srgb, #f59e0b 20%, transparent);
}

.pipeline-banner {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.875rem;
  margin-bottom: 1rem;
  border-radius: 0.625rem;
  background: color-mix(in srgb, #f59e0b 10%, transparent);
  border: 1px solid color-mix(in srgb, #f59e0b 25%, transparent);
  color: #b45309;
  font-size: 0.8125rem;
  font-weight: 600;
}

.primary-button:hover,
.secondary-button:hover,
.pipeline-button:hover {
  transform: translateY(-1px);
}

.primary-button:disabled,
.secondary-button:disabled,
.pipeline-button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
  transform: none;
}

.preview-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 1rem;
  margin-bottom: 1rem;
}

@media (min-width: 900px) {
  .preview-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

.preview-panel {
  overflow: hidden;
  border: 1px solid var(--border-color);
  border-radius: 1rem;
  background: var(--bg-surface);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--border-color);
  color: var(--text-primary);
  font-weight: 700;
}

.panel-badge {
  padding: 0.2rem 0.5rem;
  border-radius: 999px;
  background: var(--bg-elevated);
  color: var(--text-secondary);
  font-size: 0.75rem;
  font-weight: 600;
}

.panel-badge.success {
  color: #047857;
  background: color-mix(in srgb, #10b981 20%, transparent);
}

.canvas-frame {
  position: relative;
  width: 100%;
  aspect-ratio: 1 / 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  background-color: #fff;
  background-image:
    linear-gradient(45deg, #e8e8ec 25%, transparent 25%, transparent 75%, #e8e8ec 75%),
    linear-gradient(45deg, #e8e8ec 25%, transparent 25%, transparent 75%, #e8e8ec 75%);
  background-position: 0 0, 10px 10px;
  background-size: 20px 20px;
}

.canvas-frame canvas {
  max-width: 100%;
  max-height: 100%;
}

.empty-preview {
  position: absolute;
  color: #8b8b94;
  font-size: 0.875rem;
}

.action-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

@media (min-width: 768px) {
  .action-card {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
  }
}

.stats-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  color: var(--text-secondary);
  font-size: 0.8125rem;
}

.stats-bar span {
  padding: 0.25rem 0.625rem;
  border-radius: 999px;
  background: var(--bg-elevated);
}

.action-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
}

.spin-icon {
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.toast {
  position: fixed;
  left: 50%;
  bottom: 1.5rem;
  z-index: 1000;
  transform: translateX(-50%);
  padding: 0.75rem 1rem;
  border-radius: 999px;
  color: #fff;
  background: #18181b;
  box-shadow: var(--shadow-3);
  font-size: 0.875rem;
  font-weight: 700;
}

.toast.success {
  background: #10b981;
}

.toast.error {
  background: #ef4444;
}

.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.2s, transform 0.2s;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translate(-50%, 0.5rem);
}
</style>
