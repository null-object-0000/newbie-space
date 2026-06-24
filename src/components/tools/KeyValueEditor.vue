<template>
  <div class="key-value-editor">
    <div v-for="(item, index) in model" :key="index" class="kv-row">
      <input v-model="item.enabled" type="checkbox" aria-label="启用此项" />
      <input v-model="item.key" :placeholder="keyPlaceholder" @input="ensureBlankRow" />
      <input v-model="item.value" :placeholder="valuePlaceholder" @input="ensureBlankRow" />
      <button v-if="model.length > 1" class="remove" aria-label="删除此项" @click="remove(index)">×</button>
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
.key-value-editor { display: flex; flex-direction: column; gap: .5rem; }.kv-row { display: grid; grid-template-columns: auto minmax(0, .8fr) minmax(0, 1fr) 1.5rem; gap: .375rem; align-items: center; }.kv-row input[type='checkbox'] { accent-color: var(--brand-500); }.kv-row input:not([type='checkbox']) { min-width: 0; height: 2.25rem; box-sizing: border-box; border: 1px solid var(--border-color); border-radius: .375rem; background: var(--bg-elevated); color: var(--text-primary); padding: 0 .5rem; font: .8125rem var(--font-family-mono, monospace); outline: none; }.kv-row input:focus { border-color: var(--brand-500); }.remove { border: 0; background: transparent; color: var(--text-muted); cursor: pointer; font-size: 1.25rem; }.remove:hover { color: #ef4444; }
</style>
