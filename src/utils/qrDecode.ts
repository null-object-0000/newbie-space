import jsQR from 'jsqr'

export interface DecodeResult {
  data: string
  /** QR 码在图片中的位置 */
  location?: {
    topLeft: { x: number; y: number }
    topRight: { x: number; y: number }
    bottomRight: { x: number; y: number }
    bottomLeft: { x: number; y: number }
  }
}

/**
 * 从 ImageData 中解析二维码，返回解码文本。
 * 未找到二维码时返回 null。
 */
export function decodeQRCode(imageData: ImageData): DecodeResult | null {
  const code = jsQR(imageData.data, imageData.width, imageData.height, {
    inversionAttempts: 'dontInvert'
  })

  if (!code) return null

  return {
    data: code.data,
    location: code.location
  }
}

/**
 * 从 ImageData 中解析二维码，尝试多种反转策略。
 * 部分二维码图片有反色背景，需要额外尝试。
 */
export function decodeQRCodeRobust(imageData: ImageData): DecodeResult | null {
  // 先尝试原始
  let result = decodeQRCode(imageData)
  if (result) return result

  // 尝试灰度反转
  const inverted = invertImageData(imageData)
  result = decodeQRCode(inverted)
  if (result) return result

  return null
}

function invertImageData(src: ImageData): ImageData {
  const data = new Uint8ClampedArray(src.data)
  for (let i = 0; i < data.length; i += 4) {
    data[i] = 255 - data[i]         // R
    data[i + 1] = 255 - data[i + 1] // G
    data[i + 2] = 255 - data[i + 2] // B
    // Alpha 不变
  }
  return new ImageData(data, src.width, src.height)
}
