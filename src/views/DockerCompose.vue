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
            <Container :size="22" />
          </div>
          <div>
            <h1>Docker Run 转换</h1>
            <p>将 docker run 命令转换为 docker-compose.yml 格式</p>
          </div>
        </div>
      </section>

      <div class="workspace">
        <div class="panel panel-left">
          <div class="input-section">
            <label for="docker-run">Docker Run 命令</label>
            <textarea
              id="docker-run"
              v-model="dockerRunCommand"
              rows="10"
              placeholder="docker run -d --name nginx -p 80:80 -v /data:/usr/share/nginx/html nginx:latest"
              @input="handleConvert"
            ></textarea>
          </div>

          <div class="action-bar">
            <button class="btn secondary" @click="clearAll">
              <Trash2 :size="16" />
              清空
            </button>
            <button class="btn secondary" @click="loadExample">
              <Lightbulb :size="16" />
              示例
            </button>
          </div>
        </div>

        <div class="panel panel-right">
          <div class="output-section">
            <div class="output-header">
              <label>docker-compose.yml</label>
              <button
                v-if="composeOutput && !composeOutput.startsWith('# Error')"
                class="btn-copy"
                @click="copyOutput"
              >
                <Check v-if="copied" :size="14" />
                <Copy v-else :size="14" />
                {{ copied ? '已复制' : '复制' }}
              </button>
            </div>
            <pre class="output-code" :class="{ error: composeOutput.startsWith('# Error') }">{{ composeOutput || '# 在左侧输入 docker run 命令' }}</pre>
          </div>

          <div class="action-bar">
            <button
              class="btn primary"
              :disabled="!composeOutput || composeOutput.startsWith('# Error')"
              @click="downloadYaml"
            >
              <Download :size="16" />
              下载 YAML
            </button>
            <PipelineSend
              :tools="downstreamTools"
              :disabled="!composeOutput || composeOutput.startsWith('# Error')"
              @send="handlePipelineSend"
            />
          </div>
        </div>
      </div>

      <div v-if="history.length > 0" class="history-section">
        <h3>历史记录</h3>
        <div class="history-list">
          <button
            v-for="(item, index) in history"
            :key="index"
            class="history-chip"
            @click="loadFromHistory(item)"
          >
            {{ item.serviceName || 'app' }}
          </button>
        </div>
      </div>
    </main>

    <Transition name="toast">
      <div v-if="toastMessage" class="toast" :class="toastType">
        {{ toastMessage }}
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { usePipeline } from '@/composables/usePipeline'
import type { ToolItem } from '@/data/tools'
import PipelineSend from '@/components/tools/PipelineSend.vue'
import {
  ArrowLeft,
  Check,
  Container,
  Copy,
  Download,
  Lightbulb,
  Trash2
} from 'lucide-vue-next'
import { convertDockerRunToCompose } from '@/utils/dockerCompose'

const { isDark } = useTheme()

// Pipeline 集成
const { pipelineFrom, downstreamTools, sendTextTo } = usePipeline({
  toolId: 'docker-compose',
  async onIncoming(incoming) {
    if (incoming.type === 'text') {
      dockerRunCommand.value = incoming.data.text
      handleConvert()
      return true
    }
    return false
  }
})

const dockerRunCommand = ref('')
const composeOutput = ref('')
const copied = ref(false)
const toastMessage = ref('')
const toastType = ref<'success' | 'error'>('success')

interface HistoryItem {
  command: string
  output: string
  serviceName: string
}

const history = ref<HistoryItem[]>([])

const EXAMPLE_COMMAND = `docker run -d --name my-nginx -p 8080:80 -v /home/user/data:/usr/share/nginx/html -e NGINX_HOST=example.com -e NGINX_PORT=80 --restart unless-stopped nginx:latest`

onMounted(() => {
  loadHistory()
})

function handleConvert() {
  if (!dockerRunCommand.value.trim()) {
    composeOutput.value = ''
    return
  }

  try {
    composeOutput.value = convertDockerRunToCompose(dockerRunCommand.value)

    // 保存到历史记录
    if (!composeOutput.value.startsWith('# Error')) {
      addToHistory()
    }
  } catch (error) {
    composeOutput.value = `# Error: ${error instanceof Error ? error.message : '转换失败'}`
  }
}

function clearAll() {
  dockerRunCommand.value = ''
  composeOutput.value = ''
}

function loadExample() {
  dockerRunCommand.value = EXAMPLE_COMMAND
  handleConvert()
}

async function copyOutput() {
  if (!composeOutput.value || composeOutput.value.startsWith('# Error')) {
    return
  }

  try {
    await navigator.clipboard.writeText(composeOutput.value)
    copied.value = true
    showToast('已复制到剪贴板', 'success')

    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (error) {
    showToast('复制失败', 'error')
  }
}

function downloadYaml() {
  if (!composeOutput.value || composeOutput.value.startsWith('# Error')) {
    return
  }

  const blob = new Blob([composeOutput.value], { type: 'text/yaml' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = 'docker-compose.yml'
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)

  showToast('下载已开始', 'success')
}

function handlePipelineSend(target: ToolItem) {
  if (!composeOutput.value || composeOutput.value.startsWith('# Error')) {
    return
  }

  const { ok, message } = sendTextTo(target, composeOutput.value)
  showToast(message, ok ? 'success' : 'error')
}

function addToHistory() {
  const serviceName = extractServiceName(composeOutput.value)
  const item: HistoryItem = {
    command: dockerRunCommand.value,
    output: composeOutput.value,
    serviceName
  }

  // 避免重复
  const existingIndex = history.value.findIndex(h => h.command === item.command)
  if (existingIndex !== -1) {
    history.value.splice(existingIndex, 1)
  }

  history.value.unshift(item)

  // 限制历史记录数量
  if (history.value.length > 10) {
    history.value = history.value.slice(0, 10)
  }

  saveHistory()
}

function extractServiceName(yaml: string): string {
  const match = yaml.match(/services:\s*\n\s+(\w+):/)
  return match ? match[1] : 'app'
}

function loadFromHistory(item: HistoryItem) {
  dockerRunCommand.value = item.command
  composeOutput.value = item.output
}

function loadHistory() {
  try {
    const saved = localStorage.getItem('docker-compose-history')
    if (saved) {
      history.value = JSON.parse(saved)
    }
  } catch (error) {
    console.error('Failed to load history:', error)
  }
}

function saveHistory() {
  try {
    localStorage.setItem('docker-compose-history', JSON.stringify(history.value))
  } catch (error) {
    console.error('Failed to save history:', error)
  }
}

function showToast(message: string, type: 'success' | 'error') {
  toastMessage.value = message
  toastType.value = type

  setTimeout(() => {
    toastMessage.value = ''
  }, 3000)
}
</script>

<style scoped>
.heading-icon { --tool-color: #3b82f6; }

.input-section,
.output-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex: 1;
}

label {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.output-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.btn-copy {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-copy:hover {
  background: var(--bg-elevated);
  color: var(--text-primary);
}

.output-code {
  flex: 1;
  margin: 0;
  padding: 16px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.6;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--text-primary);
  min-height: 200px;
}

.output-code.error {
  color: #ef4444;
}

.action-bar {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.history-section {
  margin-top: 32px;
}

.history-section h3 {
  font-size: 16px;
  margin: 0 0 16px;
  color: var(--text-primary);
}

.history-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.history-chip {
  padding: 8px 16px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 20px;
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
  transition: all 0.2s;
}

.history-chip:hover {
  border-color: var(--brand-500);
  color: var(--brand-600);
}
</style>
