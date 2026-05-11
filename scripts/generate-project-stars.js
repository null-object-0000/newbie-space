/**
 * 构建前生成项目 GitHub star 数据。
 *
 * 运行时会优先实时请求 GitHub API；失败时使用同源的 /project-stars.json 作为兜底。
 */

import fs from 'fs/promises'
import path from 'path'
import ts from 'typescript'
import { fileURLToPath } from 'url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const rootDir = path.resolve(__dirname, '..')
const projectsFile = path.join(rootDir, 'src/data/projects.ts')
const outputFile = path.join(rootDir, 'public/project-stars.json')
const requestTimeout = 8000

async function loadProjects() {
  const source = await fs.readFile(projectsFile, 'utf-8')
  const { outputText } = ts.transpileModule(source, {
    compilerOptions: {
      module: ts.ModuleKind.ESNext,
      target: ts.ScriptTarget.ES2020
    }
  })

  const moduleUrl = `data:text/javascript;base64,${Buffer.from(outputText).toString('base64')}`
  const module = await import(moduleUrl)
  return module.projects || []
}

function getRepoApiUrl(url) {
  if (!url) return null

  try {
    const { hostname, pathname } = new URL(url)
    if (hostname !== 'github.com') return null

    const [owner, repo] = pathname.split('/').filter(Boolean)
    if (!owner || !repo) return null

    return `https://api.github.com/repos/${owner}/${repo}`
  } catch {
    return null
  }
}

async function fetchRepoStars(project) {
  const apiUrl = getRepoApiUrl(project.url)
  if (!apiUrl) return null

  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), requestTimeout)

  try {
    const response = await fetch(apiUrl, {
      headers: {
        Accept: 'application/vnd.github+json',
        'User-Agent': 'newbie-space-build'
      },
      signal: controller.signal
    })

    if (!response.ok) {
      console.warn(`⚠️  ${project.name} stars 获取失败: HTTP ${response.status}`)
      return null
    }

    const repo = await response.json()
    if (typeof repo.stargazers_count !== 'number') {
      console.warn(`⚠️  ${project.name} stars 响应缺少 stargazers_count`)
      return null
    }

    return {
      id: project.id,
      stars: repo.stargazers_count
    }
  } catch (error) {
    console.warn(`⚠️  ${project.name} stars 获取失败: ${error.message}`)
    return null
  } finally {
    clearTimeout(timer)
  }
}

async function readExistingStars() {
  try {
    const content = await fs.readFile(outputFile, 'utf-8')
    const data = JSON.parse(content)
    return data.stars || {}
  } catch {
    return {}
  }
}

async function main() {
  console.log('🚀 生成项目 GitHub stars\n')

  const projects = await loadProjects()
  const githubProjects = projects.filter(project => getRepoApiUrl(project.url))

  if (githubProjects.length === 0) {
    console.warn('⚠️  未找到 GitHub 项目，跳过生成')
    return
  }

  const existingStars = await readExistingStars()
  const results = await Promise.all(githubProjects.map(fetchRepoStars))
  const latestStars = results.reduce((acc, result) => {
    if (result) {
      acc[result.id] = result.stars
    }
    return acc
  }, {})
  const stars = {
    ...existingStars,
    ...latestStars
  }

  await fs.mkdir(path.dirname(outputFile), { recursive: true })
  await fs.writeFile(outputFile, JSON.stringify({
    updatedAt: new Date().toISOString(),
    stars
  }, null, 2), 'utf-8')

  console.log(`\n✅ 项目 stars 已生成: ${outputFile}`)
  console.log(`   本次获取 ${Object.keys(latestStars).length} 个项目，文件保留 ${Object.keys(stars).length} 个项目`)
}

main().catch(error => {
  console.error('❌ 生成项目 stars 失败:', error)
  process.exit(1)
})
