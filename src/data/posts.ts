/**
 * 博客文章索引
 * 
 * Markdown Frontmatter 规范：
 * ---
 * date: 发布日期，格式 YYYY-MM 或 YYYY-MM-DD（必填）
 * author: 作者名称（可选，默认为 "Newbie Space"）
 * series: 系列名称（可选），如 每月故障、每日一题
 * tags: 标签数组（可选）
 *   - 标签1
 *   - 标签2
 * cover: 封面图片路径（可选，null 表示无封面）
 * ---
 * 
 * 注意：
 * - 文章标题会自动从 markdown 文件的第一个 # 标题中提取
 * - 文章摘要会自动从文章开头的内容中提取（前 200 字符）
 * - 阅读时间会根据文章字数自动计算（中文 300 字/分钟，英文 200 词/分钟）
 */

import { parseFrontmatter, extractTitleFromMarkdown, extractExcerptFromMarkdown, calculateReadTime } from '@/composables/useMarkdown'
import gitTimestamps from './git-timestamps.json'

export interface PostMeta {
  slug: string
  title: string
  date: string
  author?: string         // 作者名称
  series?: string         // 系列，如 每月故障、每日一题
  excerpt?: string
  readTime?: number
  tags?: string[]
  cover?: string | null
  lastModified?: string  // Git 最后修改时间
  created?: string        // Git 创建时间
}

// 使用 Vite 的 import.meta.glob 动态导入所有 markdown 文件
const postModules = import.meta.glob<string>('@/content/posts/*.md', {
  query: '?raw',
  eager: true,
  import: 'default'
})

// 解析所有文章的 frontmatter
function parseAllPosts(): PostMeta[] {
  const posts: PostMeta[] = []
  
  for (const [path, content] of Object.entries(postModules)) {
    // 从路径中提取 slug（文件名不含扩展名）
    const match = path.match(/\/([^/]+)\.md$/)
    if (!match) continue
    
    const slug = match[1]
    const { frontmatter } = parseFrontmatter(content)
    
    // 从 markdown 内容中提取标题（第一个 # 标题）
    const title = extractTitleFromMarkdown(content)
    
    // 验证必填字段
    if (!title) {
      console.warn(`[posts] 文章 ${slug} 缺少标题（markdown 文件中应包含 # 标题）`)
      continue
    }
    
    if (!frontmatter.date) {
      console.warn(`[posts] 文章 ${slug} 缺少必填的 frontmatter 字段 (date)`)
      continue
    }
    
    // 自动提取摘要和计算阅读时间
    const excerpt = extractExcerptFromMarkdown(content)
    const readTime = calculateReadTime(content)
    
    // 获取 git 时间戳
    const timestamps = gitTimestamps[slug as keyof typeof gitTimestamps]
    
    posts.push({
      slug,
      title,
      date: frontmatter.date,
      author: frontmatter.author || 'Newbie Space',  // 默认作者
      series: frontmatter.series,
      excerpt: excerpt || undefined,
      readTime,
      tags: frontmatter.tags,
      cover: frontmatter.cover,
      lastModified: timestamps?.lastModified,
      created: timestamps?.created
    })
  }
  
  // 按日期倒序排序（最新的在前）
  return posts.sort((a, b) => b.date.localeCompare(a.date))
}

// 导出文章列表
export const posts: PostMeta[] = parseAllPosts()

export function getPostBySlug(slug: string): PostMeta | undefined {
  return posts.find(p => p.slug === slug)
}

export function getAllPosts(): PostMeta[] {
  return posts
}

/** 按系列筛选文章，返回该系列下按日期倒序的文章列表 */
export function getPostsBySeries(series: string): PostMeta[] {
  return posts.filter(p => p.series === series)
}
