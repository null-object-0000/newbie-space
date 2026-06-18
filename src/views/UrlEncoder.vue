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
            <Link2 :size="22" />
          </div>
          <div>
            <h1>{{ tool?.name }}</h1>
            <p>{{ tool?.desc }}</p>
          </div>
        </div>
      </section>

      <div v-if="pipelineFrom" class="pipeline-banner">
        <ArrowLeftRight :size="14" />
        <span>来自「{{ pipelineFrom }}」的传递数据</span>
      </div>

      <div class="workspace">
        <!-- 左侧：输入 + 模式 -->
        <div class="panel panel-left">
          <div class="mode-bar">
            <div class="segmented">
              <button :class="{ active: mode === 'encode' }" @click="mode = 'encode'">编码 Encode</button>
              <button :class="{ active: mode === 'decode' }" @click="mode = 'decode'">解码 Decode</button>
            </div>
            <button class="swap-btn" @click="swap" title="交换输入输出">
              <ArrowLeftRight :size="16" />
            </button>
          </div>

          <div v-if="mode === 'encode'" class="encode-options">
            <label class="radio-label" :class="{ checked: encodeMode === 'component' }">
              <input type="radio" v-model="encodeMode" value="component" />
              <span>encodeURIComponent</span>
              <span class="hint">编码所有特殊字符（含 / ? & = #）</span>
            </label>
            <label class="radio-label" :class="{ checked: encodeMode === 'uri' }">
              <input type="radio" v-model="encodeMode" value="uri" />
              <span>encodeURI</span>
              <span class="hint">保留 URL 结构字符，适合编码完整链接</span>
            </label>
          </div>

          <textarea
            v-model="inputText"
            :placeholder="mode === 'encode' ? '输入要编码的文本或链接…' : '输入 URL 编码后的文本…'"
            rows="8"
            @input="scheduleConvert"
          ></textarea>
        </div>

        <!-- 右侧：输出 + 操作 -->
        <div class="panel panel-right">
          <textarea
            v-model="outputText"
            readonly
            rows="8"
            :placeholder="mode === 'encode' ? '编码结果将显示在这里' : '解码结果将显示在这里'"
          ></textarea>

          <div class="meta" v-if="outputText">
            <span>输入 {{ inputText.length }} 字符 → 输出 {{ outputText.length }} 字符</span>
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
import { onUnmounted, ref, watch } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { usePipeline } from '@/composables/usePipeline'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import { findTool } from '@/utils/toolPipeline'
import {
  ArrowLeft,
  ArrowLeftRight,
  Copy,
  Link2,
  Trash2
} from 'lucide-vue-next'

const tool = findTool('url-encoder')
const { isDark } = useTheme()

// --- 流转 ---
const { pipelineFrom, downstreamTools, sendTextTo } = usePipeline({
  toolId: 'url-encoder',
  onIncoming: async (incoming) => {
    if (incoming.type !== 'text') return false
    inputText.value = incoming.data.text
    scheduleConvert()
    return true
  }
})

function sendToTool(target: typeof downstreamTools.value[number]) {
  if (!outputText.value) return
  const result = sendTextTo(target, outputText.value)
  showToast(result.message, result.ok ? 'success' : 'error')
}

// --- 状态 ---
const mode = ref<'encode' | 'decode'>('encode')
const encodeMode = ref<'component' | 'uri'>('component')
const inputText = ref('')
const outputText = ref('')
const copyLabel = ref('复制')
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')

let convertTimer: ReturnType<typeof setTimeout> | null = null
let toastTimer: ReturnType<typeof setTimeout> | null = null

// --- 转换 ---
function scheduleConvert() {
  if (convertTimer) clearTimeout(convertTimer)
  convertTimer = setTimeout(() => {
    doConvert()
  }, 150)
}

function doConvert() {
  if (!inputText.value.trim()) {
    outputText.value = ''
    return
  }

  try {
    if (mode.value === 'encode') {
      outputText.value = encodeMode.value === 'component'
        ? encodeURIComponent(inputText.value)
        : encodeURI(inputText.value)
    } else {
      outputText.value = decodeURIComponent(inputText.value)
    }
  } catch {
    outputText.value = ''
    showToast('解码失败，请检查输入是否为有效的 URL 编码文本', 'error')
  }
}

// 切换编解码模式时重新计算
watch(mode, () => {
  if (inputText.value.trim()) doConvert()
})

watch(encodeMode, () => {
  if (mode.value === 'encode' && inputText.value.trim()) doConvert()
})

// --- 操作 ---
function swap() {
  const temp = inputText.value
  inputText.value = outputText.value
  outputText.value = temp
  mode.value = mode.value === 'encode' ? 'decode' : 'encode'
}

async function copyResult() {
  if (!outputText.value) return

  try {
    await navigator.clipboard.writeText(outputText.value)
    copyLabel.value = '已复制'
    showToast('结果已复制到剪贴板', 'success')
    setTimeout(() => {
      copyLabel.value = '复制'
    }, 1500)
  } catch {
    showToast('复制失败', 'error')
  }
}

function clearAll() {
  inputText.value = ''
  outputText.value = ''
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
  if (convertTimer) clearTimeout(convertTimer)
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

/* --- 编码选项 --- */
.encode-options {
  display: flex; flex-direction: column; gap: 0.5rem;
}
.radio-label {
  display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap;
  padding: 0.5rem 0.625rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  cursor: pointer;
  transition: border-color 0.15s;
}
.radio-label.checked { border-color: var(--brand-500); }
.radio-label input[type='radio'] { accent-color: var(--brand-500); margin: 0; }
.radio-label span:first-of-type { font-weight: 600; font-size: 0.8125rem; }
.radio-label .hint {
  color: var(--text-secondary); font-size: 0.875rem;
  width: 100%; margin-left: 1.125rem;
}

/* --- 文本框 --- */
.panel-left textarea:focus { border-color: var(--brand-500); }

.panel-right textarea {
  width: 100%;
  padding: 0.625rem 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 0.8125rem;
  font-family: var(--font-family-mono, 'Cascadia Code', 'Fira Code', monospace);
  resize: vertical;
  outline: none;
  box-sizing: border-box;
  line-height: 1.6;
  cursor: default;
}
.panel-right textarea:focus { border-color: var(--brand-500); }

/* --- 元信息 --- */
.meta {
  font-size: 0.875rem;
  color: var(--text-secondary);
  font-family: var(--font-family-mono, monospace);
}
</style>
