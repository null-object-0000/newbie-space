<template>
  <div class="tool-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />
    <main class="tool-main">
      <div class="tool-topbar">
        <router-link to="/tools" class="back-link"><ArrowLeft :size="16" /><span>工具中心</span></router-link>
      </div>
      <section class="tool-header">
        <div class="tool-heading">
          <div class="heading-icon"><Activity :size="22" /></div>
          <div>
            <h1>{{ tool?.name }}</h1>
            <p>{{ tool?.desc }}</p>
          </div>
        </div>
      </section>

      <div class="workspace">
        <div class="panel panel-top" style="grid-column: 1 / -1;">
          <div class="toolbar">
            <div class="search-box">
              <Search :size="14" />
              <input v-model="search" type="text" class="search-input" placeholder="搜索进程名..." @keyup.enter="loadProcesses" />
              <button v-if="search" class="search-clear" @click="search = ''; loadProcesses()"><X :size="12" /></button>
            </div>
            <button class="btn primary" :disabled="loading" @click="loadProcesses">
              <LoaderCircle v-if="loading" :size="16" class="spin" />
              {{ loading ? '加载中' : '刷新列表' }}
            </button>
            <label class="auto-refresh">
              <input type="checkbox" v-model="autoRefresh" />
              <span>自动刷新</span>
            </label>
          </div>
        </div>

        <div class="panel panel-full" style="grid-column: 1 / -1;">
          <div v-if="error" class="result-empty">
            <AlertCircle :size="28" />
            <span>{{ error }}</span>
          </div>
          <div v-else-if="!processes.length && !loading" class="result-empty">
            <Cpu :size="28" />
            <span>点击"刷新列表"加载进程</span>
          </div>
          <div v-else class="process-table-wrap">
            <div class="table-meta">共 {{ processes.length }} 个进程</div>
            <div class="process-table" ref="tableRef">
              <div class="table-scroll">
                <table>
                  <thead>
                    <tr>
                      <th class="col-pid" @click="toggleSort('pid')">PID <span class="sort-arrow">{{ sortIcon('pid') }}</span></th>
                      <th class="col-name" @click="toggleSort('name')">名称 <span class="sort-arrow">{{ sortIcon('name') }}</span></th>
                      <th class="col-cpu" @click="toggleSort('cpu_usage')">CPU% <span class="sort-arrow">{{ sortIcon('cpu_usage') }}</span></th>
                      <th class="col-mem" @click="toggleSort('memory_mb')">内存 <span class="sort-arrow">{{ sortIcon('memory_mb') }}</span></th>
                      <th class="col-status">状态</th>
                      <th class="col-action">操作</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="p in sortedProcesses" :key="p.pid">
                      <td class="col-pid mono">{{ p.pid }}</td>
                      <td class="col-name" :title="p.exe_path ?? ''">{{ p.name }}</td>
                      <td class="col-cpu mono">{{ p.cpu_usage.toFixed(1) }}</td>
                      <td class="col-mem mono">{{ p.memory_mb.toFixed(1) }} MB</td>
                      <td class="col-status">{{ p.status }}</td>
                      <td class="col-action">
                        <button class="kill-btn" :disabled="killing === p.pid" @click="confirmKill(p)">
                          {{ killing === p.pid ? '处理中' : '终止' }}
                        </button>
                      </td>
                    </tr>
                  </tbody>
                </table>
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
      :message="`确定要终止进程「${killTarget?.name}」(PID: ${killTarget?.pid}) 吗？`"
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
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { findTool } from '@/utils/toolPipeline'
import { ArrowLeft, Activity, AlertCircle, AlertTriangle, Cpu, LoaderCircle, Search, X } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import Dialog from '@/components/Dialog.vue'

interface ProcessItem {
  pid: number
  name: string
  cpu_usage: number
  memory_mb: number
  status: string
  exe_path: string | null
}

interface KillResult {
  pid: number
  success: boolean
  message: string
}

type SortField = 'pid' | 'name' | 'cpu_usage' | 'memory_mb'

const { isDark } = useTheme()
const tool = findTool('process-manager')

const search = ref('')
const loading = ref(false)
const processes = ref<ProcessItem[]>([])
const error = ref('')
const sortField = ref<SortField>('pid')
const sortAsc = ref(true)
const autoRefresh = ref(false)
const killing = ref<number | null>(null)
const killTarget = ref<ProcessItem | null>(null)
const showConfirm = ref(false)
const tableRef = ref<HTMLElement | null>(null)
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')
let toastTimer: ReturnType<typeof setTimeout> | null = null
let refreshInterval: ReturnType<typeof setInterval> | null = null

const sortedProcesses = computed(() => {
  const arr = [...processes.value]
  arr.sort((a, b) => {
    let va: number | string, vb: number | string
    switch (sortField.value) {
      case 'pid': va = a.pid; vb = b.pid; break
      case 'name': va = a.name.toLowerCase(); vb = b.name.toLowerCase(); break
      case 'cpu_usage': va = a.cpu_usage; vb = b.cpu_usage; break
      case 'memory_mb': va = a.memory_mb; vb = b.memory_mb; break
    }
    if (va < vb) return sortAsc.value ? -1 : 1
    if (va > vb) return sortAsc.value ? 1 : -1
    return 0
  })
  return arr
})

function toggleSort(field: SortField) {
  if (sortField.value === field) {
    sortAsc.value = !sortAsc.value
  } else {
    sortField.value = field
    sortAsc.value = true
  }
}

function sortIcon(field: SortField) {
  if (sortField.value !== field) return ''
  return sortAsc.value ? '▲' : '▼'
}

async function loadProcesses() {
  loading.value = true
  error.value = ''
  try {
    const res = await invoke<ProcessItem[]>('list_processes', {
      search: search.value || null
    })
    processes.value = res
  } catch (e) {
    error.value = `加载失败：${e}`
  } finally {
    loading.value = false
  }
}

function confirmKill(p: ProcessItem) {
  killTarget.value = p
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
    if (result.success) await loadProcesses()
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

// 自动刷新
watch(autoRefresh, (on) => {
  if (on) {
    refreshInterval = setInterval(loadProcesses, 3000)
  } else {
    if (refreshInterval) { clearInterval(refreshInterval); refreshInterval = null }
  }
})

onMounted(() => { loadProcesses() })
onUnmounted(() => {
  if (toastTimer) clearTimeout(toastTimer)
  if (refreshInterval) clearInterval(refreshInterval)
})
</script>

<style scoped>
.heading-icon { --tool-color: #ef4444; }

.toolbar {
  display: flex; align-items: center; gap: 0.75rem;
  flex-wrap: wrap;
}

.search-box {
  display: flex; align-items: center; gap: 0.375rem;
  flex: 1; min-width: 180px; max-width: 320px;
  padding: 0.375rem 0.5rem; border: 1px solid var(--border-color);
  border-radius: 0.5rem; background: var(--bg-elevated);
}
.search-box:focus-within { border-color: var(--brand-500); }
.search-input {
  border: 0; background: transparent; color: var(--text-primary);
  font-size: 0.8125rem; outline: none; width: 0; flex: 1;
}
.search-clear {
  display: flex; align-items: center; border: 0; background: transparent;
  color: var(--text-secondary); cursor: pointer; padding: 0;
}

.auto-refresh {
  display: flex; align-items: center; gap: 0.375rem;
  font-size: 0.8125rem; color: var(--text-secondary); cursor: pointer;
  white-space: nowrap;
}

.result-empty {
  display: flex; flex-direction: column; align-items: center; gap: 0.5rem;
  color: var(--text-secondary); font-size: 0.875rem; padding: 3rem 0;
}

.process-table-wrap { width: 100%; }
.table-meta {
  margin-bottom: 0.5rem; font-size: 0.75rem; color: var(--text-secondary);
}

.process-table {
  border: 1px solid var(--border-color); border-radius: 0.5rem;
  overflow: hidden; background: var(--bg-elevated);
}
.table-scroll { max-height: 480px; overflow-y: auto; }

table { width: 100%; border-collapse: collapse; font-size: 0.8125rem; }
thead { position: sticky; top: 0; z-index: 1; }
thead th {
  background: var(--bg-surface); color: var(--text-secondary);
  font-weight: 600; text-align: left; padding: 0.5rem 0.625rem;
  border-bottom: 1px solid var(--border-color); user-select: none;
}
thead th:not(.col-action) { cursor: pointer; }
thead th:not(.col-action):hover { color: var(--brand-500); }
.sort-arrow { font-size: 0.625rem; margin-left: 0.125rem; }

tbody td {
  padding: 0.375rem 0.625rem; border-bottom: 1px solid var(--border-color);
  color: var(--text-primary);
}
tbody tr:last-child td { border-bottom: 0; }
tbody tr:hover { background: color-mix(in srgb, var(--brand-500) 5%, transparent); }

.col-pid { width: 72px; }
.col-cpu { width: 72px; }
.col-mem { width: 96px; }
.col-status { width: 88px; }
.col-action { width: 72px; text-align: center; }
.col-name { max-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.mono { font-family: var(--font-family-mono, monospace); }

.kill-btn {
  padding: 0.25rem 0.625rem; border: 1px solid #ef4444; border-radius: 0.375rem;
  background: transparent; color: #ef4444; font-size: 0.75rem; font-weight: 600;
  cursor: pointer; transition: all 0.15s;
}
.kill-btn:hover { background: #ef4444; color: #fff; }
.kill-btn:disabled { cursor: not-allowed; opacity: 0.5; }
.kill-btn:disabled:hover { background: transparent; color: #ef4444; }

.panel-top { padding-bottom: 0; }
.panel-full { padding-top: 0; }

.spin { animation: spin 1s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
</style>
