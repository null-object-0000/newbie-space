export interface CodeGenParams {
  method: string
  url: string
  headers: Array<{ key: string; value: string }>
  body: string
  bodyType: 'json' | 'text'
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

  if (p.body) {
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

  if (p.body) {
    lines.push(
      `${p.bodyType === 'json' ? "# body = {...}  # 也可传入 dict，requests 会自动 JSON 序列化\n" : ''}body = '''${p.body.replace(/'/g, "\\'")}'''`
    )
  }

  const args: string[] = ['url']
  if (p.headers.length) args.push('headers=headers')
  if (p.body) args.push('data=body')
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

  if (p.body) {
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

  if (p.body) {
    lines.push('  "strings"')
  }

  lines.push(')', '', 'func main() {')

  // URL
  lines.push(`  url := "${p.url.replace(/"/g, '\\"')}"`)

  // body / NewRequest
  if (p.body) {
    const escaped = p.body.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\n/g, '\\n')
    const contentType =
      p.bodyType === 'json' ? 'application/json' : 'text/plain'
    lines.push(
      `  body := strings.NewReader("${escaped}")`,
      `  req, err := http.NewRequest("${p.method}", url, body)`
    )
  } else {
    lines.push('  req, err := http.NewRequest("' + p.method + '", url, nil)')
  }

  lines.push('  if err != nil {')
  lines.push('    fmt.Println(err)')
  lines.push('    return')
  lines.push('  }')

  // headers
  for (const h of p.headers) {
    lines.push(`  req.Header.Set("${h.key.replace(/"/g, '\\"')}", "${h.value.replace(/"/g, '\\"')}")`)
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
