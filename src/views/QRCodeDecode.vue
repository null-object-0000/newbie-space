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
            <ScanLine :size="22" />
          </div>
          <div>
            <h1>{{ tool?.name }}</h1>
            <p>{{ tool?.desc }}</p>
          </div>
        </div>
      </section>

      <div v-if="pipelineFrom" class="pipeline-banner">
        <ArrowRightLeft :size="14" />
        <span>来自「{{ pipelineFrom }}」的流转图片</span>
      </div>

      <div class="workspace">
        <div class="panel panel-left">
          <label
            class="upload-zone"
            :class="{ 'drag-over': isDragging, 'has-image': Boolean(imageData) }"
            @dragenter.prevent="isDragging = true"
            @dragover.prevent="isDragging = true"
            @dragleave.prevent="isDragging = false"
            @drop.prevent="handleDrop"
          >
            <input ref="fileInputRef" type="file" accept="image/*" @change="handleFileChange">
            <UploadCloud :size="28" />
            <strong>{{ imageData ? fileName : '点击上传或拖拽到此处' }}</strong>
            <span>支持 JPG、PNG、WebP，也可从剪贴板粘贴</span>
          </label>
        </div>

        <div class="panel panel-right">
          <div class="result-area" :class="{ empty: !result, fail: scanned && !result }">
            <div v-if="result" class="result-content">
              <div class="result-label">解析结果</div>
              <div class="result-text">{{ result.data }}</div>
              <div v-if="isURL" class="result-hint">检测到链接，点击下方按钮可直接打开</div>
            </div>
            <div v-else-if="scanned && !result" class="result-empty">
              <ScanLine :size="32" />
              <span>未识别到二维码</span>
            </div>
            <div v-else class="result-empty">
              <ScanLine :size="32" />
              <span>上传图片后自动解析</span>
            </div>
          </div>

          <div class="actions">
            <button class="btn secondary" :disabled="!imageData" @click="resetAll">
              <RotateCcw :size="16" />重新上传
            </button>
            <button class="btn primary" :disabled="!result" @click="copyResult">
              <Copy :size="16" />{{ copyLabel }}
            </button>
            <button v-if="isURL" class="btn primary" :disabled="!result" @click="openURL">
              <ExternalLink :size="16" />打开链接
            </button>
            <PipelineSend
              :tools="downstreamTools"
              :disabled="!result"
              @send="handlePipelineSend"
            />
          </div>
        </div>
      </div>

      <div v-if="history.length" class="history-bar">
        <button
          v-for="(item, i) in history"
          :key="i"
          class="history-chip"
          :title="item"
        >{{ item }}</button>
      </div>
    </main>

    <Transition name="toast">
      <div v-if="toastMessage" class="toast" :class="toastType">{{ toastMessage }}</div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { usePipeline, type PipelineIncoming } from '@/composables/usePipeline'
import { findTool } from '@/utils/toolPipeline'
import type { ToolItem } from '@/data/tools'

const tool = findTool('qrcode-decode')
import PipelineSend from '@/components/tools/PipelineSend.vue'
import {
  ArrowLeft,
  ArrowRightLeft,
  Copy,
  ExternalLink,
  RotateCcw,
  ScanLine,
  UploadCloud
} from 'lucide-vue-next'
import { decodeQRCodeRobust, type DecodeResult } from '@/utils/qrDecode'

const { isDark } = useTheme()

// --- Pipeline ---
const { pipelineFrom, downstreamTools, sendTextTo } = usePipeline({
  toolId: 'qrcode-decode',
  async onIncoming(incoming: PipelineIncoming) {
    if (incoming.type === 'image') {
      const blob = dataUrlToBlob(incoming.data.dataUrl)
      const file = new File([blob], incoming.data.fileName, { type: 'image/png' })
      await loadFile(file)
      return true
    }
    return false
  }
})

function handlePipelineSend(target: ToolItem) {
  if (!result.value) return
  const { ok, message } = sendTextTo(target, result.value.data)
  showToast(message, ok ? 'success' : 'error')
}

// --- 状态 ---
const fileInputRef = ref<HTMLInputElement | null>(null)
const imageData = ref<ImageData | null>(null)
const fileName = ref('')
const result = ref<DecodeResult | null>(null)
const scanned = ref(false)
const isDragging = ref(false)
const copyLabel = ref('复制')
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')

let objectUrl: string | null = null
let toastTimer: ReturnType<typeof setTimeout> | null = null

// --- 历史 ---
const HISTORY_KEY = 'qrdecode-history'
const history = ref<string[]>([])

function loadHistory() {
  try {
    const raw = localStorage.getItem(HISTORY_KEY)
    if (raw) history.value = JSON.parse(raw)
  } catch { /* */ }
}

function saveHistory() {
  try { localStorage.setItem(HISTORY_KEY, JSON.stringify(history.value)) } catch { /* */ }
}

function pushHistory(val: string) {
  const trimmed = val.trim()
  if (!trimmed) return
  const idx = history.value.indexOf(trimmed)
  if (idx >= 0) history.value.splice(idx, 1)
  history.value.unshift(trimmed)
  if (history.value.length > 5) history.value.pop()
  saveHistory()
}

const isURL = computed(() => {
  if (!result.value) return false
  try { new URL(result.value.data); return true } catch { return false }
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
  if (file) { event.preventDefault(); loadFile(file) }
}

async function loadFile(file: File) {
  if (!file.type.startsWith('image/')) { showToast('请选择图片文件', 'error'); return }
  clearObjectUrl()
  objectUrl = URL.createObjectURL(file)
  fileName.value = file.name || '剪贴板图片'
  result.value = null
  scanned.value = false

  try {
    const img = await loadImage(objectUrl)
    const canvas = document.createElement('canvas')
    canvas.width = img.naturalWidth
    canvas.height = img.naturalHeight
    const ctx = canvas.getContext('2d')
    if (!ctx) { showToast('浏览器不支持', 'error'); return }
    ctx.drawImage(img, 0, 0)
    imageData.value = ctx.getImageData(0, 0, canvas.width, canvas.height)

    const decoded = decodeQRCodeRobust(imageData.value)
    scanned.value = true
    if (decoded) {
      result.value = decoded
      pushHistory(decoded.data)
      showToast('解析成功', 'success')
    } else {
      result.value = null
      showToast('未识别到二维码', 'error')
    }
  } catch {
    showToast('图片加载失败', 'error')
  }
}

// --- 操作 ---
function copyResult() {
  if (!result.value) return
  navigator.clipboard.writeText(result.value.data).then(() => {
    copyLabel.value = '已复制'
    showToast('已复制到剪贴板', 'success')
    setTimeout(() => { copyLabel.value = '复制' }, 1500)
  }).catch(() => showToast('复制失败', 'error'))
}

function openURL() {
  if (!result.value) return
  window.open(result.value.data, '_blank', 'noopener,noreferrer')
}

function resetAll() {
  clearObjectUrl()
  imageData.value = null
  fileName.value = ''
  result.value = null
  scanned.value = false
  if (fileInputRef.value) fileInputRef.value.value = ''
  showToast('已重置', 'success')
}

// --- 公共 ---
function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => resolve(img)
    img.onerror = () => reject(new Error('图片加载失败'))
    img.src = src
  })
}

function showToast(msg: string, type: 'success' | 'error') {
  if (toastTimer) clearTimeout(toastTimer)
  toastMessage.value = msg; toastType.value = type
  toastTimer = setTimeout(() => { toastMessage.value = '' }, 2200)
}

function clearObjectUrl() {
  if (objectUrl) { URL.revokeObjectURL(objectUrl); objectUrl = null }
}

function dataUrlToBlob(dataUrl: string): Blob {
  const parts = dataUrl.split(',')
  const mime = parts[0].match(/:(.*?);/)?.[1] || 'image/png'
  const bytes = atob(parts[1])
  const buffer = new Uint8Array(bytes.length)
  for (let i = 0; i < bytes.length; i++) buffer[i] = bytes.charCodeAt(i)
  return new Blob([buffer], { type: mime })
}

onMounted(() => {
  window.addEventListener('paste', handlePaste)
  loadHistory()
})

onUnmounted(() => {
  window.removeEventListener('paste', handlePaste)
  clearObjectUrl()
  if (toastTimer) clearTimeout(toastTimer)
})
</script>

<style scoped>
.tool-page { min-height: 100vh; background: var(--bg-main); color: var(--text-primary); }

.tool-main {
  width: 100%; max-width: 72rem; margin: 0 auto;
  padding: 5rem 1rem 2.5rem;
}
@media (min-width: 640px) { .tool-main { padding: 5.5rem 1.5rem 3rem; } }
@media (min-width: 1024px) { .tool-main { padding: 5.5rem 2rem 3rem; } }

.tool-topbar { margin-bottom: 0.75rem; }
.back-link { display: inline-flex; align-items: center; gap: 0.375rem; color: var(--text-secondary); font-size: 0.8125rem; }
.back-link:hover { color: var(--brand-500); }

.tool-header { margin-bottom: 1.25rem; }
.tool-heading { display: flex; align-items: center; gap: 0.75rem; }
.heading-icon {
  width: 2.75rem; height: 2.75rem; display: flex; align-items: center; justify-content: center;
  border-radius: 0.5rem; color: #06b6d4;
  background: color-mix(in srgb, #06b6d4 14%, transparent);
}
.tool-heading h1 { font-size: 1.375rem; margin: 0; line-height: 1.1; }
.tool-heading p { color: var(--text-secondary); font-size: 0.8125rem; margin: 0.125rem 0 0; }

.pipeline-banner {
  display: inline-flex; align-items: center; gap: 0.5rem;
  padding: 0.4375rem 0.75rem; margin-bottom: 0.75rem; border-radius: 0.5rem;
  background: color-mix(in srgb, #06b6d4 10%, transparent);
  border: 1px solid color-mix(in srgb, #06b6d4 25%, transparent);
  color: #0e7490; font-size: 0.75rem; font-weight: 600;
}

/* ====== 双栏 ====== */
.workspace {
  display: grid; grid-template-columns: 1fr; gap: 1rem;
}
@media (min-width: 768px) { .workspace { grid-template-columns: 1fr 1fr; align-items: stretch; } }

.panel {
  background: var(--bg-surface); border: 1px solid var(--border-color);
  border-radius: 1rem; padding: 0.875rem;
  display: flex; flex-direction: column; gap: 1rem;
}

.upload-zone {
  flex: 1; min-height: 12rem;
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  gap: 0.5rem; padding: 1.5rem 1rem;
  border: 2px dashed color-mix(in srgb, var(--text-secondary) 40%, transparent);
  border-radius: 0.5rem; background: var(--bg-elevated);
  color: var(--text-secondary); text-align: center; cursor: pointer;
  transition: border-color 0.2s, color 0.2s;
}
.upload-zone input { display: none; }
.upload-zone strong { color: var(--text-primary); font-size: 0.9375rem; }
.upload-zone span { font-size: 0.8125rem; }
.upload-zone.drag-over,
.upload-zone:hover { border-color: var(--brand-500); color: var(--brand-500); }
.upload-zone.has-image { border-style: solid; border-color: #06b6d4; }

/* --- 右侧结果 --- */
.result-area {
  flex: 1; min-height: 12rem; display: flex; align-items: center; justify-content: center;
  border-radius: 0.375rem; background: var(--bg-elevated); overflow: hidden;
}
.result-area.empty { background: var(--bg-elevated); }
.result-area.fail { border: 1px solid color-mix(in srgb, #ef4444 30%, transparent); }
.result-empty {
  display: flex; flex-direction: column; align-items: center; gap: 0.5rem;
  color: var(--text-secondary); font-size: 0.8125rem;
}
.result-content {
  width: 100%; padding: 0.875rem; word-break: break-all;
}
.result-label {
  color: var(--text-secondary); font-size: 0.6875rem; font-weight: 700;
  text-transform: uppercase; letter-spacing: 0.04em; margin-bottom: 0.5rem;
}
.result-text {
  font-family: var(--font-family-mono, monospace);
  font-size: 0.9375rem; line-height: 1.5; color: var(--text-primary);
  background: var(--bg-surface); padding: 0.625rem 0.75rem; border-radius: 0.5rem;
}
.result-hint {
  margin-top: 0.5rem; color: var(--brand-500); font-size: 0.75rem;
}

/* --- 按钮 --- */
.actions { display: flex; flex-wrap: wrap; gap: 0.5rem; }
.btn {
  min-height: 2.25rem; display: inline-flex; align-items: center; justify-content: center;
  gap: 0.375rem; border: 0; border-radius: 0.375rem; padding: 0 0.875rem;
  font-weight: 700; font-size: 0.8125rem; cursor: pointer;
  transition: transform 0.15s, opacity 0.15s, background 0.15s;
}
.btn.primary { background: var(--brand-500); color: #fff; }
.btn.secondary { background: var(--bg-elevated); color: var(--text-primary); }
.btn.pipeline {
  background: color-mix(in srgb, #06b6d4 12%, transparent);
  color: #0e7490; border: 1px solid color-mix(in srgb, #06b6d4 25%, transparent);
}
.btn.pipeline:hover { background: color-mix(in srgb, #06b6d4 20%, transparent); }
.btn:hover { transform: translateY(-1px); }
.btn:disabled { cursor: not-allowed; opacity: 0.5; transform: none; }

/* --- 历史 --- */
.history-bar {
  display: flex; flex-direction: column; gap: 0.25rem; margin-top: 0.75rem;
}
.history-chip {
  width: 100%; padding: 0.375rem 0.625rem;
  border: 1px solid var(--border-color); border-radius: 0.375rem;
  background: var(--bg-elevated); color: var(--text-secondary);
  font-size: 0.6875rem; text-align: left;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    outline: none;
}

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
