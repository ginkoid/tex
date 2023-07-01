import { basicSetup, EditorView } from 'codemirror'
import { StreamLanguage } from '@codemirror/language'
import { stex } from '@codemirror/legacy-modes/mode/stex'
import { keymap } from '@codemirror/view'
import { indentWithTab } from '@codemirror/commands'
import './app.css'
import preamble from '../render/preamble.tex'

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
  const downloadEl = document.getElementById('download')
  const errorEl = document.getElementById('error')

  const setError = (error) => {
    const match = error.match(/^\*?!(.*?)^!/ms)
    errorEl.textContent = match?.[1].trim() ?? error
    errorEl.classList.remove('hidden')
    imageEl.srcset = ''
    imageEl.classList.add('hidden')
    downloadEl.href = ''
    downloadEl.classList.add('hidden')
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
        downloadEl.classList.remove('hidden')
      }
    } catch (err) {
      if (err instanceof DOMException && err.name === 'AbortError') {
        return
      }
      setError(err.message)
    }
  }
  const debouncedUpdateImage = debounce(200, updateImage)
  const handleInput = (content) => {
    localStorage.content = content
    debouncedUpdateImage(content)
  }

  const view = new EditorView({
    doc: localStorage.content || initialValue,
    extensions: [
      basicSetup,
      EditorView.lineWrapping,
      keymap.of([indentWithTab]),
      StreamLanguage.define(stex),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          handleInput(update.state.doc.toString())
        }
      }),
    ],
    parent: document.getElementById('editor'),
  })
  updateImage(view.state.doc.toString())
})()
