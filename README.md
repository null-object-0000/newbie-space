# Newbie Space

一个基于 Vue 3 + Vite + SSG 的个人博客和导航站点，支持自动从 Markdown 文件读取文章，并提供丰富的自定义 Markdown 语法。

## ✨ 特性

- 📝 **自动文章管理**：从 Markdown 文件自动读取文章，无需手动维护文章列表
  - 自动提取文章标题（从第一个 `#` 标题）
  - 自动生成文章摘要（前 200 字符）
  - 自动计算阅读时间（中文 300 字/分钟，英文 200 词/分钟）
- 🎨 **自定义 Markdown 语法**：扩展了图片画廊、时间线、折叠详情块等语法
- 🚀 **静态站点生成**：使用 `vite-ssg` 生成静态 HTML，部署简单
- 📱 **响应式设计**：完美适配桌面端和移动端
- 🎯 **导航管理**：支持快速链接导航，可通过脚本添加新链接

## 🚀 快速开始

### 生产站点与自动部署

生产域名为 https://snewbie.site/，由 EdgeOne 全球可用区（含中国大陆）提供服务。

提交或合并到 `main` 后，GitHub Actions 会按根路径 `/` 构建并自动部署到 EdgeOne。工作流使用仓库 Secrets `TENCENT_CLOUD_SECRET_ID` 和 `TENCENT_CLOUD_SECRET_KEY`，也可在 Actions 中手动运行。仅发布 Tag 或 Release 不会单独触发部署。

Cloudflare Pages 保留为备用入口；GitHub Pages 和 OSS 的现有工作流保留。网站迁移只调整主域名的网站 CNAME 和新增 EdgeOne 验证记录，MX、SPF、DKIM 及邮件服务相关 CNAME 保持不变。

### 安装依赖

```bash
npm install
# 或
pnpm install
```

### 开发

```bash
npm run dev
```

### 构建

```bash
npm run build
```

### 预览构建结果

```bash
npm run preview
```

## 📁 项目结构

```
newbie-home/
├── src/
│   ├── components/          # Vue 组件
│   │   ├── blog/            # 博客相关组件
│   │   │   ├── ImageGallery.vue    # 图片画廊
│   │   │   ├── ImageViewer.vue     # 图片查看器
│   │   │   ├── Timeline.vue         # 时间线
│   │   │   └── MarkdownRenderer.vue # Markdown 渲染器
│   │   ├── nav/             # 导航相关组件
│   │   ├── AppHeader.vue     # 页面头部
│   │   └── AppFooter.vue     # 页面底部
│   ├── views/               # 页面视图
│   │   ├── Home.vue         # 首页
│   │   ├── Blog.vue          # 博客列表
│   │   ├── Post.vue          # 文章详情
│   │   ├── Nav.vue           # 导航页面
│   │   └── Projects.vue     # 项目展示
│   ├── content/             # 内容文件
│   │   └── posts/           # 博客文章（Markdown 文件）
│   ├── data/                # 数据文件
│   │   ├── posts.ts         # 文章索引（自动生成）
│   │   ├── projects.ts      # 项目数据
│   │   └── nav-data.json    # 导航数据
│   ├── composables/         # 组合式函数
│   │   ├── useMarkdown.ts   # Markdown 处理
│   │   └── useTheme.ts      # 主题切换
│   └── router/              # 路由配置
├── public/                  # 静态资源
│   ├── posts/               # 文章图片
│   └── icons/               # 图标文件
└── scripts/                 # 工具脚本
    └── add-nav.js           # 添加导航链接
```

## 📝 编写博客文章

### 创建新文章

在 `src/content/posts/` 目录下创建新的 Markdown 文件（如 `2025-05.md`）。

### Frontmatter 规范

每个 Markdown 文件需要在开头包含 frontmatter：

```yaml
---
date: 2025-05
tags:
  - 标签1
  - 标签2
cover: null  # 或图片路径
---

# 文章标题（会自动提取）

文章正文内容...
```

**必填字段：**
- `date`: 发布日期，格式 `YYYY-MM` 或 `YYYY-MM-DD`

**可选字段：**
- `tags`: 标签数组
- `cover`: 封面图片路径，`null` 表示无封面

**自动提取：**
- `title`: 从第一个 `#` 标题自动提取
- `excerpt`: 从文章开头自动提取前 200 字符
- `readTime`: 根据文章字数自动计算

### 示例

```markdown
---
date: 2025-05
tags:
  - Vue
  - 前端
  - 教程
cover: null
---

# Vue 3 组合式 API 入门指南

这是一篇关于 Vue 3 组合式 API 的文章...

## 什么是组合式 API

组合式 API 是 Vue 3 引入的新特性...
```

## 🎨 自定义 Markdown 语法

### 图片画廊 (:::images)

用于展示多张图片，支持点击查看大图。

```markdown
:::images
![图片描述1](/posts/2025-04/web_01.png)
![图片描述2](/posts/2025-04/web_02.png)
![图片描述3](/posts/2025-04/web_03.png)
:::
```

**特性：**
- 根据图片数量自动调整布局（1-5 张图片有不同的布局）
- 点击图片可在全屏查看器中查看
- 支持键盘导航（`←` / `→` 切换，`ESC` 关闭）
- 响应式设计，移动端自动适配

### 时间线 (:::timeline)

用于展示时间线事件，适合记录问题排查过程、操作步骤等。

```markdown
:::timeline
- **2025-04-09 18:16:35** 业务报演出票务后台访问异常、无法访问
- **2025-04-09 18:19:56** 通过 CI/CD 平台发现机器异常，请求全部 500
- **2025-04-09 18:22:34** 将异常机器剔除负载后应用访问正常
:::
```

**特性：**
- 时间戳使用 `**粗体**` 标记会以品牌色显示
- 自动添加时间线连接和标记点
- 响应式设计

### 折叠详情块 (:::details)

用于创建可折叠的内容块，适合放置长代码、详细说明等。

```markdown
:::details 查看完整代码
# 这里是代码
cat /data/logs/app.log
:::
```

**特性：**
- 点击标题可展开/折叠内容
- 内容区域支持所有 Markdown 语法
- 适合放置长代码块和详细日志

## 🛠️ 工具脚本

### 添加导航链接

```bash
npm run add-nav
```

运行后会交互式地添加新的导航链接到 `src/data/nav-data.json`。

## 🎯 技术栈

- **框架**: Vue 3 (Composition API)
- **构建工具**: Vite
- **静态生成**: vite-ssg
- **路由**: Vue Router 4
- **Markdown 解析**: markdown-it + markdown-it-container
- **样式**: CSS Variables + 响应式设计
- **字体**: Noto Sans SC + Outfit

## 📦 主要依赖

### 生产依赖

- `vue`: ^3.4.0
- `vue-router`: ^4.6.4
- `markdown-it`: ^14.1.0
- `markdown-it-container`: ^4.0.0
- `lucide-vue-next`: ^0.562.0 (图标库)

### 开发依赖

- `vite`: ^5.4.21
- `vite-ssg`: ^0.23.8
- `typescript`: ^5.9.3
- `@vitejs/plugin-vue`: ^5.2.4

## 🔧 配置说明

### Vite 配置

项目使用 Vite 作为构建工具，配置文件为 `vite.config.ts`。

主要配置：
- 路径别名：`@` 指向 `src` 目录
- SSG 选项：异步脚本加载、代码压缩等

### 路由配置

路由配置在 `src/router/index.ts`，支持：
- 首页 (`/`)
- 导航页 (`/nav/`)
- 博客列表 (`/posts`)
- 文章详情 (`/posts/:slug`)
- 项目展示 (`/projects`)

所有博客文章路由会在构建时自动生成。

## 📄 许可证

MIT

## 🙏 致谢

- [Vue.js](https://vuejs.org/)
- [Vite](https://vitejs.dev/)
- [vite-ssg](https://github.com/antfu/vite-ssg)
- [markdown-it](https://github.com/markdown-it/markdown-it)
