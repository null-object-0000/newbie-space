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
            <Braces :size="22" />
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
          <div class="mode-bar">
            <div class="segmented">
              <button :class="{ active: mode === 'format' }" @click="mode = 'format'">格式化</button>
              <button :class="{ active: mode === 'compact' }" @click="mode = 'compact'">压缩</button>
              <button :class="{ active: mode === 'validate' }" @click="mode = 'validate'">验证</button>
            </div>
            <button class="swap-btn" @click="pasteSample" title="粘贴示例 JSON">
              <FileJson :size="16" />
            </button>
          </div>

          <textarea
            v-model="inputJson"
            :placeholder="inputPlaceholder"
            rows="10"
            @input="scheduleProcess"
          ></textarea>

          <div v-if="validationResult" class="validation-badge" :class="validationResult.valid ? 'valid' : 'invalid'">
            <CheckCircle v-if="validationResult.valid" :size="14" />
            <AlertCircle v-else :size="14" />
            <span>{{ validationResult.message }}</span>
          </div>
        </div>

        <!-- 右侧：输出 -->
        <div class="panel panel-right">
          <textarea
            v-model="outputText"
            readonly
            rows="10"
            :placeholder="outputPlaceholder"
          ></textarea>

          <div class="meta" v-if="outputText">
            <span>输入 {{ inputJson.length.toLocaleString() }} → 输出 {{ outputText.length.toLocaleString() }} 字符</span>
            <span v-if="mode === 'compressed' && inputJson.length > 0" class="savings">
              压缩 {{ compressionRatio }}%
            </span>
          </div>

          <div class="actions">
            <button class="btn primary" :disabled="!outputText" @click="copyResult">
              <Copy :size="16" />{{ copyLabel }}
            </button>
            <button class="btn secondary" :disabled="!outputText" @click="clearAll">
              <Trash2 :size="16" />清空
            </button>
            <PipelineSend
              :tools="downstreamTools"
              :disabled="!outputText"
              @send="sendToTool"
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
import { computed, onUnmounted, ref, watch } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { usePipeline } from '@/composables/usePipeline'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import { findTool } from '@/utils/toolPipeline'
import {
  AlertCircle,
  ArrowLeft,
  ArrowRightLeft,
  Braces,
  CheckCircle,
  Copy,
  FileJson,
  Trash2
} from 'lucide-vue-next'

const tool = findTool('json-formatter')
const { isDark } = useTheme()

// --- 流转 ---
const { pipelineFrom, downstreamTools, sendTextTo } = usePipeline({
  toolId: 'json-formatter',
  onIncoming: async (incoming) => {
    if (incoming.type !== 'text') return false
    inputJson.value = incoming.data.text
    scheduleProcess()
    return true
  }
})

function sendToTool(target: typeof downstreamTools.value[number]) {
  if (!outputText.value) return
  const result = sendTextTo(target, outputText.value)
  showToast(result.message, result.ok ? 'success' : 'error')
}

// --- 状态 ---
const mode = ref<'format' | 'compact' | 'validate'>('format')
const inputJson = ref('')
const outputText = ref('')
const validationResult = ref<{ valid: boolean; message: string } | null>(null)
const copyLabel = ref('复制')
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')

let processTimer: ReturnType<typeof setTimeout> | null = null
let toastTimer: ReturnType<typeof setTimeout> | null = null

const sampleJson = `{
  "name": "Alice",
  "age": 28,
  "items": ["apple", "orange"],
  "address": {
    "city": "Beijing",
    "zip": "100000"
  }
}`

const inputPlaceholder = computed(() => {
  if (mode.value === 'validate') return '粘贴 JSON 文本进行语法验证…'
  return '粘贴需要格式化或压缩的 JSON 文本…'
})

const outputPlaceholder = computed(() => {
  if (mode.value === 'format') return '格式化结果将显示在这里'
  if (mode.value === 'compact') return '压缩结果将显示在这里'
  return ''
})

const compressionRatio = computed(() => {
  if (!inputJson.value || !outputText.value) return 0
  return Math.round((1 - outputText.value.length / inputJson.value.length) * 100)
})

// --- 处理 ---
function scheduleProcess() {
  if (processTimer) clearTimeout(processTimer)
  processTimer = setTimeout(() => {
    doProcess()
  }, 150)
}

function doProcess() {
  const raw = inputJson.value.trim()
  if (!raw) {
    outputText.value = ''
    validationResult.value = null
    return
  }

  try {
    const parsed = JSON.parse(raw)

    if (mode.value === 'format') {
      outputText.value = JSON.stringify(parsed, null, 2)
      validationResult.value = { valid: true, message: 'JSON 格式有效' }
    } else if (mode.value === 'compact') {
      outputText.value = JSON.stringify(parsed)
      validationResult.value = { valid: true, message: 'JSON 格式有效' }
    } else {
      // validate
      outputText.value = JSON.stringify(parsed, null, 2)
      const depth = getDepth(parsed)
      const keys = countKeys(parsed)
      validationResult.value = {
        valid: true,
        message: `有效 · ${keys} 个键 · 嵌套 ${depth} 层`
      }
    }
  } catch (e) {
    outputText.value = ''
    const msg = e instanceof SyntaxError ? e.message : '无效的 JSON 格式'
    validationResult.value = { valid: false, message: msg }
  }
}

function getDepth(obj: unknown): number {
  if (obj === null || typeof obj !== 'object') return 0
  if (Array.isArray(obj)) {
    return 1 + (obj.length ? Math.max(...obj.map(getDepth)) : 0)
  }
  const vals = Object.values(obj as Record<string, unknown>)
  return 1 + (vals.length ? Math.max(...vals.map(getDepth)) : 0)
}

function countKeys(obj: unknown): number {
  if (obj === null || typeof obj !== 'object') return 0
  if (Array.isArray(obj)) return obj.reduce((sum, v) => sum + countKeys(v), 0)
  const rec = obj as Record<string, unknown>
  return Object.keys(rec).length + Object.values(rec).reduce((sum, v) => sum + countKeys(v), 0)
}

// 模式切换时重新处理
watch(mode, () => {
  if (inputJson.value.trim()) doProcess()
})

// --- 操作 ---
function pasteSample() {
  inputJson.value = sampleJson
  scheduleProcess()
}

async function copyResult() {
  if (!outputText.value) return
  try {
    await navigator.clipboard.writeText(outputText.value)
    copyLabel.value = '已复制'
    showToast('结果已复制到剪贴板', 'success')
    setTimeout(() => { copyLabel.value = '复制' }, 1500)
  } catch {
    showToast('复制失败', 'error')
  }
}

function clearAll() {
  inputJson.value = ''
  outputText.value = ''
  validationResult.value = null
}

function showToast(message: string, type: 'success' | 'error') {
  if (toastTimer) clearTimeout(toastTimer)
  toastMessage.value = message
  toastType.value = type
  toastTimer = setTimeout(() => {
    toastMessage.value = ''
  }, 2200)
}

// --- 生命周期 ---
onUnmounted(() => {
  if (processTimer) clearTimeout(processTimer)
  if (toastTimer) clearTimeout(toastTimer)
})
</script>

<style scoped>
.heading-icon { --tool-color: #3b82f6; }

/* --- 模式切换 --- */
.mode-bar {
  display: flex; align-items: center; gap: 0.5rem;
}

.swap-btn {
  width: 2rem; height: 2rem; display: flex; align-items: center; justify-content: center;
  border: 1px solid var(--border-color); border-radius: 0.375rem;
  background: var(--bg-surface); color: var(--text-secondary); cursor: pointer;
  transition: border-color 0.15s, color 0.15s;
  flex-shrink: 0;
}
.swap-btn:hover { border-color: var(--brand-500); color: var(--brand-500); }

/* --- 左侧 textarea --- */
.panel-left textarea {
  width: 100%;
  padding: 0.625rem 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 0.8125rem;
  font-family: var(--font-family-mono, monospace);
  resize: vertical;
  outline: none;
  box-sizing: border-box;
  line-height: 1.6;
}
.panel-left textarea:focus { border-color: var(--brand-500); }

/* --- 验证结果 --- */
.validation-badge {
  display: inline-flex; align-items: center; gap: 0.375rem;
  padding: 0.375rem 0.625rem; border-radius: 0.375rem;
  font-size: 0.8125rem; font-weight: 600;
}
.validation-badge.valid {
  background: color-mix(in srgb, #10b981 10%, transparent);
  color: #047857;
}
.validation-badge.invalid {
  background: color-mix(in srgb, #ef4444 10%, transparent);
  color: #b91c1c;
}

/* --- 右侧 textarea --- */
.panel-right textarea {
  width: 100%;
  padding: 0.625rem 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 0.8125rem;
  font-family: var(--font-family-mono, monospace);
  resize: vertical;
  outline: none;
  box-sizing: border-box;
  line-height: 1.6;
  cursor: default;
}
.panel-right textarea:focus { border-color: var(--brand-500); }

/* --- 元信息 --- */
.meta {
  display: flex; align-items: center; gap: 0.5rem;
  font-size: 0.875rem;
  color: var(--text-secondary);
  font-family: var(--font-family-mono, monospace);
}
.savings {
  color: #047857;
  font-weight: 600;
}
</style>
