import { basicSetup, EditorView } from 'codemirror'
import { StreamLanguage } from '@codemirror/language'
import { stex } from '@codemirror/legacy-modes/mode/stex'
import { keymap } from '@codemirror/view'
import { indentWithTab } from '@codemirror/commands'

void (async () => {
    await new Promise((res) => window.addEventListener('DOMContentLoaded', res))

    const debounce = (time, cb) => {
        let timeout
        return (...args) => {
            clearTimeout(timeout)
            timeout = setTimeout(() => cb(...args), time)
        }
    }

    const imageEl = document.getElementById('image')
    const errorEl = document.getElementById('error')

    let lastAbort
    const update = async (content) => {
        lastAbort?.abort()
        lastAbort = new AbortController()
        try {
            const res = await fetch('/render', {
                method: 'POST',
                body: content,
                signal: lastAbort.signal,
            })
            if (!res.ok) {
                errorEl.textContent = await res.text()
                imageEl.src = ''
            } else {
                errorEl.textContent = ''
                imageEl.src = URL.createObjectURL(await res.blob())
            }
        } catch (err) {
            if (err instanceof DOMException && err.name === 'AbortError') {
                return
            }
            errorEl.textContent = err.message
        }
    }
    const debouncedUpdate = debounce(200, update)

    const view = new EditorView({
        doc: '\\( 1 + 1 \\)',
        extensions: [
            basicSetup,
            keymap.of([indentWithTab]),
            StreamLanguage.define(stex),
            EditorView.updateListener.of((update) => {
                if (update.docChanged) {
                    debouncedUpdate(update.state.doc.toString())
                }
            }),
        ],
        parent: document.getElementById('editor'),
    })
    update(view.state.doc.toString())
})()
