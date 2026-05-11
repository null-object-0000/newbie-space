import { computed, onMounted, ref } from 'vue'
import { Project, projects as fallbackProjects } from '@/data/projects'
import { getAssetPath } from '@/utils/path'

type GithubRepo = {
  stargazers_count?: number
}

type ProjectStars = {
  stars?: Record<number, number>
}

const projectList = ref<Project[]>(fallbackProjects.map(project => ({ ...project })))
let isLoading = false
let hasLoaded = false

const getRepoApiUrl = (url?: string) => {
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

const applyStars = (stars: Record<number, number>) => {
  projectList.value = projectList.value.map(project => {
    const starCount = stars[project.id]
    return typeof starCount === 'number'
      ? { ...project, stars: starCount }
      : project
  })
}

const fetchRealtimeStars = async () => {
  const results = await Promise.allSettled(
    fallbackProjects.map(async project => {
      const apiUrl = getRepoApiUrl(project.url)
      if (!apiUrl) return null

      const response = await fetch(apiUrl, {
        headers: {
          Accept: 'application/vnd.github+json'
        }
      })

      if (!response.ok) return null

      const repo = await response.json() as GithubRepo
      if (typeof repo.stargazers_count !== 'number') return null

      return {
        id: project.id,
        stars: repo.stargazers_count
      }
    })
  )

  return results.reduce<Record<number, number>>((acc, result) => {
    if (result.status === 'fulfilled' && result.value) {
      acc[result.value.id] = result.value.stars
    }
    return acc
  }, {})
}

const fetchGeneratedStars = async () => {
  const response = await fetch(getAssetPath('/project-stars.json'))
  if (!response.ok) return {}

  const data = await response.json() as ProjectStars
  return data.stars || {}
}

const loadProjectStars = async () => {
  if (isLoading || hasLoaded || typeof window === 'undefined') return

  isLoading = true

  try {
    const realtimeStars = await fetchRealtimeStars()
    const githubProjectCount = fallbackProjects.filter(project => getRepoApiUrl(project.url)).length
    const generatedStars = Object.keys(realtimeStars).length < githubProjectCount
      ? await fetchGeneratedStars()
      : {}
    const stars = {
      ...generatedStars,
      ...realtimeStars
    }

    if (Object.keys(stars).length > 0) {
      applyStars(stars)
    }
  } finally {
    isLoading = false
    hasLoaded = true
  }
}

export function useProjects() {
  onMounted(() => {
    void loadProjectStars()
  })

  return {
    projects: computed(() => projectList.value)
  }
}
