# 项目规范

## 新增工具

每个工具需要创建/修改以下文件：

### 必须

| 文件 | 说明 |
|------|------|
| `src/views/<Name>.vue` | 页面组件，按下方模板 |
| `src/data/tools.ts` | 注册元数据（含 `inputType` / `outputType`），**标题和描述的唯一数据源** |
| `src/router/index.ts` | 路由定义 + `getStaticRoutes()` 中添加静态路由 |
| `src/data/nav-data.json` | `自有工具` 分类下添加入口 |

### 可选

| 文件 | 说明 |
|------|------|
| `src/utils/<name>.ts` | 核心逻辑，纯函数，有可复用逻辑时抽取 |
| `bun add <lib>` | 如需第三方库 |
| `src/utils/toolPipeline.ts` | 如需新的流转数据类型 |

### 元数据唯一来源

`src/data/tools.ts` 是工具标题、描述的**唯一数据源**。页面和导航都从它读取：

```typescript
// 每个工具页面 script setup 中
import { findTool } from '@/utils/toolPipeline'
const tool = findTool('<tool-id>')
```

```html
<!-- 模板中使用 tool 显示标题和描述 -->
<h1>{{ tool?.name }}</h1>
<p>{{ tool?.desc }}</p>
```

`nav-data.json` 中的工具条目同步更新，保持 name / desc / link 与 tools.ts 一致。

### 页面模板规范

```vue
<template>
  <div class="tool-page" :class="{ 'dark-mode': isDark, 'light-mode': !isDark }">
    <AppHeader />
    <main class="tool-main">
      <!-- 返回链接 -->
      <div class="tool-topbar">
        <router-link to="/tools" class="back-link">
          <ArrowLeft :size="16" /><span>工具中心</span>
        </router-link>
      </div>

      <!-- 标题：图标 + h1 + 描述，内容来自 findTool() -->
      <section class="tool-header">
        <div class="tool-heading">
          <div class="heading-icon"><Icon :size="22" /></div>
          <div>
            <h1>{{ tool?.name }}</h1>
            <p>{{ tool?.desc }}</p>
          </div>
        </div>
      </section>

      <!-- 流转来源横幅（使用 usePipeline 自动提供 pipelineFrom） -->
      <div v-if="pipelineFrom" class="pipeline-banner">
        <ArrowRightLeft :size="14" />
        <span>来自「{{ pipelineFrom }}」的流转数据</span>
      </div>

      <!-- 双栏工作区：≥768px 左右并排 -->
      <div class="workspace">
        <div class="panel panel-left"><!-- 输入/控制 --></div>
        <div class="panel panel-right"><!-- 输出/预览 --></div>
      </div>
    </main>

    <!-- Toast -->
    <Transition name="toast">
      <div v-if="toastMessage" class="toast" :class="toastType">{{ toastMessage }}</div>
    </Transition>
  </div>
</template>
```

### 按钮规范

```html
<!-- 主操作 -->
<button class="btn primary" :disabled="!ready" @click="action">...</button>
<!-- 次要操作 -->
<button class="btn secondary" :disabled="!ready" @click="action">...</button>
```

```css
.btn {
  min-height: 2.25rem;
  display: inline-flex; align-items: center; justify-content: center;
  gap: 0.375rem; border: 0; border-radius: 0.625rem;
  padding: 0 0.875rem; font-weight: 700; font-size: 0.8125rem;
  cursor: pointer;
  transition: transform 0.15s, opacity 0.15s, background 0.15s;
}
.btn.primary { background: var(--brand-500); color: #fff; }
.btn.secondary { background: var(--bg-elevated); color: var(--text-primary); }
.btn:hover { transform: translateY(-1px); }
.btn:disabled { cursor: not-allowed; opacity: 0.5; transform: none; }
```

### 流转

```typescript
import { usePipeline, type PipelineIncoming } from '@/composables/usePipeline'
import type { ToolItem } from '@/data/tools'
import PipelineSend from '@/components/tools/PipelineSend.vue'

const { pipelineFrom, downstreamTools, sendTextTo, sendImageTo } = usePipeline({
  toolId: '<tool-id>',
  async onIncoming(incoming: PipelineIncoming) {
    // 消费上游传来的数据，返回 true 表示成功
  }
})

function handlePipelineSend(target: ToolItem) {
  const { ok, message } = sendTextTo(target, resultText) // 或 sendImageTo
  showToast(message, ok ? 'success' : 'error')
}
```

模板中：
```html
<PipelineSend
  :tools="downstreamTools"
  :disabled="!ready"
  @send="handlePipelineSend"
/>
```

`PipelineSend` 组件路径：`@/components/tools/PipelineSend.vue`。当下游工具列表为空时自动隐藏，无需手动 `v-if`。

### 模式切换（segmented control）

用于编码、格式、算法等切换的通用模式：

```html
<div class="segmented">
  <button :class="{ active: mode === 'a' }" @click="mode = 'a'">A</button>
  <button :class="{ active: mode === 'b' }" @click="mode = 'b'">B</button>
</div>
```

```css
.segmented {
  display: flex;
  padding: 0.1875rem; border-radius: 0.375rem;
  background: var(--bg-elevated);
}
.segmented button {
  flex: 1;
  min-height: 1.75rem; padding: 0 0.5rem;
  border: 0; border-radius: 0.25rem;
  background: transparent; color: var(--text-secondary);
  font-size: 0.6875rem; font-weight: 600; cursor: pointer;
}
.segmented button.active {
  background: var(--bg-surface); color: var(--text-primary); box-shadow: var(--shadow-1);
}
```

### 工具注册

```typescript
// src/data/tools.ts
{
  id: '<kebab-case>',
  name: '<中文名>',
  desc: '<一句话描述>',
  path: '/tools/<kebab-case>',
  icon: <LucideIcon>,
  tags: ['标签1', '标签2', '本地运行'],
  inputType: ['image' | 'text'][],   // 该工具接受的输入类型
  outputType: 'image' | 'text'       // 该工具产生的输出类型
}
```

### 导航注册

```jsonc
// src/data/nav-data.json → "自有工具" → "links"
{
  "name": "<中文名>",
  "icon": "<lucide-icon-kebab>",
  "link": "/tools/<kebab-case>",
  "desc": "<与 tools.ts 中一致的描述>"
}
```

### 路由

```typescript
// 动态路由
{ path: '/tools/<kebab>', name: '<kebab>', component: () => import('@/views/<Name>.vue') }
// 静态路由（getStaticRoutes 中）
'/tools/<kebab>'
```

### 样式要点

- **主题色**：每个工具用独立主题色区分
  - `#10b981` 绿 — 图片处理类
  - `#f59e0b` 橙 — 图片调整类
  - `#8b5cf6` 紫 — 生成器类
  - `#3b82f6` 蓝 — 文本处理类
  - `#ef4444` 红 — 加密/哈希类
  - `#06b6d4` 青 — 解码类
- **布局**：CSS Grid 双栏，紧凑间距（`gap: 0.75rem ~ 1rem`）
- **按钮**：`min-height: 2.25rem`，`border-radius: 0.625rem`，`font-size: 0.8125rem`
- **标题**：h1 `1.375rem`，描述 `0.8125rem`
- **Toast**：底部居中，绿成功 / 红失败

### 历史记录（可选）

- 用 `localStorage`，key 以工具 ID 命名避免冲突
- 最多 5 条，去重，新记录放最前

### 依赖安装

```bash
bun add <package>
```
