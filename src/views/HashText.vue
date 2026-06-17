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
        <span>来自「{{ pipelineFrom }}」的流转文本</span>
      </div>

      <div class="workspace">
        <!-- 左侧：输入 + 编码 -->
        <div class="panel panel-left">
          <label class="section-label" for="hash-input">输入文本</label>
          <textarea
            id="hash-input"
            ref="inputRef"
            v-model="text"
            placeholder="输入要计算哈希的文本…"
            rows="5"
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
          <span class="section-label">哈希结果</span>

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
            <button class="btn secondary" :disabled="!selectedAlgo" @click="copySelected">
              <Copy :size="14" />{{ copyLabel }}
            </button>
            <button class="btn secondary" :disabled="!results.length" @click="copyAll">
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
const copyLabel = ref('复制')
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
    copyLabel.value = '已复制'
    showToast(`${r.algo} 已复制`, 'success')
    setTimeout(() => { copyLabel.value = '复制' }, 1500)
  }).catch(() => showToast('复制失败', 'error'))
}

function copySelected() {
  const sel = results.value.find(r => r.algo === selectedAlgo.value)
  if (!sel) return
  navigator.clipboard.writeText(sel.hash).then(() => {
    copyLabel.value = '已复制'
    showToast(`${sel.algo} 已复制`, 'success')
    setTimeout(() => { copyLabel.value = '复制' }, 1500)
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
.tool-page { min-height: 100vh; background: var(--bg-main); color: var(--text-primary); }
.tool-main { width: 100%; max-width: 72rem; margin: 0 auto; padding: 5rem 1rem 2.5rem; }
@media (min-width: 640px) { .tool-main { padding: 5.5rem 1.5rem 3rem; } }
@media (min-width: 1024px) { .tool-main { padding: 5.5rem 2rem 3rem; } }

.tool-topbar { margin-bottom: 0.75rem; }
.back-link { display: inline-flex; align-items: center; gap: 0.375rem; color: var(--text-secondary); font-size: 0.8125rem; }
.back-link:hover { color: var(--brand-500); }

.tool-header { margin-bottom: 1rem; }
.tool-heading { display: flex; align-items: center; gap: 0.75rem; }
.heading-icon {
  width: 2.75rem; height: 2.75rem; display: flex; align-items: center; justify-content: center;
  border-radius: 0.5rem; color: #ef4444;
  background: color-mix(in srgb, #ef4444 14%, transparent);
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
  display: grid;
  grid-template-columns: 1fr;
  gap: 0.75rem;
}
@media (min-width: 768px) {
  .workspace { grid-template-columns: 1fr 1fr; align-items: stretch; }
}

.panel {
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  padding: 0.875rem;
  display: flex; flex-direction: column; gap: 0.75rem;
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

.encoding-bar {
  display: flex; align-items: center; gap: 0.5rem;
}
.encoding-bar .section-label { margin: 0; white-space: nowrap; }

.segmented {
  flex: 1; display: flex;
  padding: 0.1875rem; border-radius: 0.25rem; background: var(--bg-elevated);
}
.segmented button {
  flex: 1; min-height: 1.75rem; padding: 0 0.5rem;
  border: 0; border-radius: 0.1875rem;
  background: transparent; color: var(--text-secondary);
  font-size: 0.6875rem; font-weight: 600; cursor: pointer;
  transition: background 0.15s, color 0.15s;
}
.segmented button.active {
  background: var(--bg-surface); color: var(--text-primary); box-shadow: var(--shadow-1);
}

/* --- 结果 --- */
.results { flex: 1; display: flex; flex-direction: column; gap: 0.375rem; }
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
  padding: 0.375rem 0.625rem;
  border: 1px solid var(--border-color); border-radius: 0.25rem;
  background: var(--bg-elevated); color: inherit;
  font-family: var(--font-family-mono, monospace);
  cursor: pointer; text-align: left;
  width: 100%;
  transition: border-color 0.15s, background 0.15s;
}
.result-row:hover { border-color: var(--brand-500); }
.result-row.selected {
  border-color: #ef4444;
  background: color-mix(in srgb, #ef4444 8%, transparent);
}
.result-algo {
  font-size: 0.6875rem; font-weight: 700;
  color: var(--text-primary);
}
.result-hash {
  font-size: 0.75rem; line-height: 1.5;
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
