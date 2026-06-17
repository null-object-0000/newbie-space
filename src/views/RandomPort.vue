<template>
  <div class="tool-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />
    <main class="tool-main">
      <div class="tool-topbar">
        <router-link to="/tools" class="back-link"><ArrowLeft :size="16" /><span>工具中心</span></router-link>
      </div>
      <section class="tool-header">
        <div class="tool-heading">
          <div class="heading-icon"><Network :size="22" /></div>
          <div><h1>随机端口生成</h1><p>在合法端口范围 (1–65535) 内随机生成，可注册端口及以下默认排除。</p></div>
        </div>
      </section>

      <div class="workspace">
        <div class="panel panel-left">
          <label class="section-label">排除范围</label>
          <div class="preset-row">
            <button
              v-for="p in presets"
              :key="p.key"
              class="preset-btn"
              :class="{ active: range === p.key }"
              @click="range = p.key"
            >{{ p.label }}</button>
          </div>
          <div class="custom-range">
            <input v-model.number="customMin" type="number" min="1" max="65535" class="range-input" placeholder="从">
            <span class="range-sep">–</span>
            <input v-model.number="customMax" type="number" min="1" max="65535" class="range-input" placeholder="到">
            <button class="btn primary" @click="refresh" :disabled="!validRange">生成</button>
          </div>
          <div v-if="history.length" class="history-bar">
            <button v-for="(p, i) in history" :key="i" class="history-chip" @click="port = p">
              {{ p }}
            </button>
          </div>
        </div>

        <div class="panel panel-right port-display">
          <span v-if="port === null" class="port-empty">点击生成</span>
          <span v-else class="port-number">{{ port }}</span>
          <div v-if="port !== null" class="port-actions">
            <button class="btn primary" @click="copyPort"><Copy :size="16" />{{ copyLabel }}</button>
            <button class="btn secondary" @click="refresh"><RotateCcw :size="16" />刷新</button>
            <PipelineSend :tools="downstreamTools" :disabled="port === null" @send="handlePipelineSend" />
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
import { usePipeline } from '@/composables/usePipeline'
import type { ToolItem } from '@/data/tools'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import { ArrowLeft, Copy, Network, RotateCcw } from 'lucide-vue-next'

const { isDark } = useTheme()

const { pipelineFrom, downstreamTools, sendTextTo } = usePipeline({
  toolId: 'random-port',
  async onIncoming() { return false }
})

function handlePipelineSend(target: ToolItem) {
  if (port.value === null) return
  const { ok, message } = sendTextTo(target, String(port.value))
  showToast(message, ok ? 'success' : 'error')
}

type Preset = 'full' | 'unprivileged' | 'registered' | 'dynamic' | 'custom'

const presets: { key: Preset; label: string }[] = [
  { key: 'full', label: '全部 (1–65535)' },
  { key: 'unprivileged', label: '非特权 (1024+)' },
  { key: 'registered', label: '非注册 (1024–49151)' },
  { key: 'dynamic', label: '动态 (49152–65535)' },
  { key: 'custom', label: '自定义' }
]

const range = ref<Preset>('full')
const customMin = ref(3000)
const customMax = ref(9000)
const port = ref<number | null>(null)
const copyLabel = ref('复制')
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')
let toastTimer: ReturnType<typeof setTimeout> | null = null

const HISTORY_KEY = 'random-port-history'
const history = ref<number[]>([])
function loadHistory() { try { const r = localStorage.getItem(HISTORY_KEY); if (r) history.value = JSON.parse(r) } catch {} }
function saveHistory() { try { localStorage.setItem(HISTORY_KEY, JSON.stringify(history.value)) } catch {} }
function pushHistory(p: number) {
  const i = history.value.indexOf(p); if (i >= 0) history.value.splice(i, 1)
  history.value.unshift(p); if (history.value.length > 5) history.value.pop()
  saveHistory()
}

const rangeMap: Record<Preset, [number, number]> = {
  full: [1, 65535],
  unprivileged: [1024, 65535],
  registered: [1024, 49151],
  dynamic: [49152, 65535],
  custom: [0, 0] // placeholder
}

const validRange = computed(() => {
  if (range.value !== 'custom') return true
  return customMin.value >= 1 && customMax.value <= 65535 && customMin.value < customMax.value
})

function randomPort(min: number, max: number) {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

function refresh() {
  const [min, max] = range.value === 'custom' ? [customMin.value, customMax.value] : rangeMap[range.value]
  if (min >= max) return
  port.value = randomPort(min, max)
  pushHistory(port.value)
}

function copyPort() {
  if (port.value === null) return
  navigator.clipboard.writeText(String(port.value)).then(() => {
    copyLabel.value = '已复制'; showToast(`端口 ${port.value} 已复制`, 'success')
    setTimeout(() => { copyLabel.value = '复制' }, 1500)
  }).catch(() => showToast('复制失败', 'error'))
}

function showToast(m: string, t: 'success' | 'error') {
  if (toastTimer) clearTimeout(toastTimer); toastMessage.value = m; toastType.value = t
  toastTimer = setTimeout(() => { toastMessage.value = '' }, 2200)
}

onMounted(() => { loadHistory(); refresh() })
onUnmounted(() => { if (toastTimer) clearTimeout(toastTimer) })
</script>

<style scoped>
.tool-page { min-height: 100vh; background: var(--bg-main); color: var(--text-primary); }
.tool-main { width: 100%; max-width: 72rem; margin: 0 auto; padding: 5rem 1rem 2.5rem; }
@media (min-width: 640px) { .tool-main { padding: 5.5rem 1.5rem 3rem; } }
@media (min-width: 1024px) { .tool-main { padding: 5.5rem 2rem 3rem; } }

.tool-topbar { margin-bottom: 0.75rem; }
.back-link { display: inline-flex; align-items: center; gap: 0.375rem; color: var(--text-secondary); font-size: 0.8125rem; }
.back-link:hover { color: var(--brand-500); }

.tool-header { margin-bottom: 1.25rem; }
.tool-heading { display: flex; align-items: center; gap: 0.75rem; }
.heading-icon { width: 2.75rem; height: 2.75rem; display: flex; align-items: center; justify-content: center; border-radius: 0.75rem; color: #14b8a6; background: color-mix(in srgb, #14b8a6 14%, transparent); }
.tool-heading h1 { font-size: 1.375rem; margin: 0; line-height: 1.1; }
.tool-heading p { color: var(--text-secondary); font-size: 0.8125rem; margin: 0.125rem 0 0; }

.workspace { display: grid; grid-template-columns: 1fr; gap: 1rem; }
@media (min-width: 768px) { .workspace { grid-template-columns: 1fr 1fr; align-items: stretch; } }
.panel { background: var(--bg-surface); border: 1px solid var(--border-color); border-radius: 1rem; padding: 1.125rem; display: flex; flex-direction: column; gap: 1rem; }

.section-label { color: var(--text-secondary); font-size: 0.75rem; font-weight: 700; }

.preset-row { display: grid; grid-template-columns: 1fr 1fr; gap: 0.375rem; }
.preset-btn {
  min-height: 2rem; padding: 0 0.5rem;
  border: 1px solid var(--border-color); border-radius: 0.5rem;
  background: var(--bg-elevated); color: var(--text-secondary);
  font-size: 0.6875rem; font-weight: 600; cursor: pointer;
  transition: all 0.15s; text-align: center;
}
.preset-btn.active { border-color: #14b8a6; color: #0f766e; background: color-mix(in srgb, #14b8a6 10%, transparent); }
.preset-btn:hover { border-color: var(--brand-500); }

.custom-range { display: flex; align-items: center; gap: 0.375rem; }
.range-input { width: 0; flex: 1; padding: 0.4375rem 0.5rem; border: 1px solid var(--border-color); border-radius: 0.5rem; background: var(--bg-elevated); color: var(--text-primary); font-size: 0.8125rem; font-family: var(--font-family-mono, monospace); outline: none; box-sizing: border-box; text-align: center; }
.range-input:focus { border-color: var(--brand-500); }
.range-sep { color: var(--text-secondary); font-size: 0.8125rem; flex-shrink: 0; }

.history-bar { display: flex; flex-direction: column; gap: 0.25rem; }
.history-chip { width: 100%; padding: 0.375rem 0.625rem; border: 1px solid var(--border-color); border-radius: 0.375rem; background: var(--bg-elevated); color: var(--text-secondary); font-family: var(--font-family-mono, monospace); font-size: 0.8125rem; cursor: pointer; text-align: center; transition: border-color 0.15s, color 0.15s; }
.history-chip:hover { border-color: var(--brand-500); color: var(--brand-500); }

.port-display { justify-content: center; align-items: center; gap: 1.5rem; }
.port-empty { font-size: 1rem; color: var(--text-secondary); }
.port-number { font-size: 5rem; font-weight: 700; font-family: var(--font-family-mono, monospace); line-height: 1; color: #14b8a6; }
.port-actions { display: flex; flex-wrap: wrap; gap: 0.5rem; }

.btn {
  min-height: 2.25rem; display: inline-flex; align-items: center; justify-content: center;
  gap: 0.375rem; border: 0; border-radius: 0.625rem; padding: 0 0.875rem;
  font-weight: 700; font-size: 0.8125rem; cursor: pointer;
  transition: transform 0.15s, opacity 0.15s, background 0.15s;
}
.btn.primary { background: var(--brand-500); color: #fff; }
.btn.secondary { background: var(--bg-elevated); color: var(--text-primary); }
.btn:hover { transform: translateY(-1px); }
.btn:disabled { cursor: not-allowed; opacity: 0.5; transform: none; }

.toast { position: fixed; left: 50%; bottom: 1.5rem; z-index: 1000; transform: translateX(-50%); padding: 0.625rem 0.875rem; border-radius: 999px; color: #fff; background: #18181b; box-shadow: var(--shadow-3); font-size: 0.8125rem; font-weight: 700; }
.toast.success { background: #10b981; }
.toast.error { background: #ef4444; }
.toast-enter-active, .toast-leave-active { transition: opacity 0.2s, transform 0.2s; }
.toast-enter-from, .toast-leave-to { opacity: 0; transform: translate(-50%, 0.5rem); }
</style>
