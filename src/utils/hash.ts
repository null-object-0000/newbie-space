export interface HashResult {
  algo: string
  hash: string
}

export type HashEncoding = 'hex' | 'base64' | 'base64url' | 'bin'

const ALGOS = ['MD5', 'SHA-1', 'SHA-224', 'SHA-256', 'SHA-384', 'SHA-512'] as const
export type AlgoName = (typeof ALGOS)[number]

export function getAlgos(): readonly AlgoName[] {
  return ALGOS
}

const CRYPTO_MAP: Record<string, string> = {
  'SHA-1': 'SHA-1',
  'SHA-224': 'SHA-256', // truncated below
  'SHA-256': 'SHA-256',
  'SHA-384': 'SHA-384',
  'SHA-512': 'SHA-512'
}

/**
 * 计算所有算法的哈希值，按 encoding 格式化。
 */
export async function hashAll(
  text: string,
  encoding: HashEncoding = 'hex'
): Promise<HashResult[]> {
  const encoder = new TextEncoder()
  const data = encoder.encode(text)
  const results: HashResult[] = []

  for (const algo of ALGOS) {
    try {
      if (algo === 'MD5') {
        const h = md5(text)
        results.push({ algo, hash: formatHex(h, encoding) })
      } else {
        let buf = await crypto.subtle.digest(CRYPTO_MAP[algo], data)
        // SHA-224 截断 SHA-256 前 28 字节
        if (algo === 'SHA-224') buf = buf.slice(0, 28)
        const h = hex(buf)
        results.push({ algo, hash: formatHex(h, encoding) })
      }
    } catch {
      results.push({ algo, hash: '计算失败' })
    }
  }

  return results
}

// ====== 格式化 ======

/** 将 hex 字符串按 encoding 转换 */
export function formatHex(hexStr: string, encoding: HashEncoding): string {
  switch (encoding) {
    case 'hex':
      return hexStr
    case 'base64':
      return hexToBase64(hexStr)
    case 'base64url':
      return hexToBase64url(hexStr)
    case 'bin':
      return hexToBin(hexStr)
  }
}

function hex(bytes: ArrayBuffer): string {
  return Array.from(new Uint8Array(bytes))
    .map(b => b.toString(16).padStart(2, '0'))
    .join('')
}

function hexToBase64(hexStr: string): string {
  const bytes = new Uint8Array(hexStr.length / 2)
  for (let i = 0; i < hexStr.length; i += 2) {
    bytes[i / 2] = parseInt(hexStr.substring(i, i + 2), 16)
  }
  let bin = ''
  for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i])
  return btoa(bin)
}

function hexToBase64url(hexStr: string): string {
  return hexToBase64(hexStr).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

function hexToBin(hexStr: string): string {
  return hexStr
    .trim()
    .split('')
    .map(h => parseInt(h, 16).toString(2).padStart(4, '0'))
    .join('')
}

// ====== MD5 (RFC 1321) ======
function md5(input: string): string {
  const msg = unescape(encodeURIComponent(input))
  const len = msg.length
  const words: number[] = []

  for (let i = 0; i < len; i++) {
    words[i >> 2] |= msg.charCodeAt(i) << ((i % 4) << 3)
  }
  words[len >> 2] |= 0x80 << ((len % 4) << 3)

  const tail = ((len + 8) >>> 6) << 4
  words[tail + 14] = len * 8
  words[tail + 15] = 0

  let a = 0x67452301,
    b = 0xefcdab89,
    c = 0x98badcfe,
    d = 0x10325476

  for (let i = 0; i < words.length; i += 16) {
    const aa = a,
      bb = b,
      cc = c,
      dd = d
    const chunk = words.slice(i, i + 16)
    a = ff(a, b, c, d, chunk[0], 7, 0xd76aa478)
    d = ff(d, a, b, c, chunk[1], 12, 0xe8c7b756)
    c = ff(c, d, a, b, chunk[2], 17, 0x242070db)
    b = ff(b, c, d, a, chunk[3], 22, 0xc1bdceee)
    a = ff(a, b, c, d, chunk[4], 7, 0xf57c0faf)
    d = ff(d, a, b, c, chunk[5], 12, 0x4787c62a)
    c = ff(c, d, a, b, chunk[6], 17, 0xa8304613)
    b = ff(b, c, d, a, chunk[7], 22, 0xfd469501)
    a = ff(a, b, c, d, chunk[8], 7, 0x698098d8)
    d = ff(d, a, b, c, chunk[9], 12, 0x8b44f7af)
    c = ff(c, d, a, b, chunk[10], 17, 0xffff5bb1)
    b = ff(b, c, d, a, chunk[11], 22, 0x895cd7be)
    a = ff(a, b, c, d, chunk[12], 7, 0x6b901122)
    d = ff(d, a, b, c, chunk[13], 12, 0xfd987193)
    c = ff(c, d, a, b, chunk[14], 17, 0xa679438e)
    b = ff(b, c, d, a, chunk[15], 22, 0x49b40821)
    a = gg(a, b, c, d, chunk[1], 5, 0xf61e2562)
    d = gg(d, a, b, c, chunk[6], 9, 0xc040b340)
    c = gg(c, d, a, b, chunk[11], 14, 0x265e5a51)
    b = gg(b, c, d, a, chunk[0], 20, 0xe9b6c7aa)
    a = gg(a, b, c, d, chunk[5], 5, 0xd62f105d)
    d = gg(d, a, b, c, chunk[10], 9, 0x02441453)
    c = gg(c, d, a, b, chunk[15], 14, 0xd8a1e681)
    b = gg(b, c, d, a, chunk[4], 20, 0xe7d3fbc8)
    a = gg(a, b, c, d, chunk[9], 5, 0x21e1cde6)
    d = gg(d, a, b, c, chunk[14], 9, 0xc33707d6)
    c = gg(c, d, a, b, chunk[3], 14, 0xf4d50d87)
    b = gg(b, c, d, a, chunk[8], 20, 0x455a14ed)
    a = gg(a, b, c, d, chunk[13], 5, 0xa9e3e905)
    d = gg(d, a, b, c, chunk[2], 9, 0xfcefa3f8)
    c = gg(c, d, a, b, chunk[7], 14, 0x676f02d9)
    b = gg(b, c, d, a, chunk[12], 20, 0x8d2a4c8a)
    a = hh(a, b, c, d, chunk[5], 4, 0xfffa3942)
    d = hh(d, a, b, c, chunk[8], 11, 0x8771f681)
    c = hh(c, d, a, b, chunk[11], 16, 0x6d9d6122)
    b = hh(b, c, d, a, chunk[14], 23, 0xfde5380c)
    a = hh(a, b, c, d, chunk[1], 4, 0xa4beea44)
    d = hh(d, a, b, c, chunk[4], 11, 0x4bdecfa9)
    c = hh(c, d, a, b, chunk[7], 16, 0xf6bb4b60)
    b = hh(b, c, d, a, chunk[10], 23, 0xbebfbc70)
    a = hh(a, b, c, d, chunk[13], 4, 0x289b7ec6)
    d = hh(d, a, b, c, chunk[0], 11, 0xeaa127fa)
    c = hh(c, d, a, b, chunk[3], 16, 0xd4ef3085)
    b = hh(b, c, d, a, chunk[6], 23, 0x04881d05)
    a = hh(a, b, c, d, chunk[9], 4, 0xd9d4d039)
    d = hh(d, a, b, c, chunk[12], 11, 0xe6db99e5)
    c = hh(c, d, a, b, chunk[15], 16, 0x1fa27cf8)
    b = hh(b, c, d, a, chunk[2], 23, 0xc4ac5665)
    a = ii(a, b, c, d, chunk[0], 6, 0xf4292244)
    d = ii(d, a, b, c, chunk[7], 10, 0x432aff97)
    c = ii(c, d, a, b, chunk[14], 15, 0xab9423a7)
    b = ii(b, c, d, a, chunk[5], 21, 0xfc93a039)
    a = ii(a, b, c, d, chunk[12], 6, 0x655b59c3)
    d = ii(d, a, b, c, chunk[3], 10, 0x8f0ccc92)
    c = ii(c, d, a, b, chunk[10], 15, 0xffeff47d)
    b = ii(b, c, d, a, chunk[1], 21, 0x85845dd1)
    a = ii(a, b, c, d, chunk[8], 6, 0x6fa87e4f)
    d = ii(d, a, b, c, chunk[15], 10, 0xfe2ce6e0)
    c = ii(c, d, a, b, chunk[6], 15, 0xa3014314)
    b = ii(b, c, d, a, chunk[13], 21, 0x4e0811a1)
    a = ii(a, b, c, d, chunk[4], 6, 0xf7537e82)
    d = ii(d, a, b, c, chunk[11], 10, 0xbd3af235)
    c = ii(c, d, a, b, chunk[2], 15, 0x2ad7d2bb)
    b = ii(b, c, d, a, chunk[9], 21, 0xeb86d391)
    a = (a + aa) >>> 0
    b = (b + bb) >>> 0
    c = (c + cc) >>> 0
    d = (d + dd) >>> 0
  }

  return toHex(a) + toHex(b) + toHex(c) + toHex(d)
}

function ff(a: number, b: number, c: number, d: number, x: number, s: number, t: number) {
  return cmn((b & c) | (~b & d), a, b, x, s, t)
}
function gg(a: number, b: number, c: number, d: number, x: number, s: number, t: number) {
  return cmn((b & d) | (c & ~d), a, b, x, s, t)
}
function hh(a: number, b: number, c: number, d: number, x: number, s: number, t: number) {
  return cmn(b ^ c ^ d, a, b, x, s, t)
}
function ii(a: number, b: number, c: number, d: number, x: number, s: number, t: number) {
  return cmn(c ^ (b | ~d), a, b, x, s, t)
}
function cmn(q: number, a: number, b: number, x: number, s: number, t: number) {
  return add32(rol(add32(add32(a, q), add32(x || 0, t)), s), b)
}
function add32(a: number, b: number) {
  return (a + b) & 0xffffffff
}
function rol(num: number, cnt: number) {
  return (num << cnt) | (num >>> (32 - cnt))
}
function toHex(num: number): string {
  let s = ''
  for (let i = 0; i < 4; i++) s += ((num >> (i * 8)) & 0xff).toString(16).padStart(2, '0')
  return s
}
