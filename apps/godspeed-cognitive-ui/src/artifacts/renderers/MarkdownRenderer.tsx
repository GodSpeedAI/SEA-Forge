import { useEffect, useState } from 'react'
import React from 'react'
import type { SourceRendererProps } from '../registry'
import type { MarkdownBlock } from '../model'
import { highlight } from './highlight'
import './MarkdownRenderer.css'

export default function MarkdownRenderer(props: SourceRendererProps) {
  if (props.model.kind !== 'markdown') throw new Error('MarkdownRenderer expects kind="markdown"')

  const blocks = props.model.blocks as MarkdownBlock[]
  const [selectedBlockIdx, setSelectedBlockIdx] = useState<number | null>(null)

  // Search highlight
  useEffect(() => {
    let totalMatches = 0
    blocks.forEach((block) => {
      totalMatches += highlight(block.text, props.query).count
    })
    props.onMatches?.(totalMatches)
  }, [props.query, blocks, props])

  // Group consecutive li blocks into ul
  const groupedBlocks: Array<MarkdownBlock | MarkdownBlock[]> = []
  let currentList: MarkdownBlock[] = []

  for (const block of blocks) {
    if (block.type === 'li') {
      currentList.push(block)
    } else {
      if (currentList.length > 0) {
        groupedBlocks.push(currentList)
        currentList = []
      }
      groupedBlocks.push(block)
    }
  }
  if (currentList.length > 0) {
    groupedBlocks.push(currentList)
  }

  const handleBlockClick = (idx: number) => {
    setSelectedBlockIdx(selectedBlockIdx === idx ? null : idx)
  }

  return (
    <div data-testid="renderer-markdown" className="gs-r-md-root" onWheel={(e) => e.stopPropagation()}>
      <article className="gs-r-md-content">
        {groupedBlocks.map((blockOrList, idx) => {
          const isSelected = selectedBlockIdx === idx
          const isListGroup = Array.isArray(blockOrList)

          if (isListGroup) {
            return (
              <ul
                key={idx}
                className="gs-r-md-list"
                data-selected={isSelected ? 'true' : undefined}
                onClick={() => handleBlockClick(idx)}
              >
                {blockOrList.map((block, itemIdx) => (
                  <li key={itemIdx}>
                    <HighlightedText text={block.text} query={props.query} />
                  </li>
                ))}
              </ul>
            )
          }

          const block = blockOrList as MarkdownBlock

          switch (block.type) {
            case 'h1':
              return (
                <h1
                  key={idx}
                  className="gs-r-md-h1"
                  data-selected={isSelected ? 'true' : undefined}
                  onClick={() => handleBlockClick(idx)}
                >
                  <HighlightedText text={block.text} query={props.query} />
                </h1>
              )
            case 'h2':
              return (
                <h2
                  key={idx}
                  className="gs-r-md-h2"
                  data-selected={isSelected ? 'true' : undefined}
                  onClick={() => handleBlockClick(idx)}
                >
                  <HighlightedText text={block.text} query={props.query} />
                </h2>
              )
            case 'h3':
              return (
                <h3
                  key={idx}
                  className="gs-r-md-h3"
                  data-selected={isSelected ? 'true' : undefined}
                  onClick={() => handleBlockClick(idx)}
                >
                  <HighlightedText text={block.text} query={props.query} />
                </h3>
              )
            case 'p':
              return (
                <p
                  key={idx}
                  className="gs-r-md-p"
                  data-selected={isSelected ? 'true' : undefined}
                  onClick={() => handleBlockClick(idx)}
                >
                  <HighlightedText text={block.text} query={props.query} />
                </p>
              )
            case 'quote':
              return (
                <blockquote
                  key={idx}
                  className="gs-r-md-quote"
                  data-selected={isSelected ? 'true' : undefined}
                  onClick={() => handleBlockClick(idx)}
                >
                  <HighlightedText text={block.text} query={props.query} />
                </blockquote>
              )
            case 'code':
              return (
                <pre
                  key={idx}
                  className="gs-r-md-code"
                  data-selected={isSelected ? 'true' : undefined}
                  onClick={() => handleBlockClick(idx)}
                >
                  <code>
                    <HighlightedText text={block.text} query={props.query} />
                  </code>
                </pre>
              )
            default:
              return null
          }
        })}
      </article>
    </div>
  )
}

function HighlightedText({ text, query }: { text: string; query: string }) {
  const { parts } = highlight(text, query)
  return (
    <>
      {parts.map((part, i) => (
        <React.Fragment key={i}>
          {part.hit ? <mark>{part.text}</mark> : part.text}
        </React.Fragment>
      ))}
    </>
  )
}
