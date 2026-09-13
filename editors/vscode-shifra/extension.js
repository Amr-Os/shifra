const vscode = require("vscode");
const cp = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");

const VIEW_TYPE = "shifra.rtlEditor";

/** All live RTL editor panels, used to broadcast run errors. */
const errorPanels = new Set();
const errorLogPath = path.join(os.tmpdir(), "shifra-run-errors.log");

/** Live problems panel collection for Shifra source diagnostics. */
const diagnostics = vscode.languages.createDiagnosticCollection("shifra");

/** Status-bar mirror of run state (idle / running / error). */
let statusBarItem = null;
let runErrorState = false;
/** Persisted width of the hover documentation widget. */
let hoverWidthState = 380;
/** The document of the most recent run, used to publish diagnostics. */
let activeRunDoc = null;

// ---------------------------------------------------------------- settings --

function editorOptions() {
    const settings = vscode.workspace.getConfiguration("shifra.rtlEditor");
    return {
        fontFamily: settings.get("fontFamily"),
        codeFont: settings.get("fontFamily"),
        fontSize: settings.get("fontSize"),
        tabSize: settings.get("tabSize"),
        lineSpacing: settings.get("lineSpacing"),
        hoverWidth: hoverWidthState,
    };
}

/** Strip any LTR embedding markers that leaked from the webview editor. */
function stripEmbeds(text) {
    return String(text).replace(/[\u202a\u202c]/g, "");
}

// ---------------------------------------------------------------- running --

/** Quote a path/argument for the shell. */
function shellQuote(value) {
    return `"${String(value).replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

/** Walk up from `startDir` to find the nearest Cargo workspace root. */
function findWorkspaceRoot(startDir) {
    let dir = startDir;
    while (dir) {
        const cargo = path.join(dir, "Cargo.toml");
        try {
            if (fs.existsSync(cargo) && fs.readFileSync(cargo, "utf8").includes("[workspace]")) {
                return dir;
            }
        } catch {
            // ignore unreadable Cargo.toml and keep walking
        }
        const parent = path.dirname(dir);
        if (parent === dir) {
            return undefined;
        }
        dir = parent;
    }
    return undefined;
}

/** Locate the RustPython Cargo workspace root. */
function findRepoRoot() {
    const configured = vscode.workspace.getConfiguration("shifra.runtime").get("projectDir");
    if (configured) {
        const cargo = path.join(configured, "Cargo.toml");
        if (fs.existsSync(cargo) && fs.readFileSync(cargo, "utf8").includes("[workspace]")) {
            return configured;
        }
    }
    if (process.env.SHIFRA_DIR) {
        return process.env.SHIFRA_DIR;
    }
    if (process.env.RUSTPYTHON_DIR) {
        return process.env.RUSTPYTHON_DIR;
    }
    for (const folder of vscode.workspace.workspaceFolders ?? []) {
        const root = findWorkspaceRoot(folder.uri.fsPath);
        if (root) {
            return root;
        }
    }
    // When developing, the extension lives inside the Shifra checkout:
    const devRoot = findWorkspaceRoot(path.dirname(__dirname));
    if (devRoot) {
        return devRoot;
    }
    // Common install locations:
    const home = process.env.HOME || process.env.USERPROFILE;
    for (const p of [
        path.join(home, "Desktop/codes/Arabic/Shifra"),
        path.join(home, "Desktop/codes/Arabic/RustPython"),
        path.join(home, "codes/Arabic/Shifra"),
        path.join(home, "codes/Arabic/RustPython"),
        path.join(home, "Shifra"),
        path.join(home, "RustPython"),
    ]) {
        if (fs.existsSync(path.join(p, "Cargo.toml"))) {
            return p;
        }
    }
    return undefined;
}

/** Decide which executable runs `.ar` files and in which directory. */
function resolveRunner(uri) {
    const fileDir = path.dirname(uri.fsPath);
    const configured = vscode.workspace.getConfiguration("shifra.runtime").get("command");
    if (configured) {
        return { command: configured, cwd: fileDir };
    }

    const repoRoot =
        findRepoRoot() ??
        findWorkspaceRoot(fileDir) ??
        vscode.workspace.getWorkspaceFolder(uri)?.uri.fsPath ??
        fileDir;

    for (const rel of ["target/release/rustpython", "target/debug/rustpython"]) {
        const bin = path.join(repoRoot, rel);
        if (fs.existsSync(bin)) {
            return { command: shellQuote(bin), cwd: fileDir };
        }
    }
    const manifest = path.join(repoRoot, "Cargo.toml");
    return {
        command: `cargo run --release --manifest-path ${shellQuote(manifest)} --`,
        cwd: fileDir,
    };
}

/** Shifra file extensions: `.ar`, `.sf`, `.شفـ`. */
function isShifraPath(p) {
    return /\.(ar|sf|شفـ)$/.test(p);
}

function isAraDocument(document) {
    return document.languageId === "shifra" || isShifraPath(document.uri.path);
}

/** Extract the source line and message from a Python-style traceback. */
function parseError(text) {
    const re = /line (\d+)/g;
    let line = 0;
    let m;
    while ((m = re.exec(text))) {
        line = parseInt(m[1], 10);
    }
    if (!line) {
        return null;
    }
    const lines = text.trim().split("\n");
    const message = lines[lines.length - 1].trim() || "runtime error";
    return { line, message };
}

/** Status-bar wiring. The Shifra item appears only when a Shifra doc is active. */
function setRunStatus(kind) {
    if (kind === "running") {
        runErrorState = false;
        if (statusBarItem) statusBarItem.text = "$(sync~spin) Shifra";
    } else if (kind === "error") {
        runErrorState = true;
        if (statusBarItem) statusBarItem.text = "$(error) Shifra";
    } else {
        runErrorState = false;
        if (statusBarItem) statusBarItem.text = "$(code) Shifra";
    }
}

function updateStatusBar() {
    const activeShifra = vscode.window.activeTextEditor && isAraDocument(vscode.window.activeTextEditor.document);
    if (activeShifra) {
        if (!statusBarItem) {
            statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
            statusBarItem.command = "shifra.checkSyntax";
        }
        setRunStatus(runErrorState ? "error" : "idle");
        statusBarItem.show();
    } else if (statusBarItem) {
        statusBarItem.hide();
    }
}

/** Publish an error diagnostic on `line` (1-based) of `document`; clear when line is falsy. */
function setErrorDiagnostic(document, line, message) {
    if (!document) {
        return;
    }
    if (!line) {
        diagnostics.delete(document.uri);
        return;
    }
    diagnostics.set(document.uri, [
        {
            range: new vscode.Range(line - 1, 0, line - 1, Number.MAX_SAFE_INTEGER),
            message,
            severity: vscode.DiagnosticSeverity.Error,
            source: "Shifra",
        },
    ]);
}

/** Plain executable path for spawning (no shell quoting), or null. */
function interpreterCommand() {
    const configured = vscode.workspace.getConfiguration("shifra.runtime").get("command");
    if (configured) {
        return { command: configured.replace(/^"(.*)"$/, "$1"), args: [] };
    }
    const repoRoot =
        findRepoRoot() ??
        (vscode.workspace.workspaceFolders?.[0]?.uri.fsPath
            ? findWorkspaceRoot(vscode.workspace.workspaceFolders[0].uri.fsPath)
            : undefined);
    for (const rel of ["target/release/rustpython", "target/debug/rustpython"]) {
        const bin = repoRoot && path.join(repoRoot, rel);
        if (bin && fs.existsSync(bin)) {
            return { command: bin, args: [] };
        }
    }
    return null;
}

let syntaxCheckTimer = null;

function scheduleSyntaxCheck(document) {
    if (!document || document.isUntitled || !isAraDocument(document)) {
        return;
    }
    clearTimeout(syntaxCheckTimer);
    syntaxCheckTimer = setTimeout(() => runSyntaxCheck(document), 700);
}

/** Compile-only check (`--check`) that feeds the Problems panel without a terminal. */
function runSyntaxCheck(document) {
    if (!document || document.isClosed) {
        return;
    }
    const interp = interpreterCommand();
    if (!interp) {
        return;
    }
    cp.execFile(
        interp.command,
        [...interp.args, "--check", document.uri.fsPath],
        { timeout: 15000 },
        (err, _stdout, stderr) => {
            if (document.isClosed) {
                return;
            }
            if (err) {
                const error = parseError(stderr || "");
                if (error) {
                    setErrorDiagnostic(document, error.line, error.message);
                    return;
                }
            }
            diagnostics.delete(document.uri);
        },
    );
}

/** Poll the stderr log until the run settles, then report the first error. */
function watchRunOutput() {
    try {
        fs.writeFileSync(errorLogPath, "");
    } catch {
        return;
    }
    let last = "";
    let stableMs = 0;
    const started = Date.now();
    const timer = setInterval(() => {
        try {
            const data = fs.readFileSync(errorLogPath, "utf8");
            if (data !== last) {
                last = data;
                stableMs = 0;
                if (data.trim()) {
                    const error = parseError(data);
                    if (error) {
                        clearInterval(timer);
                        setRunStatus("error");
                        setErrorDiagnostic(activeRunDoc, error.line, error.message);
                        for (const panel of errorPanels) {
                            panel.webview.postMessage({ type: "error", line: error.line, message: error.message });
                        }
                        vscode.window.showErrorMessage(`Shifra — ${error.message}`);
                        return;
                    }
                }
            } else {
                stableMs += 500;
                if (stableMs >= 1500 || Date.now() - started > 30000) {
                    clearInterval(timer);
                    setRunStatus("idle");
                }
            }
        } catch {
            clearInterval(timer);
            setRunStatus("idle");
        }
    }, 500);
}

async function runDocument(document) {
    if (!isAraDocument(document)) {
        vscode.window.showWarningMessage("The Shifra Run command only works for Shifra (.ar/.sf/.شفـ) files.");
        return;
    }
    if (document.isUntitled) {
        vscode.window.showWarningMessage("Save the file before running it.");
        return;
    }

    const runner = resolveRunner(document.uri);
    let terminal = vscode.window.terminals.find(
        (t) => t.name === vscode.workspace.getConfiguration("shifra.runtime").get("terminalName"),
    );
    if (!terminal) {
        terminal = vscode.window.createTerminal({
            name: vscode.workspace.getConfiguration("shifra.runtime").get("terminalName"),
            cwd: runner.cwd,
        });
    }
    for (const panel of errorPanels) {
        panel.webview.postMessage({ type: "clearError" });
    }
    activeRunDoc = document;
    setErrorDiagnostic(document, 0, null);
    setRunStatus("running");
    const capture =
        process.platform !== "win32" &&
        vscode.workspace.getConfiguration("shifra.runtime").get("captureErrors") &&
        `2> >(tee ${shellQuote(errorLogPath)} 1>&2)`;
    terminal.show(true);
    terminal.sendText(`${runner.command} ${shellQuote(document.uri.fsPath)}${capture ? " " + capture : ""}`);
    if (capture) {
        watchRunOutput();
    }
}

async function runFile() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showWarningMessage("There is no active editor to run.");
        return;
    }
    return runDocument(editor.document);
}

// -------------------------------------------------------------- formatting --

/**
 * Normalize `.ar` formatting: 4-space indentation (structure read from the
 * existing indentation, so nothing is re-nested), trailing-space removal, a
 * single blank line cap, and a final newline. Returns undefined when the file
 * contains multi-line strings and is left untouched.
 */
function formatShifra(text) {
    const source = text.replace(/\r\n/g, "\n");
    if (source.includes('"""') || source.includes("'''")) {
        return undefined;
    }
    const out = [];
    let blankRun = 0;
    for (const raw of source.split("\n")) {
        const stripped = raw.trimEnd();
        if (stripped.trim() === "") {
            if (blankRun < 1) {
                out.push("");
                blankRun++;
            }
            continue;
        }
        blankRun = 0;
        const leadingSpaces = stripped.length - stripped.trimStart().length;
        out.push(" ".repeat(Math.max(0, Math.round((leadingSpaces - 0.5) / 4) * 4)) + stripped.trimStart());
    }
    while (out.length > 0 && out[out.length - 1] === "") {
        out.pop();
    }
    return out.length ? out.join("\n") + "\n" : "";
}

function formatDocument(document) {
    const formatted = formatShifra(document.getText());
    if (formatted === undefined) {
        return [];
    }
    if (formatted === document.getText().replace(/\r\n/g, "\n")) {
        return [];
    }
    const end = document.positionAt(document.getText().length);
    return [vscode.TextEdit.replace(new vscode.Range(0, 0, end.line, end.character), formatted)];
}

// ------------------------------------------------------------------- RTL ----

function activate(context) {
    hoverWidthState = context.workspaceState.get("shifra.hoverWidth", 380);
    context.subscriptions.push(
        vscode.workspace.onDidOpenTextDocument((doc) => {
            if (isShifraPath(doc.uri.path) && doc.languageId !== "shifra") {
                vscode.languages.setTextDocumentLanguage(doc, "shifra");
            }
            scheduleSyntaxCheck(doc);
        }),
        vscode.workspace.onDidChangeTextDocument((event) => {
            const doc = event.document;
            if (isShifraPath(doc.uri.path) && doc.languageId !== "shifra") {
                vscode.languages.setTextDocumentLanguage(doc, "shifra");
            }
            scheduleSyntaxCheck(doc);
        }),
    );

    context.subscriptions.push(
        vscode.languages.registerDocumentSymbolProvider("shifra", {
            provideDocumentSymbols(document) {
                const symbols = [];
                const lines = document.getText().split("\n");
                for (let i = 0; i < lines.length; i++) {
                    const trimmed = lines[i].trim();
                    const m =
                        /^(?:لازمني\s+)?عرف\s+([\p{L}_][\p{L}\p{N}_]*)/u.exec(trimmed) ||
                        /^صنف\s+([\p{L}_][\p{L}\p{N}_]*)/u.exec(trimmed);
                    if (!m) {
                        continue;
                    }
                    const range = new vscode.Range(i, 0, i, lines[i].length);
                    const kind = trimmed.startsWith("صنف") ? vscode.SymbolKind.Class : vscode.SymbolKind.Function;
                    symbols.push(new vscode.DocumentSymbol(m[1], kind === vscode.SymbolKind.Class ? "class" : "function", kind, range, range));
                }
                return symbols;
            },
        }),
        vscode.languages.registerFoldingRangeProvider("shifra", {
            provideFoldingRanges(document) {
                const lines = document.getText().split("\n");
                const ranges = [];
                const stack = [];
                for (let i = 0; i < lines.length; i++) {
                    const raw = lines[i].replace(/\r$/, "");
                    const stripped = raw.replace(/^\s+/, "");
                    if (stripped === "" || stripped.startsWith("#")) {
                        continue;
                    }
                    const indent = raw.length - stripped.length;
                    while (stack.length && indent <= stack[stack.length - 1].indent) {
                        const top = stack.pop();
                        if (top.end !== undefined && top.end - top.start > 1) {
                            ranges.push(new vscode.FoldingRange(top.start, top.end));
                        }
                    }
                    if (stack.length && stack[stack.length - 1].end === undefined) {
                        stack[stack.length - 1].end = i;
                    }
                    if (/:\s*(#.*)?$/.test(stripped)) {
                        stack.push({ indent, start: i });
                    }
                }
                while (stack.length) {
                    const top = stack.pop();
                    if (top.end !== undefined && top.end - top.start > 1) {
                        ranges.push(new vscode.FoldingRange(top.start, top.end));
                    }
                }
                const n = lines.length;
                for (let i = 0; i < n; i++) {
                    if (/^\s*#/.test(lines[i])) {
                        let j = i + 1;
                        while (j < n && /^\s*#/.test(lines[j])) {
                            j++;
                        }
                        if (j - 1 > i) {
                            ranges.push(new vscode.FoldingRange(i, j - 1));
                        }
                        i = j - 1;
                    }
                }
                return ranges;
            },
        }),
        vscode.commands.registerCommand("shifra.checkSyntax", () => {
            const editor = vscode.window.activeTextEditor;
            if (editor) {
                runSyntaxCheck(editor.document);
            }
        }),
        vscode.window.onDidChangeActiveTextEditor(() => updateStatusBar()),
    );

    context.subscriptions.push(
        vscode.languages.registerDocumentFormattingEditProvider("shifra", {
            provideDocumentFormattingEdits: formatDocument,
        }),
    );

    const createRtlPanel = (document, webviewPanel) => {
        const { webview } = webviewPanel;
        webview.options = { enableScripts: true };
        webview.html = webviewHtml(webview);
        errorPanels.add(webviewPanel);

        let lastAppliedText;
        const postDocument = () => webview.postMessage({ type: "document", text: document.getText() });
        const postOptions = () => webview.postMessage({ type: "options", ...editorOptions() });

        const applyText = async (text) => {
            text = stripEmbeds(text);
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

        webview.onDidReceiveMessage(
            async (message) => {
                if (message.type === "ready") {
                    // The webview script is now listening; the initial
                    // `document`/`options` messages posted before it loaded
                    // would have been dropped, so send them now.
                    postDocument();
                    postOptions();
                }
                if (message.type === "hoverWidth" && typeof message.width === "number") {
                    hoverWidthState = Math.round(Math.max(140, Math.min(message.width, 1200)));
                    await context.workspaceState.update("shifra.hoverWidth", hoverWidthState);
                }
                if (message.type === "edit" && typeof message.text === "string") {
                    await applyText(message.text);
                }
                if (message.type === "save" && typeof message.text === "string") {
                    await applyText(message.text);
                    await document.save();
                }
                if (message.type === "format") {
                    const formatted = formatShifra(document.getText());
                    if (formatted !== undefined && formatted !== document.getText().replace(/\r\n/g, "\n")) {
                        await applyText(formatted);
                    }
                }
                if (message.type === "run") {
                    await runDocument(document);
                }
            },
            undefined,
            context.subscriptions,
        );

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
            if (event.affectsConfiguration("shifra.rtlEditor")) {
                postOptions();
            }
        });
        webviewPanel.onDidDispose(() => {
            errorPanels.delete(webviewPanel);
            documentListener.dispose();
            settingsListener.dispose();
        }, undefined, context.subscriptions);

        postDocument();
        postOptions();
    };

    const provider = {
        async resolveCustomTextEditor(document, webviewPanel) {
            createRtlPanel(document, webviewPanel);
        },
    };

    context.subscriptions.push(
        vscode.window.registerCustomEditorProvider(VIEW_TYPE, provider, {
            webviewOptions: { retainContextWhenHidden: true },
        }),
        vscode.commands.registerCommand("shifra.runFile", runFile),
        vscode.commands.registerCommand("shifra.formatDocument", async () => {
            const editor = vscode.window.activeTextEditor;
            if (editor && isAraDocument(editor.document)) {
                await vscode.window.activeTextEditor.edit((edit) => {
                    for (const edit of formatDocument(editor.document)) {
                        edit(edit);
                    }
                });
            }
        }),
        vscode.commands.registerCommand("shifra.openRtlEditor", async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== "shifra") {
                vscode.window.showErrorMessage("Open a Shifra (.ar/.sf/.شفـ) file before opening the Shifra RTL editor.");
                return;
            }
            await vscode.commands.executeCommand("vscode.openWith", editor.document.uri, VIEW_TYPE);
        }),
    );

    updateStatusBar();
}

// ------------------------------------------------------------------ webview --

function webviewHtml(webview) {
    const nonce = Array.from({ length: 32 }, () => Math.floor(Math.random() * 36).toString(36)).join("");
    return /* html */ `<!doctype html>
<html lang="ar">
<head>
  <meta charset="utf-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; script-src 'nonce-${nonce}';">
  <style nonce="${nonce}">
    :root { color-scheme: light dark; --font-family: 'JetBrains Mono', 'Thmanyah Sans Term', monospace; --font-size: 16px; --code-font: inherit; --code-size: 16px; --code-tab: 4; --code-line: 24px; --pad-bottom: calc(var(--code-line) * 1.5 + 22vh);
      --active-line-bg: rgba(255, 255, 255, 0.07); --glow: rgba(86, 156, 214, 0.95); }
    body { margin: 0; height: 100vh; display: flex; flex-direction: column; overflow: hidden; background: var(--vscode-editor-background); }
    .frame { display: flex; flex-direction: row; flex: 1; min-height: 0; direction: ltr; font-family: var(--code-font); font-size: var(--code-size); line-height: var(--code-line); }
    .gutter {
      flex: none; order: 2; min-width: 2ch; box-sizing: border-box; overflow: hidden; background: var(--vscode-editorGutter-background, transparent);
      color: var(--vscode-editorLineNumber-foreground, #858585); text-align: right; direction: ltr;
      white-space: pre; user-select: none; font-variant-numeric: tabular-nums; border-left: 1px solid var(--vscode-editorGroup-border, transparent);
      font-family: var(--code-font); font-size: var(--code-size); line-height: var(--code-line);
      padding: 16px 4px var(--pad-bottom) 10px;
    }
    .gutter span.active { color: var(--vscode-editorLineNumber-activeForeground, #e6e6e6); text-shadow: 0 0 6px var(--glow); }
    .code-wrap { position: relative; flex: 1; min-width: 0; overflow: hidden; direction: ltr; }
    .highlight, #editor {
      position: absolute; inset: 0; margin: 0; box-sizing: border-box; border: 0; outline: 0;
      padding: 16px 20px var(--pad-bottom) 20px; white-space: pre; overflow: auto; direction: rtl; unicode-bidi: plaintext; text-align: right;
      font-family: var(--code-font); font-size: var(--code-size); line-height: var(--code-line); letter-spacing: 0; font-variant: normal;
      tab-size: var(--code-tab);
    }
    .active-line { position: absolute; left: 0; right: 0; height: var(--code-line); background: var(--active-line-bg); pointer-events: none; }
    .err-line { position: absolute; left: 0; right: 0; height: var(--code-line); background: rgba(244, 63, 94, 0.16); border-left: 3px solid rgba(244, 63, 94, 0.85); pointer-events: none; }
    .word-hl { position: absolute; height: var(--code-line); background: rgba(255, 255, 255, 0.08); border-radius: 2px; pointer-events: none; z-index: 1; display: none; }
    #indent-guides { position: absolute; left: 0; top: 0; right: 0; bottom: 0; pointer-events: none; z-index: 0; overflow: hidden; }
    #indent-guides .indent-guide { position: absolute; width: 1px; background: rgba(51, 51, 51, 0.55); transition: background 0.15s, box-shadow 0.15s; }
    #indent-guides .indent-guide.active { background: #505050; box-shadow: 0 0 3px rgba(80, 80, 80, 0.5); }
    .fold-layer { position: absolute; left: 0; top: 0; right: 0; bottom: 0; z-index: 11; overflow: hidden; pointer-events: none; }
    .hoverdoc { position: absolute; z-index: 30; display: none; width: var(--hover-w, 380px); padding: 10px 14px; border-radius: 8px; line-height: 1.5;
      font-family: var(--vscode-font-family, 'Noto Sans Arabic', sans-serif); font-size: calc(var(--font-size) * 0.78);
      direction: rtl; text-align: right; white-space: normal; word-break: break-word; pointer-events: auto; box-sizing: border-box;
      background: var(--vscode-editorHoverWidget-background, #21252b); color: var(--vscode-editorHoverWidget-foreground, #d4d4d4);
      border: 1px solid var(--vscode-editorHoverWidget-border, #454a52);
      box-shadow: 0 6px 20px rgba(0, 0, 0, 0.45), 0 0 0 1px rgba(127, 127, 127, 0.08);
      backdrop-filter: blur(6px); }
    .hoverdoc .hd-title { display: inline-block; font-weight: 700; font-size: calc(var(--font-size) * 0.84); font-family: var(--code-font);
      color: var(--vscode-textLink-foreground, #57a6ff); background: rgba(86, 156, 214, 0.16); border: 1px solid rgba(86, 156, 214, 0.28);
      padding: 1px 9px; border-radius: 5px; margin-bottom: 7px; }
    .hoverdoc .hd-sig { font-family: var(--code-font); font-size: calc(var(--font-size) * 0.74); color: var(--vscode-editorHoverWidget-foreground, #dcdcaa);
      background: rgba(127, 127, 127, 0.12); border-radius: 5px; padding: 3px 8px; margin: 0 0 6px; white-space: pre-wrap; direction: ltr;
      text-align: left; unicode-bidi: plaintext; }
    .hoverdoc .hd-en { color: var(--vscode-descriptionForeground, #9da5b4); }
    .hoverdoc .hd-params { margin-top: 7px; padding-top: 6px; border-top: 1px solid rgba(127, 127, 127, 0.2); }
    .hoverdoc .hd-params b { color: var(--vscode-editorHoverWidget-foreground, #dcdcdc); }
    .hoverdoc .hd-resize { position: absolute; top: 0; bottom: 0; left: 0; width: 8px; cursor: ew-resize; touch-action: none;
      background: linear-gradient(90deg, rgba(127, 127, 127, 0.35), rgba(127, 127, 127, 0.15) 55%, transparent); border-radius: 8px 0 0 8px; }
    .hoverdoc .hd-resize:hover { background: linear-gradient(90deg, rgba(127, 127, 127, 0.55), rgba(127, 127, 127, 0.2) 55%, transparent); }
    .gutter .fold-btn { display: inline-block; width: 1.5ch; margin-right: 5px; cursor: pointer; user-select: none; text-align: center;
      font-size: calc(var(--code-size) * 0.72); color: var(--vscode-editorLineNumber-foreground, #858585); opacity: 0.72; }
    .gutter .fold-btn:hover { color: var(--vscode-editorLineNumber-activeForeground, #e6e6e6); opacity: 1; }
    .gutter span.folded { opacity: 0.35; }
    .fold-bar { position: absolute; right: 0; left: 0; box-sizing: border-box; cursor: pointer; z-index: 12;
      background: var(--fold-bg, var(--vscode-editor-background, #1e1e1e));
      box-shadow: inset 0 2px 0 rgba(127, 127, 127, 0.22), inset 0 -2px 0 rgba(127, 127, 127, 0.22);
      border-radius: 6px; display: flex; align-items: center; justify-content: center; pointer-events: auto; }
    .fold-bar:hover { background: rgba(56, 96, 148, 0.28); box-shadow: inset 0 2px 0 rgba(87, 166, 255, 0.55), inset 0 -2px 0 rgba(87, 166, 255, 0.55); }
    .fold-bar::after { content: "\\22ef"; color: var(--vscode-descriptionForeground, #9da5b4); font-size: 13px; letter-spacing: 4px; }
    .gutter span.err { color: #f14c4c; text-shadow: 0 0 6px rgba(244, 63, 94, 0.7); }
    .tok-m { background: rgba(255, 195, 0, 0.22); border-radius: 2px; }
    .tok-mu { background: rgba(255, 195, 0, 0.45); box-shadow: 0 0 0 1px rgba(255, 195, 0, 0.55); }
    .highlight { pointer-events: none; color: #d8d4c4; }
    .caret { position: absolute; width: 2px; height: calc(var(--code-line) * 0.7); border-radius: 1px; background: var(--vscode-editor-foreground, #d8d4c4); animation: caret-blink 1.1s step-end infinite; z-index: 2; display: none; }
    @keyframes caret-blink { 50% { opacity: 0; } }
    .measure { position: absolute; top: 0; left: 0; right: 0; visibility: hidden; pointer-events: none; box-sizing: border-box;
      padding: 0 20px; direction: rtl; unicode-bidi: plaintext; text-align: right; white-space: pre;
      font-family: var(--code-font); font-size: var(--code-size); line-height: var(--code-line); tab-size: var(--code-tab); }
    #editor { background: transparent; color: transparent; -webkit-text-fill-color: transparent; text-shadow: none; caret-color: transparent; resize: none; }
    #editor::selection { background: rgba(38, 79, 120, 0.45); }
    .tok-keyword, .tok-literal { color: #569cd6; }
    .tok-builtin { color: #4ec9b0; }
    .tok-method { color: #c586c0; }
    .tok-attr { color: #9cdcfe; }
    .tok-call { color: #dcdcaa; }
    .tok-def { color: #dcdcaa; }
    .tok-id { color: #9cdcfe; }
    .tok-string { color: #ce9178; }
    .tok-number { color: #b5cea8; }
    .tok-comment { color: #6a9955; }
    .findbar { position: absolute; top: 8px; right: 12px; z-index: 20; display: none; flex-direction: row; gap: 6px; align-items: center;
      padding: 6px; border-radius: 4px; background: var(--vscode-editorWidget-background, #252526);
      border: 1px solid var(--vscode-widget-border, #444); box-shadow: 0 2px 8px rgba(0, 0, 0, 0.35); direction: ltr; }
    .findbar input { background: var(--vscode-input-background, #3c3c3c); color: var(--vscode-input-foreground, #ccc);
      border: 1px solid var(--vscode-input-border, transparent); padding: 3px 6px; border-radius: 2px; font: inherit; min-width: 150px; }
    .findbar button { background: var(--vscode-button-background, #0e639c); color: var(--vscode-button-foreground, #fff);
      border: none; padding: 3px 8px; border-radius: 2px; cursor: pointer; font: inherit; }
    .findbar button:hover { background: var(--vscode-button-hoverBackground, #1177bb); }
    .findbar .fb-count { color: var(--vscode-descriptionForeground, #a0a0a0); font-size: 12px; min-width: 46px; text-align: center; }
    @media (prefers-color-scheme: light) {
      #indent-guides .indent-guide { background: rgba(0, 0, 0, 0.22); }
      #indent-guides .indent-guide.active { background: rgba(0, 0, 0, 0.5); box-shadow: 0 0 3px rgba(0, 0, 0, 0.25); }
      .highlight { color: #202020; }
      .active-line { background: rgba(0, 0, 0, 0.07); }
      .word-hl { background: rgba(0, 0, 0, 0.07); }
      .gutter span.active { text-shadow: 0 0 5px rgba(0, 0, 255, 0.7); }
      .tok-keyword, .tok-literal { color: #0000ff; }
      .tok-builtin { color: #0070c1; }
      .tok-method { color: #af00db; }
      .tok-attr { color: #001080; }
      .tok-call { color: #795e26; }
      .tok-def { color: #795e26; }
      .tok-id { color: #001080; }
      .tok-string { color: #a31515; }
      .tok-number { color: #098658; }
      .tok-comment { color: #008000; }
      #editor::selection { background: rgba(0, 120, 215, 0.30); }
    }
  </style>
</head>
<body>
  <div class="findbar" id="findbar">
    <input id="find-input" placeholder="بحث\u2026" spellcheck="false" />
    <input id="replace-input" placeholder="استبدال\u2026" spellcheck="false" />
    <span class="fb-count" id="match-count"></span>
    <button id="fb-prev" title="السابق">\u25B2</button>
    <button id="fb-next" title="التالي">\u25BC</button>
    <button id="fb-replace" title="استبدل">استبدل</button>
    <button id="fb-all" title="استبدل الكل">استبدل الكل</button>
    <button id="fb-close" title="إغلاق">\u2715</button>
  </div>
  <div class="frame">
    <div class="code-wrap">
      <div class="highlight" id="highlight"></div>
      <div class="active-line" id="active-line"></div>
      <div class="err-line" id="err-line"></div>
      <div class="word-hl" id="word-hl"></div>
      <div class="indent-guides" id="indent-guides"></div>
      <div class="fold-layer" id="fold-layer"></div>
      <div class="caret" id="caret"></div>
      <div class="hoverdoc" id="hoverdoc"><div class="hd-resize" id="hd-resize"></div></div>
      <div class="measure" id="measure"></div>
      <textarea id="editor" spellcheck="false" aria-label="محرر العربية البرمجية"></textarea>
    </div>
    <div class="gutter" id="gutter">1</div>
  </div>
  <script nonce="${nonce}">
    const vscode = acquireVsCodeApi();
    const editor = document.getElementById('editor');
    const highlight = document.getElementById('highlight');
    const gutter = document.getElementById('gutter');
    const activeLineEl = document.getElementById('active-line');
    const hoverdoc = document.getElementById('hoverdoc');
    const TOP_PAD = 16;
    let tabSize = 4;
    let linePx = 24;
    let activeLine = 1;
    let timer;
    let hoverWidth = 380;

    const KEYWORDS = new Set(['و','باسم','تحقق','لازمني','انتظر','توقف','حالة','صنف','استمر','عرف','احذف','وإذا','وإلا','التقط','ختاما','لكل','من','عام','إذا','استورد','ضمن','هو','لامبدا','عدم','لامحلي','ليس','أو','تجاوز','ارم','أعد','صحيح','جرب','نوع','طالما','مع','انتج','طابق','معطيات_الصنوف','معطيات_الصنف','تصنيف','أي_نوع','الكل__','__الكل__','الاسم__','__الاسم__']);
    const LITERALS = new Set(['صحيح','خاطئ','عدم']);
    const BUILTINS = new Set(['اطبع','ادخل','طول','نطاق','قائمة','قاموس','مجموعة','متسلسلة','مجموع','اصغر','اكبر','عدد','كسري','مركب','منطقي','نص','بايت','بيتات','مطلق','الكل','أي','كرر','التالي','مكرر_غير_متزامن','التالي_غير_متزامن','ترقيم','رشح','طبق','دمج','كائن','خاصية','دالة_صنف','دالة_ثابتة','دور','قوة','قسمة_باقية','معكوس','مرتب','نسق','تمثيل','هوية','قيم','نفذ','هاش','مساعدة','افتح','حرف','رمز_الحرف','ثنائي','ثماني','ست_عشري','أسكي','شريحة','دليل','متغيرات','عالميات','محليات','اجمع','نقطة_توقف','عرض_الذاكرة','علوي','اجلب_سمة','عين_سمة','احذف_سمة','هل_له_سمة','مثيل','مشتق','استدعائي','غير_منفذ','ثلاث_نقاط','خطأ','خطأ_أساسي','خطأ_حسابي','خطأ_تحقق','خطأ_سمة','خطأ_نظام_التشغيل','خطأ_نهاية_الملف','خطأ_استيراد','حيد','خطأ_مفتاح','خطأ_بحث','خطأ_ذاكرة','خطأ_اسم','خطأ_غير_منفذ','خطأ_فيضان','خطأ_مرجع','خطأ_وقت_التشغيل','خطأ_إزاحة','خطأ_صياغة','خطأ_تبويب','خطأ_نوع','خطأ_قيمة','خطأ_القسمة_على_صفر','خطأ_متغير_محلي','خطأ_مخزن','خطأ_استدعاء_ذاتي','خطأ_وحدة_غير_موجودة','خطأ_ملف_غير_موجود','خطأ_ملف_موجود','خطأ_صلاحية','خطأ_اتصال','خطأ_اتصال_مرفوض','خطأ_اتصال_مقاطع','خطأ_اتصال_معاد','خطأ_أنبوب_مكسور','خطأ_إدخال_محجوب','خطأ_إدخال_إخراج','خطأ_بيئة','خطأ_نقطة_كسرية','خطأ_مقاطع','خطأ_مسار_دليل','خطأ_ليس_دليلا','خطأ_عملية_غير_موجودة','خطأ_عملية_طفل','خطأ_يونيكود','خطأ_فك_يونيكود','خطأ_ترميز_يونيكود','خطأ_ترجمة_يونيكود','خطأ_نظام','مجموعة_أخطاء','مجموعة_أخطاء_أساسية','توقف_التكرار','توقف_التكرار_غير_متزامن','خروج_النظام','خروج_المولد','انقطاع_لوحة_المفاتيح','تحذير','تحذير_تقادم','تحذير_ترميز','تحذير_مستقبلي','تحذير_استيراد','تحذير_صياغة','تحذير_وقت_التشغيل','تحذير_مستخدم','تحذير_بايت','تحذير_موارد']);
    const METHODS = new Set(['أضف','امسح','انسخ','عد','مدد','موقع','ادرج','اخرج','احذف','عكس','رتب','من_مفاتيح','اجلب','عناصر','مفاتيح','قيم','اخرج_آخر','عين_مبدئي','دمج','ضم','فرق','فرق_حدث','تجاهل','تقاطع','تقاطع_حدث','منفصل','مجموعة_فرعية','مجموعة_فوق','فرق_متماثل','فرق_متماثل_حدث','اتحاد','كبر','صغر','حرف_الأول','طوى','وسط','رمز','ينتهي_ب','يبدأ_ب','وسع_الجداول','ابحث','ابحث_من_آخر','موقع_من_آخر','نسق','نسق_من_قاموس','هل_حروف','هل_حروف_ورقام','هل_أسكي','هل_عشري','هل_رقم','هل_رقمي','هل_معرف','هل_صغار','هل_كبار','هل_قابل_للطباعة','هل_مسافة','هل_عناوين','اربط','برر_يسار','برر_يمين','جرد','جرد_يسار','جرد_يمين','جهز_ترجمة','اقتسم','اقتسم_من_آخر','احذف_بادئة','احذف_لاحقة','استبدل','قسم','قسم_من_آخر','قسم_الأسطر','بدل_الحالة','بعناوين','ترجم','املأ_اصفار','فك_رمز','من_ست_عشري','ست_عشري']);
    const AR_DIGITS = '0-9\\u0660-\\u0669\\u06F0-\\u06F9';
    const IS_DIGIT = new RegExp('[' + AR_DIGITS + ']');
    const NUMBER_RE = new RegExp('[' + AR_DIGITS + ']+(?:[.][' + AR_DIGITS + ']+)?');

    const DOCS = Object.assign(Object.create(null), {
      'و': ['عامل و (and)', 'and — دمج شرطين، يتحقق من كليهما'],
      'باسم': ['عامل باسم (as)', 'as — إعطاء اسم مستعار عند الاستيراد أو الالتقاط'],
      'تحقق': ['تحقق (assert)', 'assert شرط — يرفع خطأ_تحقق إذا كان الشرط خاطئا'],
      'لازمني': ['لازمني (async)', 'async — تعريف دالة لامتزامنة'],
      'انتظر': ['انتظر (await)', 'await — انتظار قيمة دالة لامتزامنة'],
      'توقف': ['توقف (break)', 'break — الخروج من الحلقة باكرا'],
      'حالة': ['حالة (case)', 'case — فرع داخل طابق (match)'],
      'صنف': ['صنف (class)', 'class — تعريف صنف جديد'],
      'استمر': ['استمر (continue)', 'continue — القفز إلى التكرار التالي'],
      'عرف': ['عرف (def)', 'def — تعريف دالة\\nعرف اسم(وسائط):'],
      'احذف': ['احذف (del)', 'del — حذف متغير أو عنصر'],
      'وإذا': ['وإذا (elif)', 'elif — فرع شرطي إضافي'],
      'وإلا': ['وإلا (else)', 'else — الفرع الافتراضي'],
      'التقط': ['التقط (except)', 'except — التقاط استثناء\\nالتقط خطأ_قيمة باسم خطأ:'],
      'ختاما': ['ختاما (finally)', 'finally — كتلة تنفذ دائما'],
      'لكل': ['لكل (for)', 'for — حلقة تكرار\\nلكل عنصر ضمن نطاق:'],
      'من': ['من (from)', 'from وحدة استورد اسم — استيراد محدد'],
      'عام': ['عام (global)', 'global — تعديل متغير عام داخل دالة'],
      'إذا': ['إذا (if)', 'if — شرط\\nإذا شرط:'],
      'استورد': ['استورد (import)', 'import — استيراد وحدة أو صنف'],
      'ضمن': ['ضمن (in)', 'in — اختبار عضوية أو تكرار'],
      'هو': ['هو (is)', 'is — اختبار تطابق الهوية (ليس القيمة)'],
      'لامبدا': ['لامبدا (lambda)', 'lambda — دالة مجهولة\\nq = لامبدا x: x * 2'],
      'عدم': ['عدم (None)', 'None — لا قيمة (فارغ)، عكس صحيح وخاطئ'],
      'لامحلي': ['لامحلي (nonlocal)', 'nonlocal — تعديل متغير الكتلة الخارجية'],
      'ليس': ['ليس (not)', 'not — نفي منطقي'],
      'أو': ['أو (or)', 'or — صحيح إذا تحقق أحد الشرطين'],
      'تجاوز': ['تجاوز (pass)', 'pass — كتلة فارغة بدون فعل'],
      'ارم': ['ارم (raise)', 'raise — رفع استثناء عمدا'],
      'أعد': ['أعد (return)', 'return — إرجاع قيمة من دالة'],
      'صحيح': ['صحيح (True)', 'True — القيمة المنطقية الصحيحة'],
      'خاطئ': ['خاطئ (False)', 'False — القيمة المنطقية الخاطئة'],
      'جرب': ['جرب (try)', 'try — محاولة تنفيذ قد يلقي استثناء'],
      'نوع': ['نوع (type)', 'type — إرجاع نوع القيمة أو تعريف نوع'],
      'طالما': ['طالما (while)', 'while — حلقة ما دام الشرط صحيحا'],
      'مع': ['مع (with)', 'with — إدارة الموارد تلقائيا\\nمع افتح(...) باسم ملف:'],
      'انتج': ['انتج (yield)', 'yield — توليد قيمة في دالة مولدة'],
      'طابق': ['طابق (match)', 'match — مطابقة الأنماط\\nطابق القيمة:\\n  حالة 1:'],
      'خطأ_أساسي': ['خطأ_أساسي (BaseException)', 'BaseException — جذر كل الاستثناءات'],
      'خطأ': ['خطأ (Exception)', 'Exception — الاستثناء العام، أساس كل الأخطاء العادية'],
      'توقف_التكرار': ['توقف_التكرار (StopIteration)', 'يُرفع عند نفاد المكرر؛ يوقفه التالي'],
      'خروج_النظام': ['خروج_النظام (SystemExit)', 'SystemExit — طلب إنهاء البرنامج'],
      'اطبع': ['اطبع (print)', 'print(*قيم, sep=" ", end="\\n") — الكتابة إلى المخرجات'],
      'ادخل': ['ادخل (input)', 'input() — قراءة سطر من الإدخال'],
      'طول': ['طول (len)', 'len(شيء) — عدد العناصر'],
      'نطاق': ['نطاق (range)', 'range(بداية, نهاية, خطوة) — متتالية أعداد'],
      'قائمة': ['قائمة (list)', 'list() — قائمة قابلة للتعديل'],
      'قاموس': ['قاموس (dict)', 'dict() — جدول مفتاح:قيمة'],
      'مجموعة': ['مجموعة (set)', 'set() — مجموعة عناصر فريدة غير مرتبة'],
      'متسلسلة': ['متسلسلة (tuple)', 'tuple() — متسلسلة غير قابلة للتعديل'],
      'مجموع': ['مجموع (sum)', 'sum(قيم) — جمع عناصر'],
      'اصغر': ['اصغر (min)', 'min(قيم) — أصغر عنصر'],
      'اكبر': ['اكبر (max)', 'max(قيم) — أكبر عنصر'],
      'عدد': ['عدد (int)', 'int(x) — تحويل إلى عدد صحيح'],
      'كسري': ['كسري (float)', 'float(x) — تحويل إلى عدد عائم'],
      'مركب': ['مركب (complex)', 'complex(x) — عدد مركب'],
      'منطقي': ['منطقي (bool)', 'bool(x) — تحويل إلى قيمة منطقية'],
      'نص': ['نص (str)', 'str(x) — تحويل إلى نص'],
      'بايت': ['بايت (bytes)', 'bytes(x) — تسلسل بايتات'],
      'بيتات': ['بيتات (bytearray)', 'bytearray(x) — تسلسل بايتات قابل للتعديل'],
      'مطلق': ['مطلق (abs)', 'abs(x) — القيمة المطلقة'],
      'الكل': ['الكل (all)', 'all(قيم) — صحيح إذا كانت كلها صحيحة'],
      'أي': ['أي (any)', 'any(قيم) — صحيح إذا كان أحدها صحيحا'],
      'كرر': ['كرر (iter)', 'iter(شيء) — إنشاء مكرر'],
      'التالي': ['التالي (next)', 'next(مكرر) — العنصر التالي'],
      'ترقيم': ['ترقيم (enumerate)', 'enumerate(قيم, بداية=0) — تكرار مع ترقيم (فهرس، قيمة)'],
      'رشح': ['رشح (filter)', 'filter(دالة, قيم) — تصفية حسب الشرط'],
      'طبق': ['طبق (map)', 'map(دالة, قيم) — تطبيق دالة على كل عنصر'],
      'دمج': ['دمج (zip)', 'zip(*متتاليات) — دمج عدة متتاليات معا'],
      'كائن': ['كائن (object)', 'object — أصل كل الأصناف'],
      'خاصية': ['خاصية (property)', 'property() — تعريف خاصية القراءة/الكتابة'],
      'دالة_صنف': ['دالة_صنف (classmethod)', 'classmethod — دالة تستقبل الصنف بدل النسخة'],
      'دالة_ثابتة': ['دالة_ثابتة (staticmethod)', 'staticmethod — دالة بلا نفس ولا صنف'],
      'دور': ['دور (round)', 'round(x, رقام=0) — تدوير رقم'],
      'قوة': ['قوة (pow)', 'pow(أساس, أس) — رفع لقوة'],
      'قسمة_باقية': ['قسمة_باقية (divmod)', 'divmod(a, b) — (قسمة صحيحة، باقي القسمة)'],
      'معكوس': ['معكوس (reversed)', 'reversed(قيم) — مكرر بالترتيب المعكوس'],
      'مرتب': ['مرتب (sorted)', 'sorted(قيم, key=None) — قائمة مرتبة جديدة'],
      'نسق': ['نسق (format)', 'format(x) — صياغة قيمة كنص أو نسق النص'],
      'تمثيل': ['تمثيل (repr)', 'repr(x) — تمثيل دقيق للقيمة'],
      'هوية': ['هوية (id)', 'id(x) — معرف الذاكرة للكائن'],
      'قيم': ['قيم (eval)', 'eval(نص) — تنفيذ نص تعبيري وإرجاع نتيجته'],
      'نفذ': ['نفذ (exec)', 'exec(نص) — تنفيذ نص برمجي'],
      'هاش': ['هاش (hash)', 'hash(x) — القيمة التجزئية للكائن'],
      'مساعدة': ['مساعدة (help)', 'help() — وثائق مساعدة (قد لا تتوفر عند تضمين المفسر)'],
      'افتح': ['افتح (open)', 'open(مسار, صيغة="r") — فتح ملف'],
      'حرف': ['حرف (chr)', 'chr(رقم) — الحرف الموافق للرقم'],
      'رمز_الحرف': ['رمز_الحرف (ord)', 'ord(حرف) — رقم يونيكود للحرف'],
      'ثنائي': ['ثنائي (bin)', 'bin(x) — تمثيل ثنائي'],
      'ثماني': ['ثماني (oct)', 'oct(x) — تمثيل ثماني'],
      'ست_عشري': ['ست_عشري (hex)', 'hex(x) — تمثيل الست عشري'],
      'أسكي': ['أسكي (ascii)', 'ascii(x) — تمثيل أسكي آمن'],
      'شريحة': ['شريحة (slice)', 'slice(بداية, نهاية, خطوة) — شرائح من القيم'],
      'دليل': ['دليل (dir)', 'dir() — أسماء متاحة أو سمات كائن'],
      'متغيرات': ['متغيرات (vars)', 'vars() — قاموس المتغيرات المحلية أو سمات كائن'],
      'عالميات': ['عالميات (globals)', 'globals() — قاموس المتغيرات العالمية'],
      'محليات': ['محليات (locals)', 'locals() — قاموس المتغيرات المحلية'],
      'اجمع': ['اجمع (compile)', 'compile(نص, مسار, صيغة) — تجميع نص إلى كائن برمجي'],
      'نقطة_توقف': ['نقطة_توقف (breakpoint)', 'breakpoint() — محطة توقف المصحح'],
      'عرض_الذاكرة': ['عرض_الذاكرة (memoryview)', 'memoryview(x) — عرض ذاكرة بدون نسخ'],
      'علوي': ['علوي (super)', 'super() — استدعاء دالة الصنف الأب'],
      'اجلب_سمة': ['اجلب_سمة (getattr)', 'getattr(كائن, اسم) — قراءة سمة بالاسم'],
      'عين_سمة': ['عين_سمة (setattr)', 'setattr(كائن, اسم, قيمة) — تعيين سمة بالاسم'],
      'احذف_سمة': ['احذف_سمة (delattr)', 'delattr(كائن, اسم) — حذف سمة بالاسم'],
      'هل_له_سمة': ['هل_له_سمة (hasattr)', 'hasattr(كائن, اسم) — هل توجد السمة؟'],
      'مثيل': ['مثيل (isinstance)', 'isinstance(شيء, نوع) — هل هو من النوع؟'],
      'مشتق': ['مشتق (issubclass)', 'issubclass(صنف, نوع) — هل هو صنف فرعي؟'],
      'استدعائي': ['استدعائي (callable)', 'callable(x) — هل يمكن استدعاؤه؟'],
      'أضف': ['أضف (.append)', 'list.append(عنصر) — إضافة عنصر في نهاية القائمة'],
      'امسح': ['امسح (.clear)', 'clear() — إفراغ المجموعة أو القاموس'],
      'انسخ': ['انسخ (.copy)', 'copy() — نسخة سطحية'],
      'عد': ['عد (.count)', 'count(قيمة) — عدد التكرارات'],
      'مدد': ['مدد (.extend)', 'list.extend(قيم) — دمج قيم في القائمة'],
      'موقع': ['موقع (.index)', 'index(قيمة) — أول مؤشر للقيمة'],
      'ادرج': ['ادرج (.insert)', 'list.insert(مؤشر, قيمة) — إدراج في موضع'],
      'اخرج': ['اخرج (.pop)', 'pop(مؤشر=-1) — إخراج عنصر وإرجاعه'],
      'عكس': ['عكس (.reverse)', 'list.reverse() — عكس الترتيب'],
      'رتب': ['رتب (.sort)', 'list.sort(key=None) — ترتيب في الموقع'],
      'من_مفاتيح': ['من_مفاتيح (.fromkeys)', 'dict.fromkeys(مفاتيح, قيمة=None) — قاموس من مفاتيح'],
      'اجلب': ['اجلب (.get)', 'dict.get(مفتاح, مبدئي=None) — قراءة دون خطأ إن غاب المفتاح'],
      'عناصر': ['عناصر (.items)', 'dict.items() — عرض المفاتيح والقيم'],
      'مفاتيح': ['مفاتيح (.keys)', 'dict.keys() — عرض المفاتيح'],
      'قيم': ['قيم (.values)', 'dict.values() — عرض القيم (و دالة eval لتنفيذ نص)'],
      'اخرج_آخر': ['اخرج_آخر (.popitem)', 'dict.popitem() — إخراج آخر عنصر'],
      'عين_مبدئي': ['عين_مبدئي (.setdefault)', 'dict.setdefault(مفتاح, مبدئي) — قراءة أو تعيين مبدئي'],
      'دمج': ['دمج (.update)', 'dict.update(قاموس) — دمج قاموس آخر'],
      'ضم': ['ضم (.add)', 'set.add(عنصر) — إضافة عنصر للمجموعة'],
      'فرق': ['فرق (.difference)', 'set.difference(أخرى) — عناصر ليست في الأخرى'],
      'فرق_حدث': ['فرق_حدث (.difference_update)', 'set.difference_update(أخرى) — حذف عناصر الأخرى'],
      'تجاهل': ['تجاهل (.discard)', 'set.discard(عنصر) — حذف دون خطأ إن غاب'],
      'تقاطع': ['تقاطع (.intersection)', 'set.intersection(أخرى) — العناصر المشتركة'],
      'تقاطع_حدث': ['تقاطع_حدث (.intersection_update)', 'set.intersection_update(أخرى) — إبقاء المشترك'],
      'منفصل': ['منفصل (.isdisjoint)', 'set.isdisjoint(أخرى) — هل لا توجد عناصر مشتركة؟'],
      'مجموعة_فرعية': ['مجموعة_فرعية (.issubset)', 'set.issubset(أخرى) — هل مجموعة فرعية؟'],
      'مجموعة_فوق': ['مجموعة_فوق (.issuperset)', 'set.issuperset(أخرى) — هل تحتوي الأخرى؟'],
      'فرق_متماثل': ['فرق_متماثل (.symmetric_difference)', 'set.symmetric_difference(أخرى) — عناصر في واحدة فقط'],
      'فرق_متماثل_حدث': ['فرق_متماثل_حدث (.symmetric_difference_update)', 'set.symmetric_difference_update(أخرى) — تحديث بالفرق المتماثل'],
      'اتحاد': ['اتحاد (.union)', 'set.union(أخرى) — اتحاد المجموعتين'],
      'كبر': ['كبر (.upper)', 'str.upper() — تحويل إلى حروف كبيرة'],
      'صغر': ['صغر (.lower)', 'str.lower() — تحويل إلى حروف صغيرة'],
      'حرف_الأول': ['حرف_الأول (.capitalize)', 'str.capitalize() — تكبير أول حرف'],
      'طوى': ['طوى (.casefold)', 'str.casefold() — تطبيع الحروف (أقوى من lower)'],
      'وسط': ['وسط (.center)', 'str.center(عرض, حشو=" ") — توسيط النص'],
      'رمز': ['رمز (.encode)', 'str.encode(ترميز="utf-8") — تحويل النص إلى بايت'],
      'ينتهي_ب': ['ينتهي_ب (.endswith)', 'str.endswith(لاحقة) — هل ينتهي النص بـ؟'],
      'يبدأ_ب': ['يبدأ_ب (.startswith)', 'str.startswith(بادئة) — هل يبدأ النص بـ؟'],
      'وسع_الجداول': ['وسع_الجداول (.expandtabs)', 'str.expandtabs(حجم=8) — استبدال الجداول بمسافات'],
      'ابحث': ['ابحث (.find)', 'str.find(فرعي) — أول مؤشر أو -1'],
      'ابحث_من_آخر': ['ابحث_من_آخر (.rfind)', 'str.rfind(فرعي) — آخر مؤشر أو -1'],
      'موقع_من_آخر': ['موقع_من_آخر (.rindex)', 'str.rindex(فرعي) — آخر مؤشر أو خطأ'],
      'نسق_من_قاموس': ['نسق_من_قاموس (.format_map)', 'str.format_map(قاموس) — صياغة باسماء من قاموس'],
      'هل_حروف': ['هل_حروف (.isalpha)', 'str.isalpha() — هل كل الحروف أبجدية؟'],
      'هل_حروف_ورقام': ['هل_حروف_ورقام (.isalnum)', 'str.isalnum() — هل حروف ورقام؟'],
      'هل_أسكي': ['هل_أسكي (.isascii)', 'str.isascii() — هل كل الحروف أسكي؟'],
      'هل_عشري': ['هل_عشري (.isdecimal)', 'str.isdecimal() — هل كل الحروف عشرية؟'],
      'هل_رقم': ['هل_رقم (.isdigit)', 'str.isdigit() — هل كل الحروف أرقام؟'],
      'هل_رقمي': ['هل_رقمي (.isnumeric)', 'str.isnumeric() — هل كل الحروف رقمية؟'],
      'هل_معرف': ['هل_معرف (.isidentifier)', 'str.isidentifier() — هل اسم صالح؟'],
      'هل_صغار': ['هل_صغار (.islower)', 'str.islower() — هل كل الحروف صغيرة؟'],
      'هل_كبار': ['هل_كبار (.isupper)', 'str.isupper() — هل كل الحروف كبيرة؟'],
      'هل_قابل_للطباعة': ['هل_قابل_للطباعة (.isprintable)', 'str.isprintable() — هل كلها قابلة للطباعة؟'],
      'هل_مسافة': ['هل_مسافة (.isspace)', 'str.isspace() — هل كلها مسافات؟'],
      'هل_عناوين': ['هل_عناوين (.istitle)', 'str.istitle() — هل بنمط عناوين؟'],
      'اربط': ['اربط (.join)', 'str.join(قيم) — ربط قيم النص بالنص الفاصل'],
      'برر_يسار': ['برر_يسار (.ljust)', 'str.ljust(عرض, حشو=" ") — محاذاة لليمين'],
      'برر_يمين': ['برر_يمين (.rjust)', 'str.rjust(عرض, حشو=" ") — محاذاة لليسار'],
      'جرد': ['جرد (.strip)', 'str.strip(حروف=None) — إزالة المسافات من الطرفين'],
      'جرد_يسار': ['جرد_يسار (.lstrip)', 'str.lstrip(حروف=None) — إزالة من البداية'],
      'جرد_يمين': ['جرد_يمين (.rstrip)', 'str.rstrip(حروف=None) — إزالة من النهاية'],
      'جهز_ترجمة': ['جهز_ترجمة (.maketrans)', 'str.maketrans(من, إلى) — جدول ترجمة'],
      'اقتسم': ['اقتسم (.partition)', 'str.partition(فاصل) — (قبل, فاصل, بعد)'],
      'اقتسم_من_آخر': ['اقتسم_من_آخر (.rpartition)', 'str.rpartition(فاصل) — القسمة من النهاية'],
      'احذف_بادئة': ['احذف_بادئة (.removeprefix)', 'str.removeprefix(بادئة) — حذف بادئة إن وجدت'],
      'احذف_لاحقة': ['احذف_لاحقة (.removesuffix)', 'str.removesuffix(لاحقة) — حذف لاحقة إن وجدت'],
      'استبدل': ['استبدل (.replace)', 'str.replace(قديم, جديد, عدد=-1) — استبدال نص'],
      'قسم': ['قسم (.split)', 'str.split(فاصل=None) — تقطيع نص إلى قائمة'],
      'قسم_من_آخر': ['قسم_من_آخر (.rsplit)', 'str.rsplit(فاصل=None) — تقطيع من النهاية'],
      'قسم_الأسطر': ['قسم_الأسطر (.splitlines)', 'str.splitlines() — تقطيع على الأسطر'],
      'بدل_الحالة': ['بدل_الحالة (.swapcase)', 'str.swapcase() — عكس الأحجام'],
      'بعناوين': ['بعناوين (.title)', 'str.title() — تكبير أول كل كلمة'],
      'ترجم': ['ترجم (.translate)', 'str.translate(جدول) — استبدال حسب جدول'],
      'املأ_اصفار': ['املأ_اصفار (.zfill)', 'str.zfill(عرض) — تعبئة أصفار من البداية'],
      'فك_رمز': ['فك_رمز (.decode)', 'bytes.decode(ترميز="utf-8") — تحويل البايت إلى نص'],
      'من_ست_عشري': ['من_ست_عشري (.fromhex)', 'bytes.fromhex(نص) — بايت من تمثيل سداسي'],
      'معطيات_الصنوف': ['معطيات_الصنوف (dataclasses)', 'وحدة دوال المعطيات — من معطيات_الصنوف استورد معطيات_الصنف'],
      'تصنيف': ['تصنيف (typing)', 'وحدة التصنيف — من تصنيف استورد أي_نوع'],
      'معطيات_الصنف': ['معطيات_الصنف (dataclass)', '@معطيات_الصنف — مولّد أسلوب __init__ تلقائيا للصنف'],
      'أي_نوع': ['أي_نوع (Any)', 'Any — نوع يقبل أي قيمة في تصنيف الدوال'],
      'الكل__': ['الكل__ (__all__)', '__all__ — قائمة الأسماء المُصدَّرة من الوحدة\\nالكل__ = ["اسم1", "اسم2"]'],
      'الاسم__': ['الاسم__ (__name__)', '__name__ — اسم الوحدة الحالية؛ "__main__" للبرنامج الرئيسي\\nإن الاسم__ == "__main__":']});

    function positionHoverDoc() {
      const hoverdoc = document.getElementById('hoverdoc');
      if (!hoverdoc) return;
      if (editor.selectionStart !== editor.selectionEnd || editor.value.length === 0) {
        hoverdoc.style.display = 'none';
        return;
      }
      const i = editor.selectionStart || 0;
      const word = getWordAt(i);
      const doc = word && DOCS[editor.value.slice(word[0], word[1])];
      if (!doc) {
        hoverdoc.innerHTML = '';
        hoverdoc.style.display = 'none';
        return;
      }
      const body = String(doc[1] || '');
      const parts = body.split('\\n');
      const sig = parts.length > 1 ? parts[0] : '';
      const desc = parts.length > 1 ? parts.slice(1).join('\\n') : body;
      hoverdoc.innerHTML =
        '<div class="hd-title">' + esc(doc[0]) + '</div>' +
        (sig ? '<div class="hd-sig">' + esc(sig) + '</div>' : '') +
        (desc ? '<div class="hd-en">' + esc(desc).replace(/\\n/g, '<br/>') + '</div>' : '');
      const wh = document.getElementById('word-hl');
      const wordLeft =
        wh && wh.style.display === 'block'
          ? Math.max(0, parseFloat(wh.style.left) || 0)
          : 0;
      const wrap = document.querySelector('.code-wrap').getBoundingClientRect();
      const wrapW = wrap.width;
      const wantW = Math.round(hoverWidth);
      let effW = Math.max(64, Math.min(wantW, wrapW - 16));
      let boxRight;
      if (wordLeft >= effW + 8) {
        boxRight = wordLeft - 8;
      } else {
        boxRight = Math.min(wordLeft + 8 + effW, Math.max(8, wrapW - 8));
        effW = Math.max(64, Math.min(effW, boxRight - 8, wrapW - 16));
      }
      hoverdoc.style.width = effW + 'px';
      hoverdoc.style.right = Math.max(4, Math.round(wrapW - boxRight)) + 'px';
      hoverdoc.style.display = 'block';
      const base = wordTopBase !== null ? wordTopBase : TOP_PAD;
      const height = hoverdoc.offsetHeight || 0;
      hoverdoc.style.top = Math.max(0, Math.round(base - height - 6 - (editor.scrollTop || 0))) + 'px';
    }

    function esc(s) { return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;'); }

    function ltrEmbedsLine(line) {
      let out = '';
      let i = 0;
      const n = line.length;
      while (i < n) {
        const c = line[i];
        if (c === '(' || c === '[' || c === '{') {
          const close = c === '(' ? ')' : c === '[' ? ']' : '}';
          let depth = 0;
          let matched = -1;
          for (let j = i; j < n; j++) {
            if (line[j] === c) depth++;
            else if (line[j] === close) { depth--; if (depth === 0) { matched = j; break; } }
          }
          if (matched >= 0) {
            out += '\u202a' + line.slice(i, matched + 1) + '\u202c';
            i = matched + 1;
            continue;
          }
        }
        out += c;
        i++;
      }
      return out;
    }
    function ltrEmbeds(text) {
      return stripEmbeds(text).split(/\\r?\\n/).map(ltrEmbedsLine).join('\\n');
    }
    function stripEmbeds(text) {
      return text.replace(/[\u202a\u202c]/g, '');
    }

    function highlightSource(source, marks, curMark) {
      let html = '';
      let i = 0;
      let pendingDef = false;
      const n = source.length;
      const push = (cls, str, pos) => {
        if (!str) return;
        let clsOut = cls ? 'tok-' + cls : '';
        if (marks) {
          for (let k = 0; k < marks.length; k++) {
            const m = marks[k];
            if (pos + str.length > m[0] && pos < m[1]) { clsOut += ' m'; break; }
          }
          if (curMark && pos + str.length > curMark[0] && pos < curMark[1]) clsOut += ' mu';
        }
        html += clsOut ? '<span class="' + clsOut + '">' + esc(str) + '</span>' : esc(str);
      };
      while (i < n) {
        const ch = source[i];
        if (ch === '#') {
          let j = i + 1;
          while (j < n && source[j] !== '\\n') j++;
          push('comment', source.slice(i, j), i);
          i = j;
          continue;
        }
        if (ch === '"' || ch === "'") {
          const triple = source.slice(i, i + 3) === ch.repeat(3);
          const closer = triple ? ch.repeat(3) : ch;
          const start = i;
          i += triple ? 3 : 1;
          while (i < n && !source.startsWith(closer, i)) {
            if (source[i] === '\\\\') i += 2; else i++;
          }
          if (i <= n) i = Math.min(i + (triple ? 3 : 1), n);
          push('string', source.slice(start, i), start);
          continue;
        }
        if (/[\\p{L}_]/u.test(ch)) {
          const start = i;
          while (i < n && /[\\p{L}\\p{M}\\p{N}_]/u.test(source[i])) i++;
          const word = source.slice(start, i);
          const prevDot = start > 0 && source[start - 1] === '.';
          let cls;
          if (pendingDef) { cls = 'def'; pendingDef = false; }
          else if (LITERALS.has(word)) cls = 'literal';
          else if (KEYWORDS.has(word)) {
            cls = 'keyword';
            if (word === 'عرف' || word === 'صنف') pendingDef = true;
          }
          else if (BUILTINS.has(word)) cls = 'builtin';
          else if (prevDot && METHODS.has(word)) cls = 'method';
          else if (prevDot) cls = 'attr';
          else if (i < n && source[i] === '(') cls = 'call';
          else cls = 'id';
          push(cls, word, start);
          continue;
        }
        if (IS_DIGIT.test(ch)) {
          const m = NUMBER_RE.exec(source.slice(i));
          push('number', m[0], i);
          i += m[0].length;
          continue;
        }
        push(null, ch, i);
        i++;
      }
      return html;
    }

    let findQuery = '';
    let replaceQuery = '';
    let matches = [];
    let matchIdx = -1;
    let marks = [];
    let curMark = null;
    let findOpen = false;
    let errLine = null;
    let caretTopBase = null;
    let wordTopBase = null;
    let indentGuideData = null;
    let foldBarData = null;
    const foldOpeners = new Set();

    function computeFoldRanges() {
      const { spans } = computeIndentGuides();
      const ranges = new Map();
      for (const s of spans) {
        const opener = s.start - 1;
        if (opener < 0) continue;
        const ex = ranges.get(opener);
        if (ex) {
          ex.begin = Math.min(ex.begin, s.start);
          ex.end = Math.max(ex.end, s.end);
        } else {
          ranges.set(opener, { begin: s.start, end: s.end });
        }
      }
      return ranges;
    }
    function drawFolds() {
      const layer = document.getElementById('fold-layer');
      const ranges = computeFoldRanges();
      const frag = [];
      const bars = [];
      for (const [opener, r] of ranges) {
        if (!foldOpeners.has(opener + 1)) continue;
        const span = r.end - r.begin + 1;
        const topBase = TOP_PAD + r.begin * linePx;
        bars.push({ topBase, height: span * linePx });
        frag.push(
          '<div class="fold-bar" data-line="' + (opener + 1) + '" style="top:' +
          Math.round(topBase - (editor.scrollTop || 0)) + 'px;height:' + (span * linePx) + 'px"></div>',
        );
      }
      layer.innerHTML = frag.join('');
      foldBarData = bars;
    }
    function positionFoldBars() {
      const layer = document.getElementById('fold-layer');
      if (!layer.children || !layer.children.length || !foldBarData || !foldBarData.length) return;
      const scrollTop = editor.scrollTop || 0;
      for (let i = 0; i < layer.children.length; i++) {
        const d = foldBarData[i];
        if (!d) continue;
        layer.children[i].style.top = Math.round(d.topBase - scrollTop) + 'px';
      }
    }
    function toggleFold(line) {
      if (foldOpeners.has(line)) foldOpeners.delete(line);
      else foldOpeners.add(line);
      render();
      updateUi();
    }
    function renderGutter() {
      const lines = editor.value.split('\\n').length;
      const ranges = computeFoldRanges();
      const hidden = new Set();
      for (const [opener, r] of ranges) {
        if (!foldOpeners.has(opener + 1)) continue;
        for (let l = r.begin + 1; l <= r.end + 1; l++) hidden.add(l);
      }
      let nums = '';
      for (let i = 1; i <= lines; i++) {
        let cls = '';
        if (i === errLine) cls += ' err';
        if (i === activeLine) cls += ' active';
        if (hidden.has(i)) cls += ' folded';
        const arrow = ranges.has(i - 1)
          ? '<span class="fold-btn" data-line="' + i + '">' + (foldOpeners.has(i) ? '▸' : '▾') + '</span>'
          : '';
        nums += '<span class="' + (cls || '').trim() + '">' + arrow + i + '</span>\\n';
      }
      gutter.innerHTML = nums;
    }
    function computeIndentGuides() {
      const lines = editor.value.split('\\n');
      const n = lines.length;
      const lvl = new Array(n).fill(0);
      let last = 0;
      for (let i = 0; i < n; i++) {
        const m = lines[i].match(/^[ \t]*/)[0];
        let c = 0;
        for (let k = 0; k < m.length; k++) c += m[k] === '\\t' ? tabSize : 1;
        lvl[i] = m.length === lines[i].length ? last : c;
        last = lvl[i];
      }
      const levels = [];
      for (let i = 0; i < n; i++) {
        if (lvl[i] > 0 && levels.indexOf(lvl[i]) === -1) levels.push(lvl[i]);
      }
      levels.sort(function (a, b) { return a - b; });
      const spans = [];
      for (let li = 0; li < levels.length; li++) {
        const L = levels[li];
        for (let i = 0; i < n; i++) {
          if (lvl[i] < L) continue;
          let j = i;
          while (j + 1 < n && lvl[j + 1] >= L) j++;
          spans.push({ k: L, start: i, end: j });
          i = j;
        }
      }
      return { lvl, levels, spans };
    }
    function measureSpaceWidth() {
      try {
        const measureEl = document.getElementById('measure');
        measureEl.textContent = '          ';
        const node = measureEl.firstChild;
        const range = document.createRange();
        range.setStart(node, 0);
        range.setEnd(node, 10);
        const box = range.getBoundingClientRect();
        return box && box.width ? box.width / 10 : 0;
      } catch (e) {
        return 0;
      }
    }
    function drawIndentGuides() {
      const container = document.getElementById('indent-guides');
      const { lvl, spans } = computeIndentGuides();
      const wrap = document.querySelector('.code-wrap').getBoundingClientRect();
      const spaceW = measureSpaceWidth();
      if (spaceW <= 0) {
        container.innerHTML = '';
        indentGuideData = [];
        return;
      }
      const caretLine = activeLine ? activeLine - 1 : -1;
      const caretLvl = caretLine >= 0 && caretLine < lvl.length ? lvl[caretLine] : 0;
      const frag = [];
      indentGuideData = [];
      for (const s of spans) {
        const x = wrap.width - 20 - s.k * spaceW;
        const span = s.end - s.start + 1;
        const active = s.k === caretLvl && caretLine >= s.start && caretLine <= s.end;
        indentGuideData.push({ xBase: x, start: s.start, span });
        frag.push(
          '<div class="indent-guide' + (active ? ' active' : '') + '" style="left:' +
          (Math.round(x) - 0.5) + 'px;top:' +
          (TOP_PAD + s.start * linePx - (editor.scrollTop || 0)) + 'px;height:' +
          (span * linePx) + 'px"></div>',
        );
      }
      container.innerHTML = frag.join('');
    }
    function positionIndentGuides() {
      const container = document.getElementById('indent-guides');
      const kids = container.children;
      if (!kids || !kids.length) return;
      const scrollTop = editor.scrollTop || 0;
      for (let i = 0; i < kids.length; i++) {
        const d = indentGuideData && indentGuideData[i];
        if (!d) continue;
        kids[i].style.left = (Math.round(d.xBase) - 0.5) + 'px';
        kids[i].style.top = (TOP_PAD + d.start * linePx - scrollTop) + 'px';
      }
    }
    function render() {
      const text = editor.value;
      highlight.innerHTML = highlightSource(text, marks, curMark);
      renderGutter();
      drawIndentGuides();
      drawFolds();
      syncScroll();
    }
    function positionActiveLine() {
      activeLineEl.style.top = (TOP_PAD + (activeLine - 1) * linePx - editor.scrollTop) + 'px';
    }
    function setActiveLine() {
      const line = editor.value.slice(0, editor.selectionStart || 0).split('\\n').length;
      if (line !== activeLine) {
        activeLine = line;
        renderGutter();
        drawIndentGuides();
      }
      positionActiveLine();
    }
    function positionErrLine() {
      const el = document.getElementById('err-line');
      if (!errLine) { el.style.display = 'none'; return; }
      el.style.display = 'block';
      el.style.top = (TOP_PAD + (errLine - 1) * linePx - editor.scrollTop) + 'px';
    }
    function getWordAt(i) {
      const v = editor.value;
      if (i > v.length) return null;
      const isW = (ch) => /[\\p{L}\\p{M}\\p{N}_]/u.test(ch);
      let a = i;
      while (a > 0 && isW(v[a - 1])) a--;
      let b = i;
      while (b < v.length && isW(v[b])) b++;
      return a < b ? [a, b] : null;
    }
    function positionCaret() {
      const caretEl = document.getElementById('caret');
      if (editor.selectionStart !== editor.selectionEnd || editor.value.length === 0) {
        caretEl.style.display = 'none';
        caretTopBase = null;
        return;
      }
      const v = editor.value;
      const caretIndex = editor.selectionStart || 0;
      if (caretIndex > v.length) {
        caretEl.style.display = 'none';
        caretTopBase = null;
        return;
      }
      const lineStart = v.lastIndexOf('\\n', caretIndex - 1) + 1;
      let lineEnd = v.indexOf('\\n', caretIndex);
      if (lineEnd < 0) lineEnd = v.length;
      const lineText = v.slice(lineStart, lineEnd);
      const col = caretIndex - lineStart;
      const lineNum = v.slice(0, caretIndex).split('\\n').length;
      const measure = document.getElementById('measure');
      try {
        measure.textContent = lineText.slice(0, col) + '\\u200b' + lineText.slice(col);
        const node = measure.firstChild;
        const range = document.createRange();
        range.setStart(node, col);
        range.setEnd(node, col + 1);
        const box = range.getBoundingClientRect();
        const wrap = document.querySelector('.code-wrap').getBoundingClientRect();
        caretTopBase = TOP_PAD + (lineNum - 1) * linePx + (linePx - Math.round(linePx * 0.7)) / 2;
        caretEl.style.display = 'block';
        caretEl.style.left = Math.round(box.left - wrap.left) + 'px';
        caretEl.style.top = Math.round(caretTopBase - editor.scrollTop) + 'px';
      } catch (e) {
        caretEl.style.display = 'none';
        caretTopBase = null;
      }
    }
    function positionWordHl() {
      const el = document.getElementById('word-hl');
      if (editor.selectionStart !== editor.selectionEnd || editor.value.length === 0) {
        el.style.display = 'none';
        wordTopBase = null;
        return;
      }
      const v = editor.value;
      const i = editor.selectionStart || 0;
      const word = getWordAt(i);
      if (!word) { el.style.display = 'none'; wordTopBase = null; return; }
      const lineStart = v.lastIndexOf('\\n', i - 1) + 1;
      let lineEnd = v.indexOf('\\n', i);
      if (lineEnd < 0) lineEnd = v.length;
      const lineText = v.slice(lineStart, lineEnd);
      const lineNum = v.slice(0, i).split('\\n').length;
      const measure = document.getElementById('measure');
      try {
        const colA = word[0] - lineStart;
        const colB = word[1] - lineStart;
        measure.textContent = lineText.slice(0, colA) + '\\u200b' +
          lineText.slice(colA, colB) + '\\u200b' +
          lineText.slice(colB);
        const node = measure.firstChild;
        const range = document.createRange();
        range.setStart(node, colA);
        range.setEnd(node, colB + 1);
        const box = range.getBoundingClientRect();
        const wrap = document.querySelector('.code-wrap').getBoundingClientRect();
        wordTopBase = TOP_PAD + (lineNum - 1) * linePx;
        el.style.display = 'block';
        el.style.left = Math.round(box.left - wrap.left) + 'px';
        el.style.top = Math.round(wordTopBase - editor.scrollTop) + 'px';
        el.style.width = Math.max(2, Math.round(box.width)) + 'px';
      } catch (e) {
        el.style.display = 'none';
        wordTopBase = null;
      }
    }
    function updateUi() {
      setActiveLine();
      positionCaret();
      positionWordHl();
      positionHoverDoc();
    }
    function escRegExp(s) {
      let out = '';
      for (const ch of s) {
        out += '.*+?^$(){}[]|\\\\'.includes(ch) ? String.fromCharCode(92) + ch : ch;
      }
      return out;
    }
    function computeMatches() {
      matches = [];
      matchIdx = -1;
      if (!findQuery) { marks = []; curMark = null; return; }
      const re = new RegExp(escRegExp(findQuery), 'gi');
      let m;
      const v = editor.value;
      while ((m = re.exec(v))) {
        matches.push({ start: m.index, end: m.index + m[0].length });
        if (m.index === re.lastIndex) re.lastIndex++;
      }
      marks = matches.map((x) => [x.start, x.end]);
      curMark = null;
    }
    function updateMatchCount() {
      document.getElementById('match-count').textContent = matches.length ? (matchIdx + 1) + ' / ' + matches.length : '0';
    }
    function gotoMatch(delta) {
      if (!matches.length) return;
      matchIdx = (matchIdx + delta + matches.length) % matches.length;
      const m = matches[matchIdx];
      editor.focus();
      editor.setSelectionRange(m.start, m.end);
      curMark = [m.start, m.end];
      render();
      updateUi();
      updateMatchCount();
    }
    function onFindInput() {
      findQuery = document.getElementById('find-input').value;
      computeMatches();
      matchIdx = matches.length ? 0 : -1;
      curMark = matches.length ? marks[0] : null;
      render();
      updateUi();
      updateMatchCount();
    }
    function replaceCurrent() {
      if (!matches.length || matchIdx < 0) return;
      const m = matches[matchIdx];
      editor.focus();
      editor.setRangeText(replaceQuery, m.start, m.end, 'end');
      editor.dispatchEvent(new Event('input'));
    }
    function replaceAll() {
      if (!matches.length) return;
      const v = editor.value;
      let out = '';
      let last = 0;
      for (const m of matches) {
        out += v.slice(last, m.start) + replaceQuery;
        last = m.end;
      }
      out += v.slice(last);
      editor.value = out;
      editor.dispatchEvent(new Event('input'));
    }
    function showFind(replaceMode) {
      findOpen = true;
      document.getElementById('findbar').style.display = 'flex';
      const rm = document.getElementById('replace-input');
      const rb = document.getElementById('fb-replace');
      const ra = document.getElementById('fb-all');
      rm.style.display = replaceMode ? 'inline-block' : 'none';
      rb.style.display = replaceMode ? 'inline-block' : 'none';
      ra.style.display = replaceMode ? 'inline-block' : 'none';
      const fi = document.getElementById('find-input');
      if (!fi.value && editor.selectionStart !== editor.selectionEnd) {
        fi.value = editor.value.slice(editor.selectionStart, editor.selectionEnd);
      }
      fi.focus();
      fi.select();
      onFindInput();
    }
    function hideFind() {
      findOpen = false;
      document.getElementById('findbar').style.display = 'none';
      matches = [];
      matchIdx = -1;
      marks = [];
      curMark = null;
      editor.focus();
      render();
      updateUi();
    }
    function selectNextOccurrence() {
      const v = editor.value;
      if (editor.selectionStart === editor.selectionEnd) {
        const word = getWordAt(editor.selectionStart || 0);
        if (word) editor.setSelectionRange(word[0], word[1]);
        return;
      }
      const sel = v.slice(editor.selectionStart, editor.selectionEnd);
      if (!sel) return;
      let idx = v.indexOf(sel, editor.selectionEnd);
      if (idx < 0) idx = v.indexOf(sel);
      if (idx >= 0) editor.setSelectionRange(idx, idx + sel.length);
    }
    function selectLine() {
      const v = editor.value;
      const i = editor.selectionStart || 0;
      const s = v.lastIndexOf('\\n', i - 1) + 1;
      let e = v.indexOf('\\n', i);
      if (e < 0) e = v.length;
      editor.setSelectionRange(s, e);
    }
    function deleteLine() {
      const v = editor.value;
      const i = editor.selectionStart || 0;
      const s = v.lastIndexOf('\\n', i - 1) + 1;
      let e = v.indexOf('\\n', i);
      if (e < 0) e = v.length;
      else e += 1;
      editor.setRangeText('', s, e, 'end');
      editor.dispatchEvent(new Event('input'));
    }
function dupLine() {
      const v = editor.value;
      const i = editor.selectionStart || 0;
      const s = v.lastIndexOf('\\n', i - 1) + 1;
      let e = v.indexOf('\\n', i);
      if (e < 0) e = v.length;
      const line = v.slice(s, e);
      if (e < v.length) {
        editor.setRangeText(line + '\\n', e + 1, e + 1, 'end');
      } else {
        editor.setRangeText('\\n' + line, e, e, 'end');
      }
      editor.dispatchEvent(new Event('input'));
    }
    function toggleComment() {
      const v = editor.value;
      const i = editor.selectionStart || 0;
      const s = v.lastIndexOf('\\n', i - 1) + 1;
      let e = v.indexOf('\\n', i);
      if (e < 0) e = v.length;
      const line = v.slice(s, e);
      let out;
      if (/^# ?/.test(line)) out = line.replace(/^# ?/, '');
      else out = '# ' + line;
      editor.setRangeText(out, s, e, 'end');
      editor.dispatchEvent(new Event('input'));
    }
    function run() {
      clearTimeout(timer);
      render();
      vscode.postMessage({ type: 'run' });
    }
    function syncScroll() {
      highlight.scrollTop = editor.scrollTop;
      highlight.scrollLeft = editor.scrollLeft;
      gutter.scrollTop = editor.scrollTop;
      positionActiveLine();
      positionErrLine();
      if (caretTopBase !== null) {
        document.getElementById('caret').style.top = Math.round(caretTopBase - editor.scrollTop) + 'px';
      } else {
        positionCaret();
      }
      if (wordTopBase !== null) {
        document.getElementById('word-hl').style.top = Math.round(wordTopBase - editor.scrollTop) + 'px';
      } else {
        positionWordHl();
      }
      positionHoverDoc();
      positionIndentGuides();
      positionFoldBars();
    }

    function sendEdit() {
      clearTimeout(timer);
      const text = stripEmbeds(editor.value);
      render();
      vscode.postMessage({ type: 'edit', text });
    }
    editor.addEventListener('input', () => {
      clearTimeout(timer);
      timer = setTimeout(sendEdit, 120);
      if (findOpen) { computeMatches(); updateMatchCount(); }
      render();
      updateUi();
    });
    editor.addEventListener('scroll', syncScroll);
    window.addEventListener('resize', () => { render(); positionCaret(); positionWordHl(); positionHoverDoc(); positionIndentGuides(); positionFoldBars(); });
    for (const ev of ['keyup', 'mouseup', 'click', 'focus']) editor.addEventListener(ev, updateUi);

    document.getElementById('hd-resize').addEventListener('pointerdown', (downEvent) => {
      downEvent.preventDefault();
      downEvent.stopPropagation();
      const handle = document.getElementById('hd-resize');
      handle.setPointerCapture(downEvent.pointerId);
      const startX = downEvent.clientX;
      const startW = hoverdoc.offsetWidth || hoverWidth;
      const wrapW = document.querySelector('.code-wrap').getBoundingClientRect().width;
      const onMove = (moveEvent) => {
        const nextW = Math.max(140, Math.min(startW + (startX - moveEvent.clientX), wrapW - 16));
        hoverdoc.style.width = nextW + 'px';
      };
      const onUp = () => {
        handle.removeEventListener('pointermove', onMove);
        handle.removeEventListener('pointerup', onUp);
        const finalW = Math.max(140, Math.min(hoverdoc.offsetWidth || hoverWidth, wrapW - 16));
        hoverWidth = Math.round(finalW);
        document.documentElement.style.setProperty('--hover-w', hoverWidth + 'px');
        vscode.postMessage({ type: 'hoverWidth', width: hoverWidth });
      };
      handle.addEventListener('pointermove', onMove);
      handle.addEventListener('pointerup', onUp);
    });

    gutter.addEventListener('click', (e) => {
      const btn = e.target.closest('.fold-btn');
      if (btn) toggleFold(parseInt(btn.getAttribute('data-line'), 10) || 1);
    });
    document.getElementById('fold-layer').addEventListener('click', (e) => {
      const bar = e.target.closest('.fold-bar');
      if (bar) toggleFold(parseInt(bar.getAttribute('data-line'), 10) || 1);
    });

    document.getElementById('fb-close').addEventListener('click', hideFind);
    document.getElementById('fb-prev').addEventListener('click', () => gotoMatch(-1));
    document.getElementById('fb-next').addEventListener('click', () => gotoMatch(1));
    document.getElementById('fb-replace').addEventListener('click', replaceCurrent);
    document.getElementById('fb-all').addEventListener('click', replaceAll);
    const findInput = document.getElementById('find-input');
    const replaceInput = document.getElementById('replace-input');
    findInput.addEventListener('input', onFindInput);
    replaceInput.addEventListener('input', () => { replaceQuery = replaceInput.value; });
    findInput.addEventListener('keydown', (event) => {
      if (event.key === 'Enter') { event.preventDefault(); gotoMatch(event.shiftKey ? -1 : 1); }
      if (event.key === 'Escape') { event.preventDefault(); hideFind(); }
    });
    replaceInput.addEventListener('keydown', (event) => {
      if (event.key === 'Enter') { event.preventDefault(); replaceCurrent(); }
      if (event.key === 'Escape') { event.preventDefault(); hideFind(); }
    });

    editor.addEventListener('keydown', (event) => {
      const ctrl = event.ctrlKey || event.metaKey;
      const key = event.key.toLowerCase();
      if (event.key === 'F5') { event.preventDefault(); run(); }
      if (ctrl && event.key === 'Enter') { event.preventDefault(); run(); }
      if (event.key === 'Tab') {
        event.preventDefault();
        const start = editor.selectionStart;
        const end = editor.selectionEnd;
        editor.setRangeText(' '.repeat(tabSize), start, end, 'end');
        editor.dispatchEvent(new Event('input'));
      }
      if (event.altKey && event.shiftKey && (event.key === 'ArrowDown' || event.key === 'ArrowUp')) {
        event.preventDefault();
        dupLine();
        return;
      }
      if (ctrl && event.shiftKey && key === 'k') { event.preventDefault(); deleteLine(); }
      if (ctrl && key === 'd') { event.preventDefault(); selectNextOccurrence(); }
      if (ctrl && key === 'f') { event.preventDefault(); showFind(false); }
      if (ctrl && key === 'h') { event.preventDefault(); showFind(true); }
      if (ctrl && key === 'l') { event.preventDefault(); selectLine(); }
      if (ctrl && key === '/') { event.preventDefault(); toggleComment(); }
      if (ctrl && event.shiftKey && key === 'i') { event.preventDefault(); vscode.postMessage({ type: 'format' }); }
      if (ctrl && key === 's') {
        event.preventDefault();
        clearTimeout(timer);
        const text = stripEmbeds(editor.value);
        render();
        vscode.postMessage({ type: 'save', text });
      }
    });

    window.addEventListener('message', (event) => {
      const message = event.data;
      if (message.type === 'document' && ltrEmbeds(message.text) !== editor.value) {
        const start = editor.selectionStart;
        const end = editor.selectionEnd;
        editor.value = ltrEmbeds(message.text);
        editor.setSelectionRange(Math.min(start, editor.value.length), Math.min(end, editor.value.length));
        computeMatches();
        updateMatchCount();
        render();
        updateUi();
      }
      if (message.type === 'options') {
        if (Number.isFinite(message.tabSize)) {
          tabSize = Math.max(1, Math.min(16, message.tabSize));
        }
        linePx = Math.max(10, Math.round((message.fontSize || 16) * (message.lineSpacing || 1.5)));
        hoverWidth = message.hoverWidth || hoverWidth;
        document.documentElement.style.setProperty('--font-family', message.fontFamily);
        document.documentElement.style.setProperty('--font-size', message.fontSize + 'px');
        document.documentElement.style.setProperty('--code-font', message.codeFont || 'inherit');
        document.documentElement.style.setProperty('--code-size', message.fontSize + 'px');
        document.documentElement.style.setProperty('--code-tab', String(tabSize));
        document.documentElement.style.setProperty('--code-line', linePx + 'px');
        document.documentElement.style.setProperty('--hover-w', hoverWidth + 'px');
        render();
        updateUi();
      }
      if (message.type === 'hoverWidth' && typeof message.width === 'number') {
        hoverWidth = message.width;
        document.documentElement.style.setProperty('--hover-w', hoverWidth + 'px');
      }
      if (message.type === 'error') {
        errLine = message.line || null;
        render();
        updateUi();
      }
      if (message.type === 'clearError') {
        errLine = null;
        render();
      }
    });
    vscode.postMessage({ type: 'ready' });
    render();
    updateUi();
  </script>
</body>
</html>`;
}

function deactivate() {}

module.exports = { activate, deactivate };