// 项目数据
export interface Project {
  id: number
  name: string
  stars: number | string
  desc: string
  stack: string[]
  url?: string
  demoUrl?: string
}

export const projects: Project[] = [
  {
    id: 1,
    name: 'Newbie Space',
    stars: 'New',
    desc: '一个基于 Vue 3 的极简个人博客主题，支持暗黑模式和动态路由。',
    stack: ['vue'],
    url: 'https://github.com/null-object-0000/newbie-space',
    demoUrl: 'https://null-object-0000.github.io/newbie-space/'
  },
  {
    id: 2,
    name: 'HTTP Log Snap',
    stars: 'New',
    desc: '一个轻量级的 Java HTTP 请求/响应日志记录库，快照式捕获完整的 HTTP 交互。同时支持客户端（如 OkHttp）和服务端（如 Spring MVC）场景。',
    stack: ['java'],
    url: 'https://github.com/null-object-0000/http-log-snap'
  },
  {
    id: 3,
    name: 'AI Clash',
    stars: 2,
    desc: '告别来回切网页！在侧边栏一键向 DeepSeek、豆包、千问等多个 AI 同步提问，并自动为你提炼最终的“最优总结”。',
    stack: ['chrome-extension'],
    url: 'https://github.com/null-object-0000/ai-clash'
  },
  {
    id: 4,
    name: 'Newbie Java Doctor',
    stars: 'New',
    desc: '你的 Java 全链路性能调优沙盒与容量规划可视化推演平台。',
    stack: ['java', 'vue'],
    url: 'https://github.com/null-object-0000/newbie-java-doctor',
    demoUrl: 'https://null-object-0000.github.io/newbie-java-doctor/'
  }
]

export function getProjectById(id: number): Project | undefined {
  return projects.find(p => p.id === id)
}
