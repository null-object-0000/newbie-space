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
            <Crop :size="22" />
          </div>
          <div>
            <h1>{{ tool?.name }}</h1>
            <p>{{ tool?.desc }}</p>
          </div>
        </div>
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
          <label>原始尺寸</label>
          <span class="origin-size">{{ originalWidth }} × {{ originalHeight }} px</span>
        </div>

        <div class="control-group">
          <label for="resizeWidth">宽度 (px)</label>
          <div class="size-input-group">
            <input
              id="resizeWidth"
              v-model.number="targetWidth"
              type="number"
              min="1"
              max="9999"
              step="1"
              @input="onWidthChange"
            >
            <span class="percent-hint" v-if="percentMode && originalWidth">
              {{ widthPercent }}%
            </span>
          </div>
        </div>

        <div class="lock-control">
          <button
            class="lock-btn"
            :class="{ locked: keepAspectRatio }"
            type="button"
            @click="keepAspectRatio = !keepAspectRatio"
            :title="keepAspectRatio ? '点击解锁宽高比' : '点击锁定宽高比'"
          >
            <Link v-if="keepAspectRatio" :size="18" />
            <Unlink v-else :size="18" />
          </button>
        </div>

        <div class="control-group">
          <label for="resizeHeight">高度 (px)</label>
          <div class="size-input-group">
            <input
              id="resizeHeight"
              v-model.number="targetHeight"
              type="number"
              min="1"
              max="9999"
              step="1"
              @input="onHeightChange"
            >
            <span class="percent-hint" v-if="percentMode && originalHeight">
              {{ heightPercent }}%
            </span>
          </div>
        </div>

      </section>

      <section v-if="originalImage" class="quick-actions">
        <span class="quick-label">快捷尺寸：</span>
        <button
          v-for="preset in presets"
          :key="preset.label"
          class="preset-btn"
          type="button"
          @click="applyPreset(preset)"
        >{{ preset.label }}</button>
        <button
          class="preset-btn"
          :class="{ active: percentMode }"
          type="button"
          @click="percentMode = !percentMode"
        >{{ percentMode ? '按像素' : '按百分比' }}</button>
      </section>

      <section v-if="originalImage" class="preview-grid">
        <div class="preview-panel">
          <div class="panel-header">
            <span>原始图片</span>
            <span class="panel-badge">{{ originalWidth }} × {{ originalHeight }}</span>
          </div>
          <div class="canvas-frame">
            <img v-if="originalDataUrl" :src="originalDataUrl" alt="原始图片">
          </div>
        </div>

        <div class="preview-panel">
          <div class="panel-header">
            <span>调整后</span>
            <span class="panel-badge success">{{ targetWidth }} × {{ targetHeight }}</span>
          </div>
          <div class="canvas-frame">
            <img v-if="resizedDataUrl" :src="resizedDataUrl" alt="调整后图片">
            <span v-else class="empty-preview">调整尺寸后将在此预览</span>
          </div>
        </div>
      </section>

      <section v-if="originalImage" class="action-card">
        <div class="stats-bar">
          <span>原始 {{ formatSize(originalSizeBytes) }}</span>
          <span v-if="resizedSizeBytes">调整后 {{ formatSize(resizedSizeBytes) }}</span>
          <span v-if="sizeRatio">
            {{ sizeRatio > 1 ? '增大' : '减小' }} {{ Math.round(Math.abs(sizeRatio - 1) * 100) }}%
          </span>
        </div>
        <div class="action-buttons">
          <button class="btn primary" type="button" @click="applyResize">
            <Maximize2 :size="16" />
            <span>预览调整效果</span>
          </button>
          <button class="btn primary" type="button" :disabled="!resizedDataUrl" @click="downloadImage">
            <Download :size="16" />
            <span>下载 PNG</span>
          </button>
          <PipelineSend
            :tools="downstreamTools"
            :disabled="!resizedDataUrl"
            @send="sendToTool"
          />
        </div>
      </section>
    </main>

    <Transition name="toast">
      <div v-if="toastMessage" class="toast" :class="toastType">{{ toastMessage }}</div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useTheme } from '@/composables/useTheme'
import {
  ArrowLeft,
  ArrowRightLeft,
  Crop,
  Download,
  Link,
  Maximize2,
  RotateCcw,
  Unlink,
  UploadCloud
} from 'lucide-vue-next'
import {
  resizeImage,
  calcAspectHeight,
  calcAspectWidth,
  formatSize,
  type ResizeOptions
} from '@/utils/imageResize'
import { usePipeline } from '@/composables/usePipeline'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import { findTool } from '@/utils/toolPipeline'

const tool = findTool('image-resize')
const { isDark } = useTheme()

// --- 图片状态 ---
const fileInputRef = ref<HTMLInputElement | null>(null)
const originalImage = ref<HTMLImageElement | null>(null)
const originalDataUrl = ref('')
const originalFileName = ref('')
const originalWidth = ref(0)
const originalHeight = ref(0)
const originalSizeBytes = ref(0)
const isDragging = ref(false)

// --- 尺寸参数 ---
const targetWidth = ref(800)
const targetHeight = ref(600)
const keepAspectRatio = ref(true)
const percentMode = ref(false)

// --- 结果 ---
const resizedDataUrl = ref('')
const resizedSizeBytes = ref(0)

// --- UI ---
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')

let objectUrl: string | null = null
let toastTimer: ReturnType<typeof setTimeout> | null = null

// --- 流转 ---
const { pipelineFrom, downstreamTools, sendImageTo } = usePipeline({
  toolId: 'image-resize',
  onIncoming: async (incoming) => {
    if (incoming.type !== 'image') return false
    try {
      const blob = dataUrlToBlob(incoming.data.dataUrl)
      const file = new File([blob], incoming.data.fileName, { type: 'image/png' })
      await loadFile(file)
      showToast(`已接收来自「${incoming.data.fromTool}」的图片`, 'success')
      return true
    } catch {
      showToast('读取流转图片失败', 'error')
      return false
    }
  }
})

function sendToTool(target: typeof downstreamTools.value[number]) {
  if (!resizedDataUrl.value) {
    showToast('请先预览调整效果', 'error')
    return
  }
  const result = sendImageTo(target, {
    dataUrl: resizedDataUrl.value,
    fileName: originalFileName.value || 'resized.png',
    width: targetWidth.value,
    height: targetHeight.value
  })
  showToast(result.message, result.ok ? 'success' : 'error')
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

const widthPercent = computed(() => {
  if (!originalWidth.value || originalWidth.value <= 0) return 0
  return Math.round((targetWidth.value / originalWidth.value) * 100)
})

const heightPercent = computed(() => {
  if (!originalHeight.value || originalHeight.value <= 0) return 0
  return Math.round((targetHeight.value / originalHeight.value) * 100)
})

const sizeRatio = computed(() => {
  if (!originalSizeBytes.value || !resizedSizeBytes.value) return null
  return resizedSizeBytes.value / originalSizeBytes.value
})

const presets = [
  { label: '16×16', w: 16, h: 16 },
  { label: '32×32', w: 32, h: 32 },
  { label: '128×128', w: 128, h: 128 },
  { label: '512×512', w: 512, h: 512 },
  { label: '800×600', w: 800, h: 600 },
  { label: '1024×768', w: 1024, h: 768 },
  { label: '1920×1080', w: 1920, h: 1080 },
  { label: '原始尺寸', w: 0, h: 0 }
]

// --- 事件处理 ---
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
  originalSizeBytes.value = file.size

  try {
    const img = await loadImage(objectUrl)
    originalImage.value = img
    originalWidth.value = img.naturalWidth
    originalHeight.value = img.naturalHeight

    // 绘制原始预览
    const canvas = document.createElement('canvas')
    canvas.width = img.naturalWidth
    canvas.height = img.naturalHeight
    const ctx = canvas.getContext('2d')
    if (ctx) {
      ctx.drawImage(img, 0, 0)
      originalDataUrl.value = canvas.toDataURL('image/png')
    }

    // 初始目标尺寸设置为原始尺寸
    targetWidth.value = img.naturalWidth
    targetHeight.value = img.naturalHeight
    resizedDataUrl.value = ''
    resizedSizeBytes.value = 0

    await nextTick()
  } catch (error) {
    console.error(error)
    showToast('图片加载失败', 'error')
  }
}

// --- 宽高联动 ---
function onWidthChange() {
  if (keepAspectRatio.value && originalImage.value && targetWidth.value > 0) {
    targetHeight.value = calcAspectHeight(originalWidth.value, originalHeight.value, targetWidth.value)
  }
}

function onHeightChange() {
  if (keepAspectRatio.value && originalImage.value && targetHeight.value > 0) {
    targetWidth.value = calcAspectWidth(originalWidth.value, originalHeight.value, targetHeight.value)
  }
}

// 锁定按钮切换时同步
watch(keepAspectRatio, (locked) => {
  if (locked && originalImage.value) {
    targetHeight.value = calcAspectHeight(originalWidth.value, originalHeight.value, targetWidth.value)
  }
})

// --- 应用预设 ---
async function applyPreset(preset: typeof presets[number]) {
  if (preset.w === 0 && preset.h === 0) {
    // 原始尺寸
    if (originalImage.value) {
      targetWidth.value = originalWidth.value
      targetHeight.value = originalHeight.value
    }
  } else {
    targetWidth.value = preset.w
    targetHeight.value = preset.h
    if (keepAspectRatio.value && originalImage.value) {
      targetHeight.value = calcAspectHeight(originalWidth.value, originalHeight.value, targetWidth.value)
    }
  }
  await nextTick()
  applyResize()
}

// --- 执行缩放 ---
function applyResize() {
  if (!originalImage.value) {
    showToast('请先上传图片', 'error')
    return
  }

  if (!targetWidth.value || !targetHeight.value || targetWidth.value <= 0 || targetHeight.value <= 0) {
    showToast('请输入有效的尺寸', 'error')
    return
  }

  try {
    const opts: ResizeOptions = {
      width: targetWidth.value,
      height: targetHeight.value,
      format: 'png',
      quality: 1
    }

    const result = resizeImage(originalImage.value, opts)
    resizedDataUrl.value = result.dataUrl
    resizedSizeBytes.value = result.sizeBytes
    showToast('调整完成', 'success')
  } catch (error) {
    console.error(error)
    showToast(error instanceof Error ? error.message : '处理失败', 'error')
  }
}

// --- 下载 ---
function downloadImage() {
  if (!resizedDataUrl.value) {
    showToast('请先预览调整效果', 'error')
    return
  }

  const baseName = originalFileName.value.replace(/\.[^.]+$/, '') || 'resized'
  const downloadName = `${baseName}-${targetWidth.value}x${targetHeight.value}.png`

  const a = document.createElement('a')
  a.href = resizedDataUrl.value
  a.download = downloadName
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  showToast('下载已开始', 'success')
}

function resetAll() {
  clearObjectUrl()
  originalImage.value = null
  originalDataUrl.value = ''
  originalFileName.value = ''
  originalWidth.value = 0
  originalHeight.value = 0
  originalSizeBytes.value = 0
  targetWidth.value = 800
  targetHeight.value = 600
  resizedDataUrl.value = ''
  resizedSizeBytes.value = 0
  keepAspectRatio.value = true
  percentMode.value = false
  if (fileInputRef.value) fileInputRef.value.value = ''
  showToast('已重置', 'success')
}

// --- 公共工具 ---
function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => resolve(img)
    img.onerror = () => reject(new Error('图片加载失败'))
    img.src = src
  })
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
.tool-page {
  min-height: 100vh;
  background: var(--bg-main);
  color: var(--text-primary);
}

.tool-main {
  width: 100%;
  max-width: 72rem;
  margin: 0 auto;
  padding: 5rem 1rem 2.5rem;
}
@media (min-width: 640px) { .tool-main { padding: 5.5rem 1.5rem 3rem; } }
@media (min-width: 1024px) { .tool-main { padding: 5.5rem 2rem 3rem; } }

.tool-topbar { margin-bottom: 0.75rem; }
.back-link {
  display: inline-flex; align-items: center; gap: 0.375rem;
  color: var(--text-secondary); font-size: 0.8125rem;
}
.back-link:hover { color: var(--brand-500); }

.tool-header { margin-bottom: 1.25rem; }
.tool-heading {
  display: flex; align-items: center; gap: 0.75rem;
}
.heading-icon {
  width: 2.75rem; height: 2.75rem;
  display: flex; align-items: center; justify-content: center;
  border-radius: 0.5rem;
  color: #f59e0b;
  background: color-mix(in srgb, #f59e0b 14%, transparent);
}
.tool-kicker {
  color: var(--brand-500); font-size: 0.6875rem; font-weight: 700;
  letter-spacing: 0.06em; text-transform: uppercase; margin-bottom: 0.125rem;
}
.tool-header h1 { font-size: 1.375rem; line-height: 1.1; margin: 0; }
.tool-header p {
  color: var(--text-secondary); font-size: 0.8125rem; margin: 0.25rem 0 0; max-width: 36rem;
}

/* ====== 卡片 ====== */
.upload-card,
.controls-card,
.action-card {
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  padding: 0.875rem;
  margin-bottom: 0.75rem;
}

/* --- 上传区 --- */
.upload-zone {
  min-height: 8rem;
  display: flex; flex-direction: column;
  align-items: center; justify-content: center;
  gap: 0.5rem; padding: 1.5rem 1rem;
  border: 2px dashed color-mix(in srgb, var(--text-secondary) 40%, transparent);
  border-radius: 0.5rem;
  background: var(--bg-elevated);
  color: var(--text-secondary);
  text-align: center; cursor: pointer;
  transition: border-color 0.2s, color 0.2s;
}
.upload-zone input { display: none; }
.upload-zone strong { color: var(--text-primary); font-size: 0.9375rem; }
.upload-zone span { font-size: 0.8125rem; }
.upload-zone.drag-over,
.upload-zone:hover { border-color: var(--brand-500); color: var(--brand-500); }
.upload-zone.has-image { border-style: solid; border-color: #f59e0b; }

/* --- 控制区 --- */
.controls-card {
  display: grid;
  grid-template-columns: 1fr 1fr auto 1fr;
  gap: 0.75rem;
  align-items: end;
}
@media (max-width: 767px) { .controls-card { grid-template-columns: 1fr 1fr; } }

.control-group { display: flex; flex-direction: column; gap: 0.375rem; }
.control-group label {
  color: var(--text-secondary); font-size: 0.75rem; font-weight: 700;
}
.origin-size {
  font-family: var(--font-family-mono, monospace);
  font-size: 0.8125rem; font-weight: 600;
  color: var(--text-primary);
  padding: 0.4375rem 0.625rem; border-radius: 0.5rem;
  background: var(--bg-elevated);
}
.size-input-group { display: flex; align-items: center; gap: 0.5rem; }
.size-input-group input {
  width: 100%; min-height: 2.25rem;
  padding: 0.3125rem 0.625rem;
  border: 1px solid var(--border-color); border-radius: 0.5rem;
  background: var(--bg-surface); color: var(--text-primary);
  font-family: var(--font-family-mono, monospace); font-size: 0.875rem;
}
.percent-hint {
  color: var(--text-secondary); font-size: 0.75rem; white-space: nowrap;
}
.lock-control {
  display: flex; align-items: flex-end; justify-content: center; padding-bottom: 0.125rem;
}
.lock-btn {
  width: 2.25rem; height: 2.25rem;
  display: flex; align-items: center; justify-content: center;
  border: 1px solid var(--border-color); border-radius: 0.5rem;
  background: var(--bg-surface); color: var(--text-secondary);
  cursor: pointer; transition: all 0.15s;
}
.lock-btn.locked {
  border-color: var(--brand-500); color: var(--brand-500);
  background: color-mix(in srgb, var(--brand-500) 12%, transparent);
}
.lock-btn:hover { border-color: var(--brand-500); color: var(--brand-500); }

.mode-toggle {
  display: grid; grid-template-columns: 1fr 1fr 1fr;
  padding: 0.25rem; border-radius: 0.5rem; background: var(--bg-elevated);
}
.mode-toggle button {
  min-height: 2rem; border: 0; border-radius: 0.375rem;
  background: transparent; color: var(--text-secondary);
  font-weight: 600; font-size: 0.75rem; cursor: pointer;
  transition: background 0.15s, color 0.15s;
}
.mode-toggle button.active {
  background: var(--bg-surface); color: var(--text-primary); box-shadow: var(--shadow-1);
}
.controls-card input[type='range'] { width: 100%; accent-color: var(--brand-500); }

/* --- 快捷操作 --- */
.quick-actions {
  display: flex; flex-wrap: wrap; align-items: center;
  gap: 0.375rem; margin-bottom: 0.75rem;
}
.quick-label {
  color: var(--text-secondary); font-size: 0.75rem; font-weight: 600;
}
.preset-btn {
  min-height: 1.75rem; padding: 0.1875rem 0.5rem;
  border: 1px solid var(--border-color); border-radius: 0.375rem;
  background: var(--bg-surface); color: var(--text-secondary);
  font-size: 0.6875rem; font-weight: 600; cursor: pointer;
  transition: all 0.15s;
}
.preset-btn:hover,
.preset-btn.active { border-color: var(--brand-500); color: var(--brand-500); }

/* --- 按钮 --- */
.btn {
  min-height: 2.25rem;
  display: inline-flex; align-items: center; justify-content: center;
  gap: 0.375rem; border: 0; border-radius: 0.625rem;
  padding: 0 0.875rem; font-weight: 700; font-size: 0.8125rem;
  cursor: pointer;
  transition: transform 0.15s, opacity 0.15s, background 0.15s;
}
.btn.primary { background: var(--brand-500); color: #fff; }
.btn:hover { transform: translateY(-1px); }
.btn:disabled { cursor: not-allowed; opacity: 0.5; transform: none; }

.pipeline-banner {
  display: inline-flex; align-items: center; gap: 0.5rem;
  padding: 0.4375rem 0.75rem; margin-bottom: 0.75rem;
  border-radius: 0.5rem;
  background: color-mix(in srgb, #10b981 10%, transparent);
  border: 1px solid color-mix(in srgb, #10b981 25%, transparent);
  color: #047857; font-size: 0.75rem; font-weight: 600;
}

/* --- 预览区 --- */
.preview-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 0.75rem;
  margin-bottom: 0.75rem;
}
@media (min-width: 900px) { .preview-grid { grid-template-columns: repeat(2, 1fr); } }

.preview-panel {
  overflow: hidden;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-surface);
}
.panel-header {
  display: flex; align-items: center; justify-content: space-between;
  gap: 0.75rem; padding: 0.5rem 0.75rem;
  border-bottom: 1px solid var(--border-color);
  color: var(--text-primary); font-weight: 700; font-size: 0.8125rem;
}
.panel-badge {
  padding: 0.1875rem 0.4375rem; border-radius: 999px;
  background: var(--bg-elevated); color: var(--text-secondary);
  font-size: 0.6875rem; font-weight: 600;
}
.panel-badge.success {
  color: #b45309;
  background: color-mix(in srgb, #f59e0b 20%, transparent);
}

.canvas-frame {
  position: relative; width: 100%; aspect-ratio: 1 / 1;
  display: flex; align-items: center; justify-content: center;
  overflow: hidden;
}
.canvas-frame img { max-width: 100%; max-height: 100%; object-fit: contain; }
.empty-preview {
  position: absolute; color: var(--text-secondary); font-size: 0.8125rem;
}

/* --- 操作区 --- */
.action-card {
  display: flex; flex-direction: column; gap: 0.75rem;
}
@media (min-width: 768px) {
  .action-card { flex-direction: row; align-items: center; justify-content: space-between; }
}
.stats-bar {
  display: flex; flex-wrap: wrap; gap: 0.375rem;
  color: var(--text-secondary); font-size: 0.75rem;
}
.stats-bar span {
  padding: 0.1875rem 0.5rem; border-radius: 999px; background: var(--bg-elevated);
}
.action-buttons { display: flex; flex-wrap: wrap; gap: 0.5rem; }

/* ====== Toast ====== */
.toast {
  position: fixed; left: 50%; bottom: 1.5rem; z-index: 1000;
  transform: translateX(-50%); padding: 0.625rem 0.875rem; border-radius: 999px;
  color: #fff; background: #18181b; box-shadow: var(--shadow-3);
  font-size: 0.8125rem; font-weight: 700;
}
.toast.success { background: #10b981; }
.toast.error { background: #ef4444; }
.toast-enter-active, .toast-leave-active { transition: opacity 0.2s, transform 0.2s; }
.toast-enter-from, .toast-leave-to { opacity: 0; transform: translate(-50%, 0.5rem); }
</style>
