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

      <section class="tool-header http-header">
        <div class="tool-heading">
          <div class="heading-icon">
            <Send :size="22" />
          </div>
          <div>
            <h1>{{ tool?.name }}</h1>
            <p>{{ tool?.desc }}</p>
          </div>
        </div>
        <div class="header-meta">
          <span title="浏览器会拦截未开放 CORS 的接口响应">Browser fetch</span>
          <span>{{ requestSummary }}</span>
        </div>
      </section>

      <section class="request-command">
        <div class="urlbar-row">
          <div class="method-select" :style="methodStyle">
            <select v-model="method" aria-label="请求方法">
              <option v-for="m in methods" :key="m" :value="m">{{ m }}</option>
            </select>
            <ChevronDown :size="12" class="method-chevron" />
          </div>
          <input
            v-model.trim="url"
            class="url-input"
            type="text"
            placeholder="https://api.example.com/users"
            @keyup.enter="sendRequest"
          />
          <button class="btn primary send-btn" :disabled="loading || !url" @click="sendRequest">
            <LoaderCircle v-if="loading" :size="16" class="spin" />
            <Send v-else :size="15" />
            <span>{{ loading ? '发送中' : '发送' }}</span>
          </button>
        </div>
        <div class="command-footer">
          <span class="cors-badge" title="请求由当前浏览器直接发出；若接口未配置 CORS，浏览器会阻止读取响应">
            CORS 由目标接口决定
          </span>
          <button class="link-action" :disabled="!hasRequestDraft" @click="clearRequest">
            <Eraser :size="14" />
            清空请求
          </button>
        </div>
      </section>

      <div class="http-console">
        <aside class="history-panel">
          <div class="panel-head">
            <div>
              <span class="eyebrow">History</span>
              <strong>最近请求</strong>
            </div>
            <div style="display:flex;gap:0.25rem;">
              <button class="btn-icon" title="从 cURL 导入" @click="showCurlImport = true">
                <Terminal :size="14" />
              </button>
              <button class="btn-icon" :disabled="!history.length" title="清空历史" @click="clearHistory">
                <Trash2 :size="14" />
              </button>
            </div>
          </div>
          <div v-if="history.length" class="history-list">
            <article
              v-for="(item, idx) in history"
              :key="idx"
              class="history-item"
              @click="restoreHistory(item)"
            >
              <div class="history-item-main">
                <span class="method-pill" :style="methodColor(item.method)">{{ item.method }}</span>
                <span class="history-url">{{ item.url }}</span>
              </div>
              <div class="history-item-sub">
                <span>{{ historyTime(item) }}</span>
                <button class="history-delete" title="删除" @click.stop="deleteHistory(idx)">
                  <X :size="13" />
                </button>
              </div>
            </article>
          </div>
          <div v-else class="history-empty">
            <Clock3 :size="22" />
            <span>发送请求后会保留最近 5 条</span>
          </div>
        </aside>

        <section class="client-workspace">
          <div class="panel panel-request">
            <div class="panel-head request-head">
              <div>
                <span class="eyebrow">Request</span>
                <strong>请求配置</strong>
              </div>
              <div class="request-stats">
                <span>{{ paramsCount }} 参数</span>
                <span>{{ headersCount }} Header</span>
                <span>{{ authSummary }}</span>
              </div>
            </div>
            <div class="req-tabs">
              <button
                v-for="tab in reqTabs"
                :key="tab.key"
                :class="{ active: activeTab === tab.key }"
                @click="activeTab = tab.key"
              >
                {{ tab.label }}
                <span v-if="tab.badge && tab.badge()" class="tab-badge">{{ tab.badge!() }}</span>
              </button>
            </div>

            <div class="req-tab-body">
              <template v-if="activeTab === 'params'">
                <div class="section-toolbar">
                  <span>Query Params</span>
                  <small>启用的参数会自动拼到 URL 上</small>
                </div>
                <KeyValueEditor v-model="params" key-placeholder="参数名" value-placeholder="参数值" />
              </template>

              <template v-else-if="activeTab === 'auth'">
                <div class="auth-grid">
                  <label class="auth-card" :class="{ active: authType === 'none' }">
                    <input v-model="authType" type="radio" value="none" />
                    <ShieldOff :size="17" />
                    <span>无认证</span>
                  </label>
                  <label class="auth-card" :class="{ active: authType === 'bearer' }">
                    <input v-model="authType" type="radio" value="bearer" />
                    <KeyRound :size="17" />
                    <span>Bearer</span>
                  </label>
                  <label class="auth-card" :class="{ active: authType === 'basic' }">
                    <input v-model="authType" type="radio" value="basic" />
                    <LockKeyhole :size="17" />
                    <span>Basic</span>
                  </label>
                </div>
                <div v-if="authType === 'bearer'" class="auth-form compact-form">
                  <label class="field-label">Token</label>
                  <div class="inline-control">
                    <input v-model="bearerToken" type="text" placeholder="输入 Token" @keyup.enter="applyBearer" />
                    <button class="btn secondary" @click="applyBearer">应用</button>
                  </div>
                </div>
                <div v-else-if="authType === 'basic'" class="auth-form compact-form">
                  <div class="two-fields">
                    <label>
                      <span class="field-label">用户名</span>
                      <input v-model="basicUsername" type="text" placeholder="username" />
                    </label>
                    <label>
                      <span class="field-label">密码</span>
                      <input v-model="basicPassword" type="password" placeholder="password" @keyup.enter="applyBasicAuth" />
                    </label>
                  </div>
                  <button class="btn secondary auth-apply" @click="applyBasicAuth">应用到 Authorization</button>
                </div>
                <p v-else class="auth-none-hint">不发送 Authorization 请求头。切换认证方式后可以一键写入请求头。</p>
              </template>

              <template v-else-if="activeTab === 'headers'">
                <div class="section-toolbar">
                  <span>Headers</span>
                  <small>大小写按原样发送</small>
                </div>
                <KeyValueEditor v-model="headers" key-placeholder="Header 名称" value-placeholder="Header 值" />
              </template>

              <template v-else-if="activeTab === 'body'">
                <div class="body-bar">
                  <div class="segmented body-type">
                    <button :class="{ active: bodyType === 'json' }" @click="bodyType = 'json'">JSON</button>
                    <button :class="{ active: bodyType === 'text' }" @click="bodyType = 'text'">Text</button>
                    <button :class="{ active: bodyType === 'form-urlencoded' }" @click="bodyType = 'form-urlencoded'">Form</button>
                    <button :class="{ active: bodyType === 'form-data' }" @click="bodyType = 'form-data'">Multipart</button>
                  </div>
                  <span v-if="bodyDisabled" class="body-disabled-hint">GET / HEAD 请求不携带请求体</span>
                </div>

                <!-- JSON / Text / Form URL-encoded body -->
                <textarea
                  v-if="bodyType !== 'form-data'"
                  v-model="body"
                  :disabled="bodyDisabled"
                  class="body-textarea"
                  rows="12"
                  :placeholder="bodyType === 'form-urlencoded' ? 'key1=value1&key2=value2' : '{ &quot;name&quot;: &quot;Ada&quot; }'"
                ></textarea>

                <!-- Multipart form-data -->
                <div v-else class="form-data-editor">
                  <div class="kv-head form-data-head">
                    <span></span>
                    <span>Field</span>
                    <span>Value</span>
                    <span>Type</span>
                    <span></span>
                  </div>
                  <div v-for="(item, index) in formFields" :key="index" class="form-data-row">
                    <input v-model="item.enabled" type="checkbox" aria-label="启用此项" />
                    <input v-model="item.key" placeholder="字段名" @input="ensureBlankFormField" />
                    <template v-if="item.type === 'file'">
                      <label class="file-picker">
                        <input type="file" @change="(e) => { const t = e.target as HTMLInputElement; if (t.files?.length) { item.file = t.files[0]; item.value = t.files[0].name } }" />
                        <span class="file-name" :class="{ empty: !item.value }">{{ item.value || '选择文件…' }}</span>
                      </label>
                    </template>
                    <input v-else v-model="item.value" placeholder="值" @input="ensureBlankFormField" />
                    <select v-model="item.type" class="type-select">
                      <option value="text">Text</option>
                      <option value="file">File</option>
                    </select>
                    <button v-if="formFields.length > 1" class="remove" aria-label="删除" title="删除" @click="removeFormField(index)">×</button>
                  </div>
                </div>
              </template>

              <template v-else>
                <div class="code-toolbar">
                  <div class="segmented code-langs">
                    <button :class="{ active: codeLang === 'curl' }" @click="codeLang = 'curl'">cURL</button>
                    <button :class="{ active: codeLang === 'python' }" @click="codeLang = 'python'">Python</button>
                    <button :class="{ active: codeLang === 'js' }" @click="codeLang = 'js'">JavaScript</button>
                    <button :class="{ active: codeLang === 'go' }" @click="codeLang = 'go'">Go</button>
                  </div>
                  <button class="btn secondary" @click="copyCode">
                    <Check v-if="codeCopied" :size="14" />
                    <Copy v-else :size="14" />
                    {{ codeCopied ? '已复制' : '复制' }}
                  </button>
                </div>
                <pre class="code-block"><code>{{ generatedCode }}</code></pre>
              </template>
            </div>
          </div>

          <div class="panel panel-response">
            <div class="panel-head response-head">
              <div>
                <span class="eyebrow">Response</span>
                <strong>响应</strong>
              </div>
              <div class="response-actions">
                <span v-if="response" class="resp-meta">{{ response.duration }} ms · {{ response.size }}</span>
                <button class="btn-icon" :disabled="!response" title="复制响应" @click="copyResponse">
                  <Copy :size="14" />
                </button>
                <button class="btn-icon" :disabled="!response && !error" title="清空响应" @click="clearResponse">
                  <X :size="14" />
                </button>
              </div>
            </div>
            <template v-if="loading">
              <div class="resp-empty loading-state">
                <LoaderCircle :size="28" class="spin" />
                <span>正在等待目标接口响应</span>
              </div>
            </template>
            <template v-else-if="response">
              <div class="resp-status-bar">
                <span class="resp-status-badge" :class="{ ok: response.ok, err: !response.ok }">
                  {{ response.status }} {{ response.statusText || responseSummary }}
                </span>
                <span>{{ Object.keys(response.headers).length }} headers</span>
              </div>
              <!-- Waterfall -->
              <div v-if="waterfall.length" class="waterfall">
                <div v-for="w in waterfall" :key="w.label" class="waterfall-row">
                  <span class="waterfall-label">{{ w.label }}</span>
                  <span class="waterfall-bar-wrap">
                    <span class="waterfall-bar" :style="{ width: waterfallBarWidth(w.ms), backgroundColor: waterfallColor(w.label) }"></span>
                  </span>
                  <span class="waterfall-ms">{{ w.ms }} ms</span>
                </div>
              </div>
              <div class="resp-tabs">
                <button :class="{ active: respTab === 'body' }" @click="respTab = 'body'">Body</button>
                <button :class="{ active: respTab === 'headers' }" @click="respTab = 'headers'">Headers</button>
              </div>
              <pre v-if="respTab === 'body'" class="resp-body">{{ response.displayBody }}</pre>
              <div v-else class="resp-headers-list">
                <div v-for="(val, key) in response.headers" :key="key" class="resp-header-row">
                  <code>{{ key }}</code>
                  <span>{{ val }}</span>
                </div>
              </div>
            </template>
            <div v-else class="resp-empty" :class="{ error: error }">
              <AlertCircle v-if="error" :size="30" />
              <Wifi v-else :size="30" />
              <span>{{ error || '发送请求后在这里查看响应' }}</span>
            </div>
          </div>
        </section>
      </div>
    </main>
    <Transition name="toast">
      <div v-if="toastMessage" class="toast" :class="toastType">{{ toastMessage }}</div>
    </Transition>

    <!-- cURL Import Modal -->
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="showCurlImport" class="modal-backdrop" @click.self="showCurlImport = false">
          <div class="modal-content curl-modal">
            <div class="modal-head">
              <strong>从 cURL 导入</strong>
              <button class="btn-icon" @click="showCurlImport = false"><X :size="16" /></button>
            </div>
            <textarea
              v-model="curlImportText"
              class="curl-textarea"
              rows="8"
              placeholder="粘贴 cURL 命令…&#10;例如: curl -X POST 'https://api.example.com' -H 'Content-Type: application/json' -d '{&quot;key&quot;:&quot;value&quot;}'"
            ></textarea>
            <div class="modal-actions">
              <button class="btn secondary" @click="showCurlImport = false">取消</button>
              <button class="btn primary" @click="importCurl" :disabled="!curlImportText.trim()">导入</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import {
  AlertCircle,
  ArrowLeft,
  Check,
  ChevronDown,
  Clock3,
  Copy,
  Eraser,
  KeyRound,
  LoaderCircle,
  LockKeyhole,
  Send,
  ShieldOff,
  Terminal,
  Trash2,
  Wifi,
  X
} from 'lucide-vue-next'
import { useTheme } from '@/composables/useTheme'
import KeyValueEditor from '@/components/tools/KeyValueEditor.vue'
import { findTool } from '@/utils/toolPipeline'
import {
  generateCurlCommand,
  generateJavaScriptCode,
  generatePythonCode,
  generateGoCode,
  parseCurlCommand,
  type CodeGenFormField
} from '@/utils/httpClient'

type KeyValue = { enabled: boolean; key: string; value: string }
type FormField = { enabled: boolean; key: string; value: string; type: 'text' | 'file'; file: File | null }
type HttpResponse = { ok: boolean; status: number; statusText: string; duration: number; size: string; headers: Record<string, string>; displayBody: string }
interface WaterfallEntry { label: string; ms: number }
interface HistoryItem {
  method: string
  url: string
  params: KeyValue[]
  headers: KeyValue[]
  body: string
  bodyType: string
  formFields: FormField[]
  createdAt?: number
}

const { isDark } = useTheme()
const tool = findTool('http-client')
const methods = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS']

// --- Request state ---
const method = ref('GET')
const url = ref('')
const params = ref<KeyValue[]>([{ enabled: true, key: '', value: '' }])
const headers = ref<KeyValue[]>([{ enabled: true, key: '', value: '' }])
const body = ref('')
const bodyType = ref<'json' | 'text' | 'form-urlencoded' | 'form-data'>('json')
const formFields = ref<FormField[]>([{ enabled: true, key: '', value: '', type: 'text', file: null }])
const bodyDisabled = computed(() => ['GET', 'HEAD'].includes(method.value))
const loading = ref(false)
const error = ref('')
const response = ref<HttpResponse | null>(null)
const waterfall = ref<WaterfallEntry[]>([])
const curlImportText = ref('')
const showCurlImport = ref(false)
const paramsCount = computed(() => params.value.filter(i => i.enabled && i.key).length)
const headersCount = computed(() => headers.value.filter(i => i.enabled && i.key).length)
const hasRequestDraft = computed(() => Boolean(
  url.value ||
  body.value ||
  params.value.some(i => i.key || i.value) ||
  headers.value.some(i => i.key || i.value) ||
  bearerToken.value ||
  basicUsername.value ||
  basicPassword.value
))
const authSummary = computed(() => {
  if (authType.value === 'bearer') return 'Bearer'
  if (authType.value === 'basic') return 'Basic'
  return 'No Auth'
})
const requestSummary = computed(() => {
  const pieces = [`${method.value}`, `${paramsCount.value} params`, `${headersCount.value} headers`]
  if (!bodyDisabled.value) {
    if (bodyType.value === 'form-data') {
      const count = formFields.value.filter(f => f.enabled && f.key).length
      if (count) pieces.push(`${count} form fields`)
    } else if (body.value) {
      pieces.push(`${formatSize(new Blob([body.value]).size)} body`)
    }
  }
  return pieces.join(' · ')
})
const responseSummary = computed(() => response.value?.ok ? 'OK' : 'Error')

// --- Tabs ---
const activeTab = ref<'params' | 'auth' | 'headers' | 'body' | 'code'>('params')
const respTab = ref<'body' | 'headers'>('body')
const reqTabs = [
  { key: 'params' as const, label: '参数', badge: () => params.value.filter(i => i.enabled && i.key).length || null },
  { key: 'auth' as const, label: '认证' },
  { key: 'headers' as const, label: '请求头', badge: () => headers.value.filter(i => i.enabled && i.key).length || null },
  { key: 'body' as const, label: '请求体' },
  { key: 'code' as const, label: '代码' },
]

// --- Method color ---
const methodColors: Record<string, string> = {
  GET: '#10b981',
  POST: '#3b82f6',
  PUT: '#f59e0b',
  PATCH: '#8b5cf6',
  DELETE: '#ef4444',
  HEAD: '#6b7280',
  OPTIONS: '#6b7280',
}
const methodStyle = computed(() => ({
  '--method-color': methodColors[method.value] || '#6b7280',
}))
function methodColor(m: string) {
  return { color: methodColors[m] || '#6b7280' }
}

// --- Toast ---
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')
let toastTimer: ReturnType<typeof setTimeout> | null = null
function showToast(m: string, t: 'success' | 'error') {
  if (toastTimer) clearTimeout(toastTimer)
  toastMessage.value = m
  toastType.value = t
  toastTimer = setTimeout(() => { toastMessage.value = '' }, 2200)
}

// --- History ---
const HISTORY_KEY = 'http-client-history'
const history = ref<HistoryItem[]>([])
function loadHistory() {
  try { const raw = localStorage.getItem(HISTORY_KEY); if (raw) history.value = JSON.parse(raw) } catch { /* ignore */ }
}
function saveHistory() {
  try { localStorage.setItem(HISTORY_KEY, JSON.stringify(history.value)) } catch { /* ignore */ }
}
function pushHistory() {
  if (!url.value) return
  const item: HistoryItem = {
    method: method.value, url: url.value,
    params: JSON.parse(JSON.stringify(params.value)),
    headers: JSON.parse(JSON.stringify(headers.value)),
    body: body.value, bodyType: bodyType.value,
    formFields: JSON.parse(JSON.stringify(formFields.value.map(f => ({ enabled: f.enabled, key: f.key, value: f.value, type: f.type, file: null })))),
    createdAt: Date.now(),
  }
  const idx = history.value.findIndex(h => h.method === item.method && h.url === item.url)
  if (idx >= 0) history.value.splice(idx, 1)
  history.value.unshift(item)
  if (history.value.length > 5) history.value.pop()
  saveHistory()
}
function restoreHistory(item: HistoryItem) {
  method.value = item.method
  url.value = item.url
  params.value = JSON.parse(JSON.stringify(item.params))
  headers.value = JSON.parse(JSON.stringify(item.headers))
  body.value = item.body
  bodyType.value = item.bodyType as 'json' | 'text' | 'form-urlencoded' | 'form-data'
  if (item.formFields?.length) {
    formFields.value = item.formFields.map(f => ({ enabled: f.enabled, key: f.key, value: f.value, type: f.type, file: null }))
  }
}
function deleteHistory(idx: number) { history.value.splice(idx, 1); saveHistory() }
function clearHistory() { history.value = []; saveHistory() }
function historyTime(item: HistoryItem) {
  if (!item.createdAt) return '刚刚'
  const diff = Date.now() - item.createdAt
  if (diff < 60_000) return '刚刚'
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)} 分钟前`
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)} 小时前`
  return new Date(item.createdAt).toLocaleDateString()
}

// --- Auth ---
const authType = ref<'none' | 'bearer' | 'basic'>('none')
const bearerToken = ref('')
const basicUsername = ref('')
const basicPassword = ref('')

function setAuthHeader(headerValue: string) {
  const existing = headers.value.find(h => h.key.toLowerCase() === 'authorization' && h.enabled)
  if (existing) { existing.value = headerValue; return }
  const empty = headers.value.find(h => !h.key)
  if (empty) { empty.key = 'Authorization'; empty.value = headerValue; empty.enabled = true }
  else { headers.value.splice(headers.value.length - 1, 0, { enabled: true, key: 'Authorization', value: headerValue }) }
}
function applyBearer() {
  if (!bearerToken.value.trim()) { showToast('请输入 Token', 'error'); return }
  setAuthHeader(`Bearer ${bearerToken.value.trim()}`)
  showToast('Bearer Token 已填入请求头', 'success')
}
function applyBasicAuth() {
  if (!basicUsername.value.trim() || !basicPassword.value.trim()) { showToast('请输入用户名和密码', 'error'); return }
  setAuthHeader(`Basic ${btoa(`${basicUsername.value.trim()}:${basicPassword.value.trim()}`)}`)
  showToast('Basic Auth 已填入请求头', 'success')
}

// --- Body type helpers ---
function ensureBlankFormField() {
  const last = formFields.value[formFields.value.length - 1]
  if (last && (last.key || last.value)) formFields.value.push({ enabled: true, key: '', value: '', type: 'text', file: null })
}
function removeFormField(index: number) {
  if (formFields.value.length <= 1) return
  formFields.value.splice(index, 1)
}

// --- cURL import ---
function importCurl() {
  const text = curlImportText.value.trim()
  if (!text) { showToast('请粘贴 cURL 命令', 'error'); return }
  const parsed = parseCurlCommand(text)
  if (!parsed) { showToast('无法解析 cURL 命令，请检查格式', 'error'); return }
  method.value = parsed.method
  url.value = parsed.url
  if (parsed.headers.length) {
    headers.value = parsed.headers.map(h => ({ enabled: true, key: h.key, value: h.value }))
    headers.value.push({ enabled: true, key: '', value: '' })
  }
  if (parsed.body) {
    body.value = parsed.body
    bodyType.value = parsed.bodyType
    activeTab.value = 'body'
  }
  if (parsed.bodyType === 'form-data' && parsed.formFields?.length) {
    body.value = ''
    bodyType.value = 'form-data'
    formFields.value = parsed.formFields.map(f => ({
      enabled: true,
      key: f.key,
      value: f.value,
      type: f.type,
      file: null,
    }))
    formFields.value.push({ enabled: true, key: '', value: '', type: 'text', file: null })
    activeTab.value = 'body'
  }
  // Detect auth
  const authHeader = parsed.headers.find(h => h.key.toLowerCase() === 'authorization')
  if (authHeader) {
    const val = authHeader.value
    if (val.startsWith('Bearer ')) {
      authType.value = 'bearer'
      bearerToken.value = val.slice(7)
    } else if (val.startsWith('Basic ')) {
      authType.value = 'basic'
      try {
        const decoded = atob(val.slice(6))
        const colonIdx = decoded.indexOf(':')
        if (colonIdx >= 0) {
          basicUsername.value = decoded.slice(0, colonIdx)
          basicPassword.value = decoded.slice(colonIdx + 1)
        }
      } catch { /* ignore */ }
    }
  }
  curlImportText.value = ''
  showCurlImport.value = false
  showToast('cURL 已导入', 'success')
}

// --- Code gen ---
const codeLang = ref<'curl' | 'python' | 'js' | 'go'>('curl')
const codeCopied = ref(false)
function safeBuildUrl(): string { try { return buildUrl() } catch { return '' } }
const generatedCode = computed(() => {
  const target = safeBuildUrl()
  if (!target) return '# 请先输入有效的 URL'
  const activeFormFields = formFields.value.filter(f => f.enabled && f.key)
  const codeGenFormFields: CodeGenFormField[] = activeFormFields.map(f => ({ key: f.key, value: f.value, type: f.type }))
  const p = {
    method: method.value, url: target,
    headers: headers.value.filter(h => h.enabled && h.key),
    body: body.value, bodyType: bodyType.value,
    formFields: codeGenFormFields.length ? codeGenFormFields : undefined,
  }
  switch (codeLang.value) {
    case 'curl': return generateCurlCommand(p)
    case 'python': return generatePythonCode(p)
    case 'js': return generateJavaScriptCode(p)
    case 'go': return generateGoCode(p)
  }
})
async function copyCode() {
  try { await navigator.clipboard.writeText(generatedCode.value); codeCopied.value = true; showToast('代码已复制', 'success'); setTimeout(() => { codeCopied.value = false }, 2000) }
  catch { showToast('复制失败', 'error') }
}

// --- Request ---
function formatSize(size: number) { return size < 1024 ? `${size} B` : `${(size / 1024).toFixed(1)} KB` }
function buildUrl() {
  const target = new URL(url.value)
  params.value.filter(i => i.enabled && i.key).forEach(i => target.searchParams.set(i.key, i.value))
  return target.toString()
}
async function sendRequest() {
  error.value = ''; response.value = null; waterfall.value = []
  let target: string
  try { target = buildUrl() } catch { error.value = '请输入有效的完整 URL（例如 https://api.example.com）'; return }
  const reqHeaders: Record<string, string> = {}
  headers.value.filter(i => i.enabled && i.key).forEach(i => { reqHeaders[i.key] = i.value })
  const init: RequestInit = { method: method.value, headers: reqHeaders }

  // Attach body
  if (!bodyDisabled.value) {
    if (bodyType.value === 'form-data') {
      const activeFields = formFields.value.filter(f => f.enabled && f.key)
      if (activeFields.length) {
        const fd = new FormData()
        for (const f of activeFields) {
          if (f.type === 'file' && f.file) {
            fd.append(f.key, f.file, f.file.name)
          } else {
            fd.append(f.key, f.value)
          }
        }
        init.body = fd
        // Let fetch set Content-Type with boundary, but remove manual Content-Type if present
        for (const key of Object.keys(reqHeaders)) {
          if (key.toLowerCase() === 'content-type') delete reqHeaders[key]
        }
        init.headers = reqHeaders
      }
    } else if (bodyType.value === 'form-urlencoded' && body.value) {
      init.body = body.value
      if (!Object.keys(reqHeaders).some(k => k.toLowerCase() === 'content-type'))
        init.headers = { ...reqHeaders, 'Content-Type': 'application/x-www-form-urlencoded' }
    } else if (body.value) {
      init.body = body.value
      if (bodyType.value === 'json' && !Object.keys(reqHeaders).some(k => k.toLowerCase() === 'content-type'))
        init.headers = { ...reqHeaders, 'Content-Type': 'application/json' }
    }
  }

  loading.value = true
  const t0 = performance.now()
  try {
    const res = await fetch(target, init)
    const text = await res.text()
    const duration = Math.round(performance.now() - t0)
    let displayBody = text
    try { displayBody = JSON.stringify(JSON.parse(text), null, 2) } catch { /* raw */ }
    response.value = {
      ok: res.ok, status: res.status, statusText: res.statusText,
      duration,
      size: formatSize(new Blob([text]).size),
      headers: Object.fromEntries(res.headers.entries()),
      displayBody,
    }
    pushHistory()

    // Waterfall: resolve timing from Resource Timing API
    try {
      const entries = performance.getEntriesByType('resource') as PerformanceResourceTiming[]
      const own = entries[entries.length - 1]
      if (own && own.name === target) {
        // Values are relative to navigation start or redirect start
        const start = own.redirectEnd || own.startTime
        const dns = Math.max(0, own.domainLookupEnd - own.domainLookupStart)
        const tcp = Math.max(0, own.connectEnd - own.connectStart)
        const tls = Math.max(0, own.secureConnectionStart > 0 ? own.connectEnd - own.secureConnectionStart : 0)
        const ttfb = Math.max(0, own.responseStart - own.requestStart)
        const dl = Math.max(0, own.responseEnd - own.responseStart)
        if (dns + tcp + tls + ttfb + dl > 0) {
          waterfall.value = [
            { label: 'DNS', ms: Math.round(dns) },
            { label: 'TCP', ms: Math.round(tcp) },
            { label: 'TLS', ms: Math.round(tls) },
            { label: 'TTFB', ms: Math.round(ttfb) },
            { label: 'Download', ms: Math.round(dl) },
          ].filter(e => e.ms > 0)
        }
      }
    } catch { /* waterfall not available */ }
  } catch (cause: any) {
    error.value = cause?.message
      ? `请求失败：${cause.message}。请检查网络、URL 和接口 CORS 配置。`
      : '请求失败：请检查网络、URL 和接口 CORS 配置。'
  } finally { loading.value = false }
}
async function copyResponse() {
  if (!response.value) return
  await navigator.clipboard.writeText(response.value.displayBody)
  showToast('响应已复制', 'success')
}
function clearResponse() {
  response.value = null
  error.value = ''
  waterfall.value = []
}
function clearRequest() {
  method.value = 'GET'
  url.value = ''
  params.value = [{ enabled: true, key: '', value: '' }]
  headers.value = [{ enabled: true, key: '', value: '' }]
  body.value = ''
  bodyType.value = 'json'
  formFields.value = [{ enabled: true, key: '', value: '', type: 'text', file: null }]
  authType.value = 'none'
  bearerToken.value = ''
  basicUsername.value = ''
  basicPassword.value = ''
  response.value = null
  error.value = ''
  waterfall.value = []
  activeTab.value = 'params'
  respTab.value = 'body'
}

onMounted(() => { loadHistory() })
onUnmounted(() => { if (toastTimer) clearTimeout(toastTimer) })

// Waterfall helpers
const waterfallMaxMs = computed(() => {
  if (!waterfall.value.length) return 100
  return Math.max(...waterfall.value.map(w => w.ms), 1) * 1.3
})
function waterfallBarWidth(ms: number) {
  return `${(ms / waterfallMaxMs.value) * 100}%`
}
function waterfallColor(label: string) {
  const m: Record<string, string> = {
    DNS: '#8b5cf6',
    TCP: '#f59e0b',
    TLS: '#ef4444',
    TTFB: '#3b82f6',
    Download: '#10b981',
  }
  return m[label] || '#6b7280'
}
</script>

<style scoped>
/* ── heading ── */
.heading-icon { --tool-color: #f97316; }

/* ── URL bar ── */
.urlbar {
  margin-bottom: 0.625rem;
}
.urlbar-row {
  display: flex;
  align-items: stretch;
  gap: 0;
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  overflow: hidden;
  background: var(--bg-surface);
  transition: border-color 0.15s;
}
.urlbar-row:focus-within {
  border-color: var(--brand-500);
}
.method-select {
  position: relative;
  display: flex;
  align-items: center;
  flex-shrink: 0;
  background: color-mix(in srgb, var(--method-color) 12%, transparent);
  border-right: 1px solid var(--border-color);
}
.method-select select {
  appearance: none;
  height: 2.75rem;
  padding: 0 1.75rem 0 0.875rem;
  border: 0;
  background: transparent;
  color: var(--method-color);
  font-weight: 800;
  font-size: 0.8125rem;
  cursor: pointer;
  outline: none;
}
.method-chevron {
  position: absolute;
  right: 0.375rem;
  pointer-events: none;
  color: var(--method-color);
}
.url-input {
  flex: 1;
  min-width: 0;
  height: 2.75rem;
  padding: 0 0.75rem;
  border: 0;
  background: transparent;
  color: var(--text-primary);
  font: 0.875rem var(--font-family-mono, monospace);
  outline: none;
}
.send-btn {
  flex-shrink: 0;
  height: 2.75rem;
  padding: 0 1.25rem;
  border-radius: 0;
  font-weight: 700;
  font-size: 0.8125rem;
}
.urlbar-hint {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  margin-top: 0.3125rem;
}
.cors-badge {
  font-size: 0.6875rem;
  color: var(--text-muted);
  cursor: help;
  border-bottom: 1px dotted var(--text-muted);
}

/* ── History ── */
.history-strip {
  display: flex;
  align-items: flex-start;
  gap: 0.375rem;
  margin-bottom: 0.75rem;
}
.history-scroll {
  flex: 1;
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  min-width: 0;
}
.history-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--border-color);
  border-radius: 999px;
  background: var(--bg-surface);
  cursor: pointer;
  transition: border-color 0.12s, background 0.12s;
  max-width: 260px;
}
.history-chip:hover { border-color: var(--brand-500); background: var(--bg-elevated); }
.chip-method {
  font-weight: 800;
  font-size: 0.625rem;
  text-transform: uppercase;
  flex-shrink: 0;
}
.chip-url {
  font-size: 0.71875rem;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--font-family-mono, monospace);
}
.chip-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border: 0;
  border-radius: 50%;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  flex-shrink: 0;
  transition: background 0.12s, color 0.12s;
}
.chip-close:hover { background: #fee2e2; color: #ef4444; }
.history-clear {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.625rem;
  height: 1.625rem;
  border: 0;
  border-radius: 0.375rem;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  flex-shrink: 0;
  margin-top: 0.0625rem;
}
.history-clear:hover { background: #fee2e2; color: #ef4444; }

/* ── Workspace ── */
.http-workspace {
  grid-template-columns: 1fr;
  gap: 0.75rem;
}
@media (min-width: 900px) {
  .http-workspace {
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  }
}

/* ── Request panel ── */
.panel-request, .panel-response {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color);
  border-radius: 0.75rem;
  background: var(--bg-surface);
  overflow: hidden;
}
.req-tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--border-color);
  padding: 0 0.5rem;
  overflow-x: auto;
}
.req-tabs button {
  position: relative;
  padding: 0.625rem 0.75rem;
  border: 0;
  background: transparent;
  color: var(--text-secondary);
  font-size: 0.78125rem;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: color 0.12s;
}
.req-tabs button.active {
  color: var(--brand-500);
}
.req-tabs button.active::after {
  content: '';
  position: absolute;
  bottom: -1px;
  left: 0;
  right: 0;
  height: 2px;
  background: var(--brand-500);
  border-radius: 2px 2px 0 0;
}
.tab-badge {
  font-size: 0.625rem;
  font-weight: 700;
  margin-left: 0.1875rem;
  opacity: 0.7;
}
.req-tab-body {
  flex: 1;
  padding: 0.75rem;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* ── Auth tab ── */
.auth-type-select {
  margin-bottom: 0.75rem;
}
.auth-type-select select {
  height: 2.25rem;
  padding: 0 0.625rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 0.8125rem;
  font-weight: 600;
  outline: none;
}
.auth-type-select select:focus { border-color: var(--brand-500); }
.field-label {
  display: block;
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--text-secondary);
  margin-bottom: 0.25rem;
  margin-top: 0.5rem;
}
.field-label:first-child { margin-top: 0; }
.auth-form input {
  width: 100%;
  height: 2.25rem;
  padding: 0 0.625rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font: 0.8125rem var(--font-family-mono, monospace);
  outline: none;
  box-sizing: border-box;
}
.auth-form input:focus { border-color: var(--brand-500); }
.auth-apply {
  margin-top: 0.625rem;
}
.auth-none-hint {
  color: var(--text-muted);
  font-size: 0.8125rem;
}

/* ── Body tab ── */
.body-bar {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  margin-bottom: 0.625rem;
}
.body-bar select {
  height: 2rem;
  padding: 0 0.5rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 0.75rem;
  font-weight: 600;
  outline: none;
}
.body-disabled-hint {
  font-size: 0.75rem;
  color: var(--text-muted);
}
.body-textarea {
  flex: 1;
  min-height: 14rem;
  padding: 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font: 0.8125rem/1.55 var(--font-family-mono, monospace);
  outline: none;
  resize: vertical;
  box-sizing: border-box;
}
.body-textarea:focus { border-color: var(--brand-500); }
.body-textarea:disabled { opacity: 0.4; cursor: not-allowed; }

/* ── Code tab ── */
.code-langs {
  margin-bottom: 0.625rem;
}
.code-block {
  flex: 1;
  min-height: 12rem;
  max-height: 32rem;
  margin: 0;
  padding: 0.75rem;
  border-radius: 0.5rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font: 0.75rem/1.55 var(--font-family-mono, monospace);
  white-space: pre-wrap;
  word-break: break-word;
  overflow: auto;
}
.code-copy-btn {
  align-self: flex-start;
  margin-top: 0.625rem;
}

/* ── Response panel ── */
.resp-status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.625rem 0.75rem;
  border-bottom: 1px solid var(--border-color);
  gap: 0.5rem;
}
.resp-status-left {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  min-width: 0;
}
.resp-status-badge {
  padding: 0.1875rem 0.5rem;
  border-radius: 0.25rem;
  font-weight: 700;
  font-size: 0.75rem;
  font-family: var(--font-family-mono, monospace);
  white-space: nowrap;
}
.resp-status-badge.ok { color: #047857; background: #d1fae5; }
.resp-status-badge.err { color: #b91c1c; background: #fee2e2; }
.resp-meta {
  font-size: 0.75rem;
  color: var(--text-secondary);
  font-family: var(--font-family-mono, monospace);
}
.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border: 0;
  border-radius: 0.375rem;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  flex-shrink: 0;
}
.btn-icon:hover { background: var(--bg-elevated); color: var(--text-primary); }

.resp-tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--border-color);
  padding: 0 0.5rem;
}
.resp-tabs button {
  position: relative;
  padding: 0.5625rem 0.75rem;
  border: 0;
  background: transparent;
  color: var(--text-secondary);
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
}
.resp-tabs button.active { color: var(--brand-500); }
.resp-tabs button.active::after {
  content: '';
  position: absolute;
  bottom: -1px;
  left: 0;
  right: 0;
  height: 2px;
  background: var(--brand-500);
  border-radius: 2px 2px 0 0;
}

.resp-body {
  flex: 1;
  min-height: 18rem;
  max-height: 40rem;
  margin: 0;
  padding: 0.875rem;
  overflow: auto;
  color: var(--text-primary);
  font: 0.8125rem/1.55 var(--font-family-mono, monospace);
  white-space: pre-wrap;
  word-break: break-word;
}
.resp-headers-list {
  flex: 1;
  min-height: 18rem;
  max-height: 40rem;
  overflow: auto;
  padding: 0.5rem 0.75rem;
}
.resp-header-row {
  display: flex;
  gap: 0.75rem;
  padding: 0.3125rem 0;
  border-bottom: 1px solid var(--border-color);
  font-size: 0.75rem;
  word-break: break-word;
}
.resp-header-row code {
  color: var(--brand-500);
  flex-shrink: 0;
  font-family: var(--font-family-mono, monospace);
}
.resp-header-row span {
  color: var(--text-primary);
  font-family: var(--font-family-mono, monospace);
}

.resp-empty {
  flex: 1;
  min-height: 20rem;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 0.5rem;
  color: var(--text-muted);
  font-size: 0.875rem;
  text-align: center;
  padding: 2rem;
}
.resp-error-detail {
  font-size: 0.75rem;
  color: #ef4444;
  max-width: 100%;
  word-break: break-word;
}

/* ── Animations ── */
.spin { animation: spin .8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

/* ── Responsive: 小屏时 URL bar 换行 ── */
@media (max-width: 600px) {
  .urlbar-row {
    flex-wrap: wrap;
    border-radius: 0.5rem;
  }
  .method-select {
    width: 100%;
    border-right: 0;
    border-bottom: 1px solid var(--border-color);
    border-radius: 0;
  }
  .url-input {
    height: 2.5rem;
    font-size: 0.8125rem;
  }
  .send-btn {
    height: 2.5rem;
    flex: 1;
    min-width: 0;
  }
}

/* ── Console refresh ── */
.tool-main {
  max-width: 1320px;
}

.http-header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 1rem;
}

.header-meta {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 0.375rem;
  color: var(--text-muted);
  font: 0.75rem var(--font-family-mono, monospace);
}

.header-meta span {
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--border-color);
  border-radius: 999px;
  background: var(--bg-surface);
}

.request-command {
  margin-bottom: 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-surface);
  box-shadow: var(--shadow-1);
}

.request-command .urlbar-row {
  border: 0;
  border-radius: 0.5rem 0.5rem 0 0;
  background: transparent;
}

.request-command .urlbar-row:focus-within {
  box-shadow: inset 0 0 0 1px var(--brand-500);
}

.request-command .method-select {
  min-width: 7.25rem;
  background: color-mix(in srgb, var(--method-color) 10%, transparent);
}

.request-command .method-select select {
  width: 100%;
  height: 3rem;
  font-size: 0.875rem;
}

.request-command .url-input {
  height: 3rem;
  font-size: 0.875rem;
}

.request-command .send-btn {
  height: 3rem;
  min-width: 7rem;
  gap: 0.375rem;
}

.command-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  min-height: 2rem;
  padding: 0 0.625rem;
  border-top: 1px solid var(--border-color);
  background: color-mix(in srgb, var(--bg-elevated) 60%, transparent);
}

.link-action {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  border: 0;
  background: transparent;
  color: var(--text-secondary);
  font-size: 0.75rem;
  font-weight: 700;
  cursor: pointer;
}

.link-action:hover:not(:disabled) {
  color: var(--brand-500);
}

.link-action:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.http-console {
  display: grid;
  grid-template-columns: 1fr;
  gap: 0.75rem;
  align-items: stretch;
}

.client-workspace {
  display: grid;
  grid-template-columns: 1fr;
  gap: 0.75rem;
  min-width: 0;
}

@media (min-width: 1100px) {
  .http-console {
    grid-template-columns: 260px minmax(0, 1fr);
  }

  .client-workspace {
    grid-template-columns: minmax(0, 1.02fr) minmax(420px, 0.98fr);
  }
}

.history-panel,
.panel-request,
.panel-response {
  display: flex;
  min-width: 0;
  min-height: 30rem;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-surface);
  box-shadow: var(--shadow-1);
}

.panel-request,
.panel-response {
  padding: 0;
  gap: 0;
}

.panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  min-height: 3.25rem;
  padding: 0.625rem 0.75rem;
  border-bottom: 1px solid var(--border-color);
}

.panel-head > div:first-child {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 0.125rem;
}

.panel-head strong {
  color: var(--text-primary);
  font-size: 0.875rem;
}

.eyebrow {
  color: var(--text-muted);
  font-size: 0.625rem;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.request-stats,
.response-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
  color: var(--text-muted);
  font-size: 0.75rem;
}

.request-stats span {
  white-space: nowrap;
}

.history-list {
  display: flex;
  flex: 1;
  min-height: 0;
  flex-direction: column;
  overflow: auto;
  padding: 0.5rem;
  gap: 0.375rem;
}

.history-item {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  padding: 0.625rem;
  border: 1px solid transparent;
  border-radius: 0.5rem;
  background: var(--bg-elevated);
  cursor: pointer;
  transition: border-color 0.15s, background 0.15s;
}

.history-item:hover {
  border-color: color-mix(in srgb, var(--brand-500) 42%, var(--border-color));
  background: var(--bg-surface);
}

.history-item-main {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}

.method-pill {
  flex: 0 0 auto;
  min-width: 3.35rem;
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  background: color-mix(in srgb, currentColor 12%, transparent);
  font-size: 0.6875rem;
  font-weight: 900;
  text-align: center;
}

.history-url {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font: 0.75rem var(--font-family-mono, monospace);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.history-item-sub {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  color: var(--text-muted);
  font-size: 0.6875rem;
}

.history-delete {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.375rem;
  height: 1.375rem;
  border: 0;
  border-radius: 0.25rem;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
}

.history-delete:hover {
  background: #fee2e2;
  color: #dc2626;
}

.history-empty {
  display: flex;
  flex: 1;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  gap: 0.5rem;
  padding: 1rem;
  color: var(--text-muted);
  text-align: center;
  font-size: 0.8125rem;
}

.section-toolbar {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 0.625rem;
}

.section-toolbar span {
  font-size: 0.8125rem;
  font-weight: 800;
}

.section-toolbar small {
  color: var(--text-muted);
  font-size: 0.75rem;
}

.req-tabs {
  min-height: 2.75rem;
  padding: 0 0.625rem;
  background: color-mix(in srgb, var(--bg-elevated) 55%, transparent);
}

.req-tabs button {
  min-height: 2.75rem;
}

.req-tab-body {
  padding: 0.875rem;
}

.auth-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.5rem;
  margin-bottom: 0.875rem;
}

.auth-card {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
  min-height: 3rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-elevated);
  color: var(--text-secondary);
  font-size: 0.8125rem;
  font-weight: 800;
  cursor: pointer;
}

.auth-card input {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}

.auth-card.active {
  border-color: color-mix(in srgb, var(--brand-500) 62%, var(--border-color));
  background: color-mix(in srgb, var(--brand-500) 10%, var(--bg-surface));
  color: var(--brand-500);
}

.compact-form {
  max-width: 100%;
}

.inline-control {
  display: flex;
  gap: 0.5rem;
}

.inline-control input {
  flex: 1;
  min-width: 0;
}

.two-fields {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.625rem;
}

.body-bar,
.code-toolbar {
  justify-content: space-between;
}

.body-type {
  width: 17rem;
}

.code-toolbar {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.625rem;
}

.code-langs {
  width: min(100%, 22rem);
  margin: 0;
}

.code-block,
.body-textarea,
.resp-body {
  border-radius: 0.375rem;
  background: color-mix(in srgb, var(--bg-elevated) 72%, transparent);
}

.body-textarea {
  min-height: 20rem;
}

.code-block {
  min-height: 22rem;
}

.panel-response {
  background: color-mix(in srgb, var(--bg-surface) 92%, var(--bg-elevated));
}

.resp-status-bar {
  min-height: 2.5rem;
  padding: 0.5rem 0.75rem;
  background: color-mix(in srgb, var(--bg-elevated) 55%, transparent);
  color: var(--text-muted);
  font-size: 0.75rem;
}

.resp-status-badge {
  border-radius: 0.375rem;
}

.resp-tabs {
  min-height: 2.5rem;
  background: var(--bg-surface);
}

.resp-tabs button {
  min-height: 2.5rem;
}

.resp-body,
.resp-headers-list {
  min-height: 24rem;
  max-height: 48rem;
}

.resp-empty {
  min-height: 26rem;
  color: var(--text-muted);
}

.resp-empty.error {
  color: #ef4444;
}

.loading-state {
  color: var(--brand-500);
}

.btn-icon:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

@media (max-width: 720px) {
  .http-header {
    align-items: flex-start;
    flex-direction: column;
  }

  .header-meta {
    justify-content: flex-start;
  }

  .request-command .method-select {
    width: 100%;
  }

  .command-footer,
  .request-stats {
    align-items: flex-start;
    flex-direction: column;
    padding-top: 0.5rem;
    padding-bottom: 0.5rem;
  }

  .auth-grid,
  .two-fields {
    grid-template-columns: 1fr;
  }

  .inline-control,
  .code-toolbar {
    align-items: stretch;
    flex-direction: column;
  }

  .body-type,
  .code-langs {
    width: 100%;
  }
}

/* ── Form data editor ── */
.form-data-editor {
  overflow: hidden;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-surface);
}

.form-data-head {
  min-height: 2rem;
  border-bottom: 1px solid var(--border-color);
  background: color-mix(in srgb, var(--bg-elevated) 70%, transparent);
}

.form-data-row {
  display: grid;
  grid-template-columns: 2.25rem minmax(0, 0.7fr) minmax(0, 0.7fr) 5rem 2rem;
  align-items: center;
  min-height: 2.625rem;
  border-bottom: 1px solid var(--border-color);
}

.form-data-row:last-child {
  border-bottom: 0;
}

.form-data-row input[type='checkbox'] {
  justify-self: center;
  accent-color: var(--brand-500);
}

.form-data-row input:not([type='checkbox']):not([type='file']) {
  min-width: 0;
  width: 100%;
  height: 2.625rem;
  box-sizing: border-box;
  border: 0;
  border-left: 1px solid var(--border-color);
  background: transparent;
  color: var(--text-primary);
  padding: 0 0.625rem;
  font: 0.8125rem var(--font-family-mono, monospace);
  outline: none;
}

.form-data-row input:not([type='checkbox']):not([type='file']):focus {
  background: color-mix(in srgb, var(--brand-500) 8%, transparent);
}

.form-data-row .type-select {
  min-width: 0;
  width: 100%;
  height: 2.625rem;
  box-sizing: border-box;
  border: 0;
  border-left: 1px solid var(--border-color);
  background: transparent;
  color: var(--text-secondary);
  padding: 0 0.5rem;
  font-size: 0.75rem;
  font-weight: 700;
  outline: none;
  cursor: pointer;
}

.file-picker {
  display: flex;
  align-items: center;
  height: 2.625rem;
  border-left: 1px solid var(--border-color);
  cursor: pointer;
}

.file-picker input[type='file'] {
  position: absolute;
  opacity: 0;
  pointer-events: none;
  width: 0;
}

.file-name {
  padding: 0 0.625rem;
  font: 0.78125rem var(--font-family-mono, monospace);
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-name.empty {
  color: var(--text-muted);
}

/* ── Waterfall ── */
.waterfall {
  padding: 0.625rem 0.75rem;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  gap: 0.3125rem;
}

.waterfall-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.waterfall-label {
  width: 5.5rem;
  flex-shrink: 0;
  font-size: 0.6875rem;
  font-weight: 800;
  color: var(--text-secondary);
  text-align: right;
}

.waterfall-bar-wrap {
  flex: 1;
  min-width: 0;
  height: 0.5rem;
  border-radius: 0.25rem;
  background: var(--bg-elevated);
  overflow: hidden;
}

.waterfall-bar {
  height: 100%;
  border-radius: 0.25rem;
  min-width: 2px;
  transition: width 0.3s ease;
}

.waterfall-ms {
  width: 3.5rem;
  flex-shrink: 0;
  font-size: 0.6875rem;
  font-family: var(--font-family-mono, monospace);
  color: var(--text-secondary);
}

/* ── cURL Import Modal ── */
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(4px);
}

.modal-content {
  width: min(90vw, 560px);
  border-radius: 0.75rem;
  background: var(--bg-surface);
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.25);
  padding: 0;
  overflow: hidden;
}

.modal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.875rem 1rem;
  border-bottom: 1px solid var(--border-color);
}

.modal-head strong {
  font-size: 0.9375rem;
  color: var(--text-primary);
}

.curl-textarea {
  width: 100%;
  min-height: 10rem;
  padding: 0.875rem 1rem;
  border: 0;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font: 0.8125rem/1.55 var(--font-family-mono, monospace);
  outline: none;
  resize: vertical;
  box-sizing: border-box;
}

.modal-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  border-top: 1px solid var(--border-color);
  background: color-mix(in srgb, var(--bg-elevated) 50%, transparent);
}

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-active .modal-content,
.modal-leave-active .modal-content {
  transition: transform 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-content {
  transform: scale(0.96) translateY(8px);
}

.modal-leave-to .modal-content {
  transform: scale(0.96) translateY(8px);
}

@media (max-width: 720px) {
  .form-data-row {
    grid-template-columns: 2rem minmax(0, 1fr) 4rem 1.5rem;
    grid-template-rows: repeat(2, 2.5rem);
  }

  .form-data-row input:not([type='checkbox']):not([type='file']) {
    grid-column: 2;
    height: 2.5rem;
    border-left: 1px solid var(--border-color);
  }

  .form-data-row input:not([type='checkbox']):not([type='file']) + .file-picker {
    grid-column: 2;
    grid-row: 2;
  }

  .form-data-row input:not([type='checkbox']):not([type='file']) + input:not([type='checkbox']) {
    grid-row: 2;
  }

  .form-data-row .type-select {
    grid-column: 3;
    grid-row: 1 / span 2;
  }

  .form-data-row .remove {
    grid-column: 4;
    grid-row: 1 / span 2;
  }
}
</style>
