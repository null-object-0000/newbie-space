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
        <span>来自「{{ pipelineFrom }}」的传递数据</span>
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
      <div class="diff-panel" v-if="pairedDiff.length">
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
              :disabled="!pairedDiff.length"
              @send="handlePipelineSend"
            />
          </div>
        </div>

        <div class="diff-table">
          <div class="diff-row" v-for="(pair, idx) in pairedDiff" :key="idx">
            <!-- 左侧 -->
            <div class="diff-cell" :class="pair.leftType">
              <span class="line-num">{{ pair.leftNum ?? '' }}</span>
              <div class="cell-content">
                <template v-if="pair.charDiff && pair.leftType === 'delete'">
                  <span
                    v-for="(chunk, ci) in pair.charDiff.left"
                    :key="ci"
                    :class="{ 'char-highlight': chunk.type === 'removed' }"
                  >{{ chunk.text }}</span>
                </template>
                <template v-else>{{ pair.leftContent || ' ' }}</template>
              </div>
            </div>
            <!-- 右侧 -->
            <div class="diff-cell" :class="pair.rightType">
              <span class="line-num">{{ pair.rightNum ?? '' }}</span>
              <div class="cell-content">
                <template v-if="pair.charDiff && pair.rightType === 'insert'">
                  <span
                    v-for="(chunk, ci) in pair.charDiff.right"
                    :key="ci"
                    :class="{ 'char-highlight': chunk.type === 'added' }"
                  >{{ chunk.text }}</span>
                </template>
                <template v-else>{{ pair.rightContent || ' ' }}</template>
              </div>
            </div>
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
import { computed, onUnmounted, ref } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { usePipeline, type PipelineIncoming } from '@/composables/usePipeline'
import type { ToolItem } from '@/data/tools'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import { findTool } from '@/utils/toolPipeline'
import { diffLines, diffStats, pairDiffLines } from '@/utils/textDiff'
import { ArrowLeft, ArrowRightLeft, Copy, GitCompare, X } from 'lucide-vue-next'

const tool = findTool('text-diff')
const { isDark } = useTheme()

// --- 流转 ---
const { pipelineFrom, downstreamTools, sendTextTo } = usePipeline({
  toolId: 'text-diff',
  async onIncoming(incoming: PipelineIncoming) {
    if (incoming.type === 'text') {
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
const rawDiff = ref<ReturnType<typeof diffLines>>([])
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')

let diffTimer: ReturnType<typeof setTimeout> | null = null
let toastTimer: ReturnType<typeof setTimeout> | null = null

const stats = computed(() => diffStats(rawDiff.value))
const pairedDiff = computed(() => pairDiffLines(rawDiff.value))

const diffSummary = computed(() => {
  if (!rawDiff.value.length) return ''
  return rawDiff.value.map(l => {
    const prefix = l.type === 'equal' ? ' ' : l.type === 'insert' ? '+' : '-'
    return `${prefix} ${l.content}`
  }).join('\n')
})

// --- Diff ---
function scheduleDiff() {
  if (diffTimer) clearTimeout(diffTimer)
  diffTimer = setTimeout(doDiff, 200)
}

function doDiff() {
  if (!textA.value.trim() || !textB.value.trim()) {
    rawDiff.value = []
    return
  }
  rawDiff.value = diffLines(textA.value, textB.value)
}

// --- 操作 ---
async function pasteA() {
  try { textA.value = await navigator.clipboard.readText(); scheduleDiff() } catch {}
}
async function pasteB() {
  try { textB.value = await navigator.clipboard.readText(); scheduleDiff() } catch {}
}
function clearA() { textA.value = ''; rawDiff.value = [] }
function clearB() { textB.value = ''; rawDiff.value = [] }

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
.heading-icon { --tool-color: #3b82f6; }

/* ====== 输入区 ====== */
.input-grid {
  display: grid; grid-template-columns: 1fr; gap: 0.75rem; margin-bottom: 0.75rem;
}
@media (min-width: 768px) {
  .input-grid { grid-template-columns: 1fr 1fr; }
}

.panel-head {
  display: flex; align-items: center; gap: 0.375rem;
}

.section-label {
  font-size: 0.8125rem; font-weight: 700;
  color: var(--text-secondary);
  text-transform: uppercase; letter-spacing: 0.04em;
  flex: 1;
}

.tiny-btn {
  min-width: 2.5rem; height: 1.75rem;
  display: flex; align-items: center; justify-content: center;
  padding: 0 0.5rem;
  border: 1px solid var(--border-color); border-radius: 0.25rem;
  background: var(--bg-elevated); color: var(--text-secondary);
  cursor: pointer; font-size: 0.8125rem; font-weight: 600;
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
.stat { font-size: 0.8125rem; font-weight: 700; font-family: var(--font-family-mono, monospace); }
.stat.added { color: #16a34a; }
.stat.removed { color: #dc2626; }

.diff-head-right {
  margin-left: auto; display: flex; align-items: center; gap: 0.5rem;
}

.diff-table {
  max-height: 500px; overflow-y: auto;
  font-family: var(--font-family-mono, monospace);
  font-size: 0.875rem;
}

.diff-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  border-bottom: 1px solid color-mix(in srgb, var(--border-color) 50%, transparent);
}

.diff-cell {
  display: flex; align-items: flex-start; gap: 0.5rem;
  padding: 0.25rem 0.625rem;
  min-height: 1.5rem;
}

.diff-cell.delete { background: color-mix(in srgb, #dc2626 8%, transparent); }
.diff-cell.insert { background: color-mix(in srgb, #16a34a 8%, transparent); }

.line-num {
  width: 2.5rem; flex-shrink: 0;
  text-align: right; color: var(--text-muted);
  font-size: 0.625rem; padding-top: 0.125rem;
  user-select: none;
}

.cell-content {
  flex: 1; line-height: 1.6; white-space: pre-wrap; word-break: break-all;
  color: var(--text-primary);
}

.char-highlight {
  background: color-mix(in srgb, #f59e0b 30%, transparent);
  border-radius: 0.125rem;
  padding: 0 0.125rem;
}

.diff-cell.delete .char-highlight {
  background: color-mix(in srgb, #dc2626 20%, transparent);
}
.diff-cell.insert .char-highlight {
  background: color-mix(in srgb, #16a34a 20%, transparent);
}

.diff-empty {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  gap: 0.5rem; padding: 3rem 1rem;
  border: 1px solid var(--border-color); border-radius: 0.5rem;
  background: var(--bg-surface); color: var(--text-secondary);
  font-size: 0.8125rem;
}
</style>
