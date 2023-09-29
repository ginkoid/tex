import { EditorView } from 'codemirror'
import extensions from './codemirror'
import './app.css'
import preamble from '../render/preamble.tex'

const maximumUrlLength = 2000
const initialValue = String.raw`
\TeX.flag.sh can render \textbf{text}, \( m \alpha \tau h \), and even pictures:
\center{\scalebox{0.3}{\begin{tikzpicture} \duck[hat] \end{tikzpicture}}}
`.trimStart()

void (async () => {
  await new Promise(resolve => addEventListener('DOMContentLoaded', resolve))

  document.getElementById('preamble').textContent = preamble

  const debounce = (time, cb) => {
    let timeout
    return (...args) => {
      clearTimeout(timeout)
      timeout = setTimeout(() => cb(...args), time)
    }
  }

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
    if (url.href.length > maximumUrlLength) {
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

  const handleInput = (content) => {
    localStorage.content = content
    renderEl.classList.add('dirty')
    updateLink(content)
    debouncedUpdateImage(content)
  }

  const view = new EditorView({
    doc: localStorage.content || initialValue,
    extensions: [
      extensions,
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          handleInput(update.state.doc.toString())
        }
      }),
    ],
    parent: document.getElementById('editor'),
  })
  handleInput(view.state.doc.toString())
})()
