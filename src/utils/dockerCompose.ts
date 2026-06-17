/**
 * Docker Run to Docker Compose 转换器
 */

interface DockerComposeService {
  image?: string
  container_name?: string
  ports?: string[]
  volumes?: string[]
  environment?: string[]
  env_file?: string[]
  working_dir?: string
  networks?: string[]
  restart?: string
  hostname?: string
  user?: string
  privileged?: boolean
  cap_add?: string[]
  cap_drop?: string[]
  labels?: string[]
  entrypoint?: string[]
  command?: string
  healthcheck?: {
    test?: string
    interval?: string
    timeout?: string
    retries?: number
  }
}

interface ParseResult {
  valid: boolean
  error?: string
  service?: DockerComposeService
  serviceName?: string
}

/**
 * 解析 docker run 命令并转换为 docker-compose.yml 格式
 */
export function convertDockerRunToCompose(dockerRunCommand: string): string {
  const result = parseDockerRun(dockerRunCommand)

  if (!result.valid || !result.service) {
    return `# Error: ${result.error || 'Invalid docker run command'}`
  }

  return generateComposeYaml(result.serviceName || 'app', result.service)
}

/**
 * 解析 docker run 命令
 */
function parseDockerRun(command: string): ParseResult {
  const trimmed = command.trim()

  // 检查是否以 docker run 开头
  if (!trimmed.toLowerCase().startsWith('docker run')) {
    return { valid: false, error: '命令必须以 "docker run" 开头' }
  }

  const service: DockerComposeService = {}
  const args = tokenize(trimmed.substring(10).trim()) // 移除 "docker run"

  let i = 0
  let imageFound = false
  let imageName = ''
  let commandArgs: string[] = []

  while (i < args.length) {
    const arg = args[i]

    // 如果已经找到镜像，剩余的都是 command 参数
    if (imageFound) {
      commandArgs.push(arg)
      i++
      continue
    }

    // 解析选项
    if (arg.startsWith('-')) {
      if (arg === '--name' && i + 1 < args.length) {
        service.container_name = args[i + 1]
        i += 2
      } else if (arg === '-p' || arg === '--publish') {
        if (i + 1 < args.length) {
          if (!service.ports) service.ports = []
          service.ports.push(args[i + 1])
          i += 2
        } else {
          i++
        }
      } else if (arg === '-v' || arg === '--volume') {
        if (i + 1 < args.length) {
          if (!service.volumes) service.volumes = []
          service.volumes.push(args[i + 1])
          i += 2
        } else {
          i++
        }
      } else if (arg === '-e' || arg === '--env') {
        if (i + 1 < args.length) {
          if (!service.environment) service.environment = []
          service.environment.push(args[i + 1])
          i += 2
        } else {
          i++
        }
      } else if (arg === '--env-file') {
        if (i + 1 < args.length) {
          if (!service.env_file) service.env_file = []
          service.env_file.push(args[i + 1])
          i += 2
        } else {
          i++
        }
      } else if (arg === '-w' || arg === '--workdir') {
        if (i + 1 < args.length) {
          service.working_dir = args[i + 1]
          i += 2
        } else {
          i++
        }
      } else if (arg === '--network') {
        if (i + 1 < args.length) {
          if (!service.networks) service.networks = []
          service.networks.push(args[i + 1])
          i += 2
        } else {
          i++
        }
      } else if (arg === '--restart') {
        if (i + 1 < args.length) {
          service.restart = args[i + 1]
          i += 2
        } else {
          i++
        }
      } else if (arg === '--hostname') {
        if (i + 1 < args.length) {
          service.hostname = args[i + 1]
          i += 2
        } else {
          i++
        }
      } else if (arg === '--user' || arg === '-u') {
        if (i + 1 < args.length) {
          service.user = args[i + 1]
          i += 2
        } else {
          i++
        }
      } else if (arg === '--privileged') {
        service.privileged = true
        i++
      } else if (arg === '--cap-add') {
        if (i + 1 < args.length) {
          if (!service.cap_add) service.cap_add = []
          service.cap_add.push(args[i + 1])
          i += 2
        } else {
          i++
        }
      } else if (arg === '--cap-drop') {
        if (i + 1 < args.length) {
          if (!service.cap_drop) service.cap_drop = []
          service.cap_drop.push(args[i + 1])
          i += 2
        } else {
          i++
        }
      } else if (arg === '--label' || arg === '-l') {
        if (i + 1 < args.length) {
          if (!service.labels) service.labels = []
          service.labels.push(args[i + 1])
          i += 2
        } else {
          i++
        }
      } else if (arg === '--entrypoint') {
        if (i + 1 < args.length) {
          service.entrypoint = args[i + 1].split(' ')
          i += 2
        } else {
          i++
        }
      } else if (arg === '-d' || arg === '--detach') {
        // 忽略，compose 中不需要
        i++
      } else if (arg === '--health-cmd') {
        if (i + 1 < args.length) {
          if (!service.healthcheck) service.healthcheck = {}
          service.healthcheck.test = ['CMD-SHELL', args[i + 1]] as any
          i += 2
        } else {
          i++
        }
      } else if (arg === '--health-interval') {
        if (i + 1 < args.length) {
          if (!service.healthcheck) service.healthcheck = {}
          service.healthcheck.interval = args[i + 1]
          i += 2
        } else {
          i++
        }
      } else if (arg === '--health-timeout') {
        if (i + 1 < args.length) {
          if (!service.healthcheck) service.healthcheck = {}
          service.healthcheck.timeout = args[i + 1]
          i += 2
        } else {
          i++
        }
      } else if (arg === '--health-retries') {
        if (i + 1 < args.length) {
          if (!service.healthcheck) service.healthcheck = {}
          service.healthcheck.retries = parseInt(args[i + 1], 10)
          i += 2
        } else {
          i++
        }
      } else {
        // 未知选项，跳过
        i++
      }
    } else {
      // 这是镜像名称
      imageName = arg
      imageFound = true
      i++
    }
  }

  if (!imageName) {
    return { valid: false, error: '未找到镜像名称' }
  }

  service.image = imageName

  if (commandArgs.length > 0) {
    service.command = commandArgs.join(' ')
  }

  // 从镜像名称生成服务名
  const serviceName = generateServiceName(imageName)

  return { valid: true, service, serviceName }
}

/**
 * 将命令字符串分割为参数数组，处理引号
 */
function tokenize(command: string): string[] {
  const tokens: string[] = []
  let current = ''
  let inQuote = false
  let quoteChar = ''

  for (let i = 0; i < command.length; i++) {
    const char = command[i]

    if (!inQuote && (char === '"' || char === "'")) {
      inQuote = true
      quoteChar = char
    } else if (inQuote && char === quoteChar) {
      inQuote = false
      quoteChar = ''
    } else if (!inQuote && char === ' ') {
      if (current) {
        tokens.push(current)
        current = ''
      }
    } else {
      current += char
    }
  }

  if (current) {
    tokens.push(current)
  }

  return tokens
}

/**
 * 从镜像名称生成服务名
 */
function generateServiceName(image: string): string {
  // 移除标签和 registry
  let name = image

  // 移除 @sha256:...
  if (name.includes('@')) {
    name = name.split('@')[0]
  }

  // 移除 :tag
  if (name.includes(':')) {
    name = name.split(':')[0]
  }

  // 取最后一部分（移除 registry/repo）
  const parts = name.split('/')
  name = parts[parts.length - 1]

  // 清理非法字符
  name = name.replace(/[^a-zA-Z0-9-_]/g, '-')

  return name || 'app'
}

/**
 * 生成 docker-compose.yml 内容
 */
function generateComposeYaml(serviceName: string, service: DockerComposeService): string {
  const lines: string[] = [
    'version: "3.8"',
    '',
    'services:',
    `  ${serviceName}:`
  ]

  if (service.image) {
    lines.push(`    image: ${service.image}`)
  }

  if (service.container_name) {
    lines.push(`    container_name: ${service.container_name}`)
  }

  if (service.ports && service.ports.length > 0) {
    lines.push('    ports:')
    service.ports.forEach(p => {
      lines.push(`      - "${p}"`)
    })
  }

  if (service.volumes && service.volumes.length > 0) {
    lines.push('    volumes:')
    service.volumes.forEach(v => {
      lines.push(`      - ${v}`)
    })
  }

  if (service.environment && service.environment.length > 0) {
    lines.push('    environment:')
    service.environment.forEach(e => {
      lines.push(`      - ${e}`)
    })
  }

  if (service.env_file && service.env_file.length > 0) {
    lines.push('    env_file:')
    service.env_file.forEach(f => {
      lines.push(`      - ${f}`)
    })
  }

  if (service.working_dir) {
    lines.push(`    working_dir: ${service.working_dir}`)
  }

  if (service.networks && service.networks.length > 0) {
    lines.push('    networks:')
    service.networks.forEach(n => {
      lines.push(`      - ${n}`)
    })
  }

  if (service.restart) {
    lines.push(`    restart: ${service.restart}`)
  }

  if (service.hostname) {
    lines.push(`    hostname: ${service.hostname}`)
  }

  if (service.user) {
    lines.push(`    user: ${service.user}`)
  }

  if (service.privileged) {
    lines.push('    privileged: true')
  }

  if (service.cap_add && service.cap_add.length > 0) {
    lines.push('    cap_add:')
    service.cap_add.forEach(c => {
      lines.push(`      - ${c}`)
    })
  }

  if (service.cap_drop && service.cap_drop.length > 0) {
    lines.push('    cap_drop:')
    service.cap_drop.forEach(c => {
      lines.push(`      - ${c}`)
    })
  }

  if (service.labels && service.labels.length > 0) {
    lines.push('    labels:')
    service.labels.forEach(l => {
      lines.push(`      - ${l}`)
    })
  }

  if (service.entrypoint) {
    lines.push(`    entrypoint: [${service.entrypoint.map(e => `"${e}"`).join(', ')}]`)
  }

  if (service.command) {
    lines.push(`    command: ${service.command}`)
  }

  if (service.healthcheck) {
    lines.push('    healthcheck:')
    if (service.healthcheck.test) {
      lines.push(`      test: [${(service.healthcheck.test as any[]).map(t => `"${t}"`).join(', ')}]`)
    }
    if (service.healthcheck.interval) {
      lines.push(`      interval: ${service.healthcheck.interval}`)
    }
    if (service.healthcheck.timeout) {
      lines.push(`      timeout: ${service.healthcheck.timeout}`)
    }
    if (service.healthcheck.retries !== undefined) {
      lines.push(`      retries: ${service.healthcheck.retries}`)
    }
  }

  return lines.join('\n')
}
