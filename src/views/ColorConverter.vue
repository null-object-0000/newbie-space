<template>
  <div class="tool-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />
    <main class="tool-main">
      <div class="tool-topbar">
        <router-link to="/tools" class="back-link"><ArrowLeft :size="16" /><span>工具中心</span></router-link>
      </div>
      <section class="tool-header">
        <div class="tool-heading">
          <div class="heading-icon"><Palette :size="22" /></div>
          <div><h1>颜色转换</h1><p>输入任意格式的颜色值，自动转换为 HEX / RGB / HSL / HSV / LCH / CMYK 等 10 种格式。</p></div>
        </div>
      </section>

      <div class="workspace">
        <div class="panel panel-left">
          <label class="input-label">输入颜色</label>
          <div class="input-row">
            <input
              v-model="input"
              type="text"
              placeholder="#3b82f6 / rgb(59,130,246) / hsl(217,91%,60%) / blue…"
              class="color-input"
              @input="scheduleConvert"
            >
            <label class="picker-wrap" title="从调色板选色">
              <input
                type="color"
                :value="hex || '#000000'"
                class="color-picker"
                @input="onPickerInput"
              >
              <Pipette :size="16" />
            </label>
          </div>
          <div class="swatch-row" v-if="hex">
            <div class="swatch" :style="{ background: hex }"></div>
            <span class="swatch-hex">{{ hex }}</span>
          </div>
          <div v-else class="swatch-empty">等待输入有效颜色</div>

          <div v-if="history.length" class="history-bar">
            <button
              v-for="(item, i) in history" :key="i"
              class="history-chip" @click="pickHistory(item)"
            >
              <span class="chip-swatch" :style="{ background: item }"></span>
              {{ item }}
            </button>
          </div>
        </div>

        <div class="panel panel-right">
          <div v-if="results.length" class="results-list">
            <div v-for="r in results" :key="r.label" class="result-row" @click="copyResult(r)">
              <span class="result-label">{{ r.label }}</span>
              <span class="result-value">{{ r.value }}</span>
              <span class="result-action">{{ copyMap[r.label] || '复制' }}</span>
            </div>
          </div>
          <div v-else class="results-empty">
            <Palette :size="32" />
            <span>输入颜色后自动转换</span>
          </div>
          <div class="actions">
            <PipelineSend :tools="downstreamTools" :disabled="!hex" @send="handlePipelineSend" />
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
import { onMounted, onUnmounted, reactive, ref } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { usePipeline, type PipelineIncoming } from '@/composables/usePipeline'
import type { ToolItem } from '@/data/tools'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import { ArrowLeft, Palette, Pipette } from 'lucide-vue-next'
import { convertColor, type ColorResult } from '@/utils/colorConverter'

const { isDark } = useTheme()

const { pipelineFrom, downstreamTools, sendTextTo } = usePipeline({
  toolId: 'color-converter',
  async onIncoming(incoming: PipelineIncoming) {
    if (incoming.type === 'text') { input.value = incoming.data.text; doConvert(); return true }
    return false
  }
})

function handlePipelineSend(target: ToolItem) {
  if (!hex.value) return
  const { ok, message } = sendTextTo(target, hex.value)
  showToast(message, ok ? 'success' : 'error')
}

const input = ref('')
const hex = ref('')
const results = ref<ColorResult[]>([])
const copyMap = reactive<Record<string, string>>({})
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')
let convertTimer: ReturnType<typeof setTimeout> | null = null
let toastTimer: ReturnType<typeof setTimeout> | null = null

const HISTORY_KEY = 'color-converter-history'
const history = ref<string[]>([])
function loadHistory() { try { const r = localStorage.getItem(HISTORY_KEY); if (r) history.value = JSON.parse(r) } catch {} }
function saveHistory() { try { localStorage.setItem(HISTORY_KEY, JSON.stringify(history.value)) } catch {} }
function pushHistory(v: string) {
  const t = v.trim(); if (!t) return
  const i = history.value.indexOf(t); if (i >= 0) history.value.splice(i, 1)
  history.value.unshift(t); if (history.value.length > 5) history.value.pop()
  saveHistory()
}
function pickHistory(val: string) { input.value = val; doConvert() }

function scheduleConvert() { if (convertTimer) clearTimeout(convertTimer); convertTimer = setTimeout(doConvert, 200) }
function onPickerInput(e: Event) {
  const val = (e.target as HTMLInputElement).value
  input.value = val
  doConvert()
}
function doConvert() {
  const v = input.value.trim()
  if (!v) { hex.value = ''; results.value = []; return }
  const c = convertColor(v)
  if (!c) { hex.value = ''; results.value = []; return }
  hex.value = c.hex; results.value = c.results
  pushHistory(c.hex)
}

function copyResult(r: ColorResult) {
  navigator.clipboard.writeText(r.raw).then(() => {
    copyMap[r.label] = '已复制'
    showToast(`${r.label} 已复制`, 'success')
    setTimeout(() => { delete copyMap[r.label] }, 1500)
  }).catch(() => showToast('复制失败', 'error'))
}

function showToast(m: string, t: 'success' | 'error') {
  if (toastTimer) clearTimeout(toastTimer); toastMessage.value = m; toastType.value = t
  toastTimer = setTimeout(() => { toastMessage.value = '' }, 2200)
}

onMounted(() => { loadHistory() })
onUnmounted(() => { if (convertTimer) clearTimeout(convertTimer); if (toastTimer) clearTimeout(toastTimer) })
</script>

<style scoped>
.heading-icon { --tool-color: #3b82f6; }

.input-label { color: var(--text-secondary); font-size: 0.875rem; font-weight: 700; }
.input-row { display: flex; gap: 0.5rem; align-items: stretch; }
.color-input { flex: 1; min-width: 0; padding: 0.625rem 0.75rem; border: 1px solid var(--border-color); border-radius: 0.625rem; background: var(--bg-elevated); color: var(--text-primary); font-size: 0.9375rem; font-family: var(--font-family-mono, monospace); outline: none; box-sizing: border-box; }
.color-input:focus { border-color: var(--brand-500); }

.picker-wrap {
  display: flex; align-items: center; justify-content: center;
  width: 2.75rem; flex-shrink: 0;
  border: 1px solid var(--border-color); border-radius: 0.625rem;
  background: var(--bg-elevated); color: var(--text-secondary);
  cursor: pointer; position: relative; overflow: hidden;
  transition: border-color 0.15s;
}
.picker-wrap:hover { border-color: var(--brand-500); color: var(--brand-500); }
.color-picker {
  position: absolute; inset: 0; opacity: 0;
  width: 100%; height: 100%; cursor: pointer;
  border: 0; padding: 0;
}

.swatch-row { display: flex; align-items: center; gap: 0.625rem; padding: 0.5rem 0.75rem; border-radius: 0.5rem; background: var(--bg-elevated); }
.swatch { width: 2rem; height: 2rem; border-radius: 0.375rem; border: 1px solid rgba(0,0,0,.1); flex-shrink: 0; }
.swatch-hex { font-family: var(--font-family-mono, monospace); font-size: 0.9375rem; font-weight: 600; }
.swatch-empty { display: flex; align-items: center; justify-content: center; padding: 1.5rem; border-radius: 0.5rem; background: var(--bg-elevated); color: var(--text-secondary); font-size: 0.8125rem; }

.history-bar { display: flex; flex-direction: column; gap: 0.25rem; }
.history-chip { width: 100%; padding: 0.3125rem 0.5rem; border: 1px solid var(--border-color); border-radius: 0.375rem; background: var(--bg-elevated); color: var(--text-secondary); font-size: 0.8125rem; cursor: pointer; text-align: left; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; transition: border-color 0.15s, color 0.15s; display: flex; align-items: center; gap: 0.375rem; }
.history-chip:hover { border-color: var(--brand-500); color: var(--brand-500); }
.chip-swatch { width: 0.875rem; height: 0.875rem; border-radius: 3px; flex-shrink: 0; border: 1px solid rgba(0,0,0,.1); }

.results-list { display: flex; flex-direction: column; gap: 0.25rem; }
.result-row { display: flex; align-items: center; gap: 0.5rem; padding: 0.4375rem 0.625rem; border-radius: 0.375rem; cursor: pointer; transition: background 0.15s; }
.result-row:hover { background: var(--bg-elevated); }
.result-label { font-size: 0.8125rem; font-weight: 700; color: var(--text-secondary); min-width: 3rem; }
.result-value { flex: 1; font-family: var(--font-family-mono, monospace); font-size: 0.8125rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.result-action { font-size: 0.8125rem; color: var(--brand-500); opacity: 0; transition: opacity 0.15s; flex-shrink: 0; }
.result-row:hover .result-action { opacity: 1; }

.results-empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.5rem; color: var(--text-secondary); font-size: 0.8125rem; }
</style>
