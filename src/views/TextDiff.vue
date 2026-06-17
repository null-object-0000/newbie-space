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
          <div class="heading-icon"><GitCompare :size="22" /></div>
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

      <!-- 输入区 -->
      <div class="input-grid">
        <div class="panel">
          <div class="panel-head">
            <span class="section-label">原始文本</span>
            <button class="tiny-btn" @click="pasteA" title="粘贴">粘贴</button>
            <button class="tiny-btn" @click="clearA" title="清空"><X :size="14" /></button>
          </div>
          <textarea
            v-model="textA"
            placeholder="粘贴原始文本…"
            rows="8"
            @input="scheduleDiff"
          ></textarea>
        </div>
        <div class="panel">
          <div class="panel-head">
            <span class="section-label">修改文本</span>
            <button class="tiny-btn" @click="pasteB" title="粘贴">粘贴</button>
            <button class="tiny-btn" @click="clearB" title="清空"><X :size="14" /></button>
          </div>
          <textarea
            v-model="textB"
            placeholder="粘贴修改后的文本…"
            rows="8"
            @input="scheduleDiff"
          ></textarea>
        </div>
      </div>

      <!-- Diff 结果 -->
      <div class="diff-panel" v-if="diff.length">
        <div class="diff-head">
          <span class="section-label">差异对比</span>
          <span class="diff-stats" v-if="stats.added || stats.removed">
            <span class="stat added">+{{ stats.added }} 行</span>
            <span class="stat removed">-{{ stats.removed }} 行</span>
          </span>
          <div class="diff-head-right">
            <button class="btn secondary" @click="copyDiff">复制差异</button>
            <PipelineSend
              :tools="downstreamTools"
              :disabled="!diff.length"
              @send="handlePipelineSend"
            />
          </div>
        </div>

        <div class="diff-lines">
          <div
            v-for="(line, idx) in diff"
            :key="idx"
            class="diff-line"
            :class="line.type"
          >
            <span class="line-num">{{ lineNumLabel(line) }}</span>
            <span class="line-prefix">{{ prefixMap[line.type] }}</span>
            <code class="line-content">{{ line.content || ' ' }}</code>
          </div>
        </div>
      </div>

      <div v-else-if="textA || textB" class="diff-empty">
        <GitCompare :size="28" />
        <span>输入两侧文本后自动对比</span>
      </div>
    </main>

    <Transition name="toast">
      <div v-if="toastMessage" class="toast" :class="toastType">{{ toastMessage }}</div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { usePipeline, type PipelineIncoming } from '@/composables/usePipeline'
import type { ToolItem } from '@/data/tools'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import { findTool } from '@/utils/toolPipeline'
import { diffLines, diffStats, type DiffLine } from '@/utils/textDiff'
import { ArrowLeft, ArrowRightLeft, Copy, GitCompare, X } from 'lucide-vue-next'

const tool = findTool('text-diff')
const { isDark } = useTheme()

// --- 流转 ---
const { pipelineFrom, downstreamTools, sendTextTo } = usePipeline({
  toolId: 'text-diff',
  async onIncoming(incoming: PipelineIncoming) {
    if (incoming.type === 'text') {
      // 先填 A，再填 B 则交换
      if (!textA.value) { textA.value = incoming.data.text }
      else { textB.value = incoming.data.text }
      scheduleDiff()
      return true
    }
    return false
  }
})

function handlePipelineSend(target: ToolItem) {
  const summary = diffSummary.value
  if (!summary) return
  const { ok, message } = sendTextTo(target, summary)
  showToast(message, ok ? 'success' : 'error')
}

// --- 状态 ---
const textA = ref('')
const textB = ref('')
const diff = ref<DiffLine[]>([])
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')

let diffTimer: ReturnType<typeof setTimeout> | null = null
let toastTimer: ReturnType<typeof setTimeout> | null = null

const prefixMap: Record<string, string> = { equal: ' ', insert: '+', delete: '-' }

const stats = computed(() => diffStats(diff.value))

const diffSummary = computed(() => {
  if (!diff.value.length) return ''
  return diff.value.map(l => `${prefixMap[l.type]} ${l.content}`).join('\n')
})

// --- Diff ---
function scheduleDiff() {
  if (diffTimer) clearTimeout(diffTimer)
  diffTimer = setTimeout(doDiff, 200)
}

function doDiff() {
  if (!textA.value.trim() || !textB.value.trim()) {
    diff.value = []
    return
  }
  diff.value = diffLines(textA.value, textB.value)
}

// --- 行号 ---
function lineNumLabel(line: DiffLine): string {
  if (line.type === 'delete') return String(line.lineNumA ?? '')
  if (line.type === 'insert') return String(line.lineNumB ?? '')
  return `${line.lineNumA ?? ''} → ${line.lineNumB ?? ''}`
}

// --- 操作 ---
async function pasteA() {
  try { textA.value = await navigator.clipboard.readText(); scheduleDiff() } catch {}
}
async function pasteB() {
  try { textB.value = await navigator.clipboard.readText(); scheduleDiff() } catch {}
}
function clearA() { textA.value = ''; diff.value = [] }
function clearB() { textB.value = ''; diff.value = [] }

async function copyDiff() {
  try {
    await navigator.clipboard.writeText(diffSummary.value)
    showToast('差异已复制', 'success')
  } catch { showToast('复制失败', 'error') }
}

function showToast(m: string, t: 'success' | 'error') {
  if (toastTimer) clearTimeout(toastTimer)
  toastMessage.value = m; toastType.value = t
  toastTimer = setTimeout(() => { toastMessage.value = '' }, 2200)
}

onUnmounted(() => {
  if (diffTimer) clearTimeout(diffTimer)
  if (toastTimer) clearTimeout(toastTimer)
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
  border-radius: 0.5rem; color: #f59e0b;
  background: color-mix(in srgb, #f59e0b 14%, transparent);
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

/* ====== 输入区 ====== */
.input-grid {
  display: grid; grid-template-columns: 1fr; gap: 0.75rem; margin-bottom: 0.75rem;
}
@media (min-width: 768px) {
  .input-grid { grid-template-columns: 1fr 1fr; }
}

.panel {
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  padding: 0.875rem;
  display: flex; flex-direction: column; gap: 0.5rem;
}

.panel-head {
  display: flex; align-items: center; gap: 0.375rem;
}

.section-label {
  font-size: 0.6875rem; font-weight: 700;
  color: var(--text-secondary);
  text-transform: uppercase; letter-spacing: 0.04em;
  flex: 1;
}

.tiny-btn {
  width: 1.625rem; height: 1.625rem;
  display: flex; align-items: center; justify-content: center;
  border: 1px solid var(--border-color); border-radius: 0.25rem;
  background: var(--bg-elevated); color: var(--text-secondary);
  cursor: pointer; font-size: 0.625rem;
  transition: border-color 0.15s, color 0.15s;
}
.tiny-btn:hover { border-color: var(--brand-500); color: var(--brand-500); }

textarea {
  width: 100%; padding: 0.5rem 0.75rem;
  border: 1px solid var(--border-color); border-radius: 0.375rem;
  background: var(--bg-elevated); color: var(--text-primary);
  font-size: 0.8125rem; font-family: var(--font-family-mono, monospace);
  line-height: 1.6; resize: vertical; outline: none; box-sizing: border-box;
}
textarea:focus { border-color: var(--brand-500); }

/* ====== Diff 结果 ====== */
.diff-panel {
  border: 1px solid var(--border-color); border-radius: 0.5rem;
  background: var(--bg-surface); overflow: hidden;
}

.diff-head {
  display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap;
  padding: 0.625rem 0.875rem;
  border-bottom: 1px solid var(--border-color);
}
.diff-stats { display: flex; gap: 0.5rem; }
.stat { font-size: 0.6875rem; font-weight: 700; font-family: var(--font-family-mono, monospace); }
.stat.added { color: #16a34a; }
.stat.removed { color: #dc2626; }

.diff-head-right {
  margin-left: auto; display: flex; align-items: center; gap: 0.5rem;
}

.diff-lines {
  max-height: 500px; overflow-y: auto;
}

.diff-line {
  display: flex; align-items: baseline; gap: 0.5rem;
  padding: 0.125rem 0.75rem;
  font-family: var(--font-family-mono, monospace);
  font-size: 0.75rem; line-height: 1.6;
  border-bottom: 1px solid color-mix(in srgb, var(--border-color) 50%, transparent);
}
.diff-line.insert { background: color-mix(in srgb, #16a34a 10%, transparent); }
.diff-line.delete { background: color-mix(in srgb, #dc2626 10%, transparent); }

.line-num {
  width: 3.5rem; flex-shrink: 0;
  text-align: right; color: var(--text-muted);
  font-size: 0.6875rem;
}

.line-prefix {
  width: 1rem; flex-shrink: 0; text-align: center;
  font-weight: 700;
}
.diff-line.insert .line-prefix { color: #16a34a; }
.diff-line.delete .line-prefix { color: #dc2626; }

.line-content {
  flex: 1; white-space: pre-wrap; word-break: break-all;
}

.diff-empty {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  gap: 0.5rem; padding: 3rem 1rem;
  border: 1px solid var(--border-color); border-radius: 0.5rem;
  background: var(--bg-surface); color: var(--text-secondary);
  font-size: 0.8125rem;
}

/* --- 按钮 --- */
.btn {
  min-height: 2rem; display: inline-flex; align-items: center; justify-content: center;
  gap: 0.375rem; border: 0; border-radius: 0.375rem; padding: 0 0.75rem;
  font-weight: 600; font-size: 0.75rem; cursor: pointer;
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
