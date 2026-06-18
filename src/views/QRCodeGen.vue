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
        <span>来自「{{ pipelineFrom }}」的传递数据</span>
      </div>

      <div class="workspace">
        <!-- 左侧：输入 + 控制 -->
        <div class="panel panel-left">
          <div class="textarea-wrap">
            <textarea
              v-model="text"
              rows="3"
              maxlength="2000"
              placeholder="请输入网址（如 https://example.com）或任意文本内容..."
              @input="scheduleGenerate"
            ></textarea>
            <span class="char-count">{{ text.length }} / 2000</span>
          </div>

          <div class="ctrl-grid">
            <div class="ctrl-item">
              <label>尺寸</label>
              <div class="segmented">
                <button
                  v-for="s in sizePresets" :key="s"
                  :class="{ active: size === s }"
                  @click="setSize(s)"
                >{{ s }}</button>
              </div>
            </div>
            <div class="ctrl-item">
              <label>边距（空白区）</label>
              <div class="segmented">
                <button
                  v-for="(m, i) in marginPresets" :key="i"
                  :class="{ active: marginIndex === i }"
                  @click="setMargin(i)"
                >{{ m }}%</button>
              </div>
            </div>
            <div class="ctrl-item">
              <label>纠错等级</label>
              <div class="segmented">
                <button
                  v-for="lvl in ecLevels" :key="lvl"
                  :class="{ active: errorCorrectionLevel === lvl }"
                  @click="setECLevel(lvl)"
                >{{ lvl }}</button>
              </div>
              <span class="ctrl-hint">纠错等级越高，容错能力越强，但密度也更高。</span>
            </div>
            <div class="ctrl-item colors">
              <label>颜色</label>
              <div class="color-pair">
                <div class="color-group">
                  <input v-model="darkColor" type="color" class="pick" @input="scheduleGenerate" title="前景色">
                  <span class="color-label">前景色</span>
                </div>
                <div class="color-group">
                  <input v-model="lightColor" type="color" class="pick" @input="scheduleGenerate" title="背景色" :disabled="transparentBg">
                  <span class="color-label">背景色</span>
                </div>
                <button class="tiny-btn" @click="resetColors" title="重置默认"><RotateCcw :size="12" /></button>
                <label class="checkbox-label">
                  <input type="checkbox" v-model="transparentBg" @change="scheduleGenerate">
                  <span>透明背景</span>
                </label>
              </div>
            </div>
          </div>
        </div>

        <!-- 右侧：预览 + 操作 -->
        <div class="panel panel-right">
          <h3 class="panel-title">二维码预览</h3>
          <div class="preview-area" :class="{ empty: !qrDataUrl }">
            <img v-if="qrDataUrl" :src="qrDataUrl" alt="二维码">
            <div v-else class="preview-empty">
              <QrCode :size="36" />
              <span>请在左侧输入内容生成二维码</span>
            </div>
          </div>
          <div class="actions">
            <button class="btn primary" :disabled="!qrDataUrl" @click="downloadQR">
              <Download :size="16" />下载 PNG
            </button>
            <button class="btn secondary" :disabled="!qrDataUrl" @click="copyQR">
              <Copy :size="16" />{{ copyLabel }}
            </button>
            <PipelineSend
              :tools="downstreamTools"
              :disabled="!qrDataUrl"
              @send="sendQRTo"
            />
          </div>
        </div>
      </div>

      <div class="tips-bar">
        <Info :size="14" />
        <span>建议使用 M 级纠错，在保证扫描成功率的同时，二维码密度更适中，美观度更高。</span>
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
  Info,
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
const sizePresets = [256, 512, 768, 1024]
const size = ref(512)
const marginPresets = [0, 10, 20, 30, 40]
const marginMap = [0, 2, 4, 6, 8] // 百分比 → 模块数
const marginIndex = ref(2) // 默认 20%
const errorCorrectionLevel = ref<ErrorCorrectionLevel>('M')
const darkColor = ref('#111827')
const lightColor = ref('#ffffff')
const transparentBg = ref(false)
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

function setSize(s: number) {
  size.value = s
  scheduleGenerate()
}

function setMargin(i: number) {
  marginIndex.value = i
  scheduleGenerate()
}

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
      margin: marginMap[marginIndex.value],
      color: {
        dark: darkColor.value,
        light: lightColor.value
      },
      errorCorrectionLevel: errorCorrectionLevel.value,
      transparentBg: transparentBg.value
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
  darkColor.value = '#111827'
  lightColor.value = '#ffffff'
  transparentBg.value = false
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
.heading-icon { --tool-color: #8b5cf6; }

/* --- Textarea with char count --- */
.textarea-wrap {
  position: relative;
}
.char-count {
  position: absolute;
  right: 0.5rem; bottom: 0.375rem;
  font-size: 0.8125rem;
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
  pointer-events: none;
}

/* --- Panel title --- */
.panel-title {
  font-size: 0.875rem; font-weight: 700; margin: 0;
}

/* --- Controls --- */
.ctrl-grid {
  display: flex; flex-direction: column; gap: 1rem;
}
.ctrl-item {
  display: flex; flex-direction: column; gap: 0.375rem;
}
.ctrl-item label {
  color: var(--text-secondary); font-size: 0.875rem; font-weight: 700;
}
.ctrl-item.colors { grid-column: span 2; }
.ctrl-hint {
  font-size: 0.8125rem; color: var(--text-muted); line-height: 1.4;
}

.color-pair {
  display: flex; align-items: center; gap: 0.75rem; flex-wrap: wrap;
}
.color-group {
  display: flex; align-items: center; gap: 0.375rem;
}
.color-label {
  font-size: 0.8125rem; color: var(--text-secondary);
}
.pick {
  width: 2.25rem; height: 2.25rem; padding: 0.125rem;
  border: 1px solid var(--border-color); border-radius: 0.5rem;
  background: var(--bg-surface); cursor: pointer;
}
.pick:disabled { opacity: 0.4; cursor: not-allowed; }
.tiny-btn {
  width: 2rem; height: 2rem; display: flex; align-items: center; justify-content: center;
  border: 1px solid var(--border-color); border-radius: 0.375rem;
  background: var(--bg-surface); color: var(--text-secondary); cursor: pointer;
  transition: border-color 0.15s, color 0.15s;
}
.tiny-btn:hover { border-color: var(--brand-500); color: var(--brand-500); }

.checkbox-label {
  display: flex; align-items: center; gap: 0.375rem;
  font-size: 0.875rem; color: var(--text-primary); cursor: pointer;
  white-space: nowrap;
}
.checkbox-label input[type='checkbox'] {
  accent-color: var(--brand-500);
}

/* --- Preview --- */
.preview-area {
  flex: 1; display: flex; align-items: center; justify-content: center;
  min-height: 20rem; border-radius: 0.375rem; background: #fff; overflow: hidden;
  /* 透明背景棋盘格 */
  background-image:
    linear-gradient(45deg, #ccc 25%, transparent 25%),
    linear-gradient(-45deg, #ccc 25%, transparent 25%),
    linear-gradient(45deg, transparent 75%, #ccc 75%),
    linear-gradient(-45deg, transparent 75%, #ccc 75%);
  background-size: 16px 16px;
  background-position: 0 0, 0 8px, 8px -8px, -8px 0;
}
.preview-area.empty {
  background: var(--bg-elevated);
  background-image: none;
}
.preview-area img { display: block; max-width: 100%; max-height: 360px; object-fit: contain; }
.preview-empty {
  display: flex; flex-direction: column; align-items: center; gap: 0.625rem;
  color: var(--text-secondary); font-size: 0.875rem;
}

/* --- Tips bar --- */
.tips-bar {
  display: flex; align-items: center; gap: 0.5rem;
  margin-top: 1rem; padding: 0.625rem 0.875rem;
  border-radius: 0.5rem;
  background: color-mix(in srgb, var(--brand-500) 6%, transparent);
  border: 1px solid color-mix(in srgb, var(--brand-500) 15%, transparent);
  color: var(--text-secondary); font-size: 0.875rem; line-height: 1.5;
}
.tips-bar svg { flex-shrink: 0; color: var(--brand-500); }

/* --- History --- */
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
  font-size: 0.8125rem; cursor: pointer; text-align: left;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  transition: border-color 0.15s, color 0.15s;
}
.history-chip:hover { border-color: var(--brand-500); color: var(--brand-500); }
</style>
