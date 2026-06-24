<template>
  <div class="tool-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />
    <main class="tool-main">
      <div class="tool-topbar"><router-link to="/tools" class="back-link"><ArrowLeft :size="16" />工具中心</router-link></div>
      <section class="tool-header"><div class="tool-heading"><div class="heading-icon"><Send :size="22" /></div><div><h1>HTTP 请求</h1><p>在浏览器中快速调试 HTTP API。请求不会经过本站服务器。</p></div></div></section>

      <div class="request-panel">
        <div class="request-line">
          <select v-model="method" aria-label="请求方法"><option v-for="item in methods" :key="item" :value="item">{{ item }}</option></select>
          <input v-model.trim="url" class="url-input" type="url" placeholder="https://api.example.com/users" @keyup.enter="sendRequest" />
          <button class="btn primary send-btn" :disabled="loading || !url" @click="sendRequest"><LoaderCircle v-if="loading" class="spin" :size="16" /><Send v-else :size="16" />{{ loading ? '发送中' : '发送' }}</button>
        </div>
        <p class="cors-tip">请求由当前浏览器直接发出；若接口未配置 CORS，浏览器会阻止读取响应。</p>
      </div>

      <div class="workspace http-workspace">
        <div class="panel">
          <div class="tabs"><button :class="{ active: activeTab === 'params' }" @click="activeTab = 'params'">参数 <span>{{ enabledParams }}</span></button><button :class="{ active: activeTab === 'headers' }" @click="activeTab = 'headers'">请求头 <span>{{ enabledHeaders }}</span></button><button :class="{ active: activeTab === 'body' }" @click="activeTab = 'body'">请求体</button></div>
          <template v-if="activeTab === 'params'"><p class="section-help">参数会自动附加到 URL。取消勾选可临时停用。</p><KeyValueEditor v-model="params" key-placeholder="参数名" value-placeholder="值" /></template>
          <template v-else-if="activeTab === 'headers'"><p class="section-help">例如 <code>Authorization: Bearer token</code>。</p><KeyValueEditor v-model="headers" key-placeholder="Header 名称" value-placeholder="值" /></template>
          <template v-else><div class="body-toolbar"><select v-model="bodyType"><option value="json">JSON</option><option value="text">Text</option></select><span v-if="bodyDisabled" class="muted">此请求方法通常不携带请求体</span></div><textarea v-model="body" :disabled="bodyDisabled" rows="14" placeholder='{ "name": "Ada" }'></textarea></template>
        </div>

        <div class="panel response-panel">
          <div class="response-header"><span class="section-label">响应</span><span v-if="response" class="status" :class="response.ok ? 'ok' : 'bad'">{{ response.status }} {{ response.statusText }}</span><span v-if="response" class="response-meta">{{ response.duration }} ms · {{ response.size }}</span><button v-if="response" class="copy-response" @click="copyResponse"><Copy :size="14" />复制</button></div>
          <pre v-if="response" class="response-body">{{ response.displayBody }}</pre>
          <div v-else class="response-empty"><Wifi :size="28" /><span>{{ error || '发送请求后，响应内容会显示在这里' }}</span></div>
          <details v-if="response" class="response-headers"><summary>响应头（{{ Object.keys(response.headers).length }}）</summary><div v-for="(value, key) in response.headers" :key="key"><code>{{ key }}</code><span>{{ value }}</span></div></details>
        </div>
      </div>
    </main>
    <Transition name="toast"><div v-if="toast" class="toast success">{{ toast }}</div></Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { ArrowLeft, Copy, LoaderCircle, Send, Wifi } from 'lucide-vue-next'
import { useTheme } from '@/composables/useTheme'
import KeyValueEditor from '@/components/tools/KeyValueEditor.vue'

type KeyValue = { enabled: boolean; key: string; value: string }
type HttpResponse = { ok: boolean; status: number; statusText: string; duration: number; size: string; headers: Record<string, string>; displayBody: string }
const { isDark } = useTheme()
const methods = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS']
const method = ref('GET'), url = ref(''), activeTab = ref<'params' | 'headers' | 'body'>('params')
const params = ref<KeyValue[]>([{ enabled: true, key: '', value: '' }])
const headers = ref<KeyValue[]>([{ enabled: true, key: '', value: '' }])
const body = ref(''), bodyType = ref<'json' | 'text'>('json'), loading = ref(false), error = ref(''), response = ref<HttpResponse | null>(null), toast = ref('')
const bodyDisabled = computed(() => ['GET', 'HEAD'].includes(method.value))
const enabledParams = computed(() => params.value.filter(item => item.enabled && item.key).length)
const enabledHeaders = computed(() => headers.value.filter(item => item.enabled && item.key).length)

function formatSize(size: number) { return size < 1024 ? `${size} B` : `${(size / 1024).toFixed(1)} KB` }
function buildUrl() {
  const target = new URL(url.value)
  params.value.filter(item => item.enabled && item.key).forEach(item => target.searchParams.set(item.key, item.value))
  return target.toString()
}
async function sendRequest() {
  error.value = ''; response.value = null
  let target: string
  try { target = buildUrl() } catch { error.value = '请输入有效的完整 URL（例如 https://api.example.com）'; return }
  const requestHeaders: Record<string, string> = {}
  headers.value.filter(item => item.enabled && item.key).forEach(item => { requestHeaders[item.key] = item.value })
  const init: RequestInit = { method: method.value, headers: requestHeaders }
  if (!bodyDisabled.value && body.value) {
    init.body = body.value
    if (bodyType.value === 'json' && !Object.keys(requestHeaders).some(key => key.toLowerCase() === 'content-type')) init.headers = { ...requestHeaders, 'Content-Type': 'application/json' }
  }
  loading.value = true
  const startedAt = performance.now()
  try {
    const result = await fetch(target, init)
    const text = await result.text()
    let displayBody = text
    try { displayBody = JSON.stringify(JSON.parse(text), null, 2) } catch { /* 保持原始文本 */ }
    response.value = { ok: result.ok, status: result.status, statusText: result.statusText, duration: Math.round(performance.now() - startedAt), size: formatSize(new Blob([text]).size), headers: Object.fromEntries(result.headers.entries()), displayBody }
  } catch (cause) {
    error.value = cause instanceof Error && cause.message ? `请求失败：${cause.message}。请检查网络、URL 和接口 CORS 配置。` : '请求失败：请检查网络、URL 和接口 CORS 配置。'
  } finally { loading.value = false }
}
async function copyResponse() { if (!response.value) return; await navigator.clipboard.writeText(response.value.displayBody); toast.value = '响应已复制'; setTimeout(() => toast.value = '', 1600) }
</script>

<style scoped>
.heading-icon { --tool-color: #f97316; }.request-panel { padding: .875rem; margin-bottom: .75rem; border: 1px solid var(--border-color); border-radius: .75rem; background: var(--bg-surface); }.request-line { display: flex; gap: .5rem; }.request-line select, .body-toolbar select { border: 1px solid var(--border-color); border-radius: .375rem; background: var(--bg-elevated); color: var(--text-primary); padding: 0 .625rem; font-weight: 700; }.url-input { flex: 1; min-width: 0; border: 1px solid var(--border-color); border-radius: .375rem; background: var(--bg-elevated); color: var(--text-primary); padding: 0 .75rem; font: inherit; outline: none; }.url-input:focus { border-color: var(--brand-500); }.send-btn { flex-shrink: 0; }.cors-tip, .section-help { margin: .625rem 0 0; color: var(--text-secondary); font-size: .8125rem; }.http-workspace { grid-template-columns: 1fr; } @media (min-width: 900px) { .http-workspace { grid-template-columns: minmax(0, .85fr) minmax(0, 1.15fr); } }.tabs { display: flex; gap: .25rem; border-bottom: 1px solid var(--border-color); }.tabs button { padding: .5rem .625rem; border: 0; border-bottom: 2px solid transparent; background: transparent; color: var(--text-secondary); font: inherit; font-size: .8125rem; cursor: pointer; }.tabs button.active { color: var(--brand-500); border-color: var(--brand-500); }.tabs span { font-size: .6875rem; }.body-toolbar { display: flex; align-items: center; gap: .625rem; }.body-toolbar select { height: 2rem; }.muted { color: var(--text-muted); font-size: .8125rem; }.response-header { display: flex; align-items: center; gap: .5rem; min-height: 1.75rem; }.status { padding: .125rem .375rem; border-radius: .25rem; font: 700 .75rem var(--font-family-mono, monospace); }.status.ok { color: #047857; background: #d1fae5; }.status.bad { color: #b91c1c; background: #fee2e2; }.response-meta { color: var(--text-secondary); font: .75rem var(--font-family-mono, monospace); }.copy-response { margin-left: auto; display: inline-flex; gap: .25rem; align-items: center; border: 0; background: transparent; color: var(--text-secondary); cursor: pointer; }.response-body { flex: 1; min-height: 16rem; max-height: 38rem; margin: 0; overflow: auto; padding: .75rem; border-radius: .375rem; background: var(--bg-elevated); color: var(--text-primary); font: .8125rem/1.55 var(--font-family-mono, monospace); white-space: pre-wrap; word-break: break-word; }.response-empty { flex: 1; min-height: 16rem; display: flex; flex-direction: column; justify-content: center; align-items: center; gap: .625rem; color: var(--text-muted); text-align: center; font-size: .875rem; }.response-headers { color: var(--text-secondary); font-size: .75rem; }.response-headers summary { cursor: pointer; }.response-headers div { display: flex; gap: .5rem; padding: .25rem 0; border-bottom: 1px solid var(--border-color); word-break: break-word; }.response-headers code { color: var(--text-primary); }.spin { animation: spin .8s linear infinite; } @keyframes spin { to { transform: rotate(360deg); } } @media (max-width: 600px) { .request-line { flex-wrap: wrap; }.request-line select { height: 2.5rem; }.url-input { width: calc(100% - 6rem); height: 2.5rem; }.send-btn { width: 100%; } }
</style>
