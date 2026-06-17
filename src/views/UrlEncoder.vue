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
        <span>来自「{{ pipelineFrom }}」的流转文本</span>
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
.tool-page {
  min-height: 100vh;
  background: var(--bg-main);
  color: var(--text-primary);
}

.tool-main {
  width: 100%;
  max-width: 72rem;
  margin: 0 auto;
  padding: 5rem 1rem 2.5rem;
}
@media (min-width: 640px) { .tool-main { padding: 5.5rem 1.5rem 3rem; } }
@media (min-width: 1024px) { .tool-main { padding: 5.5rem 2rem 3rem; } }

.tool-topbar { margin-bottom: 0.75rem; }
.back-link {
  display: inline-flex; align-items: center; gap: 0.375rem;
  color: var(--text-secondary); font-size: 0.8125rem;
}
.back-link:hover { color: var(--brand-500); }

.tool-header { margin-bottom: 1.25rem; }
.tool-heading {
  display: flex; align-items: center; gap: 0.75rem;
}
.heading-icon {
  width: 2.75rem; height: 2.75rem;
  display: flex; align-items: center; justify-content: center;
  border-radius: 0.75rem;
  color: #3b82f6;
  background: color-mix(in srgb, #3b82f6 14%, transparent);
}
.tool-heading h1 { font-size: 1.375rem; margin: 0; line-height: 1.1; }
.tool-heading p {
  color: var(--text-secondary); font-size: 0.8125rem; margin: 0.125rem 0 0;
}

/* ====== 双栏工作区 ====== */
.workspace {
  display: grid;
  grid-template-columns: 1fr;
  gap: 1rem;
}
@media (min-width: 768px) {
  .workspace {
    grid-template-columns: 1fr 1fr;
    align-items: stretch;
  }
}

.panel {
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 1rem;
  padding: 0.875rem;
  display: flex; flex-direction: column; gap: 0.875rem;
}

/* --- 模式切换 --- */
.mode-bar {
  display: flex; align-items: center; gap: 0.5rem;
}

.segmented {
  flex: 1;
  display: grid; grid-template-columns: repeat(2, 1fr);
  padding: 0.25rem; border-radius: 0.5rem; background: var(--bg-elevated);
}
.segmented button {
  min-height: 2rem; border: 0; border-radius: 0.375rem;
  background: transparent; color: var(--text-secondary);
  font-weight: 600; font-size: 0.8125rem; cursor: pointer;
  transition: background 0.15s, color 0.15s;
}
.segmented button.active {
  background: var(--bg-surface); color: var(--text-primary); box-shadow: var(--shadow-1);
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
  color: var(--text-secondary); font-size: 0.75rem;
  width: 100%; margin-left: 1.125rem;
}

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
}
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
  font-size: 0.75rem;
  color: var(--text-secondary);
  font-family: var(--font-family-mono, monospace);
}

/* --- 操作 --- */
.actions {
  display: flex; gap: 0.5rem;
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

.pipeline-banner {
  display: inline-flex; align-items: center; gap: 0.5rem;
  padding: 0.4375rem 0.75rem; margin-bottom: 0.75rem;
  border-radius: 0.5rem;
  background: color-mix(in srgb, #3b82f6 10%, transparent);
  border: 1px solid color-mix(in srgb, #3b82f6 25%, transparent);
  color: #1d4ed8; font-size: 0.75rem; font-weight: 600;
}

/* ====== Toast ====== */
.toast {
  position: fixed; left: 50%; bottom: 1.5rem; z-index: 1000;
  transform: translateX(-50%); padding: 0.625rem 0.875rem; border-radius: 999px;
  color: #fff; background: #18181b; box-shadow: var(--shadow-3);
  font-size: 0.8125rem; font-weight: 700;
}
.toast.success { background: #10b981; }
.toast.error { background: #ef4444; }
.toast-enter-active, .toast-leave-active { transition: opacity 0.2s, transform 0.2s; }
.toast-enter-from, .toast-leave-to { opacity: 0; transform: translate(-50%, 0.5rem); }
</style>
