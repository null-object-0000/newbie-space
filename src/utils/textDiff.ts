/** 单行差异类型 */
export type DiffOp = 'equal' | 'insert' | 'delete'

export interface DiffLine {
  type: DiffOp
  content: string
  lineNumA?: number // 原始文本行号
  lineNumB?: number // 修改文本行号
}

/**
 * 行级别 diff（LCS 算法）。
 * 输入两个文本字符串，返回带差异标记的行列表。
 */
export function diffLines(textA: string, textB: string): DiffLine[] {
  const linesA = textA.split('\n')
  const linesB = textB.split('\n')

  // LCS table
  const m = linesA.length
  const n = linesB.length
  const dp: number[][] = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0))

  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      if (linesA[i - 1] === linesB[j - 1]) {
        dp[i][j] = dp[i - 1][j - 1] + 1
      } else {
        dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1])
      }
    }
  }

  // Backtrack to build diff
  const result: DiffLine[] = []
  let i = m, j = n

  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && linesA[i - 1] === linesB[j - 1]) {
      result.unshift({ type: 'equal', content: linesA[i - 1], lineNumA: i, lineNumB: j })
      i--; j--
    } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
      result.unshift({ type: 'insert', content: linesB[j - 1], lineNumB: j })
      j--
    } else {
      result.unshift({ type: 'delete', content: linesA[i - 1], lineNumA: i })
      i--
    }
  }

  return result
}

/** 统计差异行数 */
export function diffStats(diff: DiffLine[]) {
  let added = 0, removed = 0
  for (const line of diff) {
    if (line.type === 'insert') added++
    else if (line.type === 'delete') removed++
  }
  return { added, removed }
}
