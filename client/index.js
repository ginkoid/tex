import { basicSetup, EditorView } from 'codemirror'

void (async () => {
    await new Promise((res) => window.addEventListener('load', res))

    new EditorView({
        doc: '\\( 1 + 1 \\)',
        extensions: [basicSetup],
        parent: document.body,
    })
})()
