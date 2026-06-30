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

      <section class="tool-header">
        <div class="tool-heading">
          <div class="heading-icon">
            <Braces :size="22" />
          </div>
          <div>
            <h1>{{ tool?.name }}</h1>
            <p>{{ tool?.desc }}</p>
          </div>
        </div>
      </section>

      <div v-if="pipelineFrom" class="pipeline-banner">
        <ArrowRightLeft :size="14" />
        <span>来自「{{ pipelineFrom }}」的传递数据</span>
      </div>

      <div class="workspace">
        <!-- 左侧：输入 -->
        <div class="panel panel-left">
          <div class="mode-bar">
            <div class="segmented">
              <button :class="{ active: mode === 'format' }" @click="mode = 'format'">格式化</button>
              <button :class="{ active: mode === 'compact' }" @click="mode = 'compact'">压缩</button>
              <button :class="{ active: mode === 'validate' }" @click="mode = 'validate'">验证</button>
            </div>
            <button class="swap-btn" @click="pasteSample" title="粘贴示例 JSON">
              <FileJson :size="16" />
            </button>
          </div>

          <div class="option-row">
            <label class="section-label">缩进</label>
            <div class="segmented indent-control">
              <button :class="{ active: indent === 2 }" @click="indent = 2">2</button>
              <button :class="{ active: indent === 4 }" @click="indent = 4">4</button>
              <button :class="{ active: indent === 'tab' }" @click="indent = 'tab'">Tab</button>
            </div>
            <label class="checkbox-inline">
              <input v-model="sortKeys" type="checkbox" />
              <span>排序键</span>
            </label>
          </div>

          <div class="quick-tools">
            <button class="tool-chip" :disabled="!inputJson" @click="escapeText">转义</button>
            <button class="tool-chip" :disabled="!inputJson" @click="unescapeText">去转义</button>
            <button class="tool-chip" :disabled="!inputJson" @click="decodeUnicode">Unicode 解码</button>
            <button class="tool-chip" :disabled="!inputJson" @click="removeWhitespace">去空白</button>
          </div>

          <div class="path-row">
            <input
              v-model="pathQuery"
              type="text"
              placeholder="路径查询，如 $.data.items[0].name"
              @keyup.enter="queryPath"
            />
            <button class="btn secondary" :disabled="!stats || !pathQuery" @click="queryPath">查询</button>
          </div>

          <textarea
            v-model="inputJson"
            :placeholder="inputPlaceholder"
            rows="14"
            @input="scheduleProcess"
          ></textarea>

          <div v-if="validationResult" class="validation-badge" :class="validationResult.valid ? 'valid' : 'invalid'">
            <CheckCircle v-if="validationResult.valid" :size="14" />
            <AlertCircle v-else :size="14" />
            <span>{{ validationResult.message }}</span>
          </div>

          <div v-if="pathResult" class="path-result" :class="{ error: !pathResult.ok }">
            <span>{{ pathResult.message }}</span>
          </div>

          <div class="input-actions">
            <button class="btn secondary" :disabled="!inputJson" @click="copyInput">
              <Copy :size="16" />复制输入
            </button>
            <button class="btn secondary" :disabled="!outputText" @click="replaceInputWithOutput">
              <ArrowRightLeft :size="16" />回填
            </button>
          </div>
        </div>

        <!-- 右侧：输出 -->
        <div class="panel panel-right">
          <div class="panel-header-row">
            <h3 class="panel-title">处理结果</h3>
            <span v-if="stats" class="summary-badge">
              {{ stats.nodeCount }} 节点 · {{ stats.depth }} 层
            </span>
          </div>

          <textarea
            v-model="outputText"
            readonly
            rows="14"
            :placeholder="outputPlaceholder"
          ></textarea>

          <div class="meta" v-if="outputText">
            <span>输入 {{ inputJson.length.toLocaleString() }} → 输出 {{ outputText.length.toLocaleString() }} 字符</span>
            <span v-if="mode === 'compact' && inputJson.length > 0" class="savings">
              压缩 {{ compressionRatio }}%
            </span>
          </div>

          <div v-if="stats" class="stats-grid">
            <div><span>对象</span><strong>{{ stats.objects }}</strong></div>
            <div><span>数组</span><strong>{{ stats.arrays }}</strong></div>
            <div><span>键</span><strong>{{ stats.keys }}</strong></div>
            <div><span>字符串</span><strong>{{ stats.strings }}</strong></div>
            <div><span>数字</span><strong>{{ stats.numbers }}</strong></div>
            <div><span>布尔</span><strong>{{ stats.booleans }}</strong></div>
            <div><span>null</span><strong>{{ stats.nulls }}</strong></div>
            <div><span>大小</span><strong>{{ outputSize }}</strong></div>
          </div>

          <div v-if="treeRows.length" class="tree-panel">
            <div class="panel-header-row">
              <h3 class="panel-title">结构预览</h3>
              <span class="summary-badge">最多显示 {{ treeRows.length }} 行</span>
            </div>
            <div class="tree-list">
              <div
                v-for="row in treeRows"
                :key="row.path"
                class="tree-row"
                :style="{ paddingLeft: `${row.depth * 0.875 + 0.5}rem` }"
              >
                <span class="tree-key">{{ row.key }}</span>
                <span class="tree-type">{{ row.type }}</span>
                <code class="tree-value">{{ row.preview }}</code>
              </div>
            </div>
          </div>

          <div class="actions">
            <button class="btn primary" :disabled="!outputText" @click="copyResult">
              <Copy :size="16" />{{ copyLabel }}
            </button>
            <button class="btn secondary" :disabled="!outputText" @click="downloadResult">
              <Download :size="16" />下载
            </button>
            <button class="btn secondary" :disabled="!outputText" @click="clearAll">
              <Trash2 :size="16" />清空
            </button>
            <PipelineSend
              :tools="downstreamTools"
              :disabled="!outputText"
              @send="sendToTool"
            />
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
import { computed, onUnmounted, ref, watch } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { usePipeline } from '@/composables/usePipeline'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import { findTool } from '@/utils/toolPipeline'
import {
  AlertCircle,
  ArrowLeft,
  ArrowRightLeft,
  Braces,
  CheckCircle,
  Copy,
  Download,
  FileJson,
  Trash2
} from 'lucide-vue-next'

const tool = findTool('json-formatter')
const { isDark } = useTheme()

// --- 流转 ---
const { pipelineFrom, downstreamTools, sendTextTo } = usePipeline({
  toolId: 'json-formatter',
  onIncoming: async (incoming) => {
    if (incoming.type !== 'text') return false
    inputJson.value = incoming.data.text
    scheduleProcess()
    return true
  }
})

function sendToTool(target: typeof downstreamTools.value[number]) {
  if (!outputText.value) return
  const result = sendTextTo(target, outputText.value)
  showToast(result.message, result.ok ? 'success' : 'error')
}

// --- 状态 ---
const mode = ref<'format' | 'compact' | 'validate'>('format')
const indent = ref<2 | 4 | 'tab'>(2)
const sortKeys = ref(false)
const inputJson = ref('')
const outputText = ref('')
const validationResult = ref<{ valid: boolean; message: string } | null>(null)
const stats = ref<JsonStats | null>(null)
const parsedJson = ref<unknown>(null)
const pathQuery = ref('')
const pathResult = ref<{ ok: boolean; message: string } | null>(null)
const copyLabel = ref('复制')
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')

let processTimer: ReturnType<typeof setTimeout> | null = null
let toastTimer: ReturnType<typeof setTimeout> | null = null

const sampleJson = `{
  "name": "Alice",
  "age": 28,
  "items": ["apple", "orange"],
  "address": {
    "city": "Beijing",
    "zip": "100000"
  }
}`

interface JsonStats {
  objects: number
  arrays: number
  keys: number
  strings: number
  numbers: number
  booleans: number
  nulls: number
  depth: number
  nodeCount: number
}

interface TreeRow {
  key: string
  path: string
  type: string
  preview: string
  depth: number
}

const inputPlaceholder = computed(() => {
  if (mode.value === 'validate') return '粘贴 JSON 文本进行语法验证…'
  return '粘贴需要格式化或压缩的 JSON 文本…'
})

const outputPlaceholder = computed(() => {
  if (mode.value === 'format') return '格式化结果将显示在这里'
  if (mode.value === 'compact') return '压缩结果将显示在这里'
  return '验证通过后会显示规范化结果'
})

const compressionRatio = computed(() => {
  if (!inputJson.value || !outputText.value) return 0
  return Math.round((1 - outputText.value.length / inputJson.value.length) * 100)
})

const outputSize = computed(() => formatBytes(new Blob([outputText.value]).size))

const treeRows = computed(() => {
  if (!stats.value) return []
  return flattenTree(parsedJson.value, '$', '$', 0, 160)
})

// --- 处理 ---
function scheduleProcess() {
  if (processTimer) clearTimeout(processTimer)
  processTimer = setTimeout(() => {
    doProcess()
  }, 150)
}

function doProcess() {
  const raw = inputJson.value.trim()
  if (!raw) {
    outputText.value = ''
    validationResult.value = null
    stats.value = null
    parsedJson.value = null
    pathResult.value = null
    return
  }

  try {
    const parsed = sortKeys.value ? sortJsonValue(JSON.parse(raw)) : JSON.parse(raw)
    const currentStats = collectStats(parsed)
    stats.value = currentStats
    parsedJson.value = parsed
    pathResult.value = null

    if (mode.value === 'format') {
      outputText.value = JSON.stringify(parsed, null, indentValue.value)
      validationResult.value = { valid: true, message: `有效 · ${currentStats.keys} 个键 · 嵌套 ${currentStats.depth} 层` }
    } else if (mode.value === 'compact') {
      outputText.value = JSON.stringify(parsed)
      validationResult.value = { valid: true, message: `有效 · ${currentStats.keys} 个键 · 已压缩` }
    } else {
      // validate
      outputText.value = JSON.stringify(parsed, null, indentValue.value)
      validationResult.value = {
        valid: true,
        message: `有效 · ${currentStats.keys} 个键 · 嵌套 ${currentStats.depth} 层`
      }
    }
  } catch (e) {
    outputText.value = ''
    stats.value = null
    parsedJson.value = null
    pathResult.value = null
    const msg = getJsonErrorMessage(e, raw)
    validationResult.value = { valid: false, message: msg }
  }
}

const indentValue = computed(() => indent.value === 'tab' ? '\t' : indent.value)

function sortJsonValue(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sortJsonValue)
  if (value === null || typeof value !== 'object') return value

  const record = value as Record<string, unknown>
  return Object.keys(record).sort().reduce<Record<string, unknown>>((acc, key) => {
    acc[key] = sortJsonValue(record[key])
    return acc
  }, {})
}

function collectStats(value: unknown): JsonStats {
  const result: JsonStats = {
    objects: 0,
    arrays: 0,
    keys: 0,
    strings: 0,
    numbers: 0,
    booleans: 0,
    nulls: 0,
    depth: 0,
    nodeCount: 0
  }

  walkStats(value, 0, result)
  return result
}

function flattenTree(value: unknown, key: string, path: string, depth: number, limit: number): TreeRow[] {
  if (limit <= 0) return []

  const rows: TreeRow[] = [{
    key,
    path,
    type: getValueType(value),
    preview: getValuePreview(value),
    depth
  }]

  if (rows.length >= limit) return rows

  if (Array.isArray(value)) {
    for (let index = 0; index < value.length && rows.length < limit; index++) {
      rows.push(...flattenTree(value[index], `[${index}]`, `${path}[${index}]`, depth + 1, limit - rows.length))
    }
  } else if (value !== null && typeof value === 'object') {
    const record = value as Record<string, unknown>
    for (const keyName of Object.keys(record)) {
      if (rows.length >= limit) break
      const childPath = isIdentifier(keyName) ? `${path}.${keyName}` : `${path}[${JSON.stringify(keyName)}]`
      rows.push(...flattenTree(record[keyName], keyName, childPath, depth + 1, limit - rows.length))
    }
  }

  return rows
}

function getValueType(value: unknown): string {
  if (value === null) return 'null'
  if (Array.isArray(value)) return `array(${value.length})`
  if (typeof value === 'object') return `object(${Object.keys(value as Record<string, unknown>).length})`
  return typeof value
}

function getValuePreview(value: unknown): string {
  if (value === null) return 'null'
  if (Array.isArray(value)) return '[...]'
  if (typeof value === 'object') return '{...}'
  const text = typeof value === 'string' ? value : String(value)
  return text.length > 80 ? `${text.slice(0, 80)}...` : text
}

function isIdentifier(value: string): boolean {
  return /^[A-Za-z_$][\w$]*$/.test(value)
}

function walkStats(value: unknown, depth: number, result: JsonStats) {
  result.nodeCount += 1
  result.depth = Math.max(result.depth, depth)

  if (value === null) {
    result.nulls += 1
    return
  }

  if (Array.isArray(value)) {
    result.arrays += 1
    for (const item of value) walkStats(item, depth + 1, result)
    return
  }

  if (typeof value === 'object') {
    result.objects += 1
    const values = Object.values(value as Record<string, unknown>)
    result.keys += values.length
    for (const item of values) walkStats(item, depth + 1, result)
    return
  }

  if (typeof value === 'string') result.strings += 1
  else if (typeof value === 'number') result.numbers += 1
  else if (typeof value === 'boolean') result.booleans += 1
}

function getJsonErrorMessage(error: unknown, raw: string): string {
  const fallback = error instanceof SyntaxError ? error.message : '无效的 JSON 格式'
  const match = fallback.match(/position (\d+)/i)
  if (!match) return fallback

  const position = Number(match[1])
  const before = raw.slice(0, position)
  const lines = before.split(/\r\n|\r|\n/)
  const line = lines.length
  const column = lines[lines.length - 1].length + 1
  return `${fallback} · 第 ${line} 行，第 ${column} 列`
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}

function queryJsonPath(source: unknown, path: string): unknown {
  let cursor = source
  const normalized = path.trim().replace(/^\$/, '')
  if (!normalized) return cursor

  const tokens: Array<string | number> = []
  const matcher = /(?:\.([A-Za-z_$][\w$]*))|\[(?:(\d+)|"([^"]+)"|'([^']+)')\]/g
  let matched = ''
  let match: RegExpExecArray | null

  while ((match = matcher.exec(normalized))) {
    matched += match[0]
    if (match[1]) tokens.push(match[1])
    else if (match[2]) tokens.push(Number(match[2]))
    else tokens.push(match[3] ?? match[4])
  }

  if (matched !== normalized) {
    throw new Error('路径格式不支持')
  }

  for (const token of tokens) {
    if (cursor === null || typeof cursor !== 'object') throw new Error('路径不存在')
    if (!(token in (cursor as Record<string | number, unknown>))) throw new Error('路径不存在')
    cursor = (cursor as Record<string | number, unknown>)[token]
  }

  return cursor
}

// 模式切换时重新处理
watch(mode, () => {
  if (inputJson.value.trim()) doProcess()
})

watch([indent, sortKeys], () => {
  if (inputJson.value.trim()) doProcess()
})

// --- 操作 ---
function pasteSample() {
  inputJson.value = sampleJson
  scheduleProcess()
}

function setOutput(text: string, successMessage: string) {
  outputText.value = text
  validationResult.value = { valid: true, message: successMessage }
  showToast(successMessage, 'success')
}

function escapeText() {
  if (!inputJson.value) return
  setOutput(JSON.stringify(inputJson.value).slice(1, -1), '已转义为字符串内容')
}

function unescapeText() {
  if (!inputJson.value) return
  try {
    const raw = inputJson.value.trim()
    const decoded = raw.startsWith('"') && raw.endsWith('"')
      ? JSON.parse(raw)
      : JSON.parse(`"${raw.replace(/"/g, '\\"')}"`)
    setOutput(String(decoded), '已去转义')
  } catch {
    showToast('去转义失败，请检查反斜杠序列', 'error')
  }
}

function decodeUnicode() {
  if (!inputJson.value) return
  try {
    const decoded = inputJson.value.replace(/\\u([0-9a-fA-F]{4})/g, (_, hex: string) => {
      return String.fromCharCode(parseInt(hex, 16))
    })
    setOutput(decoded, 'Unicode 已解码')
  } catch {
    showToast('Unicode 解码失败', 'error')
  }
}

function removeWhitespace() {
  if (!inputJson.value) return
  try {
    const parsed = JSON.parse(inputJson.value)
    setOutput(JSON.stringify(parsed), '已移除 JSON 空白')
  } catch {
    outputText.value = inputJson.value.replace(/\s+/g, '')
    validationResult.value = { valid: false, message: '输入不是有效 JSON，已按纯文本移除空白' }
  }
}

function queryPath() {
  if (!stats.value) {
    doProcess()
    if (!stats.value) return
  }
  try {
    const value = queryJsonPath(parsedJson.value, pathQuery.value)
    const text = typeof value === 'string' ? value : JSON.stringify(value, null, indentValue.value)
    outputText.value = text ?? 'undefined'
    pathResult.value = { ok: true, message: `${pathQuery.value.trim() || '$'} 查询成功` }
    showToast('路径查询成功', 'success')
  } catch (error) {
    pathResult.value = {
      ok: false,
      message: error instanceof Error ? error.message : '路径查询失败'
    }
  }
}

async function copyInput() {
  if (!inputJson.value) return
  try {
    await navigator.clipboard.writeText(inputJson.value)
    showToast('输入已复制到剪贴板', 'success')
  } catch {
    showToast('复制失败', 'error')
  }
}

async function copyResult() {
  if (!outputText.value) return
  try {
    await navigator.clipboard.writeText(outputText.value)
    copyLabel.value = '已复制'
    showToast('结果已复制到剪贴板', 'success')
    setTimeout(() => { copyLabel.value = '复制' }, 1500)
  } catch {
    showToast('复制失败', 'error')
  }
}

function replaceInputWithOutput() {
  if (!outputText.value) return
  inputJson.value = outputText.value
  scheduleProcess()
  showToast('已回填到输入区', 'success')
}

function downloadResult() {
  if (!outputText.value) return
  const blob = new Blob([outputText.value], { type: 'application/json;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = mode.value === 'compact' ? 'compact.json' : 'formatted.json'
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
  showToast('下载已开始', 'success')
}

function clearAll() {
  inputJson.value = ''
  outputText.value = ''
  validationResult.value = null
  stats.value = null
  parsedJson.value = null
  pathResult.value = null
}

function showToast(message: string, type: 'success' | 'error') {
  if (toastTimer) clearTimeout(toastTimer)
  toastMessage.value = message
  toastType.value = type
  toastTimer = setTimeout(() => {
    toastMessage.value = ''
  }, 2200)
}

// --- 生命周期 ---
onUnmounted(() => {
  if (processTimer) clearTimeout(processTimer)
  if (toastTimer) clearTimeout(toastTimer)
})
</script>

<style scoped>
.heading-icon { --tool-color: #3b82f6; }

/* --- 模式切换 --- */
.mode-bar {
  display: flex; align-items: center; gap: 0.5rem;
}
.mode-bar .segmented { flex: 1; }

.swap-btn {
  width: 2rem; height: 2rem; display: flex; align-items: center; justify-content: center;
  border: 1px solid var(--border-color); border-radius: 0.375rem;
  background: var(--bg-surface); color: var(--text-secondary); cursor: pointer;
  transition: border-color 0.15s, color 0.15s;
  flex-shrink: 0;
}
.swap-btn:hover { border-color: var(--brand-500); color: var(--brand-500); }

.option-row {
  display: flex; align-items: center; flex-wrap: wrap; gap: 0.5rem;
}
.option-row .section-label { margin-right: -0.25rem; }
.indent-control { width: 9rem; flex: 0 0 auto; }
.checkbox-inline {
  display: inline-flex; align-items: center; gap: 0.375rem;
  color: var(--text-secondary); font-size: 0.8125rem; font-weight: 700;
  cursor: pointer;
}
.checkbox-inline input {
  margin: 0;
  accent-color: var(--brand-500);
}

.quick-tools {
  display: flex; flex-wrap: wrap; gap: 0.375rem;
}
.tool-chip {
  min-height: 2rem;
  padding: 0 0.625rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 0.8125rem;
  font-weight: 700;
  cursor: pointer;
}
.tool-chip:hover { border-color: var(--brand-500); color: var(--brand-500); }
.tool-chip:disabled { opacity: 0.5; cursor: not-allowed; }

.path-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.5rem;
}
.path-row input {
  min-width: 0;
  padding: 0.5rem 0.625rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 0.8125rem;
  font-family: var(--font-family-mono, monospace);
  outline: none;
}
.path-row input:focus { border-color: var(--brand-500); }

/* --- 左侧 textarea --- */
.panel-left textarea {
  width: 100%;
  padding: 0.625rem 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 0.8125rem;
  font-family: var(--font-family-mono, monospace);
  resize: vertical;
  outline: none;
  box-sizing: border-box;
  line-height: 1.6;
}
.panel-left textarea:focus { border-color: var(--brand-500); }

.input-actions {
  display: flex; flex-wrap: wrap; gap: 0.5rem;
}

/* --- 验证结果 --- */
.validation-badge {
  display: inline-flex; align-items: center; gap: 0.375rem;
  padding: 0.375rem 0.625rem; border-radius: 0.375rem;
  font-size: 0.8125rem; font-weight: 600;
}
.validation-badge.valid {
  background: color-mix(in srgb, #10b981 10%, transparent);
  color: #047857;
}
.validation-badge.invalid {
  background: color-mix(in srgb, #ef4444 10%, transparent);
  color: #b91c1c;
}
.path-result {
  padding: 0.375rem 0.625rem;
  border-radius: 0.375rem;
  background: color-mix(in srgb, #3b82f6 10%, transparent);
  color: #1d4ed8;
  font-size: 0.8125rem;
  font-weight: 700;
}
.path-result.error {
  background: color-mix(in srgb, #ef4444 10%, transparent);
  color: #b91c1c;
}

.panel-header-row {
  display: flex; align-items: center; justify-content: space-between; gap: 0.5rem;
}
.panel-title {
  margin: 0;
  font-size: 0.875rem;
  font-weight: 800;
}
.summary-badge {
  padding: 0.1875rem 0.5rem;
  border-radius: 999px;
  background: var(--bg-elevated);
  color: var(--text-muted);
  font-size: 0.75rem;
  font-weight: 700;
  white-space: nowrap;
}

/* --- 右侧 textarea --- */
.panel-right textarea {
  width: 100%;
  padding: 0.625rem 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 0.8125rem;
  font-family: var(--font-family-mono, monospace);
  resize: vertical;
  outline: none;
  box-sizing: border-box;
  line-height: 1.6;
  cursor: default;
  min-height: 22rem;
}
.panel-right textarea:focus { border-color: var(--brand-500); }

/* --- 元信息 --- */
.meta {
  display: flex; align-items: center; gap: 0.5rem;
  font-size: 0.875rem;
  color: var(--text-secondary);
  font-family: var(--font-family-mono, monospace);
}
.savings {
  color: #047857;
  font-weight: 600;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.5rem;
}
.stats-grid div {
  display: flex; flex-direction: column; gap: 0.125rem;
  padding: 0.5rem;
  border-radius: 0.375rem;
  background: var(--bg-elevated);
}
.stats-grid span {
  color: var(--text-muted);
  font-size: 0.6875rem;
  font-weight: 700;
}
.stats-grid strong {
  color: var(--text-primary);
  font-size: 0.875rem;
  font-family: var(--font-family-mono, monospace);
}

.tree-panel {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.tree-list {
  max-height: 17rem;
  overflow: auto;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-elevated);
}
.tree-row {
  display: grid;
  grid-template-columns: minmax(5rem, 0.7fr) auto minmax(0, 1fr);
  gap: 0.5rem;
  align-items: center;
  min-height: 1.875rem;
  border-bottom: 1px solid color-mix(in srgb, var(--border-color) 70%, transparent);
  font-family: var(--font-family-mono, monospace);
  font-size: 0.75rem;
}
.tree-row:last-child { border-bottom: 0; }
.tree-key {
  color: var(--text-primary);
  font-weight: 800;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.tree-type {
  color: #2563eb;
  font-weight: 700;
}
.tree-value {
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 520px) {
  .stats-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .indent-control { width: 100%; }
  .path-row { grid-template-columns: 1fr; }
  .tree-row { grid-template-columns: minmax(4rem, 0.8fr) auto; }
  .tree-value { grid-column: 1 / -1; }
}
</style>
