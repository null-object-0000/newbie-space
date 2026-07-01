<template>
  <div class="tool-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />
    <main class="tool-main">
      <div class="tool-topbar">
        <router-link to="/tools" class="back-link"><ArrowLeft :size="16" /><span>工具中心</span></router-link>
      </div>
      <section class="tool-header">
        <div class="tool-heading">
          <div class="heading-icon"><FolderSearch :size="22" /></div>
          <div>
            <h1>{{ tool?.name }}</h1>
            <p>{{ tool?.desc }}</p>
          </div>
        </div>
      </section>

      <div class="workspace">
        <div class="panel panel-left">
          <label class="section-label">文件路径</label>
          <div class="ip-row">
            <input v-model="filePath" type="text" class="path-input" placeholder="输入绝对路径，如 /var/log/nginx/access.log" @keyup.enter="checkUsage" />
            <button class="btn primary" :disabled="!valid || loading" @click="checkUsage">
              <LoaderCircle v-if="loading" :size="16" class="spin" />
              <Search v-else :size="16" />
              {{ loading ? '检测中' : '检测' }}
            </button>
          </div>
          <div class="path-hint">
            <Info :size="14" />
            <span>支持文件路径和目录路径</span>
          </div>
          <div v-if="history.length" class="history-bar">
            <span class="section-label">历史记录</span>
            <button v-for="(p, i) in history" :key="i" class="history-chip" :title="p" @click="filePath = p; checkUsage()">
              {{ p.length > 40 ? '...' + p.slice(-37) : p }}
            </button>
          </div>
        </div>

        <div class="panel panel-right">
          <div v-if="error" class="result-empty">
            <AlertCircle :size="28" />
            <span>{{ error }}</span>
          </div>
          <div v-else-if="results === null && !loading" class="result-empty">
            <FolderOpen :size="28" />
            <span>输入文件路径检测占用进程</span>
          </div>
          <div v-else-if="loading" class="result-loading">
            <LoaderCircle :size="24" class="spin" />
            <span>检测中...</span>
          </div>
          <div v-else-if="!results.length" class="result-empty">
            <CheckCircle :size="28" />
            <span>未检测到占用该文件/目录的进程</span>
          </div>
          <div v-else class="result-table-wrap">
            <div class="table-header">
              <span class="result-count">共 {{ results.length }} 个进程占用</span>
            </div>
            <div class="result-table" ref="tableRef">
              <div v-for="item in results" :key="item.pid" class="result-row">
                <router-link :to="'/tools/process-manager'" class="usage-link">
                  <span class="usage-name">{{ item.process_name }}</span>
                  <span class="usage-pid">PID {{ item.pid }}</span>
                </router-link>
                <code v-if="item.file_path !== filePath" class="usage-path" :title="item.file_path">{{ item.file_path }}</code>
                <button
                  v-if="item.pid > 0"
                  class="kill-btn"
                  :disabled="killing === item.pid"
                  @click="confirmKill(item)"
                >
                  {{ killing === item.pid ? '处理中' : '终止' }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>

    <!-- 确认弹窗 -->
    <Dialog
      v-model="showConfirm"
      title="确认终止进程"
      :message="`确定要终止进程「${killTarget?.process_name}」(PID: ${killTarget?.pid}) 吗？`"
      sub-message="强制终止进程可能导致数据丢失，请谨慎操作。"
      :icon="AlertTriangle"
      cancel-text="取消"
      confirm-text="确认终止"
      @confirm="doKill"
      @cancel="killTarget = null"
    />

    <Transition name="toast">
      <div v-if="toastMessage" class="toast" :class="toastType">{{ toastMessage }}</div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { findTool } from '@/utils/toolPipeline'
import { ArrowLeft, AlertCircle, AlertTriangle, CheckCircle, FolderOpen, FolderSearch, Info, LoaderCircle, Search } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import Dialog from '@/components/Dialog.vue'

interface FileUsageItem {
  pid: number
  process_name: string
  file_path: string
}

interface KillResult {
  pid: number
  success: boolean
  message: string
}

const { isDark } = useTheme()
const tool = findTool('file-usage-check')

const filePath = ref('')
const loading = ref(false)
const results = ref<FileUsageItem[] | null>(null)
const error = ref('')
const killing = ref<number | null>(null)
const killTarget = ref<FileUsageItem | null>(null)
const showConfirm = ref(false)
const tableRef = ref<HTMLElement | null>(null)
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')
let toastTimer: ReturnType<typeof setTimeout> | null = null

const valid = computed(() => filePath.value.trim().length > 0)

const HISTORY_KEY = 'file-usage-history'
const history = ref<string[]>([])
function loadHistory() { try { const r = localStorage.getItem(HISTORY_KEY); if (r) history.value = JSON.parse(r) } catch {} }
function saveHistory() { try { localStorage.setItem(HISTORY_KEY, JSON.stringify(history.value)) } catch {} }
function pushHistory(p: string) {
  const i = history.value.indexOf(p); if (i >= 0) history.value.splice(i, 1)
  history.value.unshift(p); if (history.value.length > 5) history.value.pop()
  saveHistory()
}

async function checkUsage() {
  if (!valid.value || loading.value) return
  const path = filePath.value.trim()
  loading.value = true
  results.value = null
  error.value = ''
  try {
    const res = await invoke<FileUsageItem[]>('check_file_usage', { path })
    // 过滤掉错误占位条目 (pid === 0)
    const realResults = res.filter(item => item.pid > 0)
    // 检查是否有错误消息（pid === 0 且 process_name 非空）
    const errItem = res.find(item => item.pid === 0 && item.process_name !== '')
    if (errItem) {
      error.value = errItem.file_path
      results.value = []
    } else {
      results.value = realResults
    }
    pushHistory(path)
  } catch (e) {
    error.value = `检测失败：${e}`
  } finally {
    loading.value = false
  }
}

function confirmKill(item: FileUsageItem) {
  killTarget.value = item
  showConfirm.value = true
}

async function doKill() {
  showConfirm.value = false
  const target = killTarget.value
  killTarget.value = null
  if (!target) return

  killing.value = target.pid
  try {
    const result = await invoke<KillResult>('kill_process', { pid: target.pid })
    showToast(result.message, result.success ? 'success' : 'error')
    if (result.success) await checkUsage()
  } catch (e) {
    showToast(`操作失败：${e}`, 'error')
  } finally {
    killing.value = null
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
.heading-icon { --tool-color: #f59e0b; }

.path-input {
  width: 0; flex: 1; padding: 0.5rem 0.625rem;
  border: 1px solid var(--border-color); border-radius: 0.5rem;
  background: var(--bg-elevated); color: var(--text-primary);
  font-size: 0.875rem; font-family: var(--font-family-mono, monospace);
  outline: none; box-sizing: border-box;
}
.path-input:focus { border-color: var(--brand-500); }

.path-hint {
  display: flex; align-items: center; gap: 0.375rem;
  margin-top: 0.5rem; color: var(--text-secondary); font-size: 0.75rem;
}

.history-bar { display: flex; flex-direction: column; gap: 0.25rem; }
.history-chip {
  width: 100%; padding: 0.375rem 0.625rem;
  border: 1px solid var(--border-color); border-radius: 0.375rem;
  background: var(--bg-elevated); color: var(--text-secondary);
  font-family: var(--font-family-mono, monospace); font-size: 0.75rem;
  cursor: pointer; text-align: left; transition: border-color 0.15s, color 0.15s;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.history-chip:hover { border-color: var(--brand-500); color: var(--brand-500); }

.result-empty, .result-loading {
  display: flex; flex-direction: column; align-items: center; gap: 0.5rem;
  color: var(--text-secondary); font-size: 0.875rem; padding: 2rem 0;
}

.result-table-wrap { width: 100%; }
.table-header {
  display: flex; justify-content: space-between; align-items: center;
  margin-bottom: 0.5rem; font-size: 0.75rem;
}
.result-count { color: var(--text-secondary); }

.result-table {
  max-height: 420px; overflow-y: auto;
  border: 1px solid var(--border-color); border-radius: 0.5rem;
  background: var(--bg-elevated);
}
.result-row {
  display: flex; align-items: center; gap: 0.5rem;
  padding: 0.4375rem 0.625rem;
  border-bottom: 1px solid var(--border-color);
}
.result-row:last-child { border-bottom: 0; }

.usage-link {
  display: flex; align-items: center; gap: 0.375rem;
  text-decoration: none; color: var(--text-primary); flex: 1; min-width: 0;
  transition: color 0.15s;
}
.usage-link:hover { color: var(--brand-500); }
.usage-name { font-size: 0.8125rem; font-weight: 600; }
.usage-pid { font-size: 0.75rem; color: var(--text-secondary); font-family: var(--font-family-mono, monospace); }

.usage-path {
  font-family: var(--font-family-mono, monospace);
  font-size: 0.6875rem; color: var(--text-secondary);
  word-break: break-all; max-width: 200px; overflow: hidden;
  text-overflow: ellipsis; white-space: nowrap;
}

.kill-btn {
  padding: 0.25rem 0.625rem; border: 1px solid #ef4444; border-radius: 0.375rem;
  background: transparent; color: #ef4444; font-size: 0.75rem; font-weight: 600;
  cursor: pointer; transition: all 0.15s; white-space: nowrap; flex-shrink: 0;
}
.kill-btn:hover { background: #ef4444; color: #fff; }
.kill-btn:disabled { cursor: not-allowed; opacity: 0.5; }
.kill-btn:disabled:hover { background: transparent; color: #ef4444; }

.ip-row { display: flex; gap: 0.5rem; }

.spin { animation: spin 1s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
</style>
