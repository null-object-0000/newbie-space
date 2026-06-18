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
            <BarChart3 :size="22" />
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
        <!-- 左侧：输入 -->
        <div class="panel panel-left">
          <textarea
            v-model="text"
            placeholder="输入要统计的文本…"
            rows="10"
          ></textarea>

          <div class="actions">
            <button class="btn secondary" :disabled="!text" @click="clearAll">
              <Trash2 :size="16" />清空
            </button>
            <PipelineSend
              :tools="downstreamTools"
              :disabled="!text"
              @send="sendToTool"
            />
          </div>
        </div>

        <!-- 右侧：统计结果 -->
        <div class="panel panel-right">
          <div class="stats-grid">
            <div class="stat-card">
              <span class="stat-value">{{ stats.charCount }}</span>
              <span class="stat-label">字符数（含空格）</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ stats.charCountNoSpaces }}</span>
              <span class="stat-label">字符数（不含空格）</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ stats.wordCount }}</span>
              <span class="stat-label">单词数</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ stats.chineseCount }}</span>
              <span class="stat-label">中文字数</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ stats.lineCount }}</span>
              <span class="stat-label">行数</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ stats.paragraphCount }}</span>
              <span class="stat-label">段落数</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ stats.sentenceCount }}</span>
              <span class="stat-label">句子数</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ stats.byteSize }}</span>
              <span class="stat-label">字节大小（UTF-8）</span>
            </div>
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
import { computed, onUnmounted, ref, watch } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { usePipeline } from '@/composables/usePipeline'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import { findTool } from '@/utils/toolPipeline'
import { getTextStats, type TextStats } from '@/utils/textStatistics'
import {
  ArrowLeft,
  ArrowRightLeft,
  BarChart3,
  Trash2
} from 'lucide-vue-next'

const tool = findTool('text-statistics')
const { isDark } = useTheme()

// --- 流转 ---
const { pipelineFrom, downstreamTools, sendTextTo } = usePipeline({
  toolId: 'text-statistics',
  onIncoming: async (incoming) => {
    if (incoming.type !== 'text') return false
    text.value = incoming.data.text
    return true
  }
})

function sendToTool(target: typeof downstreamTools.value[number]) {
  if (!text.value) return
  const result = sendTextTo(target, text.value)
  showToast(result.message, result.ok ? 'success' : 'error')
}

// --- 状态 ---
const text = ref('')
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')

let toastTimer: ReturnType<typeof setTimeout> | null = null

// --- 统计 ---
const stats = computed<TextStats>(() => getTextStats(text.value))

// 监听流转数据变化
watch(text, () => {
  // 计算由 computed 自动完成
})

// --- 操作 ---
function clearAll() {
  text.value = ''
}

function showToast(message: string, type: 'success' | 'error') {
  if (toastTimer) clearTimeout(toastTimer)
  toastMessage.value = message
  toastType.value = type
  toastTimer = setTimeout(() => {
    toastMessage.value = ''
  }, 2200)
}

onUnmounted(() => {
  if (toastTimer) clearTimeout(toastTimer)
})
</script>

<style scoped>
.heading-icon { --tool-color: #06b6d4; }

/* --- 文本框 --- */
.panel-left textarea {
  width: 100%;
  padding: 0.625rem 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 0.9375rem;
  font-family: inherit;
  resize: vertical;
  outline: none;
  box-sizing: border-box;
  line-height: 1.6;
  flex: 1;
}
.panel-left textarea:focus { border-color: var(--brand-500); }

/* --- 统计卡片网格 --- */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 0.625rem;
  align-content: start;
}

.stat-card {
  display: flex; flex-direction: column; align-items: center;
  padding: 0.75rem 0.5rem;
  border-radius: 0.625rem;
  background: var(--bg-elevated);
  border: 1px solid var(--border-color);
  text-align: center;
  transition: border-color 0.15s;
}
.stat-card:hover { border-color: #06b6d4; }

.stat-value {
  font-size: 1.5rem;
  font-weight: 700;
  color: #06b6d4;
  line-height: 1.2;
  font-family: var(--font-family-mono, 'Cascadia Code', 'Fira Code', monospace);
}

.stat-label {
  font-size: 0.8125rem;
  color: var(--text-secondary);
  margin-top: 0.25rem;
  line-height: 1.3;
}
</style>
