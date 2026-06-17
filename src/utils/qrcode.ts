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

  const dataUrl = canvas.toDataURL('image/png')

  return {
    dataUrl,
    width: options.width
  }
}
