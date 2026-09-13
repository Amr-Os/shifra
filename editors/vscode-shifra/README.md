# Shifra (شِفرة) for VS Code

This extension makes Shifra source files (`.sf`, `.ar`, `.شفـ`) first-class citizens in VS Code. Shifra is Arabic Python — a Python 3 interpreter whose keywords and standard library are spelled in Arabic. Arabic code is unreadable without right-to-left layout, so this extension also ships **Shifra RTL Editor**: a real editing surface that lays the code out right-to-left, with line numbers on the right edge (the natural side for an Arabic reader), syntax colors, hover documentation, live diagnostics, and one-key running.

## Features

- **RTL-friendly editing** of Arabic source (right-aligned text, correct word order), with line numbers on the right edge.
- **Syntax highlighting** for keywords, literals, builtins, methods, numbers, strings, and comments (Arabic-Indic ٠-٩ and Persian ۰-۹ digits included). The word tables mirror the Shifra dictionary: every builtin (functions, types, constants, exceptions — e.g. `اطبع`, `طول`, `مرتب`, `خطأ_قيمة`) is colored, and member names after a `.` (`.أضف`/`append`, `.اخرج`/`pop`, `.اجلب`/`get`, `.قسم`/`split`, …) get their own color.
- **Live diagnostics** — every Shifra file is syntax-checked as you type (`rustpython --check`), reported into the Problems panel without opening a terminal. `Ctrl+Shift+C` re-checks the active file instantly.
- **Run support**: press `F5` (or `Ctrl/Cmd+Enter`) to execute the current Shifra file.
- **Run error feedback**: a failed run highlights the failing line (red bar + red line number), publishes an error diagnostic to the Problems panel, and pops up the exception text.
- **Status-bar state**: a Shifra status-bar item shows a spinner while running and reports an error state (`Ctrl+Click` re-checks syntax).
- **Hover documentation**: place the cursor on any keyword, builtin, or method (e.g. `اطبع`, `نطاق`, `.قسم`) and a tooltip over the word shows its Python equivalent and Arabic explanation right inside the RTL editor.
- **Outline (symbols)**: `عرف` (def) and `صنف` (class) definitions are listed in the Outline view.
- **Code folding**: block structure (indentation and `:` blocks plus `#` comment blocks) folds in the standard editor.
- **Smart snippets**: `إذا`, `وإلا`, `وإذا`, `عرف`, `أعد`, `صنف`, `لكل`, `طالما`, `طابق`/`حالة`, `استورد`, `من…استورد`, `مع`, `جرب`/`التقط`/`ختاما`, `لامبدا`, and more.
- **Find & replace** in the RTL editor: `Ctrl+F` search bar, `Ctrl+H` adds replace (`Enter`/`Shift+Enter` move between matches, `Replace All` included), `Esc` closes.
- **Multi-cursor-style additions**: `Ctrl+D` selects the next occurrence, `Ctrl+L` selects the whole line.
- **Line editing**: `Ctrl+Shift+K` deletes the line, `Alt+Shift+↑/↓` duplicates it, `Ctrl+/` toggles a `#` comment.
- **Format support**: `Ctrl+Shift+I` normalizes 4-space indentation, trims trailing spaces, and cleans the file end. The RTL editor applies the same rule with `Ctrl+Shift+I`.
- **Focused-line feedback**: the cursor line gets a subtle highlight, its number glows, and the word under the cursor is softly marked.
- The standard text editor with the Arabic grammar is still available via **Reopen Editor With → Text Editor**.

## Run configuration

The `F5` run works from anywhere (the Shifra file doesn't have to live inside the Shifra checkout). The extension locates the interpreter in this order:

1. `shifra.runtime.command` — a custom command, if set.
2. A built `rustpython` binary under `target/debug` or `target/release` of the Shifra checkout.
3. `cargo run --release` in the Shifra checkout (found via `shifra.runtime.projectDir`, the `SHIFRA_DIR` environment variable, the legacy `RUSTPYTHON_DIR`, an open workspace, or the extension's own folder during development).

> From a plain terminal, run `cargo run --release --manifest-path /path/to/Shifra/Cargo.toml -- /path/to/file.sf` — a bare `cargo run -- file.sf` only works when your current directory is already inside the checkout.

Output appears in an integrated terminal named **Shifra** (configurable via `shifra.runtime.terminalName`).

> **Note:** error capture uses a bash process substitution to tee the script's stderr into a log file (`shifra.runtime.captureErrors`, on by default). If your default shell isn't bash and a run fails to start, disable this setting.

> **Note:** if another extension has taken over the `F5` key (for example a user-assigned Code Runner binding), your own bindings always win over extension bindings in the standard editor. Inside the Shifra RTL Editor, `F5` is handled directly by the editor itself, so it works regardless. You can always run with `Ctrl/Cmd+Enter` as well.

## Settings

| Setting | Default | Purpose |
| --- | --- | --- |
| `shifra.runtime.command` | *(empty)* | Command to run Shifra files; empty = auto-detect `rustpython`. |
| `shifra.runtime.projectDir` | *(empty)* | Path to the Shifra checkout for interpreter auto-detection. |
| `shifra.runtime.terminalName` | `Shifra` | Name of the terminal used for runs. |
| `shifra.runtime.captureErrors` | `true` | Capture script stderr to highlight the failing line. |
| `shifra.rtlEditor.fontFamily` | `JetBrains Mono, Thmanyah Sans Term, monospace` | Monospace font for the RTL editor (Arabic must stay grid-aligned). `Thmanyah Sans Term` is the bundled monospace-tuned Arabic face. |
| `shifra.rtlEditor.fontSize` | `16` | Font size for the RTL editor. |
| `shifra.rtlEditor.tabSize` | `4` | Spaces inserted per Tab in the RTL editor. |
| `shifra.rtlEditor.lineSpacing` | `1.5` | Line height multiplier (keeps code, numbers, and caret aligned). |

## Use

Install the extension, open any Shifra file (`.sf`), and it opens in the Shifra RTL Editor. Press `F5` to run it.

## Fonts

The RTL editor is monospace so code stays grid-aligned; the default font stack is `JetBrains Mono, Thmanyah Sans Term, monospace`. Two monospace-tuned Arabic faces ship in this repo at `../fonts/`:

- `ThmanyahSansTerm.ttf` — Thmanyah Sans Medium with each letter kept at its natural shape and position in a 600-unit grid (recommended).
- `ShifraNaskhTerm.ttf` — Noto Naskh Arabic re-tuned to fill the monospace cell edge-to-edge.

Install them (`cp editors/fonts/*.ttf ~/.fonts && fc-cache -f`) or use any other monospaced Arabic-capable family, and set it via `shifra.rtlEditor.fontFamily`.

## Package

Install `@vscode/vsce`, then package from this directory:

```bash
npm install --global @vscode/vsce
vsce package
```

Install the generated `.vsix`: **Extensions → ⋯ → Install from VSIX…** Then fully reload VS Code (File → Reopen Window).