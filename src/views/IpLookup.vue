<template>
  <div class="tool-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />
    <main class="tool-main">
      <div class="tool-topbar"><router-link to="/tools" class="back-link"><ArrowLeft :size="16" />工具中心</router-link></div>
      <section class="tool-header"><div class="tool-heading"><div class="heading-icon"><Network :size="22" /></div><div><h1>IP 信息解析</h1><p>从多个公开数据源查询 IP 的网络和地理信息；结果仅供参考。</p></div></div></section>

      <section class="lookup-box">
        <div class="lookup-row"><input v-model.trim="ip" placeholder="输入 IPv4 或 IPv6；留空查询当前公网 IP" @keyup.enter="lookup" /><select v-model="selectedProvider"><option value="all">聚合查询（推荐）</option><option v-for="provider in providers" :key="provider.id" :value="provider.id">{{ provider.name }}</option></select><button class="btn primary" :disabled="loading" @click="lookup"><LoaderCircle v-if="loading" class="spin" :size="16" /><Search v-else :size="16" />{{ loading ? '查询中' : '查询' }}</button><button class="refresh-btn" :disabled="loading" title="忽略缓存并重新请求" @click="lookup(true)"><RefreshCw :size="16" />刷新</button></div>
        <p v-if="error" class="error-tip">{{ error }}</p><p v-else class="lookup-tip">{{ cacheMessage || '指定 IP 地址时，成功响应会在浏览器 IndexedDB 中缓存 1 小时；当前公网 IP 每次都会实时查询。遇到 429 会按服务建议的等待时间暂停重试。' }}</p>
      </section>

      <section v-if="summary" class="summary-card"><div class="summary-main"><div class="ip-value">{{ summary.ip }}</div><div class="place"><MapPin :size="16" />{{ summary.location || '未提供地理位置' }}</div></div><div class="summary-grid"><div><span>网络</span><strong>{{ summary.network || '—' }}</strong></div><div><span>时区</span><strong>{{ summary.timezone || '—' }}</strong></div><div><span>坐标</span><strong>{{ summary.coordinates || '—' }}</strong></div></div></section>

      <div v-if="results.length" class="result-grid"><article v-for="result in results" :key="result.id" class="provider-card"><div class="provider-title"><span>{{ result.name }}</span><span class="source-status" :class="result.error ? 'failed' : result.cached ? 'cached' : 'success'">{{ result.error ? '不可用' : result.cached ? '缓存结果' : '已返回' }}</span></div><p v-if="result.error" class="provider-error">{{ result.error }}</p><template v-else><dl class="field-list"><template v-for="field in result.fields" :key="field.label"><dt>{{ field.label }}</dt><dd>{{ field.value || '—' }}</dd></template></dl><details><summary>查看原始数据</summary><pre>{{ JSON.stringify(result.raw, null, 2) }}</pre></details></template></article></div>
      <div v-else-if="!loading" class="empty-state"><Globe2 :size="32" /><span>输入 IP 地址开始查询</span></div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { ArrowLeft, Globe2, LoaderCircle, MapPin, Network, RefreshCw, Search } from 'lucide-vue-next'
import { useTheme } from '@/composables/useTheme'

type Field = { label: string; value: string }
type ProviderResult = { id: string; name: string; raw?: Record<string, unknown>; fields: Field[]; error?: string; cached?: boolean }
type Provider = { id: string; name: string; request: (ip: string) => string; parse: (data: Record<string, any>) => Field[]; isFailure: (data: Record<string, any>) => boolean }
type CacheRecord = { key: string; data?: Record<string, any>; results?: ProviderResult[]; expiresAt?: number; rateLimitedUntil?: number }
const CACHE_DB = 'newbie-ip-lookup', CACHE_STORE = 'responses', CACHE_TTL = 60 * 60 * 1000
let cacheDatabase: Promise<IDBDatabase> | null = null
const { isDark } = useTheme()
const ip = ref(''), selectedProvider = ref('all'), loading = ref(false), error = ref(''), results = ref<ProviderResult[]>([]), cacheMessage = ref('')
const text = (value: unknown) => value === undefined || value === null || value === '' ? '' : String(value)
const providers: Provider[] = [
  { id: 'iprust', name: 'IPRust', request: () => 'http://iprust.io/ip.json', isFailure: data => Boolean(data.error || data.message), parse: data => [{ label: 'IP', value: text(data.ip) }, { label: '国家 / 地区', value: [data.country, data.country_name, data.region, data.city].filter(Boolean).join(' · ') }, { label: 'ASN / 运营商', value: [data.asn?.number && `AS${data.asn.number}`, data.asn?.name, data.organization, data.org].filter(Boolean).join(' · ') }, { label: '时区', value: text(data.timezone) }, { label: '坐标', value: data.latitude != null || data.lat != null ? `${data.latitude ?? data.lat}, ${data.longitude ?? data.lon}` : '' }, { label: '邮编', value: text(data.postal || data.zip) }] },
  { id: 'ipwhois', name: 'ipwho.is', request: value => `https://ipwho.is/${encodeURIComponent(value)}`, isFailure: data => data.success === false, parse: data => [{ label: 'IP', value: text(data.ip) }, { label: '国家 / 地区', value: [data.country, data.region, data.city].filter(Boolean).join(' · ') }, { label: 'ASN / 运营商', value: [data.connection?.asn && `AS${data.connection.asn}`, data.connection?.org || data.connection?.isp].filter(Boolean).join(' · ') }, { label: '时区', value: text(data.timezone?.id) }, { label: '坐标', value: data.latitude != null ? `${data.latitude}, ${data.longitude}` : '' }, { label: '类型', value: text(data.type) }] },
  { id: 'ipapi', name: 'ipapi.co', request: value => `https://ipapi.co/${encodeURIComponent(value)}/json/`, isFailure: data => Boolean(data.error), parse: data => [{ label: 'IP', value: text(data.ip) }, { label: '国家 / 地区', value: [data.country_name, data.region, data.city].filter(Boolean).join(' · ') }, { label: 'ASN / 运营商', value: [data.asn, data.org].filter(Boolean).join(' · ') }, { label: '时区', value: text(data.timezone) }, { label: '坐标', value: data.latitude != null ? `${data.latitude}, ${data.longitude}` : '' }, { label: '邮编', value: text(data.postal) }] },
  { id: 'ipinfo', name: 'ipinfo.io', request: value => `https://ipinfo.io/${encodeURIComponent(value)}/json`, isFailure: data => Boolean(data.error || data.bogon), parse: data => [{ label: 'IP', value: text(data.ip) }, { label: '国家 / 地区', value: [data.country, data.region, data.city].filter(Boolean).join(' · ') }, { label: '网络', value: [data.org, data.asn?.name].filter(Boolean).join(' · ') }, { label: '时区', value: text(data.timezone) }, { label: '坐标', value: text(data.loc) }, { label: '邮编', value: text(data.postal) }] },
  { id: 'ipapicom', name: 'ip-api.com', request: value => `http://ip-api.com/json/${encodeURIComponent(value)}`, isFailure: data => data.status !== 'success', parse: data => [{ label: 'IP', value: text(data.query) }, { label: '国家 / 地区', value: [data.country, data.regionName, data.city].filter(Boolean).join(' · ') }, { label: 'ASN / 运营商', value: [data.as, data.isp, data.org].filter(Boolean).join(' · ') }, { label: '时区', value: text(data.timezone) }, { label: '坐标', value: data.lat != null ? `${data.lat}, ${data.lon}` : '' }, { label: '邮编', value: text(data.zip) }] },
  { id: 'ipapiis', name: 'ipapi.is', request: value => value ? `https://api.ipapi.is/?q=${encodeURIComponent(value)}` : 'https://api.ipapi.is/', isFailure: data => Boolean(data.error || !data.ip), parse: data => [{ label: 'IP', value: text(data.ip) }, { label: '国家 / 地区', value: [data.location?.country_name, data.location?.state, data.location?.city].filter(Boolean).join(' · ') }, { label: 'ASN / 运营商', value: [data.asn?.asn && `AS${data.asn.asn}`, data.asn?.descr_short || data.asn?.descr, data.company?.name].filter(Boolean).join(' · ') }, { label: '时区', value: text(data.location?.timezone) }, { label: '坐标', value: data.location?.latitude != null ? `${data.location.latitude}, ${data.location.longitude}` : '' }, { label: '邮编', value: text(data.location?.zip) }, { label: '类型', value: [data.is_mobile && '移动网络', data.is_datacenter && '数据中心', data.is_proxy && '代理', data.is_vpn && 'VPN', data.is_tor && 'Tor'].filter(Boolean).join(' · ') || '—' }] },
]
const summary = computed(() => {
  const valid = results.value.filter(item => !item.error)
  if (!valid.length) return null

  const collect = (getValue: (fields: Field[]) => string | undefined) =>
    valid.map(r => getValue(r.fields)).filter(Boolean) as string[]

  const ipValues = collect(f => f.find(item => item.label === 'IP')?.value)
  const locationValues = collect(f => f.find(item => item.label === '国家 / 地区')?.value)
  const networkValues = collect(f => {
    const n = f.find(item => item.label === 'ASN / 运营商' || item.label === '网络')
    return n?.value
  })
  const tzValues = collect(f => f.find(item => item.label === '时区')?.value)
  const coordValues = collect(f => f.find(item => item.label === '坐标')?.value)

  const majority = (values: string[]) => {
    if (!values.length) return ''
    const freq = new Map<string, number>()
    for (const v of values) freq.set(v, (freq.get(v) || 0) + 1)
    let best = values[0], bestCount = 1
    for (const [k, c] of freq) {
      if (c > bestCount) { best = k; bestCount = c }
    }
    return best
  }

  return {
    ip: majority(ipValues) || ip.value || '当前公网 IP',
    location: majority(locationValues) || '未提供地理位置',
    network: majority(networkValues) || '—',
    timezone: majority(tzValues) || '—',
    coordinates: majority(coordValues) || '—',
  }
})
function validIp(value: string) {
  if (!value) return true
  const ipv4 = /^(?:25[0-5]|2[0-4]\d|1?\d?\d)(?:\.(?:25[0-5]|2[0-4]\d|1?\d?\d)){3}$/
  return ipv4.test(value) || (value.includes(':') && /^[0-9a-fA-F:.]+$/.test(value))
}
function localIpResult(value: string): ProviderResult | null {
  const local = (category: string, description: string): ProviderResult => ({
    id: 'local', name: '本地解析', fields: [
      { label: 'IP', value },
      { label: '地址类型', value: category },
      { label: '处理方式', value: '纯本地识别，不会请求第三方 API' },
      { label: '说明', value: description }
    ]
  })
  if (value === '127.0.0.1') return local('IPv4 环回地址', '仅指向当前设备（localhost）。')
  if (value.startsWith('127.')) return local('IPv4 环回网段', '127.0.0.0/8 仅用于本机回环通信。')
  if (value.startsWith('10.')) return local('IPv4 私有地址', '10.0.0.0/8 为 RFC 1918 私有网段。')
  if (/^192\.168\./.test(value)) return local('IPv4 私有地址', '192.168.0.0/16 为 RFC 1918 私有网段。')
  if (/^172\.(1[6-9]|2\d|3[0-1])\./.test(value)) return local('IPv4 私有地址', '172.16.0.0/12 为 RFC 1918 私有网段。')
  if (/^169\.254\./.test(value)) return local('IPv4 链路本地地址', '169.254.0.0/16 通常由设备自动分配，不能跨路由器使用。')
  if (/^100\.(?:6[4-9]|[7-9]\d|1[01]\d|12[0-7])\./.test(value)) return local('运营商级 NAT 地址', '100.64.0.0/10 用于运营商级 NAT（CGNAT），并非公网可路由地址。')
  if (/^198\.(1[89])\./.test(value)) return local('IPv4 基准测试地址', '198.18.0.0/15 保留用于网络设备性能测试。')
  if (/^(192\.0\.2|198\.51\.100|203\.0\.113)\./.test(value)) return local('IPv4 文档示例地址', '该网段仅用于文档和示例，不能作为公网主机地址。')
  if (/^(22[4-9]|23\d)\./.test(value)) return local('IPv4 组播地址', '224.0.0.0/4 用于组播，不对应单个公网主机。')
  if (/^(24\d|25[0-5])\./.test(value)) return local('IPv4 保留地址', '240.0.0.0/4 为保留地址空间，不应进行公网归属地查询。')
  if (value === '0.0.0.0') return local('IPv4 未指定地址', '用于表示所有本地接口或尚未分配的地址。')
  const lower = value.toLowerCase()
  if (lower === '::1') return local('IPv6 环回地址', '仅指向当前设备（localhost）。')
  if (lower === '::') return local('IPv6 未指定地址', '表示尚未分配的 IPv6 地址。')
  if (/^(fc|fd)[0-9a-f]{2}:/i.test(lower)) return local('IPv6 唯一本地地址', 'fc00::/7 用于私有网络，不在公网路由。')
  if (/^fe[89ab][0-9a-f]:/i.test(lower)) return local('IPv6 链路本地地址', 'fe80::/10 仅在本地链路内有效。')
  if (/^ff[0-9a-f]{2}:/i.test(lower)) return local('IPv6 组播地址', 'ff00::/8 用于组播，不对应单个公网主机。')
  if (/^2001:0db8:/i.test(lower)) return local('IPv6 文档示例地址', '2001:db8::/32 仅用于文档和示例。')
  return null
}
function cacheKey(provider: Provider, value: string) { return `${provider.id}:${value || 'current'}` }
function openCache() {
  if (cacheDatabase) return cacheDatabase
  cacheDatabase = new Promise<IDBDatabase>((resolve, reject) => {
    const request = indexedDB.open(CACHE_DB, 1)
    request.onupgradeneeded = () => {
      if (!request.result.objectStoreNames.contains(CACHE_STORE)) request.result.createObjectStore(CACHE_STORE, { keyPath: 'key' })
    }
    request.onsuccess = () => resolve(request.result)
    request.onerror = () => { cacheDatabase = null; reject(request.error) }
  })
  return cacheDatabase
}
async function getCache(key: string) {
  try {
    const db = await openCache()
    return await new Promise<CacheRecord | undefined>((resolve, reject) => {
      const request = db.transaction(CACHE_STORE).objectStore(CACHE_STORE).get(key)
      request.onsuccess = () => resolve(request.result as CacheRecord | undefined)
      request.onerror = () => reject(request.error)
    })
  } catch { return undefined }
}
async function setCache(record: CacheRecord) {
  try {
    const db = await openCache()
    await new Promise<void>((resolve, reject) => {
      const transaction = db.transaction(CACHE_STORE, 'readwrite')
      transaction.objectStore(CACHE_STORE).put(record)
      transaction.oncomplete = () => resolve()
      transaction.onerror = () => reject(transaction.error)
      transaction.onabort = () => reject(transaction.error)
    })
  } catch { /* 私密模式或禁用 IndexedDB 时仍直接查询 */ }
}
function retryAfterMs(response: Response) {
  const value = response.headers.get('Retry-After')
  if (!value) return 5 * 60 * 1000
  const seconds = Number(value)
  if (Number.isFinite(seconds)) return Math.max(seconds * 1000, 1000)
  const date = Date.parse(value)
  return Number.isNaN(date) ? 5 * 60 * 1000 : Math.max(date - Date.now(), 1000)
}
function waitMessage(until: number) {
  const seconds = Math.max(1, Math.ceil((until - Date.now()) / 1000))
  return `请求过于频繁（HTTP 429），该数据源将在约 ${seconds} 秒后恢复查询。`
}
async function fetchProvider(provider: Provider, value: string, forceRefresh = false): Promise<ProviderResult> {
  const key = cacheKey(provider, value), cached = await getCache(key)
  if (cached?.rateLimitedUntil && cached.rateLimitedUntil > Date.now()) return { id: provider.id, name: provider.name, fields: [], error: waitMessage(cached.rateLimitedUntil) }
  /* 仅指定 IP 地址时走缓存；当前公网 IP 每次都实时请求 */
  if (value && !forceRefresh && cached?.data && cached.expiresAt && cached.expiresAt > Date.now()) return { id: provider.id, name: provider.name, raw: cached.data, fields: provider.parse(cached.data), cached: true }
  try {
    const controller = new AbortController(), timer = setTimeout(() => controller.abort(), 9000)
    const response = await fetch(provider.request(value), { signal: controller.signal }); clearTimeout(timer)
    if (response.status === 429) {
      const rateLimitedUntil = Date.now() + retryAfterMs(response)
      await setCache({ key, rateLimitedUntil })
      return { id: provider.id, name: provider.name, fields: [], error: waitMessage(rateLimitedUntil) }
    }
    const data = await response.json() as Record<string, any>
    if (!response.ok || provider.isFailure(data)) return { id: provider.id, name: provider.name, fields: [], error: text(data.reason || data.message || data.error) || `服务返回 HTTP ${response.status}` }
    /* 仅指定 IP 地址时写入缓存 */
    if (value) await setCache({ key, data, expiresAt: Date.now() + CACHE_TTL })
    return { id: provider.id, name: provider.name, raw: data, fields: provider.parse(data) }
  } catch { return { id: provider.id, name: provider.name, fields: [], error: '请求失败：服务不可达、超时或被浏览器 CORS 策略阻止' } }
}
async function lookup(forceRefresh = false) {
  if (!validIp(ip.value)) { error.value = '请输入有效的 IPv4 或 IPv6 地址'; return }
  if (ip.value && selectedProvider.value === 'iprust') { error.value = 'IPRust 的 ip.json 仅返回当前公网 IP；请输入空值后再使用该数据源。'; return }
  error.value = ''; cacheMessage.value = ''; results.value = []; loading.value = true
  const localResult = ip.value ? localIpResult(ip.value) : null
  if (localResult) {
    results.value = [localResult]
    cacheMessage.value = '已识别为特殊网络地址，结果由浏览器本地生成，未发起任何网络请求。'
    loading.value = false
    return
  }
  const targets = (selectedProvider.value === 'all' ? providers : providers.filter(item => item.id === selectedProvider.value))
    .filter(provider => !ip.value || provider.id !== 'iprust')
  results.value = await Promise.all(targets.map(provider => fetchProvider(provider, ip.value, forceRefresh)))
  const cachedCount = results.value.filter(item => item.cached).length
  if (cachedCount) cacheMessage.value = `本次查询命中 ${cachedCount}/${targets.length} 个浏览器缓存（有效期 1 小时）。`
  else if (forceRefresh && ip.value) cacheMessage.value = '已跳过正常缓存并重新请求数据源。'
  if (results.value.every(item => item.error)) error.value = '所有数据源均未返回可用结果，请稍后重试或切换数据源。'
  loading.value = false
}

onMounted(() => lookup())
</script>

<style scoped>
.heading-icon { --tool-color: #06b6d4; }.lookup-box, .summary-card, .provider-card { border: 1px solid var(--border-color); border-radius: .75rem; background: var(--bg-surface); }.lookup-box { padding: .875rem; }.lookup-row { display: flex; gap: .5rem; }.lookup-row input { flex: 1; min-width: 0; height: 2.5rem; box-sizing: border-box; border: 1px solid var(--border-color); border-radius: .375rem; padding: 0 .75rem; background: var(--bg-elevated); color: var(--text-primary); font: inherit; outline: 0; }.lookup-row input:focus { border-color: var(--brand-500); }.lookup-row select, .refresh-btn { border: 1px solid var(--border-color); border-radius: .375rem; background: var(--bg-elevated); color: var(--text-primary); padding: 0 .5rem; }.refresh-btn { display: inline-flex; align-items: center; gap: .25rem; cursor: pointer; font: 600 .8125rem inherit; }.refresh-btn:disabled { opacity: .5; cursor: not-allowed; }.lookup-tip, .error-tip { margin: .625rem 0 0; font-size: .8125rem; color: var(--text-secondary); }.error-tip { color: #ef4444; }.summary-card { margin-top: .75rem; padding: 1rem; display: flex; align-items: center; justify-content: space-between; gap: 1rem; }.ip-value { font: 700 1.5rem var(--font-family-mono, monospace); }.place { display: flex; align-items: center; gap: .25rem; margin-top: .375rem; color: var(--text-secondary); font-size: .875rem; }.summary-grid { display: flex; gap: 1.5rem; }.summary-grid div { display: flex; flex-direction: column; gap: .25rem; max-width: 15rem; }.summary-grid span { color: var(--text-muted); font-size: .75rem; }.summary-grid strong { font-size: .8125rem; word-break: break-word; }.result-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: .75rem; margin-top: .75rem; }.provider-card { overflow: hidden; }.provider-title { display: flex; justify-content: space-between; align-items: center; padding: .75rem .875rem; border-bottom: 1px solid var(--border-color); font-weight: 700; }.source-status { padding: .125rem .375rem; border-radius: 999px; font-size: .6875rem; }.source-status.success { color: #047857; background: #d1fae5; }.source-status.cached { color: #1d4ed8; background: #dbeafe; }.source-status.failed { color: #b91c1c; background: #fee2e2; }.field-list { display: grid; grid-template-columns: 6.5rem 1fr; gap: .5rem .75rem; padding: .875rem; margin: 0; font-size: .8125rem; }.field-list dt { color: var(--text-secondary); }.field-list dd { margin: 0; overflow-wrap: anywhere; }.provider-error { padding: .875rem; margin: 0; color: #ef4444; font-size: .8125rem; line-height: 1.5; }.provider-card details { border-top: 1px solid var(--border-color); padding: .625rem .875rem; color: var(--text-secondary); font-size: .75rem; }.provider-card summary { cursor: pointer; }.provider-card pre { max-height: 18rem; overflow: auto; color: var(--text-primary); white-space: pre-wrap; word-break: break-word; }.empty-state { min-height: 15rem; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: .625rem; color: var(--text-muted); }.spin { animation: spin .8s linear infinite; } @keyframes spin { to { transform: rotate(360deg); } } @media (max-width: 720px) { .lookup-row { flex-wrap: wrap; }.lookup-row input { width: 100%; }.lookup-row select, .lookup-row button { height: 2.5rem; }.summary-card { align-items: flex-start; flex-direction: column; }.summary-grid { width: 100%; justify-content: space-between; gap: .75rem; } }
</style>