<template>
  <div class="tool-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />

    <main class="tool-main">
      <div class="tool-topbar">
        <router-link to="/tools" class="back-link">
          <ArrowLeft :size="16" /><span>工具中心</span>
        </router-link>
      </div>

      <section class="tool-header">
        <div class="tool-heading">
          <div class="heading-icon"><Clock :size="22" /></div>
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
        <!-- 左侧：输入 -->
        <div class="panel panel-left">
          <span class="section-label">输入</span>
          <textarea
            ref="inputRef"
            v-model="inputText"
            placeholder="输入日期字符串、时间戳…留空则为当前时间"
            rows="2"
            @input="scheduleParse"
          ></textarea>

          <div class="now-row">
            <button class="btn secondary" @click="useNow">当前时间</button>
            <button class="btn secondary" @click="clearInput">清空</button>
          </div>

          <div class="detected" v-if="inputText.trim()">
            <span class="detected-label">识别格式</span>
            <span class="detected-value" :class="{ invalid: !dateValid }">
              {{ dateValid ? detectedFormat : '无法解析' }}
            </span>
          </div>
        </div>

        <!-- 右侧：结果 -->
        <div class="panel panel-right">
          <span class="section-label">转换结果</span>

          <div class="results">
            <button
              v-for="fmt in formats"
              :key="fmt.name"
              class="result-row"
              @click="copyFormat(fmt)"
            >
              <span class="format-name">{{ fmt.name }}</span>
              <code class="format-value" :class="{ empty: !fmt.result }">{{
                fmt.result || '—'
              }}</code>
              <Copy :size="14" class="copy-hint" />
            </button>
          </div>

          <div class="actions">
            <button class="btn secondary" :disabled="!results.length" @click="copyAll">
              <Copy :size="14" />复制全部
            </button>
            <PipelineSend
              :tools="downstreamTools"
              :disabled="!parsedDate"
              @send="handlePipelineSend"
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
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { usePipeline, type PipelineIncoming } from '@/composables/usePipeline'
import type { ToolItem } from '@/data/tools'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import { findTool } from '@/utils/toolPipeline'
import { ArrowLeft, ArrowRightLeft, Clock, Copy } from 'lucide-vue-next'

const tool = findTool('date-converter')
const { isDark } = useTheme()

// --- 流转 ---
const { pipelineFrom, downstreamTools, sendTextTo } = usePipeline({
  toolId: 'date-converter',
  async onIncoming(incoming: PipelineIncoming) {
    if (incoming.type === 'text') {
      inputText.value = incoming.data.text
      scheduleParse()
      return true
    }
    return false
  }
})

function handlePipelineSend(target: ToolItem) {
  if (!parsedDate.value) return
  const { ok, message } = sendTextTo(target, formats.value[0]?.result || '')
  showToast(message, ok ? 'success' : 'error')
}

// --- 格式 ---
interface FormatDef {
  name: string
  fn: (d: Date) => string
}

const formatDefs: FormatDef[] = [
  { name: 'ISO 8601', fn: d => d.toISOString() },
  { name: '本地时间', fn: d => formatLocal(d) },
  { name: 'Unix 时间戳 (秒)', fn: d => String(Math.floor(d.getTime() / 1000)) },
  { name: 'Unix 时间戳 (毫秒)', fn: d => String(d.getTime()) },
  { name: 'UTC 字符串', fn: d => d.toUTCString() },
  { name: '相对时间', fn: d => formatRelative(d) }
]

// --- 状态 ---
const inputRef = ref<HTMLTextAreaElement | null>(null)
const inputText = ref('')
const parsedDate = ref<Date | null>(null)
const detectedFormat = ref('')
const dateValid = ref(false)
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')

let parseTimer: ReturnType<typeof setTimeout> | null = null
let toastTimer: ReturnType<typeof setTimeout> | null = null
let nowTimer: ReturnType<typeof setInterval> | null = null

// 当前时间参考 (每秒刷新)
const nowRef = ref(Date.now())

// --- 格式化结果 ---
const formats = computed(() => {
  const d = inputText.value.trim() ? parsedDate.value : new Date(nowRef.value)
  if (!d) return []
  return formatDefs.map(f => ({
    name: f.name,
    result: d ? safeFormat(f.fn, d) : ''
  }))
})

const results = computed(() => formats.value.filter(f => f.result))

function safeFormat(fn: (d: Date) => string, d: Date) {
  try { return fn(d) } catch { return '' }
}

// --- 解析 ---
function scheduleParse() {
  if (parseTimer) clearTimeout(parseTimer)
  parseTimer = setTimeout(doParse, 100)
}

function doParse() {
  const v = inputText.value.trim()
  if (!v) {
    parsedDate.value = null
    detectedFormat.value = ''
    dateValid.value = false
    return
  }

  const d = parseDate(v)
  if (d) {
    parsedDate.value = d
    detectedFormat.value = detectFormatStr(v)
    dateValid.value = true
  } else {
    parsedDate.value = null
    detectedFormat.value = ''
    dateValid.value = false
  }
}

function parseDate(input: string): Date | null {
  // Unix timestamp (seconds): exactly 10 digits, starts with 1
  if (/^\d{10}$/.test(input) && input.startsWith('1')) {
    return new Date(Number(input) * 1000)
  }
  // Unix timestamp (milliseconds): exactly 13 digits
  if (/^\d{13}$/.test(input) && input.startsWith('1')) {
    return new Date(Number(input))
  }
  // ISO 8601 or other parseable strings
  const ms = Date.parse(input)
  if (!isNaN(ms)) return new Date(ms)
  return null
}

function detectFormatStr(input: string): string {
  if (/^\d{10}$/.test(input) && input.startsWith('1')) return 'Unix 时间戳 (秒)'
  if (/^\d{13}$/.test(input) && input.startsWith('1')) return 'Unix 时间戳 (毫秒)'
  if (/^\d{4}-\d{2}-\d{2}T/.test(input)) return 'ISO 8601'
  if (/^[A-Z][a-z]{2},\s\d{2}\s[A-Z][a-z]{2}\s\d{4}/.test(input)) return 'UTC 字符串'
  return '日期字符串'
}

// --- 时间格式化 ---
function pad(n: number) { return String(n).padStart(2, '0') }

function formatLocal(d: Date): string {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

function formatRelative(d: Date): string {
  const diff = d.getTime() - nowRef.value
  const abs = Math.abs(diff)
  const suffix = diff < 0 ? '前' : '后'

  const s = Math.floor(abs / 1000)
  const m = Math.floor(s / 60)
  const h = Math.floor(m / 60)
  const day = Math.floor(h / 24)
  const mon = Math.floor(day / 30)
  const yr = Math.floor(day / 365)

  if (yr > 0) return `${yr} 年${suffix}`
  if (mon > 0) return `${mon} 个月${suffix}`
  if (day > 0) return `${day} 天${suffix}`
  if (h > 0) return `${h} 小时${suffix}`
  if (m > 0) return `${m} 分钟${suffix}`
  return `${s} 秒${suffix}`
}

// --- 操作 ---
function useNow() {
  inputText.value = ''
  parsedDate.value = null
  detectedFormat.value = ''
  dateValid.value = false
}

function clearInput() {
  inputText.value = ''
  parsedDate.value = null
  detectedFormat.value = ''
  dateValid.value = false
}

async function copyFormat(fmt: { name: string; result: string }) {
  try {
    await navigator.clipboard.writeText(fmt.result)
    showToast(`${fmt.name} 已复制`, 'success')
  } catch {
    showToast('复制失败', 'error')
  }
}

async function copyAll() {
  const lines = formats.value.map(f => `${f.name}: ${f.result}`).join('\n')
  try {
    await navigator.clipboard.writeText(lines)
    showToast('全部结果已复制', 'success')
  } catch {
    showToast('复制失败', 'error')
  }
}

function showToast(m: string, t: 'success' | 'error') {
  if (toastTimer) clearTimeout(toastTimer)
  toastMessage.value = m
  toastType.value = t
  toastTimer = setTimeout(() => { toastMessage.value = '' }, 2200)
}

// --- 生命周期 ---
onMounted(() => {
  inputRef.value?.focus()
  nowTimer = setInterval(() => { nowRef.value = Date.now() }, 1000)
})

onUnmounted(() => {
  if (parseTimer) clearTimeout(parseTimer)
  if (toastTimer) clearTimeout(toastTimer)
  if (nowTimer) clearInterval(nowTimer)
})
</script>

<style scoped>
.tool-page { min-height: 100vh; background: var(--bg-main); color: var(--text-primary); }
.tool-main { width: 100%; max-width: 72rem; margin: 0 auto; padding: 5rem 1rem 2.5rem; }
@media (min-width: 640px) { .tool-main { padding: 5.5rem 1.5rem 3rem; } }
@media (min-width: 1024px) { .tool-main { padding: 5.5rem 2rem 3rem; } }

.tool-topbar { margin-bottom: 0.75rem; }
.back-link {
  display: inline-flex; align-items: center; gap: 0.375rem;
  color: var(--text-secondary); font-size: 0.8125rem;
}
.back-link:hover { color: var(--brand-500); }

.tool-header { margin-bottom: 0.75rem; }
.tool-heading { display: flex; align-items: center; gap: 0.75rem; }
.heading-icon {
  width: 2.75rem; height: 2.75rem; display: flex; align-items: center; justify-content: center;
  border-radius: 0.5rem; color: #8b5cf6;
  background: color-mix(in srgb, #8b5cf6 14%, transparent);
}
.tool-heading h1 { font-size: 1.375rem; margin: 0; line-height: 1.1; }
.tool-heading p { color: var(--text-secondary); font-size: 0.8125rem; margin: 0.125rem 0 0; }

.pipeline-banner {
  display: inline-flex; align-items: center; gap: 0.5rem;
  padding: 0.375rem 0.625rem; margin-bottom: 0.75rem; border-radius: 0.375rem;
  background: color-mix(in srgb, #f59e0b 10%, transparent);
  border: 1px solid color-mix(in srgb, #f59e0b 25%, transparent);
  color: #b45309; font-size: 0.75rem; font-weight: 600;
}

/* ====== 双栏 ====== */
.workspace {
  display: grid; grid-template-columns: 1fr; gap: 0.75rem;
}
@media (min-width: 768px) {
  .workspace { grid-template-columns: 1fr 1fr; align-items: stretch; }
}

.panel {
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  padding: 0.875rem;
  display: flex; flex-direction: column; gap: 0.625rem;
}

.section-label {
  font-size: 0.6875rem; font-weight: 700;
  color: var(--text-secondary);
  text-transform: uppercase; letter-spacing: 0.04em;
}

/* --- 输入 --- */
.panel-left textarea {
  width: 100%; padding: 0.625rem 0.75rem;
  border: 1px solid var(--border-color); border-radius: 0.375rem;
  background: var(--bg-elevated); color: var(--text-primary);
  font-size: 0.9375rem; font-family: inherit; line-height: 1.6;
  resize: vertical; outline: none; box-sizing: border-box;
}
.panel-left textarea:focus { border-color: var(--brand-500); }

.now-row { display: flex; gap: 0.375rem; }

.detected {
  display: flex; align-items: center; gap: 0.5rem;
  padding: 0.375rem 0.625rem; border-radius: 0.25rem;
  background: var(--bg-elevated);
}
.detected-label {
  font-size: 0.6875rem; color: var(--text-secondary);
}
.detected-value {
  font-size: 0.75rem; font-weight: 600; font-family: var(--font-family-mono, monospace);
  color: #047857;
}
.detected-value.invalid { color: #ef4444; }

/* --- 结果 --- */
.results { display: flex; flex-direction: column; gap: 0.375rem; }

.result-row {
  display: grid; grid-template-columns: 8.5rem 1fr auto;
  align-items: center; gap: 0.5rem;
  padding: 0.375rem 0.625rem;
  border: 1px solid var(--border-color); border-radius: 0.25rem;
  background: var(--bg-elevated); color: inherit;
  cursor: pointer; text-align: left;
  width: 100%;
  transition: border-color 0.15s, background 0.15s;
}
.result-row:hover { border-color: var(--brand-500); }

.format-name {
  font-size: 0.6875rem; font-weight: 700;
  color: var(--text-primary);
}
.format-value {
  font-size: 0.75rem; font-family: var(--font-family-mono, monospace);
  line-height: 1.5; word-break: break-all;
  color: var(--text-primary); overflow: hidden;
  text-overflow: ellipsis; white-space: nowrap;
}
.format-value.empty { color: var(--text-muted); }

.copy-hint {
  opacity: 0; transition: opacity 0.15s;
  color: var(--text-secondary); flex-shrink: 0;
}
.result-row:hover .copy-hint { opacity: 0.6; }

/* --- 操作 --- */
.actions {
  display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem;
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

/* ====== Toast ====== */
.toast {
  position: fixed; left: 50%; bottom: 1.5rem; z-index: 1000;
  transform: translateX(-50%); padding: 0.5rem 0.75rem; border-radius: 999px;
  color: #fff; background: #18181b; box-shadow: var(--shadow-3);
  font-size: 0.8125rem; font-weight: 700;
}
.toast.success { background: #10b981; }
.toast.error { background: #ef4444; }
.toast-enter-active, .toast-leave-active { transition: opacity 0.2s, transform 0.2s; }
.toast-enter-from, .toast-leave-to { opacity: 0; transform: translate(-50%, 0.5rem); }
</style>
