<template>
  <div class="tool-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />
    <main class="tool-main">
      <div class="tool-topbar">
        <router-link to="/tools" class="back-link"><ArrowLeft :size="16" /><span>工具中心</span></router-link>
      </div>

      <section class="tool-header">
        <div class="tool-heading">
          <div class="heading-icon"><Hash :size="22" /></div>
          <div>
            <h1>{{ tool?.name }}</h1>
            <p>{{ tool?.desc }}</p>
          </div>
        </div>
      </section>

      <div class="workspace">
        <div class="panel panel-left">
          <label class="section-label">生成模式</label>
          <div class="segmented">
            <button :class="{ active: version === 'v4' }" @click="version = 'v4'">UUID v4</button>
            <button :class="{ active: version === 'v7' }" @click="version = 'v7'">UUID v7</button>
            <button :class="{ active: version === 'random' }" @click="version = 'random'">自定义</button>
          </div>

          <template v-if="version === 'random'">
            <label class="section-label">字符串长度</label>
            <div class="length-row">
              <input v-model.number="customLength" type="range" min="4" max="64" step="1" class="length-slider" />
              <span class="length-val">{{ customLength }}</span>
            </div>

            <label class="section-label">字符集</label>
            <div class="charset-grid">
              <label class="checkbox-label" v-for="cs in charsets" :key="cs.key">
                <input type="checkbox" :value="cs.key" v-model="selectedCharsets" />
                <span>{{ cs.label }}</span>
              </label>
            </div>
          </template>

          <label class="section-label">生成数量</label>
          <div class="count-row">
            <input v-model.number="count" type="number" min="1" max="100" class="count-input" />
            <button class="btn primary" @click="generate">生成</button>
          </div>
        </div>

        <div class="panel panel-right result-panel">
          <div v-if="!results.length" class="result-empty">点击生成按钮</div>
          <div v-else class="result-list">
            <div v-for="(r, i) in results" :key="i" class="result-item" @click="copySingle(r)">
              <code>{{ r }}</code>
              <Copy :size="12" class="item-copy" />
            </div>
          </div>

          <div v-if="results.length" class="actions">
            <button class="btn primary" @click="copyAll"><Copy :size="16" />{{ copyLabel }}</button>
            <button class="btn secondary" @click="generate"><RotateCcw :size="16" />刷新</button>
            <PipelineSend :tools="downstreamTools" :disabled="!results.length" @send="handlePipelineSend" />
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
import type { ToolItem } from '@/data/tools'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import { findTool } from '@/utils/toolPipeline'
import { ArrowLeft, Copy, Hash, RotateCcw } from 'lucide-vue-next'

const tool = findTool('uuid-generator')
const { isDark } = useTheme()

const { pipelineFrom, downstreamTools, sendTextTo } = usePipeline({
  toolId: 'uuid-generator',
  async onIncoming() { return false }
})

function handlePipelineSend(target: ToolItem) {
  const text = results.value.join('\n')
  if (!text) return
  const { ok, message } = sendTextTo(target, text)
  showToast(message, ok ? 'success' : 'error')
}

const version = ref<'v4' | 'v7' | 'random'>('v4')
const customLength = ref(16)
const count = ref(1)
const results = ref<string[]>([])
const copyLabel = ref('复制')

interface CharsetItem { key: string; label: string }
const charsets: CharsetItem[] = [
  { key: 'lower', label: '小写字母 a–z' },
  { key: 'upper', label: '大写字母 A–Z' },
  { key: 'digits', label: '数字 0–9' },
  { key: 'symbols', label: '符号 !@#$%^&*' }
]
const selectedCharsets = ref<string[]>(['lower', 'upper', 'digits'])

const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')
let toastTimer: ReturnType<typeof setTimeout> | null = null

function buildCharset(): string {
  const map: Record<string, string> = {
    lower: 'abcdefghijklmnopqrstuvwxyz',
    upper: 'ABCDEFGHIJKLMNOPQRSTUVWXYZ',
    digits: '0123456789',
    symbols: '!@#$%^&*'
  }
  return selectedCharsets.value.map(k => map[k]).join('') || 'abcdefghijklmnopqrstuvwxyz'
}

function randomString(length: number, chars: string): string {
  const arr = new Uint8Array(length)
  crypto.getRandomValues(arr)
  let result = ''
  for (let i = 0; i < length; i++) {
    result += chars[arr[i] % chars.length]
  }
  return result
}

function uuidV4(): string {
  const arr = new Uint8Array(16)
  crypto.getRandomValues(arr)
  arr[6] = (arr[6] & 0x0f) | 0x40 // version 4
  arr[8] = (arr[8] & 0x3f) | 0x80 // variant 10
  const hex = Array.from(arr, b => b.toString(16).padStart(2, '0'))
  return `${hex[0]}${hex[1]}${hex[2]}${hex[3]}-${hex[4]}${hex[5]}-${hex[6]}${hex[7]}-${hex[8]}${hex[9]}-${hex[10]}${hex[11]}${hex[12]}${hex[13]}${hex[14]}${hex[15]}`
}

function uuidV7(): string {
  // UUID v7: 48-bit timestamp (ms since Unix epoch) + random
  const ts = BigInt(Date.now())
  const rand = new Uint8Array(10)
  crypto.getRandomValues(rand)

  const tsBytes = new Uint8Array(6)
  for (let i = 5; i >= 0; i--) {
    tsBytes[i] = Number(ts & 0xffn)
    // ts >>= 8n is not a thing in JS — use division instead
    // ts = ts >> 8n
  }
  // Manual bigint shift
  let remaining = ts
  for (let i = 5; i >= 0; i--) {
    tsBytes[i] = Number(remaining & 0xffn)
    remaining = remaining >> 8n
  }

  const bytes = new Uint8Array(16)
  bytes.set(tsBytes, 0)
  bytes.set(rand, 6)
  bytes[6] = (bytes[6] & 0x0f) | 0x70 // version 7
  bytes[8] = (bytes[8] & 0x3f) | 0x80 // variant 10

  const hex = Array.from(bytes, b => b.toString(16).padStart(2, '0'))
  return `${hex[0]}${hex[1]}${hex[2]}${hex[3]}-${hex[4]}${hex[5]}-${hex[6]}${hex[7]}-${hex[8]}${hex[9]}-${hex[10]}${hex[11]}${hex[12]}${hex[13]}${hex[14]}${hex[15]}`
}

function generate() {
  const n = Math.min(Math.max(count.value, 1), 100)
  const items: string[] = []
  for (let i = 0; i < n; i++) {
    if (version.value === 'v4') {
      items.push(uuidV4())
    } else if (version.value === 'v7') {
      items.push(uuidV7())
    } else {
      items.push(randomString(customLength.value, buildCharset()))
    }
  }
  results.value = items
  showToast(`已生成 ${n} 个`, 'success')
}

function copySingle(val: string) {
  navigator.clipboard.writeText(val).then(() => {
    showToast('已复制', 'success')
  }).catch(() => showToast('复制失败', 'error'))
}

async function copyAll() {
  try {
    await navigator.clipboard.writeText(results.value.join('\n'))
    copyLabel.value = '已复制'
    showToast('全部已复制到剪贴板', 'success')
    setTimeout(() => { copyLabel.value = '复制' }, 1500)
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

onMounted(() => { generate() })
onUnmounted(() => { if (toastTimer) clearTimeout(toastTimer) })
</script>

<style scoped>
.heading-icon { --tool-color: #8b5cf6; }

/* --- 长度滑块 --- */
.length-row {
  display: flex; align-items: center; gap: 0.5rem;
}
.length-slider { flex: 1; accent-color: var(--brand-500); }
.length-val {
  width: 2rem; text-align: center;
  font-family: var(--font-family-mono, monospace);
  font-size: 0.875rem; font-weight: 700; color: var(--text-primary);
}

/* --- 字符集 --- */
.charset-grid {
  display: grid; grid-template-columns: 1fr 1fr; gap: 0.25rem;
}
.checkbox-label {
  display: flex; align-items: center; gap: 0.375rem;
  font-size: 0.8125rem; color: var(--text-secondary);
  cursor: pointer; padding: 0.25rem 0.375rem;
  border-radius: 0.25rem;
  transition: background 0.15s;
}
.checkbox-label:hover { background: var(--bg-elevated); }
.checkbox-label input[type='checkbox'] { accent-color: var(--brand-500); margin: 0; }

/* --- 数量 --- */
.count-row {
  display: flex; align-items: center; gap: 0.5rem;
}
.count-input {
  width: 4rem; text-align: center;
  padding: 0.4375rem 0.5rem;
  border: 1px solid var(--border-color); border-radius: 0.5rem;
  background: var(--bg-elevated); color: var(--text-primary);
  font-size: 0.875rem; font-family: var(--font-family-mono, monospace);
  outline: none; box-sizing: border-box;
}
.count-input:focus { border-color: var(--brand-500); }

/* --- 结果 --- */
.result-panel { justify-content: center; align-items: center; gap: 1rem; }
.result-empty { font-size: 1rem; color: var(--text-secondary); }
.result-list {
  width: 100%; display: flex; flex-direction: column; gap: 0.25rem;
  max-height: 320px; overflow-y: auto;
}
.result-item {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0.5rem 0.625rem;
  border-radius: 0.375rem;
  background: var(--bg-elevated);
  cursor: pointer;
  transition: background 0.15s;
}
.result-item:hover { background: color-mix(in srgb, var(--brand-500) 32%, transparent); }
.result-item code {
  font-family: var(--font-family-mono, monospace);
  font-size: 0.875rem; color: var(--text-primary);
  word-break: break-all;
}
.item-copy { color: var(--text-secondary); flex-shrink: 0; }
.result-item:hover .item-copy { color: var(--text-primary); }
</style>
