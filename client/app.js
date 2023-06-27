import { basicSetup, EditorView } from 'codemirror'
import { StreamLanguage } from '@codemirror/language'
import { stex } from '@codemirror/legacy-modes/mode/stex'
import { keymap } from '@codemirror/view'
import { indentWithTab } from '@codemirror/commands'

void (async () => {
    await new Promise((res) => window.addEventListener('DOMContentLoaded', res))

    const imageEl = document.getElementById('image')

    const updateImage = (content) => {
        imageEl.src = '/render/' + encodeURIComponent(content)
    }

    const view = new EditorView({
        doc: '\\( 1 + 1 \\)',
        extensions: [
            basicSetup,
            keymap.of([indentWithTab]),
            StreamLanguage.define(stex),
            EditorView.updateListener.of((update) => {
                if (update.docChanged) {
                    updateImage(update.state.doc.toString())
                }
            }),
        ],
        parent: document.getElementById('editor'),
    })
    updateImage(view.state.doc.toString())
})()
