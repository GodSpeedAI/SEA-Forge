/**
 * Highlight matching substrings in text (case-insensitive).
 * Returns parts with hit flags and total match count.
 */
export interface HighlightPart {
  text: string
  hit: boolean
}

export interface HighlightResult {
  parts: HighlightPart[]
  count: number
}

export function highlight(text: string, query: string): HighlightResult {
  if (!query) {
    return { parts: [{ text, hit: false }], count: 0 }
  }

  const parts: HighlightPart[] = []
  let count = 0

  // Escape special regex characters so they match literally
  const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const regex = new RegExp(escaped, 'gi')

  let lastIndex = 0
  let match: RegExpExecArray | null

  while ((match = regex.exec(text)) !== null) {
    // Add text before the match
    if (match.index > lastIndex) {
      parts.push({ text: text.slice(lastIndex, match.index), hit: false })
    }
    // Add the match
    parts.push({ text: match[0], hit: true })
    lastIndex = regex.lastIndex
    count++
  }

  // Add remaining text
  if (lastIndex < text.length) {
    parts.push({ text: text.slice(lastIndex), hit: false })
  }

  return { parts, count }
}
