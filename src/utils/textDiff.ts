export type DiffOp = 'equal' | 'insert' | 'delete'

export interface DiffLine {
  type: DiffOp
  content: string
  lineNumA?: number
  lineNumB?: number
}

/** 字符级差异片段 */
export interface CharDiff {
  type: 'equal' | 'added' | 'removed'
  text: string
}

/**
 * 行级别 diff（LCS 算法）。
 */
export function diffLines(textA: string, textB: string): DiffLine[] {
  const linesA = textA.split('\n')
  const linesB = textB.split('\n')
  const m = linesA.length
  const n = linesB.length
  const dp: number[][] = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0))

  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      dp[i][j] = linesA[i - 1] === linesB[j - 1]
        ? dp[i - 1][j - 1] + 1
        : Math.max(dp[i - 1][j], dp[i][j - 1])
    }
  }

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

/**
 * 字符级 diff（基于 LCS）。用于行内高亮。
 */
export function diffChars(a: string, b: string): { left: CharDiff[]; right: CharDiff[] } {
  const m = a.length
  const n = b.length
  const dp: number[][] = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0))

  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      dp[i][j] = a[i - 1] === b[j - 1]
        ? dp[i - 1][j - 1] + 1
        : Math.max(dp[i - 1][j], dp[i][j - 1])
    }
  }

  const left: CharDiff[] = []
  const right: CharDiff[] = []
  let i = m, j = n

  while (i > 0 || j > 0) {
    if (i > 0 && j > 0 && a[i - 1] === b[j - 1]) {
      left.unshift({ type: 'equal', text: a[i - 1] })
      right.unshift({ type: 'equal', text: b[j - 1] })
      i--; j--
    } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
      right.unshift({ type: 'added', text: b[j - 1] })
      j--
    } else {
      left.unshift({ type: 'removed', text: a[i - 1] })
      i--
    }
  }

  return { left: mergeAdjacent(left), right: mergeAdjacent(right) }
}

function mergeAdjacent(diffs: CharDiff[]): CharDiff[] {
  const result: CharDiff[] = []
  for (const d of diffs) {
    const last = result[result.length - 1]
    if (last && last.type === d.type) {
      last.text += d.text
    } else {
      result.push({ ...d })
    }
  }
  return result
}

export function diffStats(diff: DiffLine[]) {
  let added = 0, removed = 0
  for (const line of diff) {
    if (line.type === 'insert') added++
    else if (line.type === 'delete') removed++
  }
  return { added, removed }
}

/**
 * 将行级 diff 配对为并排视图的行。
 * 连续的 delete + insert 会配对在一起（带字符级 diff）。
 */
export interface PairedDiffLine {
  leftContent: string
  rightContent: string
  leftNum?: number
  rightNum?: number
  leftType: DiffOp
  rightType: DiffOp
  /** 当 delete+insert 配对时，提供字符级 diff */
  charDiff?: { left: CharDiff[]; right: CharDiff[] }
}

export function pairDiffLines(diff: DiffLine[]): PairedDiffLine[] {
  const pairs: PairedDiffLine[] = []
  let i = 0

  while (i < diff.length) {
    const line = diff[i]

    if (line.type === 'equal') {
      pairs.push({
        leftContent: line.content,
        rightContent: line.content,
        leftNum: line.lineNumA,
        rightNum: line.lineNumB,
        leftType: 'equal',
        rightType: 'equal'
      })
      i++
    } else {
      // 收集连续的 delete 和 insert
      const deletes: DiffLine[] = []
      const inserts: DiffLine[] = []
      while (i < diff.length && (diff[i].type === 'delete' || diff[i].type === 'insert')) {
        if (diff[i].type === 'delete') deletes.push(diff[i])
        else inserts.push(diff[i])
        i++
      }

      // 配对 delete 和 insert
      const maxLen = Math.max(deletes.length, inserts.length)
      for (let k = 0; k < maxLen; k++) {
        const del = deletes[k]
        const ins = inserts[k]

        if (del && ins) {
          pairs.push({
            leftContent: del.content,
            rightContent: ins.content,
            leftNum: del.lineNumA,
            rightNum: ins.lineNumB,
            leftType: 'delete',
            rightType: 'insert',
            charDiff: diffChars(del.content, ins.content)
          })
        } else if (del) {
          pairs.push({
            leftContent: del.content,
            rightContent: '',
            leftNum: del.lineNumA,
            rightNum: undefined,
            leftType: 'delete',
            rightType: 'equal'
          })
        } else if (ins) {
          pairs.push({
            leftContent: '',
            rightContent: ins.content,
            leftNum: undefined,
            rightNum: ins.lineNumB,
            leftType: 'equal',
            rightType: 'insert'
          })
        }
      }
    }
  }

  return pairs
}
