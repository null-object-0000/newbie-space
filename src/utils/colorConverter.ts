import { colord, type RgbColor, type HslColor, type HwbColor, type LchColor, type LabColor, type XyzColor, type CmykColor, type HsvColor } from 'colord'

export interface ColorResult {
  label: string
  value: string
  /** 用于 copy 的去格式纯值，如 "255, 128, 0" */
  raw: string
}

const formatRgb = (c: RgbColor) => `rgb(${c.r}, ${c.g}, ${c.b})`
const rawRgb = (c: RgbColor) => `${c.r}, ${c.g}, ${c.b}`

const formatHsl = (c: HslColor) => `hsl(${c.h}, ${c.s}%, ${c.l}%)`
const rawHsl = (c: HslColor) => `${c.h}, ${c.s}%, ${c.l}%`

const formatHwb = (c: HwbColor) => `hwb(${c.h} ${c.w}% ${c.b}%)`
const rawHwb = (c: HwbColor) => `${c.h}, ${c.w}%, ${c.b}%`

const formatLch = (c: LchColor) => `lch(${Math.round(c.l)}% ${Math.round(c.c)} ${Math.round(c.h)})`
const rawLch = (c: LchColor) => `${Math.round(c.l)}%, ${Math.round(c.c)}, ${Math.round(c.h)}`

const formatLab = (c: LabColor) => `lab(${Math.round(c.l)}% ${Math.round(c.a)} ${Math.round(c.b)})`
const rawLab = (c: LabColor) => `${Math.round(c.l)}%, ${Math.round(c.a)}, ${Math.round(c.b)}`

const formatXyz = (c: XyzColor) => `xyz(${round2(c.x)}, ${round2(c.y)}, ${round2(c.z)})`
const rawXyz = (c: XyzColor) => `${round2(c.x)}, ${round2(c.y)}, ${round2(c.z)}`

const formatCmyk = (c: CmykColor) => `cmyk(${c.c}%, ${c.m}%, ${c.y}%, ${c.k}%)`
const rawCmyk = (c: CmykColor) => `${c.c}%, ${c.m}%, ${c.y}%, ${c.k}%`

const formatHsv = (c: HsvColor) => `hsv(${c.h}, ${c.s}%, ${c.v}%)`
const rawHsv = (c: HsvColor) => `${c.h}, ${c.s}%, ${c.v}%`

function round2(n: number) {
  return Math.round(n * 100) / 100
}

export function convertColor(input: string): { hex: string; results: ColorResult[] } | null {
  const c = colord(input)
  if (!c.isValid()) return null

  const hex = c.toHex()
  const rgba = c.toRgb()
  const hsl = c.toHsl()
  const hwb = c.toHwb()
  const lch = c.toLch()
  const lab = c.toLab()
  const xyz = c.toXyz()
  const cmyk = c.toCmyk()
  const hsv = c.toHsv()
  const name = c.toName({ closest: true })

  return {
    hex,
    results: [
      { label: 'HEX', value: hex, raw: hex },
      { label: 'RGB', value: formatRgb(rgba), raw: rawRgb(rgba) },
      { label: 'HSL', value: formatHsl(hsl), raw: rawHsl(hsl) },
      { label: 'HWB', value: formatHwb(hwb), raw: rawHwb(hwb) },
      { label: 'HSV', value: formatHsv(hsv), raw: rawHsv(hsv) },
      { label: 'LCH', value: formatLch(lch), raw: rawLch(lch) },
      { label: 'LAB', value: formatLab(lab), raw: rawLab(lab) },
      { label: 'XYZ', value: formatXyz(xyz), raw: rawXyz(xyz) },
      { label: 'CMYK', value: formatCmyk(cmyk), raw: rawCmyk(cmyk) },
      { label: 'NAME', value: name || '—', raw: name || '' }
    ]
  }
}
