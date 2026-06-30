<template>
  <div class="key-value-editor">
    <div class="kv-head">
      <span></span>
      <span>Key</span>
      <span>Value</span>
      <span></span>
    </div>
    <div v-for="(item, index) in model" :key="index" class="kv-row">
      <input v-model="item.enabled" type="checkbox" aria-label="启用此项" />
      <input v-model="item.key" :placeholder="keyPlaceholder" @input="ensureBlankRow" />
      <input v-model="item.value" :placeholder="valuePlaceholder" @input="ensureBlankRow" />
      <button v-if="model.length > 1" class="remove" aria-label="删除此项" title="删除" @click="remove(index)">×</button>
    </div>
  </div>
</template>
<script setup lang="ts">
type KeyValue = { enabled: boolean; key: string; value: string }
const model = defineModel<KeyValue[]>({ required: true })
defineProps<{ keyPlaceholder: string; valuePlaceholder: string }>()
function ensureBlankRow() { const last = model.value[model.value.length - 1]; if (last && (last.key || last.value)) model.value.push({ enabled: true, key: '', value: '' }) }
function remove(index: number) { model.value.splice(index, 1); if (!model.value.length) model.value.push({ enabled: true, key: '', value: '' }) }
</script>
<style scoped>
.key-value-editor {
  overflow: hidden;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  background: var(--bg-surface);
}

.kv-head,
.kv-row {
  display: grid;
  grid-template-columns: 2.25rem minmax(0, 0.9fr) minmax(0, 1.1fr) 2rem;
  align-items: center;
}

.kv-head {
  min-height: 2rem;
  border-bottom: 1px solid var(--border-color);
  background: color-mix(in srgb, var(--bg-elevated) 70%, transparent);
  color: var(--text-muted);
  font-size: 0.6875rem;
  font-weight: 800;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.kv-row {
  min-height: 2.625rem;
  border-bottom: 1px solid var(--border-color);
}

.kv-row:last-child {
  border-bottom: 0;
}

.kv-row input[type='checkbox'] {
  justify-self: center;
  accent-color: var(--brand-500);
}

.kv-row input:not([type='checkbox']) {
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

.kv-row input:not([type='checkbox']):focus {
  background: color-mix(in srgb, var(--brand-500) 8%, transparent);
}

.remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  justify-self: center;
  width: 1.5rem;
  height: 1.5rem;
  border: 0;
  border-radius: 0.25rem;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 1.125rem;
  line-height: 1;
}

.remove:hover {
  background: #fee2e2;
  color: #ef4444;
}

@media (max-width: 560px) {
  .kv-head {
    display: none;
  }

  .kv-row {
    grid-template-columns: 2rem minmax(0, 1fr) 2rem;
    grid-template-rows: repeat(2, 2.5rem);
  }

  .kv-row input:not([type='checkbox']) {
    grid-column: 2;
    height: 2.5rem;
    border-left: 1px solid var(--border-color);
  }

  .kv-row input:not([type='checkbox']) + input:not([type='checkbox']) {
    grid-row: 2;
  }

  .remove {
    grid-column: 3;
    grid-row: 1 / span 2;
  }
}
</style>
