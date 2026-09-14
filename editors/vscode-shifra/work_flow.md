# Shifra (شِفرة) work flow log

Running log of everything done on the Shifra (Arabic) programming-language project — the RustPython interpreter and its VS Code extension.

> This file is a living log: each new round appends a dated section at the end describing what changed, why, and any caveats.

## Phase 1 — Arabic syntax (Rust)

Implemented the Arabic-level syntax for RustPython (`advanture` → renamed **`arabiya`** crate).

- `عرف`/`استمر` function definitions; `أعد` return; `إذا`/`وإلا`/`وإذا` conditionals; `لكل` loops; repetition via `طالما`; `جرب`/`التقط`/`ختاما` try/except/finally.
- Arithmetic/comparison/comprehension/editing operators.
- Arabic-Indic digits, `صحيح`/`خاطئ`/`عدم` literals, `اطبع`, `ادخل`.
- `aby` (Byterray), `أحجية` (bytes → puzzle/byte-string prefix from the logo), a Unicode escape test, and engine trip wires.
- `multiline` test suite wiring; unicode handling for web/`Unicode` feature.
- Wasm feature-flag fix so the compiler stays consistent in `freeze-stdlib` builds.
- commit `7a88f186e` — **arabiya** crate.

## Phase 1 — VS Code extension (first working version)

- Shifra files (`.sf`, `.ar`, `.شفـ`) get: grammar + `arabiya` language, snippets, F5 run command, `arabiya.rtlEditor` custom editor, and a temp launch config.
- Built the RTL editor inside custom editor HTML with a tokenizer (keywords, literals, builtins, strings, comments).
- Integrated RustPython execution; commit `d130ef6b1`.

## Iteration rounds (editor refinement)

Each round fixes what you found breaking, then we reinstall the `.vsix`.

### Round 0.2.1
- Double-quoted strings with escapes; Arabic/Persian digits.
- RTL layout: right-to-left text, line numbers on the right, text right-aligned.
- RustPython-as-extension default (fake runner fallback), triple-quoted strings.
- Adds `keywords/Literals` sets to webview.

### Round 0.2.2
- Gutter moved to the right; identical fonts across gutter/highlight/textarea.
- Added `--code-font`, `--code-size`, `--code-tab` CSS variables; `editorOptions` sends `codeFont`.

### Round 0.2.3
- Removed the toolbar and Run button; F5/Ctrl+Enter still run.
- Auto-force `arabiya` language mode for Shifra files (`.sf`, `.ar`, `.شفـ`) (fixes "stale plaintext" white code).
- Compact gutter (right-aligned numbers). Translucent selection instead of opaque.
- Caret-height reduction, word-under-cursor highlight, and scroll bottom padding groundwork.

### Round 0.2.4 (root-cause fix for white code)
- **Actual white-code bug**: tokenizer produced `class="keyword"`, CSS used `.tok-keyword` → nothing matched. Fixed by prefixing every class with `tok-`.
- `-webkit-text-fill-color: transparent` so WebKit webviews render the colored layer.
- Function-call coloring (webview tokenizer + grammar `#calls` rule: `[\p{L}][\p{L}\p{N}_]*(?=\()`)`.
- Default text tinted warm gray.

### Round 0.2.5
- **Run command fixed to work from any directory**: `findRepoRoot()` (projectDir setting → `RUSTPYTHON_DIR` → open workspace → the extension's own folder → common checkout paths) and a `cargo --manifest-path` fallback.
- **Ctrl+Shift+I formatting** (`formatShifra`): CRLF→LF, nearest-4-space indent, trailing-space trim, blank-run cap, final newline; registered as a formatting provider + webview shortcut.

### Round 0.2.6
- Unified line heights: `--code-line` (px) on all three layers so the caret, active-line bar, and line numbers never drift during scroll.
- Active-line highlight + glowing active number; new `arabiya.rtlEditor.lineSpacing` setting (default 1.5).

### Round 0.2.7
- Custom caret shortened to 70% of the line height (native caret hidden; the caret is a blinking div positioned with a zero-width `\u200b` marker measured in RTL context).
- Word-under-cursor soft highlight.
- Bottom scroll padding (`--pad-bottom`, 1.5 lines) on editor, highlight, and gutter so the scroll limit keeps all layers aligned.

### Round 0.2.8 (VS Code-style shortcuts, cursor smoothness, run error highlighting)
- **Editor shortcuts in the RTL webview**: `Ctrl+D` (next occurrence / select word when nothing selected), `Ctrl+F` (in-editor find bar with real-time match highlighting and `n/m` counter), `Ctrl+H` (replace bar with Replace / Replace All), `Ctrl+L` (select whole line), `Ctrl+Shift+K` (delete line), `Alt+Shift+↑/↓` (duplicate line), `Ctrl+/` (toggle `#` comment). `Enter`/`Shift+Enter`/`Esc` in the find bar.
- **Cursor/lag fix**: caret movement no longer re-tokenizes the file. The old per-word `cur` tokenizer marks were replaced by a separate word-highlight overlay that is positioned with the same marker-measurement trick, so moving the caret only does a small measurement — no full re-render per word. Scrolling also no longer re-measures: the caret/word top is cached and just shifted by `scrollTop`.
- **Run error highlighting**: the run command captures the script's stderr through a bash process substitution into a temp log; the extension polls it, parses the last `line N` from a Python-style traceback, posts it to every open RTL editor (red error bar + red line number) and raises an error notification with the exception text. Configure via `arabiya.runtime.captureErrors` (on by default; disable on non-bash shells).
- Find/replace marks (`tok-m` / `tok-mu`) render on top of syntax colors.

### Round 0.2.9 (full Arabic object methods + builtins in the editor)
- The editor tokenizer now mirrors the Shifra dictionary: the full `BUILTINS` word set (all Arabic functions, types, constants and exceptions, e.g. `مطلق`, `أي` `كل`, `مرتب`, `خطأ_نوع` …) and a new `METHODS` word set for member names (`.أضف` `append`, `.اخرج` `pop`, `.اجلب` `get`, `.قسم` `split`, `.اربط` `join`, …). Words that follow a `.` are colored with the new `tok-method` class (purple) instead of being treated as calls/keywords.
- Shifra runtime gained methods/builtins per `SHIFRA.md` (see `crates/arabiya`); the editor's coloring sets must stay in sync by hand.

### Round 0.2.10 (language round: inline conditions + Shifra string escapes)
- **One-line `if/else`**: the preprocessor now splits a mid-line statement `else:`/`elif` (`وإلا`/`وإلا_إن`) onto a new line at the same indentation, because Python won't share a physical line with a finished `if` suite. A ternary `else` (no following `:`) stays inline. No tokenizer change — `وإلا`/`وإلا_إن` were already keywords; this is Rust-side (see `crates/arabiya` §7 of `SHIFRA.md`).
- **Shifra string escapes**: inside ordinary string literals, `\` + Arabic first letter now maps to a real Python escape (`\ج`→`\n`, `\ط`→`\t`, `\ر`→`\r`, `\م`→`\\`, `\ص`→`\0`, `\س`→`\b`, `\ن`→`\f`, `\ع`→`\v`, `\ت`→`\a`), per `SHIFRA.md` §6. Raw strings (`r"…"`) keep text verbatim. This is Rust-side; the tokenizer already colors the whole string.
- Re-run takes effect through the rebuilt interpreter, no vsix reinstall needed.

### Round 0.2.11 (Shifra file extensions: `.sf`, `.شفـ`)
- Shifra files changed extension from `.ar` to `.sf`; the interpreter now also accepts a fully-Arabic extension `.شفـ` by sidestep: `is_shifra_file()` in `src/lib.rs` accepts `ar`, `sf`, `شف`, and `شفـ`. Existing `.ar` files still run.
- All Shifra sources in the repo were renamed to `.sf` (`examples/*.sf`, `examples/to_be_deleted/shifra_benchmarks/*.sf`, `shifra_demo.sf`). The Python example originals, the old `arabiya.rs` bootstrap runner, and the benchmark ports moved under `examples/to_be_deleted/`.
- The extension now associates `.ar`, `.sf`, and `.شفـ` with the `arabiya` language (`package.json` `extensions`, `activationEvents`, plus `isShifraPath()` in `extension.js` for run/RTL-editor checks).
- New: `examples/to_be_deleted/shifra_benchmarks/` — 16 Shifra `.sf` ports of the repo's Python benchmarks, plus `run_benchmarks.sh` (times each `.sf` vs its `.py` original) and a `RESULTS.md` table.

### Round 0.3.0 (full rebrand to Shifra + editor intel + live diagnostics)
- **Rebrand**: the extension is now **Shifra** (`shifra-language`, v0.3.0). Internal ids renamed from `arabiya` to `shifra`: language id, grammar scope (`source.shifra`), `shifra.rtlEditor` view type, commands (`shifra.runFile` / `shifra.formatDocument` / `shifra.checkSyntax` / `shifra.openRtlEditor`), settings namespaces (`shifra.runtime.*`, `shifra.rtlEditor.*`), activation events, and the keybinding/menu `when` clauses. Files renamed accordingly (`syntaxes/shifra.tmLanguage.json`, `snippets/shifra.code-snippets`). The legacy env var `RUSTPYTHON_DIR` still works; the new `SHIFRA_DIR` takes priority. Existing user config (`arabiya.*`) breaks — accepted.
- **`--check` compile-only flag in the interpreter** (`src/settings.rs` + `src/lib.rs`): `rustpython --check file` translates Shifra → compiles → reports a `SyntaxError` with a caret pointing into the Arabic source, exit 1; silent exit 0 when valid. This is what powers the live diagnostics below.
- **Live diagnostics**: the extension runs `rustpython --check` debounced (700 ms) on every opened/changed `.ar/.sf/.شفـ` file and publishes errors into the Problems panel (`shifra` collection) — no terminal involved. Run failures also publish a diagnostic on the failing line. New `Shifra: Check Syntax` command (`Ctrl+Shift+C`) and a status-bar item (spinner while running, error state after a failed run).
- **Editor intel** (standard editor, language `shifra`): `DocumentSymbolProvider` (Outline shows `عرف` functions and `صنف` classes) and `FoldingRangeProvider` (indent/`:` blocks + `#` comment blocks). `language-configuration.json` gained `onEnterRules` (auto-indent after `عرف`/`صنف`/`إن`/`لكل`/… suite headers).
- **Hover documentation in the RTL editor**: a `DOCS` dictionary maps every keyword, builtin, method (and the main exceptions) to an Arabic explanation + its Python equivalent; placing the caret on a documented word shows a tooltip above it. Positioned with the same marker-measurement trick as the caret/word highlight.
- **Snippets expanded** from 4 to 21 (`إن`, `وإلا`, `وإلا_إن`, ternary `إنوإلا`, `عرف`, `أعد`, `صنف`, `لامبدا`, `لكل`, `ما_دام`, `استورد`, `مناستورد`, `طابق`, `حالة`, `مع`, `جرب`, `ختاما`, `ارم`, `اطبع`, `طول`, `نطاق`).
- `install_builtins()` in `crates/arabiya` now skips builtins missing from the VM (embedded interpreters lack `help`) instead of aborting; the `call_between_rust_and_python` example embeds Shifra via `import_shifra_module()` (translate → compile → run → register in `sys.modules`).
 - Webview stub test bumped to `webview-test-0.3.0.js` (extraction now unescapes the template literal so word-highlighting and line-edit assertions actually evaluate; added hover-doc assertions). All 26 pass.

### Round 0.3.1 (correct bracket order `()` in the RTL editor)
- **Problem**: in RTL text flow the Unicode Bidirectional Algorithm treats `(`/`)` as neutral characters and reorders them, so `() ` visually became `)(` — not a font issue, and mathematically unavoidable while keeping the whole paragraph RTL.
- **Fix**: the webview now wraps each balanced bracket group `(...)`, `[...]`, `{...}` (on lines where a matching close exists) in an **LTR embedding** `\u202A … \u202C`. Inside the embed the brackets render left-to-right, so `() ` appears correct. Nested brackets are covered because the whole (outermost) group becomes LTR.
- **Effect / trade-off**: a bracketed group now renders as one LTR run, so its arguments appear to the LEFT of the function name visually (e.g. `اطبع\202A("مرحبا")\202C` → `("مرحبا") اطبع`). Everything else stays RTL. This was the approved trade-off.
- **Sync / safety**: the markers are only a display layer. `ltrEmbeds()` wraps doc text when it arrives (and is idempotent — it strips before re-wrapping); `stripEmbeds()` removes every `\u202A`/`\u202C` in `sendEdit`, the webview save handler, and the extension's `applyText` choke-point, so the `.ar/.sf/.شفـ` file and interpreter input always stay clean. Caret/word/highlight positioning use the same wrapped `editor.value`, so they stay aligned.
- Verified: `node --check` on both scripts, plus a standalone run of `ltrEmbedsLine`/`stripEmbeds` (basic call, nested brackets, strings containing brackets, multi-line, idempotent re-wrap, strip round-trip). Version bumped to 0.3.1.

## Commands that matter

```bash
# check syntax + extract + lint the webview script:
node --check extension.js
node -e "const fs=require('fs');const s=fs.readFileSync('extension.js','utf8');const o=s.indexOf('<script nonce=');const c=s.indexOf('</script>',o);const js=s.slice(s.indexOf('>',o)+1,c).replace(/\\\\/g, String.fromCharCode(92));fs.writeFileSync('/tmp/opencode/html.js',js);"
node --check /tmp/opencode/html.js

# stub-DOM functional test of the webview script:
node /tmp/opencode/webview-test-0.3.0.js

# package the vsix:
npx --yes @vscode/vsce package --out /tmp/opencode/shifra-language-0.3.0.vsix
```

## Reinstall ritual (after every round)

Extensions → Shifra → Uninstall → **File → Reopen Window** → Install from VSIX → **File → Reopen Window** → confirm the new version in the Extensions panel.

## Gotchas learned

- The webview script lives inside a JS **template literal** in `extension.js`: backslashes must be doubled (`\\\\` for a regex `\\`), and invalid escapes like `\]` are a SyntaxError inside a template literal (build regex chars via char codes / `includes` instead).
- WebKit webviews ignore `color: transparent` on a textarea — you need `-webkit-text-fill-color: transparent`.
- A textarea can only have one selection, so `Ctrl+D` is a "select next occurrence" approximation, not true multi-cursor.
- `Ctrl+Shift+I` is Electron's devtools shortcut; the webview capture usually wins, but the native-editor keybinding is the reliable path.
- Error capture needs `2> >(tee … 1>&2)`. That's bash-specific → it's a setting you can switch off.