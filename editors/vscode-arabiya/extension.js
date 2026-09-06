const vscode = require("vscode");

const VIEW_TYPE = "arabiya.rtlEditor";

function editorOptions() {
    const settings = vscode.workspace.getConfiguration("arabiya.rtlEditor");
    return {
        fontFamily: settings.get("fontFamily"),
        fontSize: settings.get("fontSize"),
        tabSize: settings.get("tabSize"),
    };
}

/** @param {vscode.ExtensionContext} context */
function activate(context) {
    const provider = {
        async resolveCustomTextEditor(document, webviewPanel) {
            const { webview } = webviewPanel;
            webview.options = { enableScripts: true };
            webview.html = webviewHtml(webview);

            let lastAppliedText;
            const postDocument = () => webview.postMessage({ type: "document", text: document.getText() });
            const postOptions = () => webview.postMessage({ type: "options", ...editorOptions() });

            const applyText = async (text) => {
                if (text === document.getText()) {
                    return;
                }
                lastAppliedText = text;
                const edit = new vscode.WorkspaceEdit();
                const fullDocument = new vscode.Range(
                    document.positionAt(0),
                    document.positionAt(document.getText().length),
                );
                edit.replace(document.uri, fullDocument, text);
                await vscode.workspace.applyEdit(edit);
            };

            webview.onDidReceiveMessage(async (message) => {
                if (message.type === "edit" && typeof message.text === "string") {
                    await applyText(message.text);
                }
                if (message.type === "save" && typeof message.text === "string") {
                    await applyText(message.text);
                    await document.save();
                }
            }, undefined, context.subscriptions);

            const documentListener = vscode.workspace.onDidChangeTextDocument((event) => {
                if (event.document.uri.toString() !== document.uri.toString()) {
                    return;
                }
                if (lastAppliedText === document.getText()) {
                    lastAppliedText = undefined;
                    return;
                }
                postDocument();
            });
            const settingsListener = vscode.workspace.onDidChangeConfiguration((event) => {
                if (event.affectsConfiguration("arabiya.rtlEditor")) {
                    postOptions();
                }
            });
            webviewPanel.onDidDispose(() => {
                documentListener.dispose();
                settingsListener.dispose();
            }, undefined, context.subscriptions);

            postDocument();
            postOptions();
        },
    };

    context.subscriptions.push(
        vscode.window.registerCustomEditorProvider(VIEW_TYPE, provider, {
            webviewOptions: { retainContextWhenHidden: true },
        }),
        vscode.commands.registerCommand("arabiya.openRtlEditor", async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== "arabiya") {
                vscode.window.showErrorMessage("Open an .ar file before opening the Arabiya RTL editor.");
                return;
            }
            await vscode.commands.executeCommand("vscode.openWith", editor.document.uri, VIEW_TYPE);
        }),
    );
}

function webviewHtml(webview) {
    const nonce = Array.from({ length: 32 }, () => Math.floor(Math.random() * 36).toString(36)).join("");
    return /* html */ `<!doctype html>
<html lang="ar" dir="rtl">
<head>
  <meta charset="utf-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'nonce-${nonce}'; script-src 'nonce-${nonce}';">
  <style nonce="${nonce}">
    :root { color-scheme: light dark; }
    body { margin: 0; height: 100vh; overflow: hidden; background: var(--vscode-editor-background); }
    #editor {
      box-sizing: border-box; width: 100%; height: 100%; resize: none; border: 0; outline: 0;
      padding: 16px 20px; background: var(--vscode-editor-background); color: var(--vscode-editor-foreground);
      font-family: Cascadia Code, 'Noto Sans Arabic', monospace; font-size: 16px; line-height: 1.55;
      direction: rtl; text-align: right; unicode-bidi: plaintext; white-space: pre; overflow: auto;
    }
  </style>
</head>
<body>
  <textarea id="editor" dir="rtl" spellcheck="false" aria-label="محرر العربية البرمجية"></textarea>
  <script nonce="${nonce}">
    const vscode = acquireVsCodeApi();
    const editor = document.getElementById('editor');
    let tabSize = 4;
    let timer;

    function sendEdit() {
      clearTimeout(timer);
      vscode.postMessage({ type: 'edit', text: editor.value });
    }
    editor.addEventListener('input', () => {
      clearTimeout(timer);
      timer = setTimeout(sendEdit, 120);
    });
    editor.addEventListener('keydown', (event) => {
      if (event.key === 'Tab') {
        event.preventDefault();
        const start = editor.selectionStart;
        const end = editor.selectionEnd;
        editor.setRangeText(' '.repeat(tabSize), start, end, 'end');
        editor.dispatchEvent(new Event('input'));
      }
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') {
        event.preventDefault();
        clearTimeout(timer);
        vscode.postMessage({ type: 'save', text: editor.value });
      }
    });
    window.addEventListener('message', (event) => {
      const message = event.data;
      if (message.type === 'document' && message.text !== editor.value) {
        const start = editor.selectionStart;
        const end = editor.selectionEnd;
        editor.value = message.text;
        editor.setSelectionRange(Math.min(start, editor.value.length), Math.min(end, editor.value.length));
      }
      if (message.type === 'options') {
        tabSize = message.tabSize;
        editor.style.fontFamily = message.fontFamily;
        editor.style.fontSize = message.fontSize + 'px';
        editor.style.tabSize = String(tabSize);
      }
    });
  </script>
</body>
</html>`;
}

function deactivate() {}

module.exports = { activate, deactivate };
