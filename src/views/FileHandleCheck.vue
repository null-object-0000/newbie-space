<template>
  <div class="tool-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />
    <main class="tool-main">
      <div class="tool-topbar">
        <router-link to="/tools" class="back-link"><ArrowLeft :size="16" /><span>工具中心</span></router-link>
      </div>
      <section class="tool-header">
        <div class="tool-heading">
          <div class="heading-icon"><FileSearch :size="22" /></div>
          <div>
            <h1>{{ tool?.name }}</h1>
            <p>{{ tool?.desc }}</p>
          </div>
        </div>
      </section>

      <div class="workspace">
        <div class="panel panel-left">
          <label class="section-label">进程 ID (PID)</label>
          <div class="ip-row">
            <input v-model.number="pid" type="number" min="1" class="pid-input" placeholder="输入 PID，如 1234" @keyup.enter="checkHandles" />
            <button class="btn primary" :disabled="!valid || loading" @click="checkHandles">
              <LoaderCircle v-if="loading" :size="16" class="spin" />
              <Search v-else :size="16" />
              {{ loading ? '查询中' : '查询' }}
            </button>
          </div>
          <div class="pid-hint">
            <Info :size="14" />
            <span>可在"进程管理"工具或任务管理器中查找 PID</span>
          </div>
        </div>

        <div class="panel panel-right">
          <div v-if="error" class="result-empty">
            <AlertCircle :size="28" />
            <span>{{ error }}</span>
          </div>
          <div v-else-if="results === null && !loading" class="result-empty">
            <FolderOpen :size="28" />
            <span>输入 PID 查询进程打开的文件句柄</span>
          </div>
          <div v-else-if="loading" class="result-loading">
            <LoaderCircle :size="24" class="spin" />
            <span>查询中...</span>
          </div>
          <div v-else-if="!results.length" class="result-empty">
            <FolderOpen :size="28" />
            <span>该进程未打开任何文件句柄</span>
          </div>
          <div v-else class="handle-table-wrap">
            <div class="table-header">
              <span class="handle-count">共 {{ results.length }} 个文件句柄</span>
              <span class="handle-process">进程: {{ results[0]?.process_name }}</span>
            </div>
            <div class="handle-table" ref="tableRef">
              <div v-for="(item, i) in results" :key="i" class="handle-row">
                <code class="handle-path">{{ item.file_path }}</code>
              </div>
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
import { computed, onUnmounted, ref } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { findTool } from '@/utils/toolPipeline'
import { ArrowLeft, AlertCircle, FileSearch, FolderOpen, Info, LoaderCircle, Search } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'

interface FileHandleItem {
  pid: number
  process_name: string
  file_path: string
}

const { isDark } = useTheme()
const tool = findTool('file-handle-check')

const pid = ref<number | null>(null)
const loading = ref(false)
const results = ref<FileHandleItem[] | null>(null)
const error = ref('')
const tableRef = ref<HTMLElement | null>(null)
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')
let toastTimer: ReturnType<typeof setTimeout> | null = null

const valid = computed(() => pid.value !== null && pid.value >= 1)

async function checkHandles() {
  if (!valid.value || loading.value) return
  loading.value = true
  results.value = null
  error.value = ''
  try {
    const res = await invoke<FileHandleItem[]>('check_file_handles', { pid: pid.value! })
    results.value = res
    if (!res.length) error.value = '未找到文件句柄'
  } catch (e) {
    error.value = `查询失败：${e}`
  } finally {
    loading.value = false
  }
}

function showToast(m: string, t: 'success' | 'error') {
  if (toastTimer) clearTimeout(toastTimer); toastMessage.value = m; toastType.value = t
  toastTimer = setTimeout(() => { toastMessage.value = '' }, 2200)
}

onUnmounted(() => { if (toastTimer) clearTimeout(toastTimer) })
</script>

<style scoped>
.heading-icon { --tool-color: #f59e0b; }

.pid-input {
  width: 0; flex: 1; padding: 0.5rem 0.625rem;
  border: 1px solid var(--border-color); border-radius: 0.5rem;
  background: var(--bg-elevated); color: var(--text-primary);
  font-size: 0.875rem; font-family: var(--font-family-mono, monospace);
  outline: none; box-sizing: border-box;
}
.pid-input:focus { border-color: var(--brand-500); }

.pid-hint {
  display: flex; align-items: center; gap: 0.375rem;
  margin-top: 0.5rem; color: var(--text-secondary); font-size: 0.75rem;
}

.result-empty, .result-loading {
  display: flex; flex-direction: column; align-items: center; gap: 0.5rem;
  color: var(--text-secondary); font-size: 0.875rem; padding: 2rem 0;
}

.handle-table-wrap { width: 100%; }
.table-header {
  display: flex; justify-content: space-between; align-items: center;
  margin-bottom: 0.5rem; font-size: 0.75rem;
}
.handle-count { color: var(--text-secondary); }
.handle-process { color: var(--brand-500); font-weight: 600; }

.handle-table {
  max-height: 420px; overflow-y: auto;
  border: 1px solid var(--border-color); border-radius: 0.5rem;
  background: var(--bg-elevated);
}
.handle-row {
  padding: 0.4375rem 0.625rem;
  border-bottom: 1px solid var(--border-color);
}
.handle-row:last-child { border-bottom: 0; }
.handle-path {
  font-family: var(--font-family-mono, monospace);
  font-size: 0.75rem; color: var(--text-primary);
  word-break: break-all; line-height: 1.5;
}

.ip-row { display: flex; gap: 0.5rem; }

.spin { animation: spin 1s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
</style>
