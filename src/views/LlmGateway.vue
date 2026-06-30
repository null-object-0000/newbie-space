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
            <button class="btn" :disabled="busy" @click="testConfig">
              <FlaskConical :size="16" />
              测试渠道
            </button>
          </div>

          <div class="metric-grid">
            <div class="metric"><span>渠道</span><strong>{{ status.enabled_channel_count }}/{{ status.channel_count }}</strong></div>
            <div class="metric"><span>映射</span><strong>{{ status.route_count }}</strong></div>
            <div class="metric"><span>请求</span><strong>{{ usage.requests }}</strong></div>
            <div class="metric"><span>费用</span><strong>${{ usage.estimated_cost.toFixed(6) }}</strong></div>
          </div>

          <div class="profile-box">
            <label class="section-label">配置 Profile</label>
            <div class="profile-row">
              <select v-model="selectedProfile">
                <option value="">选择 Profile</option>
                <option v-for="profile in profiles" :key="profile.name" :value="profile.name">
                  {{ profile.active ? `* ${profile.name}` : profile.name }}
                </option>
              </select>
              <input v-model.trim="profileName" placeholder="profile-name" :disabled="status.running" />
            </div>
            <div class="button-row">
              <button class="btn" :disabled="busy || status.running || !profileName" @click="saveProfile">保存 Profile</button>
              <button class="btn" :disabled="busy || status.running || !selectedProfile" @click="loadProfile">加载</button>
              <button class="btn danger" :disabled="busy || !selectedProfile" @click="deleteProfile">删除</button>
            </div>
          </div>

          <fieldset class="config-form" :disabled="status.running">
            <div class="section-title">
              <label class="section-label">监听设置</label>
            </div>
            <div class="form-grid two">
              <label class="field">
                <span>Host</span>
                <select v-model="config.listen_host">
                  <option value="127.0.0.1">127.0.0.1</option>
                  <option value="localhost">localhost</option>
                  <option value="::1">::1</option>
                </select>
              </label>
              <label class="field">
                <span>Port</span>
                <input v-model.number="config.listen_port" type="number" min="1" max="65535" />
              </label>
            </div>
            <div class="form-grid two">
              <label class="field"><span>最大并发</span><input v-model.number="config.max_concurrent_requests" type="number" min="0" max="256" /></label>
              <label class="field"><span>每分钟请求</span><input v-model.number="config.requests_per_minute" type="number" min="0" max="60000" /></label>
            </div>

            <div class="section-title">
              <label class="section-label">上游渠道</label>
              <button type="button" class="icon-btn" @click="addChannel" title="新增渠道"><Plus :size="16" /></button>
            </div>
            <div v-if="!config.channels.length" class="empty-row">暂无渠道</div>
            <div v-for="channel in config.channels" :key="channel.id" class="config-card">
              <div class="card-head">
                <label class="toggle"><input v-model="channel.enabled" type="checkbox" />启用</label>
                <button type="button" class="icon-btn danger" @click="removeChannel(channel.id)" title="删除渠道"><Trash2 :size="16" /></button>
              </div>
              <div class="form-grid two">
                <label class="field"><span>ID</span><input v-model.trim="channel.id" /></label>
                <label class="field"><span>名称</span><input v-model.trim="channel.name" /></label>
              </div>
              <label class="field"><span>Base URL</span><input v-model.trim="channel.base_url" placeholder="https://api.example.com" /></label>
              <div class="form-grid three">
                <label class="field"><span>优先级</span><input v-model.number="channel.priority" type="number" min="1" /></label>
                <label class="field"><span>Key 策略</span><select v-model="channel.key_strategy"><option value="round_robin">轮询</option><option value="random">随机</option></select></label>
                <label class="field"><span>超时 ms</span><input v-model.number="channel.timeout_ms" type="number" min="1000" step="1000" /></label>
              </div>
              <div class="form-grid one-small">
                <label class="field"><span>失败重试</span><input v-model.number="channel.retry_count" type="number" min="0" max="5" /></label>
              </div>

              <div class="section-title nested">
                <label class="section-label">API Keys</label>
                <div class="mini-actions">
                  <button type="button" class="btn compact" @click="discoverModels(channel)">发现模型</button>
                  <button type="button" class="icon-btn" @click="addKey(channel)" title="新增 Key"><Plus :size="16" /></button>
                </div>
              </div>
              <div v-if="!channel.keys.length" class="empty-row compact">暂无 Key</div>
              <div v-for="key in channel.keys" :key="key.id" class="key-row">
                <label class="toggle"><input v-model="key.enabled" type="checkbox" /></label>
                <input v-model.trim="key.id" placeholder="key-id" />
                <input v-model.trim="key.name" placeholder="名称" />
                <input v-model="key.api_key" type="password" placeholder="sk-..." />
                <input v-model.number="key.weight" type="number" min="1" title="权重" />
                <button type="button" class="icon-btn danger" @click="removeKey(channel, key.id)" title="删除 Key"><Trash2 :size="16" /></button>
              </div>
            </div>

            <div class="section-title">
              <label class="section-label">模型映射</label>
              <button type="button" class="icon-btn" @click="addRoute" title="新增映射"><Plus :size="16" /></button>
            </div>
            <div v-if="!config.model_routes.length" class="empty-row">暂无映射</div>
            <div v-for="route in config.model_routes" :key="route.id" class="config-card route-card">
              <div class="card-head">
                <label class="toggle"><input v-model="route.enabled" type="checkbox" />启用</label>
                <button type="button" class="icon-btn danger" @click="removeRoute(route.id)" title="删除映射"><Trash2 :size="16" /></button>
              </div>
              <div class="form-grid two">
                <label class="field"><span>Public Model</span><input v-model.trim="route.public_model" /></label>
                <label class="field"><span>Upstream Model</span><input v-model.trim="route.upstream_model" /></label>
              </div>
              <div class="form-grid three">
                <label class="field"><span>渠道</span><select v-model="route.channel_id"><option v-for="channel in config.channels" :key="channel.id" :value="channel.id">{{ channel.name || channel.id }}</option></select></label>
                <label class="field"><span>优先级</span><input v-model.number="route.priority" type="number" min="1" /></label>
                <label class="field"><span>ID</span><input v-model.trim="route.id" /></label>
              </div>
              <div class="form-grid two">
                <label class="field"><span>Prompt $/1K</span><input v-model.number="route.prompt_cost_per_1k" type="number" min="0" step="0.000001" /></label>
                <label class="field"><span>Completion $/1K</span><input v-model.number="route.completion_cost_per_1k" type="number" min="0" step="0.000001" /></label>
              </div>
            </div>
          </fieldset>

          <div class="button-row save-row">
            <button class="btn primary" :disabled="busy || status.running" @click="saveConfig">
              <Save :size="16" />
              保存配置
            </button>
            <button class="btn" :disabled="busy || status.running" @click="loadExample">
              <FileJson :size="16" />
              示例
            </button>
          </div>

          <details class="advanced-json">
            <summary>高级 JSON</summary>
            <textarea v-model="configText" class="config-editor" spellcheck="false" :disabled="status.running" />
            <div class="button-row">
              <button class="btn" :disabled="busy || status.running" @click="syncConfigText">从表单同步</button>
              <button class="btn" :disabled="busy || status.running" @click="applyJsonToForm">应用 JSON</button>
            </div>
          </details>
        </div>

        <div class="panel panel-right">
          <div class="usage-strip">
            <div><span>成功</span><strong>{{ usage.success }}</strong></div>
            <div><span>失败</span><strong>{{ usage.failed }}</strong></div>
            <div><span>Prompt</span><strong>{{ usage.prompt_tokens }}</strong></div>
            <div><span>Completion</span><strong>{{ usage.completion_tokens }}</strong></div>
            <div><span>Total</span><strong>{{ usage.total_tokens }}</strong></div>
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
            <div v-for="log in logs" :key="log.id" class="log-row clickable" @click="selectedLog = log">
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

    <div v-if="selectedLog" class="modal-backdrop" @click.self="selectedLog = null">
      <div class="log-modal">
        <div class="log-header">
          <label class="section-label">请求详情</label>
          <button class="icon-btn" @click="selectedLog = null">×</button>
        </div>
        <div class="detail-grid">
          <span>ID</span><strong>{{ selectedLog.id }}</strong>
          <span>Status</span><strong>{{ selectedLog.status }}</strong>
          <span>Model</span><strong>{{ selectedLog.public_model }}</strong>
          <span>Channel</span><strong>{{ selectedLog.channel_id || '-' }}</strong>
          <span>Key</span><strong>{{ selectedLog.key_id || '-' }}</strong>
          <span>Latency</span><strong>{{ selectedLog.latency_ms }}ms</strong>
        </div>
        <div v-if="selectedLog.error" class="log-error detail-error">{{ selectedLog.error }}</div>
        <label class="section-label">Input</label>
        <pre>{{ JSON.stringify(selectedLog.input, null, 2) }}</pre>
        <label class="section-label">Output</label>
        <pre>{{ JSON.stringify(selectedLog.output, null, 2) }}</pre>
      </div>
    </div>
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
  Plus,
  RefreshCw,
  Save,
  Square,
  Trash2
} from 'lucide-vue-next'

type KeyStrategy = 'round_robin' | 'random'

interface ApiKeyConfig {
  id: string
  name: string
  api_key: string
  enabled: boolean
  weight: number
}

interface ChannelConfig {
  id: string
  name: string
  base_url: string
  enabled: boolean
  priority: number
  key_strategy: KeyStrategy
  timeout_ms: number
  retry_count: number
  keys: ApiKeyConfig[]
}

interface ModelRouteConfig {
  id: string
  public_model: string
  upstream_model: string
  channel_id: string
  enabled: boolean
  priority: number
  prompt_cost_per_1k: number
  completion_cost_per_1k: number
}

interface GatewayConfig {
  listen_host: string
  listen_port: number
  channels: ChannelConfig[]
  model_routes: ModelRouteConfig[]
  max_concurrent_requests: number
  requests_per_minute: number
}

interface ConfigProfile {
  name: string
  active: boolean
  updated_at: string | null
}

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
  input: unknown
  output: unknown
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

type ModelDiscoveryResult = ChannelTestResult

const { isDark } = useTheme()
const tool = findTool('llm-gateway')

const busy = ref(false)
const config = ref<GatewayConfig>(createDefaultConfig())
const configText = ref('')
const logs = ref<RequestLog[]>([])
const selectedLog = ref<RequestLog | null>(null)
const testResults = ref<ChannelTestResult[]>([])
const profiles = ref<ConfigProfile[]>([])
const selectedProfile = ref('')
const profileName = ref('default')
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

function createDefaultConfig(): GatewayConfig {
  return {
    listen_host: '127.0.0.1',
    listen_port: 11434,
    max_concurrent_requests: 8,
    requests_per_minute: 120,
    channels: [],
    model_routes: [],
    max_concurrent_requests: 8,
    requests_per_minute: 120
  }
}

function normalizeConfig(value: GatewayConfig): GatewayConfig {
  return {
    listen_host: value.listen_host || '127.0.0.1',
    listen_port: Number(value.listen_port || 11434),
    max_concurrent_requests: Number(value.max_concurrent_requests ?? 8),
    requests_per_minute: Number(value.requests_per_minute ?? 120),
    channels: (value.channels || []).map(channel => ({
      id: channel.id || '',
      name: channel.name || channel.id || '',
      base_url: channel.base_url || '',
      enabled: channel.enabled ?? true,
      priority: Number(channel.priority || 1),
      key_strategy: channel.key_strategy || 'round_robin',
      timeout_ms: Number(channel.timeout_ms || 60000),
      retry_count: Number(channel.retry_count || 0),
      keys: (channel.keys || []).map(key => ({
        id: key.id || '',
        name: key.name || key.id || '',
        api_key: key.api_key || '',
        enabled: key.enabled ?? true,
        weight: Number(key.weight || 1)
      }))
    })),
    model_routes: (value.model_routes || []).map(route => ({
      id: route.id || '',
      public_model: route.public_model || '',
      upstream_model: route.upstream_model || '',
      channel_id: route.channel_id || '',
      enabled: route.enabled ?? true,
      priority: Number(route.priority || 1),
      prompt_cost_per_1k: Number(route.prompt_cost_per_1k || 0),
      completion_cost_per_1k: Number(route.completion_cost_per_1k || 0)
    }))
  }
}

function syncConfigText() {
  configText.value = JSON.stringify(config.value, null, 2)
}

function applyJsonToForm() {
  try {
    config.value = normalizeConfig(JSON.parse(configText.value) as GatewayConfig)
    syncConfigText()
    showToast('JSON 已应用到表单', 'success')
  } catch (e) {
    showToast(`JSON 无效：${e}`, 'error')
  }
}

async function refreshAll() {
  const [nextStatus, nextUsage, nextLogs] = await Promise.all([
    invoke<GatewayStatus>('get_llm_gateway_status'),
    invoke<UsageSummary>('get_llm_gateway_usage'),
    invoke<RequestLog[]>('list_llm_gateway_logs', { limit: 100 })
  ])
  status.value = nextStatus
  usage.value = nextUsage
  logs.value = nextLogs
  if (selectedLog.value) {
    selectedLog.value = nextLogs.find(log => log.id === selectedLog.value?.id) || selectedLog.value
  }
}

async function loadConfig() {
  const nextConfig = await invoke<GatewayConfig>('get_llm_gateway_config')
  config.value = normalizeConfig(nextConfig)
  syncConfigText()
  await loadProfiles()
}

async function saveConfig() {
  busy.value = true
  try {
    syncConfigText()
    await invoke('save_llm_gateway_config', { config: config.value })
    await refreshAll()
    await loadProfiles()
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
    selectedLog.value = null
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
    syncConfigText()
    testResults.value = await invoke<ChannelTestResult[]>('test_llm_gateway_config', { config: config.value })
    const okCount = testResults.value.filter(result => result.ok).length
    showToast(`测试完成：${okCount}/${testResults.value.length} 个渠道可用`, okCount ? 'success' : 'error')
  } catch (e) {
    showToast(`测试失败：${e}`, 'error')
  } finally {
    busy.value = false
  }
}

async function loadProfiles() {
  profiles.value = await invoke<ConfigProfile[]>('list_llm_gateway_profiles')
  const active = profiles.value.find(profile => profile.active)
  selectedProfile.value = active?.name || selectedProfile.value
  if (active?.name) profileName.value = active.name
}

async function saveProfile() {
  busy.value = true
  try {
    syncConfigText()
    await invoke('save_llm_gateway_profile', { name: profileName.value, config: config.value })
    selectedProfile.value = profileName.value
    await loadProfiles()
    showToast('Profile 已保存', 'success')
  } catch (e) {
    showToast(`保存 Profile 失败：${e}`, 'error')
  } finally {
    busy.value = false
  }
}

async function loadProfile() {
  if (!selectedProfile.value) return
  busy.value = true
  try {
    const nextConfig = await invoke<GatewayConfig>('load_llm_gateway_profile', { name: selectedProfile.value })
    config.value = normalizeConfig(nextConfig)
    profileName.value = selectedProfile.value
    syncConfigText()
    await loadProfiles()
    await refreshAll()
    showToast('Profile 已加载', 'success')
  } catch (e) {
    showToast(`加载 Profile 失败：${e}`, 'error')
  } finally {
    busy.value = false
  }
}

async function deleteProfile() {
  if (!selectedProfile.value) return
  busy.value = true
  try {
    await invoke('delete_llm_gateway_profile', { name: selectedProfile.value })
    selectedProfile.value = ''
    await loadProfiles()
    showToast('Profile 已删除', 'success')
  } catch (e) {
    showToast(`删除 Profile 失败：${e}`, 'error')
  } finally {
    busy.value = false
  }
}

async function discoverModels(channel: ChannelConfig) {
  busy.value = true
  try {
    syncConfigText()
    const result = await invoke<ModelDiscoveryResult>('discover_llm_gateway_models', {
      config: config.value,
      channelId: channel.id
    })
    testResults.value = [result]
    if (!result.ok) {
      showToast(`发现失败：${result.message}`, 'error')
      return
    }
    const added = addRoutesFromModels(channel, result.models)
    syncConfigText()
    showToast(`发现 ${result.models.length} 个模型，新增 ${added} 条映射`, 'success')
  } catch (e) {
    showToast(`发现失败：${e}`, 'error')
  } finally {
    busy.value = false
  }
}

function addRoutesFromModels(channel: ChannelConfig, models: string[]) {
  let added = 0
  for (const model of models) {
    const exists = config.value.model_routes.some(route => route.channel_id === channel.id && route.public_model === model && route.upstream_model === model)
    if (exists) continue
    config.value.model_routes.push({
      id: `${channel.id}-${model}`.replace(/[^a-zA-Z0-9_-]/g, '-').slice(0, 80),
      public_model: model,
      upstream_model: model,
      channel_id: channel.id,
      enabled: true,
      priority: config.value.model_routes.length + 1,
      prompt_cost_per_1k: 0,
      completion_cost_per_1k: 0
    })
    added += 1
  }
  return added
}

function nextId(prefix: string) {
  return `${prefix}-${Math.random().toString(36).slice(2, 8)}`
}

function addChannel() {
  const id = nextId('channel')
  config.value.channels.push({
    id,
    name: '新渠道',
    base_url: '',
    enabled: true,
    priority: config.value.channels.length + 1,
    key_strategy: 'round_robin',
    timeout_ms: 60000,
    retry_count: 0,
    keys: []
  })
}

function removeChannel(id: string) {
  config.value.channels = config.value.channels.filter(channel => channel.id !== id)
  config.value.model_routes = config.value.model_routes.filter(route => route.channel_id !== id)
}

function addKey(channel: ChannelConfig) {
  channel.keys.push({
    id: nextId('key'),
    name: 'API Key',
    api_key: '',
    enabled: true,
    weight: 1
  })
}

function removeKey(channel: ChannelConfig, id: string) {
  channel.keys = channel.keys.filter(key => key.id !== id)
}

function addRoute() {
  const channelId = config.value.channels[0]?.id || ''
  config.value.model_routes.push({
    id: nextId('route'),
    public_model: '',
    upstream_model: '',
    channel_id: channelId,
    enabled: true,
    priority: config.value.model_routes.length + 1,
    prompt_cost_per_1k: 0,
    completion_cost_per_1k: 0
  })
}

function removeRoute(id: string) {
  config.value.model_routes = config.value.model_routes.filter(route => route.id !== id)
}

function loadExample() {
  config.value = normalizeConfig({
    listen_host: '127.0.0.1',
    listen_port: 11434,
    max_concurrent_requests: 8,
    requests_per_minute: 120,
    channels: [
      {
        id: 'work-friday',
        name: 'Work Friday',
        base_url: 'https://work-friday.nichangen.com',
        enabled: true,
        priority: 1,
        key_strategy: 'round_robin',
        timeout_ms: 60000,
        retry_count: 1,
        keys: [
          {
            id: 'work-friday-key-1',
            name: '测试 key',
            api_key: 'sk-xxx',
            enabled: true,
            weight: 1
          }
        ]
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
        id: 'deepseek-v4-pro-flash-fallback',
        public_model: 'deepseek-v4-pro',
        upstream_model: 'deepseek-v4-flash',
        channel_id: 'work-friday',
        enabled: true,
        priority: 2,
        prompt_cost_per_1k: 0,
        completion_cost_per_1k: 0
      }
    ]
  })
  syncConfigText()
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
    await loadProfiles()
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
.log-header,
.section-title,
.card-head {
  display: flex;
  align-items: center;
  gap: 0.625rem;
}

.service-row,
.log-header,
.section-title,
.card-head {
  justify-content: space-between;
}

.button-row {
  flex-wrap: wrap;
}

.profile-box {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-elevated);
}

.profile-row,
.mini-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.profile-row select,
.profile-row input {
  min-width: 0;
  flex: 1;
  box-sizing: border-box;
  padding: 0.5rem 0.625rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-muted);
  color: var(--text-primary);
  font-size: 0.8125rem;
}

.btn.compact {
  min-height: 2rem;
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
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

.status-pill.running .status-dot { background: #10b981; }

.metric-grid,
.usage-strip {
  display: grid;
  gap: 0.5rem;
}

.metric-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
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

.config-form {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 0;
  border: 0;
  margin: 0;
  min-inline-size: 0;
}

.section-title { margin-top: 0.5rem; }
.section-title.nested { margin-top: 0.75rem; }

.form-grid {
  display: grid;
  gap: 0.625rem;
}

.form-grid.two { grid-template-columns: repeat(2, minmax(0, 1fr)); }
.form-grid.three { grid-template-columns: repeat(3, minmax(0, 1fr)); }
.form-grid.one-small { grid-template-columns: minmax(8rem, 12rem); }

.field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  color: var(--text-secondary);
  font-size: 0.75rem;
}

.field input,
.field select,
.key-row input,
.key-row select {
  width: 100%;
  min-width: 0;
  box-sizing: border-box;
  padding: 0.5rem 0.625rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 0.8125rem;
  outline: none;
}

.field input:focus,
.field select:focus,
.key-row input:focus { border-color: var(--brand-500); }

.config-card {
  padding: 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: color-mix(in srgb, var(--bg-elevated) 82%, transparent);
}

.config-card,
.route-card {
  display: flex;
  flex-direction: column;
  gap: 0.625rem;
}

.toggle {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  color: var(--text-secondary);
  font-size: 0.75rem;
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-secondary);
  cursor: pointer;
}

.icon-btn:hover { color: var(--text-primary); border-color: var(--brand-500); }

.key-row {
  display: grid;
  grid-template-columns: auto minmax(6rem, 0.8fr) minmax(6rem, 0.8fr) minmax(10rem, 1.4fr) 4.5rem auto;
  align-items: center;
  gap: 0.5rem;
}

.empty-row {
  padding: 0.75rem;
  border: 1px dashed var(--border-color);
  border-radius: 0.5rem;
  color: var(--text-secondary);
  font-size: 0.8125rem;
  text-align: center;
}

.empty-row.compact { padding: 0.5rem; }
.save-row { margin-top: 0.25rem; }

.advanced-json {
  margin-top: 0.5rem;
  color: var(--text-secondary);
  font-size: 0.8125rem;
}

.advanced-json summary {
  cursor: pointer;
  margin-bottom: 0.625rem;
}

.config-editor {
  width: 100%;
  min-height: 280px;
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

.config-editor:focus { border-color: var(--brand-500); }

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

.test-row.ok { border-color: color-mix(in srgb, #10b981 55%, var(--border-color)); }

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

.model-list { margin-top: 0.5rem; }

.model-list span {
  padding: 0.125rem 0.375rem;
  border-radius: 0.375rem;
  background: var(--bg-muted);
  color: var(--text-secondary);
  font-family: var(--font-family-mono, monospace);
  font-size: 0.6875rem;
}

.log-header { margin-bottom: 0.5rem; }

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

.log-row.clickable {
  cursor: pointer;
}

.log-row.clickable:hover {
  border-color: var(--brand-500);
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

.mono { font-family: var(--font-family-mono, monospace); }

.danger {
  color: #ef4444;
  border-color: color-mix(in srgb, #ef4444 45%, var(--border-color));
}

.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 60;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
  background: rgb(15 23 42 / 0.42);
}

.log-modal {
  width: min(860px, 100%);
  max-height: 86vh;
  overflow: auto;
  padding: 1rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-primary);
  box-shadow: 0 20px 60px rgb(0 0 0 / 0.22);
}

.detail-grid {
  display: grid;
  grid-template-columns: minmax(5rem, auto) minmax(0, 1fr);
  gap: 0.375rem 0.75rem;
  margin: 0.75rem 0;
  color: var(--text-secondary);
  font-size: 0.8125rem;
}

.detail-grid strong {
  min-width: 0;
  overflow-wrap: anywhere;
  color: var(--text-primary);
  font-weight: 500;
}

.detail-error { margin-bottom: 0.75rem; }

.log-modal pre {
  max-height: 220px;
  overflow: auto;
  padding: 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-family: var(--font-family-mono, monospace);
  font-size: 0.75rem;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

@media (max-width: 980px) {
  .usage-strip,
  .form-grid.two,
  .form-grid.three {
    grid-template-columns: 1fr;
  }

  .key-row {
    grid-template-columns: 1fr;
  }
}
</style>
