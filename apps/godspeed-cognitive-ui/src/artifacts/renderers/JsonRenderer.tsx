import { useEffect, useState } from 'react'
import type { SourceRendererProps } from '../registry'
import './JsonRenderer.css'

export default function JsonRenderer(props: SourceRendererProps) {
  if (props.model.kind !== 'json') {
    throw new Error(`JsonRenderer expects kind 'json', got '${props.model.kind}'`)
  }

  const data = props.model.value
  const [expanded, setExpanded] = useState<Set<string>>(new Set())

  // Initialize expanded state: top 2 levels expanded by default
  useEffect(() => {
    const toExpand = new Set<string>()
    const traverse = (val: unknown, path: string, depth: number) => {
      if (depth < 2) {
        if (typeof val === 'object' && val !== null) {
          toExpand.add(path)
          if (Array.isArray(val)) {
            val.forEach((item, i) => traverse(item, `${path}[${i}]`, depth + 1))
          } else {
            Object.entries(val).forEach(([k, v]) => traverse(v, `${path}.${k}`, depth + 1))
          }
        }
      }
    }
    traverse(data, 'root', 0)
    setExpanded(toExpand)
  }, [data])

  const getMatchCount = () => {
    let count = 0
    const traverse = (val: unknown) => {
      if (typeof val === 'string') {
        if (val.toLowerCase().includes(props.query.toLowerCase())) count++
      } else if (typeof val === 'object' && val !== null) {
        if (Array.isArray(val)) {
          val.forEach(traverse)
        } else {
          Object.entries(val).forEach(([k, v]) => {
            if (k.toLowerCase().includes(props.query.toLowerCase())) count++
            traverse(v)
          })
        }
      }
    }
    traverse(data)
    return count
  }

  useEffect(() => {
    props.onMatches?.(getMatchCount())
  }, [props.query])

  const toggleExpand = (path: string) => {
    setExpanded((prev) => {
      const next = new Set(prev)
      if (next.has(path)) {
        next.delete(path)
      } else {
        next.add(path)
      }
      return next
    })
  }

  const expandAll = () => {
    const all = new Set<string>()
    const traverse = (val: unknown, path: string) => {
      if (typeof val === 'object' && val !== null) {
        all.add(path)
        if (Array.isArray(val)) {
          val.forEach((item, i) => traverse(item, `${path}[${i}]`))
        } else {
          Object.entries(val).forEach(([k, v]) => traverse(v, `${path}.${k}`))
        }
      }
    }
    traverse(data, 'root')
    setExpanded(all)
  }

  const collapseAll = () => {
    setExpanded(new Set())
  }

  const renderValue = (val: unknown, path: string, depth: number = 0): JSX.Element => {
    if (val === null) {
      return <span className="gs-r-json-null">null</span>
    }

    if (val === undefined) {
      return <span className="gs-r-json-undefined">undefined</span>
    }

    if (typeof val === 'boolean') {
      return <span className="gs-r-json-boolean">{String(val)}</span>
    }

    if (typeof val === 'number') {
      return <span className="gs-r-json-number">{val}</span>
    }

    if (typeof val === 'string') {
      const isMatch =
        props.query && val.toLowerCase().includes(props.query.toLowerCase())
      return (
        <span className="gs-r-json-string">
          &quot;
          {isMatch ? highlightText(val, props.query) : val}
          &quot;
        </span>
      )
    }

    if (Array.isArray(val)) {
      const isExpanded = expanded.has(path)
      const itemCount = val.length
      return (
        <div className="gs-r-json-array">
          <button
            className="gs-r-json-toggle"
            onClick={() => toggleExpand(path)}
            aria-expanded={isExpanded}
          >
            <span className="gs-r-json-arrow">{isExpanded ? '▼' : '▶'}</span>
            <span className="gs-r-json-bracket">[</span>
            <span className="gs-r-json-count">{itemCount}</span>
            <span className="gs-r-json-bracket">]</span>
          </button>
          {isExpanded && (
            <div className="gs-r-json-children">
              {val.map((item, i) => (
                <div key={i} className="gs-r-json-item">
                  <span className="gs-r-json-index">{i}</span>
                  {': '}
                  {renderValue(item, `${path}[${i}]`, depth + 1)}
                </div>
              ))}
            </div>
          )}
        </div>
      )
    }

    if (typeof val === 'object') {
      const isExpanded = expanded.has(path)
      const entries = Object.entries(val)
      const keyCount = entries.length
      return (
        <div className="gs-r-json-object">
          <button
            className="gs-r-json-toggle"
            onClick={() => toggleExpand(path)}
            aria-expanded={isExpanded}
          >
            <span className="gs-r-json-arrow">{isExpanded ? '▼' : '▶'}</span>
            <span className="gs-r-json-bracket">{'{'}</span>
            <span className="gs-r-json-count">{keyCount}</span>
            <span className="gs-r-json-bracket">{'}'}</span>
          </button>
          {isExpanded && (
            <div className="gs-r-json-children">
              {entries.map(([k, v]) => {
                const isKeyMatch =
                  props.query && k.toLowerCase().includes(props.query.toLowerCase())
                return (
                  <div key={k} className="gs-r-json-item">
                    <span
                      className={`gs-r-json-key ${isKeyMatch ? 'gs-r-json-key--match' : ''}`}
                    >
                      &quot;{isKeyMatch ? highlightText(k, props.query) : k}&quot;
                    </span>
                    {': '}
                    {renderValue(v, `${path}.${k}`, depth + 1)}
                  </div>
                )
              })}
            </div>
          )}
        </div>
      )
    }

    return <span className="gs-r-json-unknown">{String(val)}</span>
  }

  const highlightText = (text: string, query: string): JSX.Element => {
    if (!query) return <>{text}</>

    const lower = text.toLowerCase()
    const queryLower = query.toLowerCase()
    const idx = lower.indexOf(queryLower)

    if (idx < 0) return <>{text}</>

    return (
      <>
        {text.slice(0, idx)}
        <mark>{text.slice(idx, idx + query.length)}</mark>
        {text.slice(idx + query.length)}
      </>
    )
  }

  return (
    <div data-testid="renderer-json" className="gs-r-json-root">
      <div className="gs-r-json-controls">
        <button className="gs-r-json-button" onClick={expandAll}>
          Expand all
        </button>
        <button className="gs-r-json-button" onClick={collapseAll}>
          Collapse all
        </button>
      </div>

      <div className="gs-r-json-container">
        <pre className="gs-r-json-tree">{renderValue(data, 'root')}</pre>
      </div>
    </div>
  )
}
