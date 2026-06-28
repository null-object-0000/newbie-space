<template>
  <div class="tool-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />
    <main class="tool-main">
      <div class="tool-topbar"><router-link to="/tools" class="back-link"><ArrowLeft :size="16" /><span>工具中心</span></router-link></div>
      <section class="tool-header">
        <div class="tool-heading">
          <div class="heading-icon"><Send :size="22" /></div>
          <div><h1>HTTP 请求</h1><p>轻量级 API 调试工具，请求由浏览器直接发出，不经过本站服务器。</p></div>
        </div>
      </section>

      <!-- URL Bar -->
      <div class="urlbar">
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
            <span v-else>发送</span>
          </button>
        </div>
        <div class="urlbar-hint">
          <span class="cors-badge" title="请求由当前浏览器直接发出；若接口未配置 CORS，浏览器会阻止读取响应">CORS 提醒</span>
        </div>
      </div>

      <!-- History -->
      <div v-if="history.length" class="history-strip">
        <div class="history-scroll">
          <button
            v-for="(item, idx) in history"
            :key="idx"
            class="history-chip"
            @click="restoreHistory(item)"
          >
            <span class="chip-method" :style="methodColor(item.method)">{{ item.method }}</span>
            <span class="chip-url">{{ item.url }}</span>
            <button class="chip-close" @click.stop="deleteHistory(idx)"><X :size="11" /></button>
          </button>
        </div>
        <button class="history-clear" @click="clearHistory" title="清空历史"><Trash2 :size="13" /></button>
      </div>

      <!-- Workspace -->
      <div class="workspace http-workspace">
        <!-- Left: Request Builder -->
        <div class="panel panel-request">
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
            <!-- Params -->
            <template v-if="activeTab === 'params'">
              <KeyValueEditor v-model="params" key-placeholder="参数名" value-placeholder="参数值" />
            </template>

            <!-- Auth -->
            <template v-else-if="activeTab === 'auth'">
              <div class="auth-type-select">
                <select v-model="authType">
                  <option value="none">无认证</option>
                  <option value="bearer">Bearer Token</option>
                  <option value="basic">Basic Auth</option>
                </select>
              </div>
              <div v-if="authType === 'bearer'" class="auth-form">
                <label class="field-label">Token</label>
                <input v-model="bearerToken" type="text" placeholder="输入 Token…" @keyup.enter="applyBearer" />
                <button class="btn secondary auth-apply" @click="applyBearer">应用</button>
              </div>
              <div v-else-if="authType === 'basic'" class="auth-form">
                <label class="field-label">用户名</label>
                <input v-model="basicUsername" type="text" placeholder="用户名" />
                <label class="field-label">密码</label>
                <input v-model="basicPassword" type="password" placeholder="密码" />
                <button class="btn secondary auth-apply" @click="applyBasicAuth">应用</button>
              </div>
              <p v-else class="auth-none-hint">选择认证方式后，Authorization 请求头将自动填入。</p>
            </template>

            <!-- Headers -->
            <template v-else-if="activeTab === 'headers'">
              <KeyValueEditor v-model="headers" key-placeholder="Header 名称" value-placeholder="Header 值" />
            </template>

            <!-- Body -->
            <template v-else-if="activeTab === 'body'">
              <div class="body-bar">
                <select v-model="bodyType">
                  <option value="json">JSON</option>
                  <option value="text">纯文本</option>
                </select>
                <span v-if="bodyDisabled" class="body-disabled-hint">GET / HEAD 请求不携带请求体</span>
              </div>
              <textarea
                v-model="body"
                :disabled="bodyDisabled"
                class="body-textarea"
                rows="14"
                placeholder='{ "name": "Ada" }'
              ></textarea>
            </template>

            <!-- Code -->
            <template v-else>
              <div class="segmented code-langs">
                <button :class="{ active: codeLang === 'curl' }" @click="codeLang = 'curl'">cURL</button>
                <button :class="{ active: codeLang === 'python' }" @click="codeLang = 'python'">Python</button>
                <button :class="{ active: codeLang === 'js' }" @click="codeLang = 'js'">JavaScript</button>
                <button :class="{ active: codeLang === 'go' }" @click="codeLang = 'go'">Go</button>
              </div>
              <pre class="code-block"><code>{{ generatedCode }}</code></pre>
              <button class="btn secondary code-copy-btn" @click="copyCode">
                <Check v-if="codeCopied" :size="14" />
                <Copy v-else :size="14" />
                {{ codeCopied ? '已复制' : '复制代码' }}
              </button>
            </template>
          </div>
        </div>

        <!-- Right: Response Viewer -->
        <div class="panel panel-response">
          <template v-if="response">
            <div class="resp-status-bar">
              <div class="resp-status-left">
                <span class="resp-status-badge" :class="{ ok: response.ok, err: !response.ok }">
                  {{ response.status }} {{ response.statusText }}
                </span>
                <span class="resp-meta">{{ response.duration }}&thinsp;ms · {{ response.size }}</span>
              </div>
              <button class="btn-icon" title="复制响应" @click="copyResponse"><Copy :size="14" /></button>
            </div>
            <div class="resp-tabs">
              <button :class="{ active: respTab === 'body' }" @click="respTab = 'body'">响应体</button>
              <button :class="{ active: respTab === 'headers' }" @click="respTab = 'headers'">
                响应头
                <span class="tab-badge">{{ Object.keys(response.headers).length }}</span>
              </button>
            </div>
            <pre v-if="respTab === 'body'" class="resp-body">{{ response.displayBody }}</pre>
            <div v-else class="resp-headers-list">
              <div v-for="(val, key) in response.headers" :key="key" class="resp-header-row">
                <code>{{ key }}</code><span>{{ val }}</span>
              </div>
            </div>
          </template>
          <div v-else class="resp-empty">
            <Wifi :size="32" />
            <span>{{ error || '点击发送按钮发起请求' }}</span>
            <span v-if="error" class="resp-error-detail">{{ error }}</span>
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
import { ArrowLeft, Check, ChevronDown, Copy, LoaderCircle, Send, Trash2, Wifi, X } from 'lucide-vue-next'
import { useTheme } from '@/composables/useTheme'
import KeyValueEditor from '@/components/tools/KeyValueEditor.vue'
import {
  generateCurlCommand,
  generateJavaScriptCode,
  generatePythonCode,
  generateGoCode
} from '@/utils/httpClient'

type KeyValue = { enabled: boolean; key: string; value: string }
type HttpResponse = { ok: boolean; status: number; statusText: string; duration: number; size: string; headers: Record<string, string>; displayBody: string }
interface HistoryItem {
  method: string
  url: string
  params: KeyValue[]
  headers: KeyValue[]
  body: string
  bodyType: 'json' | 'text'
}

const { isDark } = useTheme()
const methods = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS']

// --- Request state ---
const method = ref('GET')
const url = ref('')
const params = ref<KeyValue[]>([{ enabled: true, key: '', value: '' }])
const headers = ref<KeyValue[]>([{ enabled: true, key: '', value: '' }])
const body = ref('')
const bodyType = ref<'json' | 'text'>('json')
const bodyDisabled = computed(() => ['GET', 'HEAD'].includes(method.value))
const loading = ref(false)
const error = ref('')
const response = ref<HttpResponse | null>(null)

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
  bodyType.value = item.bodyType
}
function deleteHistory(idx: number) { history.value.splice(idx, 1); saveHistory() }
function clearHistory() { history.value = []; saveHistory() }

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

// --- Code gen ---
const codeLang = ref<'curl' | 'python' | 'js' | 'go'>('curl')
const codeCopied = ref(false)
function safeBuildUrl(): string { try { return buildUrl() } catch { return '' } }
const generatedCode = computed(() => {
  const target = safeBuildUrl()
  if (!target) return '# 请先输入有效的 URL'
  const p = {
    method: method.value, url: target,
    headers: headers.value.filter(h => h.enabled && h.key),
    body: body.value, bodyType: bodyType.value,
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
  error.value = ''; response.value = null
  let target: string
  try { target = buildUrl() } catch { error.value = '请输入有效的完整 URL（例如 https://api.example.com）'; return }
  const reqHeaders: Record<string, string> = {}
  headers.value.filter(i => i.enabled && i.key).forEach(i => { reqHeaders[i.key] = i.value })
  const init: RequestInit = { method: method.value, headers: reqHeaders }
  if (!bodyDisabled.value && body.value) {
    init.body = body.value
    if (bodyType.value === 'json' && !Object.keys(reqHeaders).some(k => k.toLowerCase() === 'content-type'))
      init.headers = { ...reqHeaders, 'Content-Type': 'application/json' }
  }
  loading.value = true
  const t0 = performance.now()
  try {
    const res = await fetch(target, init)
    const text = await res.text()
    let displayBody = text
    try { displayBody = JSON.stringify(JSON.parse(text), null, 2) } catch { /* raw */ }
    response.value = {
      ok: res.ok, status: res.status, statusText: res.statusText,
      duration: Math.round(performance.now() - t0),
      size: formatSize(new Blob([text]).size),
      headers: Object.fromEntries(res.headers.entries()),
      displayBody,
    }
    pushHistory()
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

onMounted(() => { loadHistory() })
onUnmounted(() => { if (toastTimer) clearTimeout(toastTimer) })
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
</style>
