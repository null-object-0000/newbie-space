export interface CodeGenFormField {
  key: string
  value: string
  type: 'text' | 'file'
}

export interface CodeGenParams {
  method: string
  url: string
  headers: Array<{ key: string; value: string }>
  body: string
  bodyType: 'json' | 'text' | 'form-urlencoded' | 'form-data'
  formFields?: CodeGenFormField[]
}

function escapeShellSingleQuote(s: string): string {
  return s.replace(/'/g, "'\\''")
}

function quoteHeaderValue(v: string): string {
  return `'${escapeShellSingleQuote(v)}'`
}

function indent(text: string, spaces: number): string {
  const prefix = ' '.repeat(spaces)
  return text
    .split('\n')
    .map((line) => (line ? prefix + line : line))
    .join('\n')
}

/** 生成 cURL 命令 */
export function generateCurlCommand(p: CodeGenParams): string {
  const parts: string[] = ['curl']

  if (p.method !== 'GET') {
    parts.push(`-X ${p.method}`)
  }

  parts.push(`'${escapeShellSingleQuote(p.url)}'`)

  for (const h of p.headers) {
    parts.push(`-H '${escapeShellSingleQuote(h.key)}: ${escapeShellSingleQuote(h.value)}'`)
  }

  if (p.bodyType === 'form-data' && p.formFields?.length) {
    for (const f of p.formFields) {
      if (f.type === 'file') {
        parts.push(`-F '${escapeShellSingleQuote(f.key)}=@/path/to/file'`)
      } else {
        parts.push(`-F '${escapeShellSingleQuote(f.key)}=${escapeShellSingleQuote(f.value)}'`)
      }
    }
  } else if (p.bodyType === 'form-urlencoded' && p.body) {
    parts.push(`-d '${escapeShellSingleQuote(p.body)}'`)
  } else if (p.body) {
    parts.push(`-d '${escapeShellSingleQuote(p.body)}'`)
  }

  return parts.join(' \\\n  ')
}

/** 生成 Python requests 代码 */
export function generatePythonCode(p: CodeGenParams): string {
  const lines: string[] = ['import requests', '']
  const method = p.method.toLowerCase()

  lines.push(`url = '${p.url.replace(/'/g, "\\'")}'`)

  if (p.headers.length) {
    const headerEntries = p.headers.map((h) => `    '${h.key.replace(/'/g, "\\'")}': '${h.value.replace(/'/g, "\\'")}'`)
    lines.push(`headers = {\n${headerEntries.join(',\n')}\n}`)
  }

  if (p.bodyType === 'form-data' && p.formFields?.length) {
    const fileFields = p.formFields.filter(f => f.type === 'file')
    const textFields = p.formFields.filter(f => f.type === 'text')
    if (fileFields.length) {
      const fieldLines = textFields.map(f => `    '${f.key.replace(/'/g, "\\'")}': '${f.value.replace(/'/g, "\\'")}'`).join(',\n')
      const fileLines = fileFields.map(f => `    '${f.key.replace(/'/g, "\\'")}': open('/path/to/file', 'rb')`).join(',\n')
      lines.push(`files = {\n${fieldLines}${fieldLines && fileLines ? ',\n' : ''}${fileLines}\n}`)
    } else if (textFields.length) {
      const fieldLines = textFields.map(f => `    '${f.key.replace(/'/g, "\\'")}': '${f.value.replace(/'/g, "\\'")}'`).join(',\n')
      lines.push(`data = {\n${fieldLines}\n}`)
    }
  } else if (p.bodyType === 'form-urlencoded' && p.body) {
    const kvLines = p.body.split('&').map(pair => {
      const [k, v] = pair.split('=')
      return `    '${(k || '').replace(/'/g, "\\'")}': '${(v || '').replace(/'/g, "\\'")}'`
    }).join(',\n')
    lines.push(`data = {\n${kvLines}\n}`)
  } else if (p.body) {
    lines.push(
      `${p.bodyType === 'json' ? "# body = {...}  # 也可传入 dict，requests 会自动 JSON 序列化\n" : ''}body = '''${p.body.replace(/'/g, "\\'")}'''`
    )
  }

  const args: string[] = ['url']
  if (p.headers.length) args.push('headers=headers')
  if (p.body) {
    if (p.bodyType === 'form-data' && p.formFields?.some(f => f.type === 'file')) {
      args.push('files=files')
    } else if (p.bodyType === 'form-data' || p.bodyType === 'form-urlencoded') {
      args.push('data=data')
    } else {
      args.push('data=body')
    }
  }
  lines.push(`response = requests.${method}(${args.join(', ')})`)
  lines.push('print(response.status_code)')
  lines.push('print(response.text)')

  return lines.join('\n')
}

/** 生成 JavaScript fetch 代码 */
export function generateJavaScriptCode(p: CodeGenParams): string {
  const lines: string[] = []

  const optionsEntries: string[] = [`  method: '${p.method}'`]

  if (p.headers.length) {
    const headerEntries = p.headers.map(
      (h) => `    '${h.key.replace(/'/g, "\\'")}': '${h.value.replace(/'/g, "\\'")}'`
    )
    optionsEntries.push(`  headers: {\n${headerEntries.join(',\n')}\n  }`)
  }

  if (p.bodyType === 'form-data' && p.formFields?.length) {
    lines.push('const formData = new FormData()')
    for (const f of p.formFields) {
      if (f.type === 'file') {
        lines.push(`// formData.append('${f.key.replace(/'/g, "\\'")}', fileInput.files[0])`)
      } else {
        lines.push(`formData.append('${f.key.replace(/'/g, "\\'")}', '${f.value.replace(/'/g, "\\'")}')`)
      }
    }
    optionsEntries.push('  body: formData')
  } else if (p.bodyType === 'form-urlencoded') {
    const searchParams = new URLSearchParams()
    if (p.body) {
      p.body.split('&').forEach(pair => {
        const [k, v] = pair.split('=')
        searchParams.set(k || '', v || '')
      })
    }
    optionsEntries.push(`  body: new URLSearchParams('${searchParams.toString().replace(/'/g, "\\'")}')`)
  } else if (p.body) {
    const bodyStr =
      p.bodyType === 'json'
        ? `JSON.stringify(${p.body})`
        : `'${p.body.replace(/'/g, "\\'").replace(/\n/g, '\\n')}'`
    optionsEntries.push(`  body: ${bodyStr}`)
  }

  lines.push(`fetch('${p.url.replace(/'/g, "\\'")}', {`)
  lines.push(optionsEntries.join(',\n'))
  lines.push('})')
  lines.push('  .then(response => response.json())')
  lines.push('  .then(data => console.log(data))')
  lines.push('  .catch(err => console.error(err))')

  return lines.join('\n')
}

/** 生成 Go net/http 代码 */
export function generateGoCode(p: CodeGenParams): string {
  const lines: string[] = [
    'package main',
    '',
    'import (',
    '  "fmt"',
    '  "io"',
    '  "net/http"',
  ]

  if (p.bodyType === 'form-data' && p.formFields?.some(f => f.type === 'file')) {
    lines.push('  "bytes"')
    lines.push('  "mime/multipart"')
    lines.push('  "os"')
  } else if (p.body) {
    lines.push('  "strings"')
  }

  if (p.bodyType === 'form-urlencoded' && p.body) {
    lines.push('  "net/url"')
  }

  lines.push(')', '', 'func main() {')

  // URL
  lines.push(`  urlStr := "${p.url.replace(/"/g, '\\"')}"`)

  // body / NewRequest
  if (p.bodyType === 'form-data' && p.formFields?.length) {
    const hasFile = p.formFields.some(f => f.type === 'file')
    if (hasFile) {
      lines.push('  bodyBuf := new(bytes.Buffer)')
      lines.push('  writer := multipart.NewWriter(bodyBuf)')
      for (const f of p.formFields) {
        if (f.type === 'file') {
          lines.push(`  file, _ := os.Open("/path/to/file")`)
          lines.push(`  part, _ := writer.CreateFormFile("${f.key.replace(/"/g, '\\"')}", "filename")`)
          lines.push('  io.Copy(part, file)')
          lines.push('  file.Close()')
        } else {
          lines.push(`  writer.WriteField("${f.key.replace(/"/g, '\\"')}", "${f.value.replace(/"/g, '\\"')}")`)
        }
      }
      lines.push('  writer.Close()')
      lines.push(`  req, err := http.NewRequest("${p.method}", urlStr, bodyBuf)`)
    } else {
      lines.push(`  req, err := http.NewRequest("${p.method}", urlStr, nil)`)
    }
  } else if (p.bodyType === 'form-urlencoded' && p.body) {
    const formValues = p.body.split('&').map(pair => {
      const [k, v] = pair.split('=')
      return `  data.Set("${(k || '').replace(/"/g, '\\"')}", "${decodeURIComponent(v || '').replace(/"/g, '\\"')}")`
    }).join('\n')
    lines.push('  data := url.Values{}')
    lines.push(formValues)
    lines.push('  req, err := http.NewRequest("' + p.method + '", urlStr, strings.NewReader(data.Encode()))')
  } else if (p.body) {
    const escaped = p.body.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\n/g, '\\n')
    const contentType =
      p.bodyType === 'json' ? 'application/json' : 'text/plain'
    lines.push(
      `  body := strings.NewReader("${escaped}")`,
      `  req, err := http.NewRequest("${p.method}", urlStr, body)`
    )
  } else {
    lines.push('  req, err := http.NewRequest("' + p.method + '", urlStr, nil)')
  }

  lines.push('  if err != nil {')
  lines.push('    fmt.Println(err)')
  lines.push('    return')
  lines.push('  }')

  // headers
  for (const h of p.headers) {
    lines.push(`  req.Header.Set("${h.key.replace(/"/g, '\\"')}", "${h.value.replace(/"/g, '\\"')}")`)
  }

  if (p.bodyType === 'form-data' && p.formFields?.some(f => f.type === 'file')) {
    lines.push(`  req.Header.Set("Content-Type", writer.FormDataContentType())`)
  } else if (p.bodyType === 'form-urlencoded' && p.body) {
    lines.push(`  req.Header.Set("Content-Type", "application/x-www-form-urlencoded")`)
  }

  lines.push('')
  lines.push('  client := &http.Client{}')
  lines.push('  resp, err := client.Do(req)')
  lines.push('  if err != nil {')
  lines.push('    fmt.Println(err)')
  lines.push('    return')
  lines.push('  }')
  lines.push('  defer resp.Body.Close()')
  lines.push('')
  lines.push('  bodyBytes, err := io.ReadAll(resp.Body)')
  lines.push('  if err != nil {')
  lines.push('    fmt.Println(err)')
  lines.push('    return')
  lines.push('  }')
  lines.push('  fmt.Println(resp.StatusCode, string(bodyBytes))')
  lines.push('}')

  return lines.join('\n')
}

/** 从 cURL 命令解析请求参数 */
export interface ParsedCurl {
  method: string
  url: string
  headers: Array<{ key: string; value: string }>
  body: string
  bodyType: 'json' | 'text' | 'form-urlencoded' | 'form-data'
}

export function parseCurlCommand(curlText: string): ParsedCurl | null {
  try {
    const text = curlText.trim()
    // Strip "curl " prefix (case insensitive)
    const body = text.replace(/^curl\s+/i, '').trim()

    const result: ParsedCurl = {
      method: 'GET',
      url: '',
      headers: [],
      body: '',
      bodyType: 'text',
    }

    // Tokenize respecting single-quoted strings
    const tokens: string[] = []
    let i = 0
    while (i < body.length) {
      // Skip whitespace and line continuations
      if (body[i] === ' ' || body[i] === '\t') { i++; continue }
      if (body[i] === '\\') {
        // Skip backslash-newline continuations
        i++
        while (i < body.length && (body[i] === ' ' || body[i] === '\t' || body[i] === '\n' || body[i] === '\r')) i++
        continue
      }

      if (body[i] === "'") {
        i++ // skip opening quote
        let str = ''
        while (i < body.length && body[i] !== "'") {
          if (body[i] === '\\' && i + 1 < body.length) {
            str += body[i + 1]
            i += 2
          } else {
            str += body[i]
            i++
          }
        }
        if (i < body.length) i++ // skip closing quote
        tokens.push(str)
      } else if (body[i] === '"') {
        i++ // skip opening quote
        let str = ''
        while (i < body.length && body[i] !== '"') {
          if (body[i] === '\\' && i + 1 < body.length) {
            const next = body[i + 1]
            if (next === 'n') { str += '\n'; i += 2 }
            else if (next === 't') { str += '\t'; i += 2 }
            else { str += next; i += 2 }
          } else {
            str += body[i]
            i++
          }
        }
        if (i < body.length) i++ // skip closing quote
        tokens.push(str)
      } else {
        let tok = ''
        while (i < body.length && body[i] !== ' ' && body[i] !== '\t') {
          tok += body[i]
          i++
        }
        tokens.push(tok)
      }
    }

    // Parse tokens
    for (let ti = 0; ti < tokens.length; ti++) {
      const t = tokens[ti]

      if (t === '-X' || t === '--request') {
        result.method = tokens[++ti] || 'GET'
        continue
      }

      if (t === '-H' || t === '--header') {
        const headerStr = tokens[++ti] || ''
        const colonIdx = headerStr.indexOf(':')
        if (colonIdx >= 0) {
          result.headers.push({
            key: headerStr.slice(0, colonIdx).trim(),
            value: headerStr.slice(colonIdx + 1).trim(),
          })
        }
        continue
      }

      if (t === '-d' || t === '--data' || t === '--data-raw' || t === '--data-binary') {
        const data = tokens[++ti] || ''
        result.body = data
        // Detect body type
        if (data.trim().startsWith('{') || data.trim().startsWith('[')) {
          result.bodyType = 'json'
        } else if (data.includes('=') && !data.includes(' ')) {
          result.bodyType = 'form-urlencoded'
        }
        continue
      }

      // URL: first non-flag token that isn't a flag value
      if (!t.startsWith('-') && result.url === '') {
        result.url = t
        continue
      }
    }

    if (!result.url) return null

    return result
  } catch {
    return null
  }
}
