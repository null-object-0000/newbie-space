import QRCode from 'qrcode'

export type ErrorCorrectionLevel = 'L' | 'M' | 'Q' | 'H'

export interface QRCodeOptions {
  text: string
  width: number
  margin: number
  color: {
    dark: string  // 模块颜色
    light: string // 背景颜色
  }
  errorCorrectionLevel: ErrorCorrectionLevel
  transparentBg?: boolean // 是否将背景色替换为透明
}

export interface QRCodeResult {
  dataUrl: string
  width: number
}

const EC_LABELS: Record<ErrorCorrectionLevel, string> = {
  L: 'L — 约 7% 容错',
  M: 'M — 约 15% 容错',
  Q: 'Q — 约 25% 容错',
  H: 'H — 约 30% 容错'
}

export function getECLabel(level: ErrorCorrectionLevel): string {
  return EC_LABELS[level]
}

/**
 * 生成二维码 Data URL，全部在浏览器本地完成。
 */
export async function generateQRCode(options: QRCodeOptions): Promise<QRCodeResult> {
  if (!options.text.trim()) {
    throw new Error('请输入文字或链接')
  }

  const canvas = document.createElement('canvas')

  await QRCode.toCanvas(canvas, options.text, {
    width: options.width,
    margin: options.margin,
    color: {
      dark: options.color.dark,
      light: options.color.light
    },
    errorCorrectionLevel: options.errorCorrectionLevel
  })

  // 透明背景后处理：将背景色像素替换为透明
  if (options.transparentBg) {
    const ctx = canvas.getContext('2d')!
    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height)
    const data = imageData.data
    // 解析背景色为 RGBA
    const bg = parseColor(options.color.light)
    const tolerance = 10 // 颜色容差
    for (let i = 0; i < data.length; i += 4) {
      if (
        Math.abs(data[i] - bg.r) < tolerance &&
        Math.abs(data[i + 1] - bg.g) < tolerance &&
        Math.abs(data[i + 2] - bg.b) < tolerance
      ) {
        data[i + 3] = 0 // alpha = 0 (transparent)
      }
    }
    ctx.putImageData(imageData, 0, 0)
  }

  const dataUrl = canvas.toDataURL('image/png')

  return {
    dataUrl,
    width: options.width
  }
}

/** 将 CSS 颜色字符串解析为 RGB */
function parseColor(color: string): { r: number; g: number; b: number } {
  const ctx = document.createElement('canvas').getContext('2d')!
  ctx.fillStyle = color
  const hex = ctx.fillStyle // 浏览器会标准化为 #rrggbb 格式
  const match = hex.match(/^#([0-9a-f]{6})$/i)
  if (match) {
    const n = parseInt(match[1], 16)
    return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255 }
  }
  return { r: 255, g: 255, b: 255 }
}
