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
          <div class="heading-icon"><Fingerprint :size="22" /></div>
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
        <!-- 左侧：输入 + 编码 -->
        <div class="panel panel-left">
          <div class="panel-header-row">
            <label class="section-label" for="hash-input">输入文本</label>
            <span class="char-count">{{ text.length.toLocaleString() }} / 10,000,000</span>
          </div>
          <textarea
            id="hash-input"
            ref="inputRef"
            v-model="text"
            placeholder="输入要计算哈希的文本…"
            rows="5"
            maxlength="10000000"
            @input="scheduleHash"
          ></textarea>

          <div class="encoding-bar">
            <span class="section-label">编码</span>
            <div class="segmented">
              <button
                v-for="enc in encodings"
                :key="enc.key"
                :class="{ active: encoding === enc.key }"
                @click="encoding = enc.key"
              >{{ enc.label }}</button>
            </div>
          </div>
        </div>

        <!-- 右侧：结果 -->
        <div class="panel panel-right">
          <div class="panel-header-row">
            <h3 class="panel-title">哈希结果</h3>
            <span class="local-badge">所有结果均在本地计算，不会上传或保存</span>
          </div>

          <div class="results" :class="{ empty: !results.length }">
            <div v-if="!results.length" class="results-empty">
              <Fingerprint :size="32" />
              <span>输入文本后自动计算</span>
            </div>
            <button
              v-for="r in results"
              :key="r.algo"
              class="result-row"
              :class="{ selected: selectedAlgo === r.algo }"
              @click="selectResult(r)"
            >
              <span class="result-algo">{{ r.algo }}</span>
              <code class="result-hash" :class="{ empty: !r.hash }">{{ r.hash || '—' }}</code>
              <Copy :size="14" class="copy-hint" />
            </button>
          </div>

          <div class="actions">
            <button class="btn primary" :disabled="!results.length" @click="copyAll">
              <Copy :size="14" />复制全部
            </button>
            <button class="btn secondary" :disabled="!text" @click="clearAll">
              <RotateCcw :size="14" />清空
            </button>
            <PipelineSend
              :tools="downstreamTools"
              :disabled="!selectedAlgo"
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
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { usePipeline, type PipelineIncoming } from '@/composables/usePipeline'
import type { ToolItem } from '@/data/tools'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import { findTool } from '@/utils/toolPipeline'
import { ArrowLeft, ArrowRightLeft, Copy, Fingerprint, RotateCcw } from 'lucide-vue-next'
import { hashAll, type HashResult, type HashEncoding } from '@/utils/hash'

const tool = findTool('hash-text')
const { isDark } = useTheme()

// --- 流转 ---
const { pipelineFrom, downstreamTools, sendTextTo } = usePipeline({
  toolId: 'hash-text',
  async onIncoming(incoming: PipelineIncoming) {
    if (incoming.type === 'text') { text.value = incoming.data.text; scheduleHash(); return true }
    return false
  }
})

function handlePipelineSend(target: ToolItem) {
  const sel = results.value.find(r => r.algo === selectedAlgo.value)
  if (!sel) return
  const { ok, message } = sendTextTo(target, sel.hash)
  showToast(message, ok ? 'success' : 'error')
}

// --- 编码 ---
const encodings: { key: HashEncoding; label: string }[] = [
  { key: 'hex', label: 'Hex' },
  { key: 'base64', label: 'Base64' },
  { key: 'base64url', label: 'Base64url' },
  { key: 'bin', label: 'Bin' }
]

// --- 状态 ---
const inputRef = ref<HTMLTextAreaElement | null>(null)
const text = ref('')
const encoding = ref<HashEncoding>('hex')
const results = ref<HashResult[]>([])
const selectedAlgo = ref('')
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')

let hashTimer: ReturnType<typeof setTimeout> | null = null
let toastTimer: ReturnType<typeof setTimeout> | null = null

// --- 哈希 ---
function scheduleHash() {
  if (hashTimer) clearTimeout(hashTimer)
  hashTimer = setTimeout(doHash, 150)
}

async function doHash() {
  if (!text.value.trim()) { results.value = []; selectedAlgo.value = ''; return }
  results.value = await hashAll(text.value, encoding.value)
  if (!results.value.some(r => r.algo === selectedAlgo.value)) selectedAlgo.value = ''
}

watch(encoding, () => { if (text.value.trim()) doHash() })

// --- 选中 & 复制 ---
function selectResult(r: HashResult) {
  if (selectedAlgo.value === r.algo) { selectedAlgo.value = ''; return }
  selectedAlgo.value = r.algo
  navigator.clipboard.writeText(r.hash).then(() => {
    showToast(`${r.algo} 已复制`, 'success')
  }).catch(() => showToast('复制失败', 'error'))
}

async function copyAll() {
  const lines = results.value.map(r => `${r.algo}: ${r.hash}`).join('\n')
  try {
    await navigator.clipboard.writeText(lines)
    showToast('全部结果已复制', 'success')
  } catch { showToast('复制失败', 'error') }
}

function clearAll() { text.value = ''; results.value = []; selectedAlgo.value = '' }

function showToast(m: string, t: 'success' | 'error') {
  if (toastTimer) clearTimeout(toastTimer)
  toastMessage.value = m; toastType.value = t
  toastTimer = setTimeout(() => { toastMessage.value = '' }, 2200)
}

onMounted(() => { inputRef.value?.focus() })
onUnmounted(() => { if (hashTimer) clearTimeout(hashTimer); if (toastTimer) clearTimeout(toastTimer) })
</script>

<style scoped>
/* Tool-specific color for heading icon */
.heading-icon {
  --tool-color: #ef4444;
}

/* --- Panel header row --- */
.panel-header-row {
  display: flex; align-items: center; justify-content: space-between; gap: 0.5rem;
}
.panel-title {
  font-size: 0.875rem; font-weight: 700; margin: 0;
}
.local-badge {
  font-size: 0.625rem; font-weight: 600;
  color: var(--text-muted);
  padding: 0.1875rem 0.5rem;
  background: var(--bg-elevated);
  border-radius: 999px;
  white-space: nowrap;
}
.char-count {
  font-size: 0.8125rem;
  color: var(--text-muted);
  font-variant-numeric: tabular-nums;
}

/* --- Encoding bar --- */
.encoding-bar {
  display: flex; align-items: center; gap: 0.5rem;
}
.encoding-bar .section-label { margin: 0; white-space: nowrap; }

/* --- Results --- */
.results { flex: 1; display: flex; flex-direction: column; }
.results.empty {
  align-items: center; justify-content: center;
  border-radius: 0.375rem; background: var(--bg-elevated);
}
.results-empty {
  display: flex; flex-direction: column; align-items: center; gap: 0.5rem;
  color: var(--text-secondary); font-size: 0.8125rem;
}

.result-row {
  display: grid; grid-template-columns: 5.5rem 1fr auto;
  align-items: center; gap: 0.5rem;
  padding: 0.625rem 0;
  border: 0;
  border-bottom: 1px solid var(--border-color);
  background: transparent; color: inherit;
  font-family: var(--font-family-mono, monospace);
  cursor: pointer; text-align: left;
  width: 100%;
  transition: background 0.15s;
}
.result-row:last-child { border-bottom: 0; }
.result-row:hover { background: var(--bg-elevated); border-radius: 0.25rem; }
.result-row.selected {
  background: color-mix(in srgb, #ef4444 8%, transparent);
  border-radius: 0.25rem;
}
.result-algo {
  font-size: 0.8125rem; font-weight: 700;
  color: var(--text-primary);
}
.result-hash {
  font-size: 0.875rem; line-height: 1.5;
  word-break: break-all; color: var(--text-primary);
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.result-hash.empty { color: var(--text-muted); }

.copy-hint {
  opacity: 0; transition: opacity 0.15s;
  color: var(--text-secondary); flex-shrink: 0;
}
.result-row:hover .copy-hint,
.result-row.selected .copy-hint { opacity: 0.6; }
</style>
