import { colord, type RgbColor, type HslColor, type HsvColor } from 'colord'

export interface ColorResult {
  label: string
  value: string
  /** 用于 copy 的去格式纯值 */
  raw: string
}

// --- 手动格式转换（不依赖 colord 插件）---

const formatRgb = (c: RgbColor) => `rgb(${c.r}, ${c.g}, ${c.b})`
const rawRgb = (c: RgbColor) => `${c.r}, ${c.g}, ${c.b}`

const formatHsl = (c: HslColor) => `hsl(${c.h}, ${c.s}%, ${c.l}%)`
const rawHsl = (c: HslColor) => `${c.h}, ${c.s}%, ${c.l}%`

const formatHsv = (c: HsvColor) => `hsv(${c.h}, ${c.s}%, ${c.v}%)`
const rawHsv = (c: HsvColor) => `${c.h}, ${c.s}%, ${c.v}%`

function rgbToHwb(r: number, g: number, b: number) {
  r /= 255; g /= 255; b /= 255
  const w = Math.min(r, g, b)
  const black = 1 - Math.max(r, g, b)
  const h = rgbToHue(r, g, b)
  return { h: Math.round(h), w: Math.round(w * 100), b: Math.round(black * 100) }
}

function rgbToCmyk(r: number, g: number, b: number) {
  r /= 255; g /= 255; b /= 255
  const k = 1 - Math.max(r, g, b)
  if (k === 1) return { c: 0, m: 0, y: 0, k: 100 }
  const c = (1 - r - k) / (1 - k)
  const m = (1 - g - k) / (1 - k)
  const y = (1 - b - k) / (1 - k)
  return { c: Math.round(c * 100), m: Math.round(m * 100), y: Math.round(y * 100), k: Math.round(k * 100) }
}

function rgbToLab(r: number, g: number, b: number) {
  // RGB → XYZ → Lab
  r /= 255; g /= 255; b /= 255
  r = r > 0.04045 ? Math.pow((r + 0.055) / 1.055, 2.4) : r / 12.92
  g = g > 0.04045 ? Math.pow((g + 0.055) / 1.055, 2.4) : g / 12.92
  b = b > 0.04045 ? Math.pow((b + 0.055) / 1.055, 2.4) : b / 12.92
  let x = (r * 0.4124564 + g * 0.3575761 + b * 0.1804375) / 0.95047
  let y = (r * 0.2126729 + g * 0.7151522 + b * 0.0721750) / 1.00000
  let z = (r * 0.0193339 + g * 0.1191920 + b * 0.9503041) / 1.08883
  x = x > 0.008856 ? Math.pow(x, 1 / 3) : 7.787 * x + 16 / 116
  y = y > 0.008856 ? Math.pow(y, 1 / 3) : 7.787 * y + 16 / 116
  z = z > 0.008856 ? Math.pow(z, 1 / 3) : 7.787 * z + 16 / 116
  return {
    l: Math.round((116 * y - 16) * 100) / 100,
    a: Math.round((500 * (x - y)) * 100) / 100,
    b: Math.round((200 * (y - z)) * 100) / 100
  }
}

function rgbToLch(r: number, g: number, b: number) {
  const lab = rgbToLab(r, g, b)
  const c = Math.sqrt(lab.a * lab.a + lab.b * lab.b)
  let h = Math.atan2(lab.b, lab.a) * 180 / Math.PI
  if (h < 0) h += 360
  return { l: Math.round(lab.l), c: Math.round(c), h: Math.round(h) }
}

function rgbToXyz(r: number, g: number, b: number) {
  r /= 255; g /= 255; b /= 255
  r = r > 0.04045 ? Math.pow((r + 0.055) / 1.055, 2.4) : r / 12.92
  g = g > 0.04045 ? Math.pow((g + 0.055) / 1.055, 2.4) : g / 12.92
  b = b > 0.04045 ? Math.pow((b + 0.055) / 1.055, 2.4) : b / 12.92
  return {
    x: round2(r * 0.4124564 + g * 0.3575761 + b * 0.1804375),
    y: round2(r * 0.2126729 + g * 0.7151522 + b * 0.0721750),
    z: round2(r * 0.0193339 + g * 0.1191920 + b * 0.9503041)
  }
}

function rgbToHue(r: number, g: number, b: number) {
  const max = Math.max(r, g, b), min = Math.min(r, g, b)
  const d = max - min
  if (d === 0) return 0
  let h = 0
  if (max === r) h = ((g - b) / d + (g < b ? 6 : 0))
  else if (max === g) h = ((b - r) / d + 2)
  else h = ((r - g) / d + 4)
  return h * 60
}

// CSS 命名颜色（常用）
const CSS_NAMES: Record<string, string> = {
  black: '#000000', white: '#ffffff', red: '#ff0000', green: '#008000',
  blue: '#0000ff', yellow: '#ffff00', cyan: '#00ffff', magenta: '#ff00ff',
  orange: '#ffa500', pink: '#ffc0cb', purple: '#800080', gray: '#808080', grey: '#808080',
  brown: '#a52a2a', navy: '#000080', teal: '#008080', maroon: '#800000',
  olive: '#808000', lime: '#00ff00', aqua: '#00ffff', silver: '#c0c0c0',
  gold: '#ffd700', coral: '#ff7f50', salmon: '#fa8072', tomato: '#ff6347',
  indigo: '#4b0082', violet: '#ee82ee', crimson: '#dc143c', khaki: '#f0e68c',
  plum: '#dda0dd', orchid: '#da70d6', turquoise: '#40e0d0', tan: '#d2b48c',
  skyblue: '#87ceeb', chocolate: '#d2691b', sienna: '#a0522d', peru: '#cd853f',
  wheat: '#f5deb3', linen: '#faf0e6', ivory: '#fffff0', snow: '#fffafa',
  beige: '#f5f5dc', honeydew: '#f0fff0', mintcream: '#f5fffa', azure: '#f0ffff',
  aliceblue: '#f0f8ff', ghostwhite: '#f8f8ff', lavender: '#e6e6fa', mistyrose: '#ffe4e1',
  antiquewhite: '#faebd7', bisque: '#ffe4c4', blanchedalmond: '#ffebcd',
  cornsilk: '#fff8dc', floralwhite: '#fffaf0', gainsboro: '#dcdcdc',
  lightgray: '#d3d3d3', lightgrey: '#d3d3d3', darkgray: '#a9a9a9', darkgrey: '#a9a9a9',
  dimgray: '#696969', dimgrey: '#696969', slategray: '#708090', slategrey: '#708090',
  transparent: '#00000000'
}

// 最接近的颜色名（按 hex 反查）
const HEX_TO_NAME: Record<string, string> = {}
for (const [name, hex] of Object.entries(CSS_NAMES)) {
  if (!HEX_TO_NAME[hex]) HEX_TO_NAME[hex] = name
}

function round2(n: number) { return Math.round(n * 100) / 100 }

/**
 * 预处理输入：如果是 CSS 命名颜色，转为 hex
 */
function preprocessInput(raw: string): string {
  const trimmed = raw.trim().toLowerCase()
  if (CSS_NAMES[trimmed]) return CSS_NAMES[trimmed]
  return raw.trim()
}

export function convertColor(input: string): { hex: string; results: ColorResult[] } | null {
  const processed = preprocessInput(input)
  const c = colord(processed)
  if (!c.isValid()) return null

  const hex = c.toHex()
  const rgba = c.toRgb()
  const hsl = c.toHsl()
  const hsv = c.toHsv()
  const hwb = rgbToHwb(rgba.r, rgba.g, rgba.b)
  const cmyk = rgbToCmyk(rgba.r, rgba.g, rgba.b)
  const lab = rgbToLab(rgba.r, rgba.g, rgba.b)
  const lch = rgbToLch(rgba.r, rgba.g, rgba.b)
  const xyz = rgbToXyz(rgba.r, rgba.g, rgba.b)
  const name = HEX_TO_NAME[hex] || ''

  return {
    hex,
    results: [
      { label: 'HEX', value: hex, raw: hex },
      { label: 'RGB', value: formatRgb(rgba), raw: rawRgb(rgba) },
      { label: 'HSL', value: formatHsl(hsl), raw: rawHsl(hsl) },
      { label: 'HSV', value: formatHsv(hsv), raw: rawHsv(hsv) },
      { label: 'HWB', value: `hwb(${hwb.h} ${hwb.w}% ${hwb.b}%)`, raw: `${hwb.h}, ${hwb.w}%, ${hwb.b}%` },
      { label: 'CMYK', value: `cmyk(${cmyk.c}%, ${cmyk.m}%, ${cmyk.y}%, ${cmyk.k}%)`, raw: `${cmyk.c}%, ${cmyk.m}%, ${cmyk.y}%, ${cmyk.k}%` },
      { label: 'LAB', value: `lab(${lab.l} ${lab.a} ${lab.b})`, raw: `${lab.l}, ${lab.a}, ${lab.b}` },
      { label: 'LCH', value: `lch(${lch.l}% ${lch.c} ${lch.h})`, raw: `${lch.l}%, ${lch.c}, ${lch.h}` },
      { label: 'XYZ', value: `xyz(${xyz.x}, ${xyz.y}, ${xyz.z})`, raw: `${xyz.x}, ${xyz.y}, ${xyz.z}` },
      { label: 'NAME', value: name || '—', raw: name || '' }
    ]
  }
}
