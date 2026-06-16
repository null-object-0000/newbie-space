export type TransparentMode = 'background' | 'global'

export interface ProcessOptions {
  targetColor: string
  tolerance: number
  mode: TransparentMode
}

export interface ProcessStats {
  totalPixels: number
  transparentPixels: number
  width: number
  height: number
}

export interface ProcessResult {
  imageData: ImageData
  stats: ProcessStats
}

export function rgbDistance(r1: number, g1: number, b1: number, r2: number, g2: number, b2: number): number {
  const dr = r1 - r2
  const dg = g1 - g2
  const db = b1 - b2
  return Math.sqrt(dr * dr + dg * dg + db * db)
}

export function calculateAlpha(dist: number, coreThreshold: number, feather: number, originalAlpha: number): number {
  if (dist < coreThreshold) return 0
  if (dist >= coreThreshold + feather) return originalAlpha

  const t = (dist - coreThreshold) / feather
  const smoothT = t * t * (3 - 2 * t)
  return Math.round(originalAlpha * smoothT)
}

export function parseHexColor(hex: string): { r: number; g: number; b: number } {
  const normalized = hex.trim()
  if (!/^#[0-9a-fA-F]{6}$/.test(normalized)) {
    throw new Error('目标颜色格式无效')
  }

  return {
    r: parseInt(normalized.slice(1, 3), 16),
    g: parseInt(normalized.slice(3, 5), 16),
    b: parseInt(normalized.slice(5, 7), 16)
  }
}

export function processImageData(source: ImageData, options: ProcessOptions): ProcessResult {
  const { r, g, b } = parseHexColor(options.targetColor)
  const width = source.width
  const height = source.height
  const outData = new Uint8ClampedArray(source.data)
  const imageData = new ImageData(outData, width, height)

  const tolerance = Math.max(0, Math.min(100, options.tolerance))
  const coreThreshold = tolerance * 3.0 + 2
  const feather = Math.max(8, tolerance * 0.5 + 10)
  const totalThreshold = coreThreshold + feather

  if (options.mode === 'global') {
    processGlobal(imageData, r, g, b, coreThreshold, feather)
  } else {
    processBackgroundBFS(imageData, r, g, b, coreThreshold, feather, totalThreshold)
  }

  const totalPixels = width * height
  let transparentPixels = 0
  for (let i = 0; i < totalPixels; i++) {
    if (imageData.data[i * 4 + 3] < 255) {
      transparentPixels++
    }
  }

  return {
    imageData,
    stats: {
      totalPixels,
      transparentPixels,
      width,
      height
    }
  }
}

function processGlobal(
  imageData: ImageData,
  targetR: number,
  targetG: number,
  targetB: number,
  coreThreshold: number,
  feather: number
) {
  const data = imageData.data
  const totalPixels = imageData.width * imageData.height

  for (let i = 0; i < totalPixels; i++) {
    const idx = i * 4
    const a = data[idx + 3]
    if (a === 0) continue

    const dist = rgbDistance(data[idx], data[idx + 1], data[idx + 2], targetR, targetG, targetB)
    data[idx + 3] = calculateAlpha(dist, coreThreshold, feather, a)
  }
}

function processBackgroundBFS(
  imageData: ImageData,
  targetR: number,
  targetG: number,
  targetB: number,
  coreThreshold: number,
  feather: number,
  totalThreshold: number
) {
  const width = imageData.width
  const height = imageData.height
  const data = imageData.data
  const totalPixels = width * height
  const visited = new Uint8Array(totalPixels)
  const queue = new Int32Array(totalPixels * 2)
  let head = 0
  let tail = 0

  const isConnected = (idx: number) => {
    const a = data[idx + 3]
    if (a < 64) return true

    const dist = rgbDistance(data[idx], data[idx + 1], data[idx + 2], targetR, targetG, targetB)
    return dist < totalThreshold
  }

  const enqueueEdge = (x: number, y: number) => {
    const pixelIndex = y * width + x
    if (visited[pixelIndex]) return

    const idx = pixelIndex * 4
    if (isConnected(idx)) {
      visited[pixelIndex] = 1
      queue[tail++] = x
      queue[tail++] = y
    }
  }

  for (let x = 0; x < width; x++) {
    enqueueEdge(x, 0)
    enqueueEdge(x, height - 1)
  }

  for (let y = 1; y < height - 1; y++) {
    enqueueEdge(0, y)
    enqueueEdge(width - 1, y)
  }

  while (head < tail) {
    const cx = queue[head++]
    const cy = queue[head++]

    visitNeighbor(cx, cy - 1)
    visitNeighbor(cx, cy + 1)
    visitNeighbor(cx - 1, cy)
    visitNeighbor(cx + 1, cy)
  }

  for (let i = 0; i < totalPixels; i++) {
    if (!visited[i]) continue

    const idx = i * 4
    const a = data[idx + 3]
    if (a === 0) continue

    const dist = rgbDistance(data[idx], data[idx + 1], data[idx + 2], targetR, targetG, targetB)
    data[idx + 3] = calculateAlpha(dist, coreThreshold, feather, a)
  }

  function visitNeighbor(x: number, y: number) {
    if (x < 0 || x >= width || y < 0 || y >= height) return

    const pixelIndex = y * width + x
    if (visited[pixelIndex]) return

    const idx = pixelIndex * 4
    if (isConnected(idx)) {
      visited[pixelIndex] = 1
      queue[tail++] = x
      queue[tail++] = y
    }
  }
}
