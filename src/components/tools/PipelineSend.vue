<template>
  <div v-if="tools.length" class="pipeline-send">
    <div class="pipeline-select-wrap">
      <select
        ref="selectRef"
        v-model="selectedId"
        class="pipeline-select"
        :disabled="disabled"
      >
        <option disabled value="">— 流转到 —</option>
        <option v-for="t in tools" :key="t.id" :value="t.id">{{ t.name }}</option>
      </select>
      <ChevronDown :size="14" class="select-chevron" />
    </div>
    <button
      class="pipeline-btn"
      :disabled="disabled || !selectedId"
      @click="handleSend"
    >
      <ArrowRightLeft :size="16" />
      <span>流转</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, type PropType } from 'vue'
import { ArrowRightLeft, ChevronDown } from 'lucide-vue-next'
import type { ToolItem } from '@/data/tools'

const props = defineProps({
  tools: {
    type: Array as PropType<ToolItem[]>,
    required: true
  },
  disabled: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits<{
  send: [tool: ToolItem]
}>()

const selectedId = ref('')
const selectRef = ref<HTMLSelectElement | null>(null)

function handleSend() {
  const target = props.tools.find(t => t.id === selectedId.value)
  if (target) {
    emit('send', target)
    selectedId.value = ''
  }
}
</script>

<style scoped>
.pipeline-send {
  display: flex; align-items: center; gap: 0.5rem;
}

.pipeline-select-wrap {
  position: relative;
}

.pipeline-select {
  min-height: 2.25rem;
  padding: 0 2rem 0 0.625rem;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-elevated);
  color: var(--text-primary);
  font-size: 0.8125rem;
  font-family: inherit;
  font-weight: 600;
  cursor: pointer;
  appearance: none;
  outline: none;
  transition: border-color 0.15s;
}
.pipeline-select:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}
.pipeline-select:focus {
  border-color: var(--brand-500);
}

.select-chevron {
  position: absolute; right: 0.5rem; top: 50%;
  transform: translateY(-50%);
  color: var(--text-secondary);
  pointer-events: none;
}

.pipeline-btn {
  min-height: 2.25rem;
  display: inline-flex; align-items: center; justify-content: center;
  gap: 0.375rem; border: 0; border-radius: 0.5rem;
  padding: 0 0.75rem; font-weight: 700; font-size: 0.8125rem;
  cursor: pointer;
  background: color-mix(in srgb, #f59e0b 12%, transparent);
  color: #b45309;
  border: 1px solid color-mix(in srgb, #f59e0b 25%, transparent);
  transition: transform 0.15s, opacity 0.15s, background 0.15s;
  white-space: nowrap;
}
.pipeline-btn:hover { transform: translateY(-1px); }
.pipeline-btn:disabled { cursor: not-allowed; opacity: 0.5; transform: none; }
</style>
