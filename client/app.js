import { EditorView } from 'codemirror'
import extensions from './codemirror'
import preamble from '../render/preamble.tex'
import './app.css'

const maxUrlLength = 2000
const initialValue = String.raw`
\TeX.flag.sh can render \textbf{text}, \( m \alpha \tau h \), and even pictures:
\center{\scalebox{0.3}{\begin{tikzpicture} \duck[hat] \end{tikzpicture}}}
`.trimStart()

const debounce = (time, cb) => {
  let timeout
  return (...args) => {
    clearTimeout(timeout)
    timeout = setTimeout(() => cb(...args), time)
  }
}

addEventListener('DOMContentLoaded', () => {
  document.getElementById('preamble').textContent = preamble

  const imageEl = document.getElementById('image')
  const actionsEl = document.getElementById('actions')
  const downloadEl = document.getElementById('download')
  const copyEl = document.getElementById('copy')
  const errorEl = document.getElementById('error')
  const renderEl = document.getElementById('render')

  const setError = (error) => {
    const match = error.match(/^\*?!(.*?)^!/ms)
    errorEl.textContent = match?.[1].trim() ?? error
    errorEl.classList.remove('hidden')
    imageEl.srcset = ''
    imageEl.classList.add('hidden')
    downloadEl.href = ''
    actionsEl.classList.add('hidden')
  }

  let lastAbort
  let lastObjectUrl
  const updateImage = async (content) => {
    lastAbort?.abort()
    lastAbort = new AbortController()
    try {
      const res = await fetch('/render', {
        method: 'POST',
        body: content,
        signal: lastAbort.signal,
      })
      if (lastObjectUrl) {
        URL.revokeObjectURL(lastObjectUrl)
      }
      if (!res.ok) {
        setError(await res.text())
      } else {
        errorEl.textContent = ''
        errorEl.classList.add('hidden')
        lastObjectUrl = URL.createObjectURL(await res.blob())
        imageEl.srcset = `${lastObjectUrl} 2x`
        imageEl.classList.remove('hidden')
        downloadEl.href = lastObjectUrl
        actionsEl.classList.remove('hidden')
      }
    } catch (err) {
      if (err instanceof DOMException && err.name === 'AbortError') {
        return
      }
      setError(err.message)
    } finally {
      renderEl.classList.remove('dirty')
    }
  }
  const debouncedUpdateImage = debounce(200, updateImage)

  let imageUrl
  const updateLink = (content) => {
    const url = new URL(`/render/${encodeURIComponent(content)}`, location)
    if (url.href.length > maxUrlLength) {
      imageUrl = undefined
      copyEl.classList.add('hidden')
    } else {
      imageUrl = url.href
      copyEl.classList.remove('hidden')
    }
  }
  copyEl.addEventListener('click', () => {
    if (imageUrl) {
      navigator.clipboard.writeText(imageUrl)
    }
  })

  const handleInput = (immediate, content) => {
    localStorage.content = content
    renderEl.classList.add('dirty')
    updateLink(content)
    if (immediate) {
      updateImage(content)
    } else {
      debouncedUpdateImage(content)
    }
  }
  const value = localStorage.content || initialValue
  handleInput(true, value)
  new EditorView({
    doc: value,
    extensions: [
      extensions,
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          handleInput(false, update.state.doc.toString())
        }
      }),
    ],
    parent: document.getElementById('editor'),
  })
})
