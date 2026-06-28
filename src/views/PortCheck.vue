<template>
  <div class="tool-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />
    <main class="tool-main">
      <div class="tool-topbar">
        <router-link to="/tools" class="back-link"><ArrowLeft :size="16" /><span>工具中心</span></router-link>
      </div>
      <section class="tool-header">
        <div class="tool-heading">
          <div class="heading-icon"><Network :size="22" /></div>
          <div>
            <h1>{{ tool?.name }}</h1>
            <p>{{ tool?.desc }}</p>
          </div>
        </div>
      </section>

      <div class="workspace">
        <div class="panel panel-left">
          <label class="section-label">端口号</label>
          <div class="ip-row">
            <input v-model.number="port" type="number" min="1" max="65535" class="port-input" placeholder="输入端口号，如 443" @keyup.enter="checkPort" />
            <button class="btn primary" :disabled="!valid || checking" @click="checkPort">
              <LoaderCircle v-if="checking" :size="16" class="spin" />
              <Search v-else :size="16" />
              {{ checking ? '检测中' : '检测' }}
            </button>
          </div>
          <div v-if="history.length" class="history-bar">
            <span class="section-label">历史记录</span>
            <button v-for="(p, i) in history" :key="i" class="history-chip" @click="port = p; checkPort()">
              {{ p }}
            </button>
          </div>
        </div>

        <div class="panel panel-right result-panel">
          <div v-if="result === null && !checking" class="result-empty">
            <Wifi :size="28" />
            <span>输入端口号点击检测</span>
          </div>
          <div v-else-if="checking" class="result-loading">
            <LoaderCircle :size="24" class="spin" />
            <span>检测中...</span>
          </div>
          <div v-else class="result-body">
            <div class="result-status" :class="result.is_open ? 'open' : 'closed'">
              <div class="status-dot" />
              <span class="status-text">{{ result.is_open ? '端口开放' : '端口关闭' }}</span>
            </div>
            <div class="result-message">{{ result.message }}</div>
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
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { findTool } from '@/utils/toolPipeline'
import { ArrowLeft, LoaderCircle, Network, Search, Wifi } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'

interface PortCheckResult {
  port: number
  is_open: boolean
  message: string
}

const { isDark } = useTheme()
const tool = findTool('port-check')

const port = ref<number | null>(null)
const checking = ref(false)
const result = ref<PortCheckResult | null>(null)
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')
let toastTimer: ReturnType<typeof setTimeout> | null = null

const valid = computed(() => port.value !== null && port.value >= 1 && port.value <= 65535)

const HISTORY_KEY = 'port-check-history'
const history = ref<number[]>([])
function loadHistory() { try { const r = localStorage.getItem(HISTORY_KEY); if (r) history.value = JSON.parse(r) } catch {} }
function saveHistory() { try { localStorage.setItem(HISTORY_KEY, JSON.stringify(history.value)) } catch {} }
function pushHistory(p: number) {
  const i = history.value.indexOf(p); if (i >= 0) history.value.splice(i, 1)
  history.value.unshift(p); if (history.value.length > 5) history.value.pop()
  saveHistory()
}

async function checkPort() {
  if (!valid.value || checking.value) return
  checking.value = true
  result.value = null
  try {
    const res = await invoke<PortCheckResult>('check_port', { port: port.value! })
    result.value = res
    pushHistory(port.value!)
  } catch (e) {
    showToast(`检测失败：${e}`, 'error')
  } finally {
    checking.value = false
  }
}

function showToast(m: string, t: 'success' | 'error') {
  if (toastTimer) clearTimeout(toastTimer); toastMessage.value = m; toastType.value = t
  toastTimer = setTimeout(() => { toastMessage.value = '' }, 2200)
}

onMounted(() => { loadHistory() })
onUnmounted(() => { if (toastTimer) clearTimeout(toastTimer) })
</script>

<style scoped>
.heading-icon { --tool-color: #3b82f6; }

.port-input {
  width: 0; flex: 1; padding: 0.5rem 0.625rem;
  border: 1px solid var(--border-color); border-radius: 0.5rem;
  background: var(--bg-elevated); color: var(--text-primary);
  font-size: 0.875rem; font-family: var(--font-family-mono, monospace);
  outline: none; box-sizing: border-box;
}
.port-input:focus { border-color: var(--brand-500); }

.history-bar { display: flex; flex-direction: column; gap: 0.25rem; }
.history-chip {
  width: 100%; padding: 0.375rem 0.625rem;
  border: 1px solid var(--border-color); border-radius: 0.375rem;
  background: var(--bg-elevated); color: var(--text-secondary);
  font-family: var(--font-family-mono, monospace); font-size: 0.8125rem;
  cursor: pointer; text-align: center; transition: border-color 0.15s, color 0.15s;
}
.history-chip:hover { border-color: var(--brand-500); color: var(--brand-500); }

.result-panel { justify-content: center; align-items: center; gap: 1rem; }
.result-empty, .result-loading {
  display: flex; flex-direction: column; align-items: center; gap: 0.5rem;
  color: var(--text-secondary); font-size: 0.875rem;
}
.result-body { text-align: center; }
.result-status { display: flex; align-items: center; justify-content: center; gap: 0.5rem; margin-bottom: 0.5rem; }
.status-dot { width: 0.75rem; height: 0.75rem; border-radius: 50%; }
.result-status.open .status-dot { background: #10b981; box-shadow: 0 0 8px rgba(16, 185, 129, 0.4); }
.result-status.closed .status-dot { background: #ef4444; }
.status-text { font-size: 1.5rem; font-weight: 700; }
.result-status.open .status-text { color: #10b981; }
.result-status.closed .status-text { color: #ef4444; }
.result-message { font-size: 0.8125rem; color: var(--text-secondary); word-break: break-all; }

.ip-row { display: flex; gap: 0.5rem; }

.spin { animation: spin 1s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
</style>
