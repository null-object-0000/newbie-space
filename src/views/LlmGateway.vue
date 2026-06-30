<template>
  <div class="tool-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />
    <main class="tool-main">
      <div class="tool-topbar">
        <router-link to="/tools" class="back-link"><ArrowLeft :size="16" /><span>工具中心</span></router-link>
      </div>

      <section class="tool-header">
        <div class="tool-heading">
          <div class="heading-icon"><Bot :size="22" /></div>
          <div>
            <h1>{{ tool?.name }}</h1>
            <p>{{ tool?.desc }}</p>
          </div>
        </div>
      </section>

      <div class="workspace">
        <div class="panel panel-left">
          <div class="service-row">
            <div>
              <label class="section-label">本地服务</label>
              <div class="listen-url">{{ status.listen_url || 'http://127.0.0.1:11434' }}</div>
            </div>
            <div class="status-pill" :class="{ running: status.running }">
              <span class="status-dot" />
              {{ status.running ? '运行中' : '已停止' }}
            </div>
          </div>

          <div class="button-row">
            <button class="btn primary" :disabled="busy || status.running" @click="startServer">
              <Play :size="16" />
              启动
            </button>
            <button class="btn" :disabled="busy || !status.running" @click="stopServer">
              <Square :size="16" />
              停止
            </button>
            <button class="btn" :disabled="busy" @click="refreshAll">
              <RefreshCw :size="16" />
              刷新
            </button>
          </div>

          <div class="metric-grid">
            <div class="metric">
              <span>渠道</span>
              <strong>{{ status.enabled_channel_count }}/{{ status.channel_count }}</strong>
            </div>
            <div class="metric">
              <span>映射</span>
              <strong>{{ status.route_count }}</strong>
            </div>
            <div class="metric">
              <span>请求</span>
              <strong>{{ usage.requests }}</strong>
            </div>
            <div class="metric">
              <span>费用</span>
              <strong>${{ usage.estimated_cost.toFixed(6) }}</strong>
            </div>
          </div>

          <label class="section-label">网关配置 JSON</label>
          <textarea v-model="configText" class="config-editor" spellcheck="false" :disabled="status.running" />
          <div class="button-row">
            <button class="btn primary" :disabled="busy || status.running" @click="saveConfig">
              <Save :size="16" />
              保存配置
            </button>
            <button class="btn" :disabled="busy || status.running" @click="loadExample">
              <FileJson :size="16" />
              示例
            </button>
            <button class="btn" :disabled="busy" @click="testConfig">
              <FlaskConical :size="16" />
              测试渠道
            </button>
          </div>
        </div>

        <div class="panel panel-right">
          <div class="usage-strip">
            <div>
              <span>成功</span>
              <strong>{{ usage.success }}</strong>
            </div>
            <div>
              <span>失败</span>
              <strong>{{ usage.failed }}</strong>
            </div>
            <div>
              <span>Prompt</span>
              <strong>{{ usage.prompt_tokens }}</strong>
            </div>
            <div>
              <span>Completion</span>
              <strong>{{ usage.completion_tokens }}</strong>
            </div>
            <div>
              <span>Total</span>
              <strong>{{ usage.total_tokens }}</strong>
            </div>
          </div>

          <div v-if="testResults.length" class="test-results">
            <label class="section-label">渠道测试</label>
            <div v-for="result in testResults" :key="result.channel_id" class="test-row" :class="{ ok: result.ok }">
              <div class="test-main">
                <strong>{{ result.channel_name }}</strong>
                <span>{{ result.status ? `HTTP ${result.status}` : '未连接' }}</span>
                <span>{{ result.latency_ms }}ms</span>
              </div>
              <div class="test-message">{{ result.message }}</div>
              <div v-if="result.models.length" class="model-list">
                <span v-for="model in result.models.slice(0, 12)" :key="model">{{ model }}</span>
                <span v-if="result.models.length > 12">+{{ result.models.length - 12 }}</span>
              </div>
            </div>
          </div>


          <div class="log-header">
            <label class="section-label">请求日志</label>
            <button class="btn danger" :disabled="busy || !logs.length" @click="clearLogs">
              <Trash2 :size="16" />
              清空
            </button>
          </div>

          <div v-if="!logs.length" class="result-empty">
            <FileText :size="28" />
            <span>暂无请求记录</span>
          </div>
          <div v-else class="log-list">
            <div v-for="log in logs" :key="log.id" class="log-row">
              <div class="log-top">
                <span class="mono">{{ log.status }}</span>
                <strong>{{ log.public_model }}</strong>
                <span>{{ log.latency_ms }}ms</span>
                <span>{{ formatDate(log.created_at) }}</span>
              </div>
              <div class="log-meta">
                <span>channel: {{ log.channel_id || '-' }}</span>
                <span>upstream: {{ log.upstream_model || '-' }}</span>
                <span>tokens: {{ log.total_tokens }}</span>
                <span>${{ log.estimated_cost.toFixed(6) }}</span>
              </div>
              <div v-if="log.error" class="log-error">{{ log.error }}</div>
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
import { onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useTheme } from '@/composables/useTheme'
import { findTool } from '@/utils/toolPipeline'
import {
  ArrowLeft,
  Bot,
  FileJson,
  FileText,
  FlaskConical,
  Play,
  RefreshCw,
  Save,
  Square,
  Trash2
} from 'lucide-vue-next'

interface GatewayStatus {
  running: boolean
  listen_url: string
  channel_count: number
  enabled_channel_count: number
  route_count: number
  request_count: number
}

interface UsageSummary {
  requests: number
  success: number
  failed: number
  prompt_tokens: number
  completion_tokens: number
  total_tokens: number
  estimated_cost: number
}

interface RequestLog {
  id: string
  created_at: string
  public_model: string
  upstream_model: string | null
  channel_id: string | null
  key_id: string | null
  status: number
  latency_ms: number
  prompt_tokens: number
  completion_tokens: number
  total_tokens: number
  estimated_cost: number
  error: string | null
}
interface ChannelTestResult {
  channel_id: string
  channel_name: string
  ok: boolean
  status: number | null
  latency_ms: number
  message: string
  models: string[]
}

const { isDark } = useTheme()
const tool = findTool('llm-gateway')

const busy = ref(false)
const configText = ref('')
const logs = ref<RequestLog[]>([])
const testResults = ref<ChannelTestResult[]>([])
const status = ref<GatewayStatus>({
  running: false,
  listen_url: '',
  channel_count: 0,
  enabled_channel_count: 0,
  route_count: 0,
  request_count: 0
})
const usage = ref<UsageSummary>({
  requests: 0,
  success: 0,
  failed: 0,
  prompt_tokens: 0,
  completion_tokens: 0,
  total_tokens: 0,
  estimated_cost: 0
})
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')
let toastTimer: ReturnType<typeof setTimeout> | null = null
let refreshTimer: ReturnType<typeof setInterval> | null = null

async function refreshAll() {
  const [nextStatus, nextUsage, nextLogs] = await Promise.all([
    invoke<GatewayStatus>('get_llm_gateway_status'),
    invoke<UsageSummary>('get_llm_gateway_usage'),
    invoke<RequestLog[]>('list_llm_gateway_logs', { limit: 100 })
  ])
  status.value = nextStatus
  usage.value = nextUsage
  logs.value = nextLogs
}

async function loadConfig() {
  const config = await invoke<Record<string, unknown>>('get_llm_gateway_config')
  configText.value = JSON.stringify(config, null, 2)
}

async function saveConfig() {
  busy.value = true
  try {
    const config = JSON.parse(configText.value)
    await invoke('save_llm_gateway_config', { config })
    await refreshAll()
    showToast('配置已保存', 'success')
  } catch (e) {
    showToast(`保存失败：${e}`, 'error')
  } finally {
    busy.value = false
  }
}

async function startServer() {
  busy.value = true
  try {
    status.value = await invoke<GatewayStatus>('start_llm_gateway')
    await refreshAll()
    showToast('本地 LLM API 服务已启动', 'success')
  } catch (e) {
    showToast(`启动失败：${e}`, 'error')
  } finally {
    busy.value = false
  }
}

async function stopServer() {
  busy.value = true
  try {
    status.value = await invoke<GatewayStatus>('stop_llm_gateway')
    await refreshAll()
    showToast('本地 LLM API 服务已停止', 'success')
  } catch (e) {
    showToast(`停止失败：${e}`, 'error')
  } finally {
    busy.value = false
  }
}

async function clearLogs() {
  busy.value = true
  try {
    await invoke('clear_llm_gateway_logs')
    await refreshAll()
    showToast('日志已清空', 'success')
  } catch (e) {
    showToast(`清空失败：${e}`, 'error')
  } finally {
    busy.value = false
  }
}

async function testConfig() {
  busy.value = true
  try {
    const config = JSON.parse(configText.value)
    testResults.value = await invoke<ChannelTestResult[]>('test_llm_gateway_config', { config })
    const okCount = testResults.value.filter(result => result.ok).length
    showToast(`测试完成：${okCount}/${testResults.value.length} 个渠道可用`, okCount ? 'success' : 'error')
  } catch (e) {
    showToast(`测试失败：${e}`, 'error')
  } finally {
    busy.value = false
  }
}

function loadExample() {
  configText.value = JSON.stringify({
    listen_host: '127.0.0.1',
    listen_port: 11434,
    channels: [
      {
        id: 'work-friday',
        name: 'Work Friday',
        base_url: 'https://work-friday.nichangen.com',
        enabled: true,
        priority: 1,
        key_strategy: 'round_robin',
        timeout_ms: 60000,
        keys: [
          {
            id: 'work-friday-key-1',
            name: '测试 key',
            api_key: 'sk-xxx',
            enabled: true,
            weight: 1
          }
        ]
      },
      {
        id: 'work-friday-flash',
        name: 'Work Friday Flash',
        base_url: 'https://work-friday.nichangen.com',
        enabled: false,
        priority: 2,
        key_strategy: 'random',
        timeout_ms: 60000,
        keys: []
      }
    ],
    model_routes: [
      {
        id: 'deepseek-v4-pro-main',
        public_model: 'deepseek-v4-pro',
        upstream_model: 'deepseek-v4-pro',
        channel_id: 'work-friday',
        enabled: true,
        priority: 1,
        prompt_cost_per_1k: 0,
        completion_cost_per_1k: 0
      },
      {
        id: 'deepseek-v4-pro-backup',
        public_model: 'deepseek-v4-pro',
        upstream_model: 'deepseek-v4-flash',
        channel_id: 'work-friday',
        enabled: true,
        priority: 2,
        prompt_cost_per_1k: 0,
        completion_cost_per_1k: 0
      }
    ]
  }, null, 2)
}

function formatDate(value: string) {
  return new Date(value).toLocaleString()
}

function showToast(m: string, t: 'success' | 'error') {
  if (toastTimer) clearTimeout(toastTimer)
  toastMessage.value = m
  toastType.value = t
  toastTimer = setTimeout(() => { toastMessage.value = '' }, 2200)
}

onMounted(async () => {
  try {
    await loadConfig()
    await refreshAll()
    refreshTimer = setInterval(refreshAll, 5000)
  } catch (e) {
    showToast(`加载失败：${e}`, 'error')
  }
})

onUnmounted(() => {
  if (toastTimer) clearTimeout(toastTimer)
  if (refreshTimer) clearInterval(refreshTimer)
})
</script>

<style scoped>
.heading-icon { --tool-color: #14b8a6; }

.service-row,
.button-row,
.log-header {
  display: flex;
  align-items: center;
  gap: 0.625rem;
}

.test-results {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.test-row {
  padding: 0.625rem;
  border: 1px solid color-mix(in srgb, #ef4444 45%, var(--border-color));
  border-radius: 0.5rem;
  background: var(--bg-elevated);
}

.test-row.ok {
  border-color: color-mix(in srgb, #10b981 55%, var(--border-color));
}

.test-main,
.model-list {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.test-main {
  color: var(--text-primary);
  font-size: 0.8125rem;
}

.test-message {
  margin-top: 0.375rem;
  color: var(--text-secondary);
  font-size: 0.75rem;
}

.model-list {
  margin-top: 0.5rem;
}

.model-list span {
  padding: 0.125rem 0.375rem;
  border-radius: 0.375rem;
  background: var(--bg-muted);
  color: var(--text-secondary);
  font-family: var(--font-family-mono, monospace);
  font-size: 0.6875rem;
}

.service-row {
  justify-content: space-between;
}

.button-row {
  flex-wrap: wrap;
}

.listen-url {
  margin-top: 0.25rem;
  color: var(--text-primary);
  font-family: var(--font-family-mono, monospace);
  font-size: 0.875rem;
}

.status-pill {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--border-color);
  border-radius: 999px;
  color: var(--text-secondary);
  font-size: 0.75rem;
  white-space: nowrap;
}

.status-dot {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 999px;
  background: #94a3b8;
}

.status-pill.running {
  color: #10b981;
  border-color: color-mix(in srgb, #10b981 55%, var(--border-color));
}

.status-pill.running .status-dot {
  background: #10b981;
}

.metric-grid,
.usage-strip {
  display: grid;
  gap: 0.5rem;
}

.metric-grid {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.usage-strip {
  grid-template-columns: repeat(5, minmax(0, 1fr));
  margin-bottom: 1rem;
}

.metric,
.usage-strip > div {
  padding: 0.625rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-elevated);
}

.metric span,
.usage-strip span {
  display: block;
  color: var(--text-secondary);
  font-size: 0.75rem;
  margin-bottom: 0.25rem;
}

.metric strong,
.usage-strip strong {
  color: var(--text-primary);
  font-size: 1rem;
}

.config-editor {
  width: 100%;
  min-height: 420px;
  padding: 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-family: var(--font-family-mono, monospace);
  font-size: 0.8125rem;
  line-height: 1.5;
  resize: vertical;
  outline: none;
  box-sizing: border-box;
}

.config-editor:focus {
  border-color: var(--brand-500);
}


.log-header {
  justify-content: space-between;
  margin-bottom: 0.5rem;
}

.result-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  color: var(--text-secondary);
  font-size: 0.875rem;
  padding: 3rem 0;
}

.log-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  max-height: 620px;
  overflow: auto;
}

.log-row {
  padding: 0.625rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-elevated);
}

.log-top,
.log-meta {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  flex-wrap: wrap;
}

.log-top {
  color: var(--text-primary);
  font-size: 0.8125rem;
}

.log-meta {
  margin-top: 0.375rem;
  color: var(--text-secondary);
  font-size: 0.75rem;
}

.log-error {
  margin-top: 0.375rem;
  color: #ef4444;
  font-size: 0.75rem;
}

.mono {
  font-family: var(--font-family-mono, monospace);
}

.danger {
  color: #ef4444;
  border-color: color-mix(in srgb, #ef4444 45%, var(--border-color));
}

@media (max-width: 860px) {
  .usage-strip {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
