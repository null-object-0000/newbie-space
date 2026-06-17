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
            <QrCode :size="22" />
          </div>
          <div>
            <h1>{{ tool?.name }}</h1>
            <p>{{ tool?.desc }}</p>
          </div>
        </div>
      </section>

      <div v-if="pipelineFrom" class="pipeline-banner">
        <ArrowRightLeft :size="14" />
        <span>来自「{{ pipelineFrom }}」的流转文本</span>
      </div>

      <div class="workspace">
        <!-- 左侧：输入 + 控制 -->
        <div class="panel panel-left">
          <textarea
            v-model="text"
            rows="2"
            placeholder="输入网址或文字…"
            @input="scheduleGenerate"
          ></textarea>

          <div class="ctrl-grid">
            <div class="ctrl-item">
              <label>尺寸</label>
              <div class="slider-row">
                <input type="range" v-model.number="size" min="160" max="800" step="20" @input="scheduleGenerate">
                <span>{{ size }}px</span>
              </div>
            </div>
            <div class="ctrl-item">
              <label>边距</label>
              <div class="slider-row">
                <input type="range" v-model.number="margin" min="0" max="8" step="1" @input="scheduleGenerate">
                <span>{{ margin }}</span>
              </div>
            </div>
            <div class="ctrl-item">
              <label>纠错</label>
              <div class="segmented">
                <button v-for="lvl in ecLevels" :key="lvl" :class="{ active: errorCorrectionLevel === lvl }" @click="setECLevel(lvl)">{{ lvl }}</button>
              </div>
            </div>
            <div class="ctrl-item colors">
              <label>颜色</label>
              <div class="color-pair">
                <input v-model="darkColor" type="color" class="pick" @input="scheduleGenerate" title="模块色">
                <input v-model="lightColor" type="color" class="pick" @input="scheduleGenerate" title="背景色">
                <button class="tiny-btn" @click="resetColors" title="默认"><RotateCcw :size="12" /></button>
              </div>
            </div>
          </div>
        </div>

        <!-- 右侧：预览 + 操作 -->
        <div class="panel panel-right">
          <div class="preview-area" :class="{ empty: !qrDataUrl }">
            <img v-if="qrDataUrl" :src="qrDataUrl" alt="二维码">
            <div v-else class="preview-empty">
              <QrCode :size="36" />
              <span>输入内容后自动生成</span>
            </div>
          </div>
          <div class="actions">
            <button class="btn secondary" :disabled="!qrDataUrl" @click="copyQR">
              <Copy :size="16" />{{ copyLabel }}
            </button>
            <button class="btn primary" :disabled="!qrDataUrl" @click="downloadQR">
              <Download :size="16" />下载 PNG
            </button>
            <PipelineSend
              :tools="downstreamTools"
              :disabled="!qrDataUrl"
              @send="sendQRTo"
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
          @click="pickHistory(item)"
        >{{ item }}</button>
      </div>
    </main>

    <Transition name="toast">
      <div v-if="toastMessage" class="toast" :class="toastType">{{ toastMessage }}</div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { usePipeline } from '@/composables/usePipeline'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import { findTool } from '@/utils/toolPipeline'

const tool = findTool('qrcode-gen')
import { useTheme } from '@/composables/useTheme'
import {
  ArrowLeft,
  ArrowRightLeft,
  Copy,
  Download,
  QrCode,
  RotateCcw
} from 'lucide-vue-next'
import {
  generateQRCode,
  type ErrorCorrectionLevel
} from '@/utils/qrcode'

const { isDark } = useTheme()

// --- 状态 ---
const text = ref('')
const size = ref(320)
const margin = ref(2)
const errorCorrectionLevel = ref<ErrorCorrectionLevel>('M')
const darkColor = ref('#000000')
const lightColor = ref('#ffffff')
const qrDataUrl = ref('')
const isGenerating = ref(false)
const copyLabel = ref('复制')
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')

let generateTimer: ReturnType<typeof setTimeout> | null = null
let toastTimer: ReturnType<typeof setTimeout> | null = null

// --- 流转 ---
const { pipelineFrom, downstreamTools, sendImageTo } = usePipeline({
  toolId: 'qrcode-gen',
  onIncoming: async (incoming) => {
    if (incoming.type !== 'text') return false
    text.value = incoming.data.text
    scheduleGenerate()
    return true
  }
})

function sendQRTo(target: typeof downstreamTools.value[number]) {
  if (!qrDataUrl.value) {
    showToast('请先生成二维码', 'error')
    return
  }
  const result = sendImageTo(target, {
    dataUrl: qrDataUrl.value,
    fileName: `qrcode-${size.value}x${size.value}.png`,
    width: size.value,
    height: size.value
  })
  showToast(result.message, result.ok ? 'success' : 'error')
}

const ecLevels: ErrorCorrectionLevel[] = ['L', 'M', 'Q', 'H']

// --- 历史记录 ---
const HISTORY_KEY = 'qrcode-history'
const history = ref<string[]>([])

function loadHistory() {
  try {
    const raw = localStorage.getItem(HISTORY_KEY)
    if (raw) history.value = JSON.parse(raw)
  } catch { /* ignore */ }
}

function saveHistory() {
  try {
    localStorage.setItem(HISTORY_KEY, JSON.stringify(history.value))
  } catch { /* ignore */ }
}

function pushHistory(val: string) {
  const trimmed = val.trim()
  if (!trimmed) return
  // 去重：如果已存在则移到最前
  const idx = history.value.indexOf(trimmed)
  if (idx >= 0) history.value.splice(idx, 1)
  history.value.unshift(trimmed)
  if (history.value.length > 5) history.value.pop()
  saveHistory()
}

function pickHistory(val: string) {
  text.value = val
  scheduleGenerate()
}

// --- 生成 ---
function scheduleGenerate() {
  if (generateTimer) clearTimeout(generateTimer)
  generateTimer = setTimeout(() => {
    doGenerate()
  }, 200)
}

async function doGenerate() {
  if (!text.value.trim()) {
    qrDataUrl.value = ''
    return
  }

  if (isGenerating.value) return

  isGenerating.value = true
  try {
    const result = await generateQRCode({
      text: text.value,
      width: size.value,
      margin: margin.value,
      color: {
        dark: darkColor.value,
        light: lightColor.value
      },
      errorCorrectionLevel: errorCorrectionLevel.value
    })
    qrDataUrl.value = result.dataUrl
    pushHistory(text.value)
  } catch (error) {
    console.error(error)
    showToast(error instanceof Error ? error.message : '生成失败', 'error')
  } finally {
    isGenerating.value = false
  }
}

function setECLevel(level: ErrorCorrectionLevel) {
  if (errorCorrectionLevel.value === level) return
  errorCorrectionLevel.value = level
  scheduleGenerate()
}

function resetColors() {
  darkColor.value = '#000000'
  lightColor.value = '#ffffff'
  scheduleGenerate()
}

// --- 下载 / 复制 ---
function downloadQR() {
  if (!qrDataUrl.value) {
    showToast('请先生成二维码', 'error')
    return
  }

  const a = document.createElement('a')
  a.href = qrDataUrl.value
  a.download = `qr-${size.value}x${size.value}.png`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  showToast('下载已开始', 'success')
}

async function copyQR() {
  if (!qrDataUrl.value) {
    showToast('请先生成二维码', 'error')
    return
  }

  try {
    const blob = await (await fetch(qrDataUrl.value)).blob()
    await navigator.clipboard.write([
      new ClipboardItem({ 'image/png': blob })
    ])
    copyLabel.value = '已复制'
    showToast('二维码已复制到剪贴板', 'success')
    setTimeout(() => {
      copyLabel.value = '复制'
    }, 1500)
  } catch {
    showToast('复制失败，请尝试下载', 'error')
  }
}

function showToast(message: string, type: 'success' | 'error') {
  if (toastTimer) clearTimeout(toastTimer)
  toastMessage.value = message
  toastType.value = type
  toastTimer = setTimeout(() => {
    toastMessage.value = ''
  }, 2200)
}

onMounted(() => {
  loadHistory()
})

onUnmounted(() => {
  if (generateTimer) clearTimeout(generateTimer)
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
  color: #8b5cf6;
  background: color-mix(in srgb, #8b5cf6 14%, transparent);
}
.tool-heading h1 { font-size: 1.375rem; margin: 0; line-height: 1.1; }
.tool-heading p {
  color: var(--text-secondary); font-size: 0.8125rem; margin: 0.125rem 0 0;
}

/* ====== 双栏工作区 ====== */
.workspace {
  display: grid;
  grid-template-columns: 1fr;
  gap: 1rem;
}
@media (min-width: 768px) {
  .workspace {
    grid-template-columns: 1fr 1fr;
    align-items: stretch;
  }
}

.panel {
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 1rem;
  padding: 0.875rem;
  display: flex; flex-direction: column; gap: 1rem;
}

/* --- 左侧 --- */
.panel-left textarea {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 0.9375rem;
  font-family: inherit;
  resize: vertical;
  outline: none;
  box-sizing: border-box;
}
.panel-left textarea:focus { border-color: var(--brand-500); }

.history-bar {
  display: flex; flex-direction: column; gap: 0.25rem;
  margin-top: 0.75rem;
}
.history-chip {
  width: 100%; padding: 0.375rem 0.625rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-secondary);
  font-size: 0.6875rem; cursor: pointer; text-align: left;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  transition: border-color 0.15s, color 0.15s;
}
.history-chip:hover { border-color: var(--brand-500); color: var(--brand-500); }

.ctrl-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.875rem;
}
.ctrl-item {
  display: flex; flex-direction: column; gap: 0.375rem;
}
.ctrl-item label {
  color: var(--text-secondary); font-size: 0.75rem; font-weight: 700;
}
.ctrl-item.colors { grid-column: span 2; }
.slider-row {
  display: flex; align-items: center; gap: 0.625rem;
}
.slider-row input[type='range'] { flex: 1; accent-color: var(--brand-500); }
.slider-row span {
  font-family: var(--font-family-mono, monospace);
  font-size: 0.8125rem; color: var(--text-secondary); min-width: 3rem; text-align: right;
}
.segmented {
  display: grid; grid-template-columns: repeat(4, 1fr);
  padding: 0.25rem; border-radius: 0.5rem; background: var(--bg-elevated);
}
.segmented button {
  min-height: 2rem; border: 0; border-radius: 0.375rem;
  background: transparent; color: var(--text-secondary);
  font-weight: 600; font-size: 0.8125rem; cursor: pointer;
  transition: background 0.15s, color 0.15s;
}
.segmented button.active {
  background: var(--bg-surface); color: var(--text-primary); box-shadow: var(--shadow-1);
}
.color-pair {
  display: flex; align-items: center; gap: 0.625rem;
}
.pick {
  width: 2.25rem; height: 2.25rem; padding: 0.125rem;
  border: 1px solid var(--border-color); border-radius: 0.5rem;
  background: var(--bg-surface); cursor: pointer;
}
.tiny-btn {
  width: 2rem; height: 2rem; display: flex; align-items: center; justify-content: center;
  border: 1px solid var(--border-color); border-radius: 0.375rem;
  background: var(--bg-surface); color: var(--text-secondary); cursor: pointer;
  transition: border-color 0.15s, color 0.15s;
}
.tiny-btn:hover { border-color: var(--brand-500); color: var(--brand-500); }

/* --- 右侧 --- */
.preview-area {
  flex: 1; display: flex; align-items: center; justify-content: center;
  min-height: 20rem; border-radius: 0.375rem; background: #fff; overflow: hidden;
}
.preview-area.empty { background: var(--bg-elevated); }
.preview-area img { display: block; max-width: 100%; max-height: 360px; object-fit: contain; }
.preview-empty {
  display: flex; flex-direction: column; align-items: center; gap: 0.625rem;
  color: var(--text-secondary); font-size: 0.875rem;
}

.actions {
  display: flex; gap: 0.5rem;
}
.btn {
  min-height: 2.25rem; display: inline-flex; align-items: center; justify-content: center;
  gap: 0.375rem; border: 0; border-radius: 0.375rem; padding: 0 0.875rem;
  font-weight: 700; font-size: 0.8125rem; cursor: pointer;
  transition: transform 0.15s, opacity 0.15s, background 0.15s;
}
.btn.primary { background: var(--brand-500); color: #fff; }
.btn.secondary { background: var(--bg-elevated); color: var(--text-primary); }
.btn:hover { transform: translateY(-1px); }
.btn:disabled { cursor: not-allowed; opacity: 0.5; transform: none; }

.pipeline-banner {
  display: inline-flex; align-items: center; gap: 0.5rem;
  padding: 0.4375rem 0.75rem; margin-bottom: 0.75rem;
  border-radius: 0.5rem;
  background: color-mix(in srgb, #8b5cf6 10%, transparent);
  border: 1px solid color-mix(in srgb, #8b5cf6 25%, transparent);
  color: #6d28d9; font-size: 0.75rem; font-weight: 600;
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
