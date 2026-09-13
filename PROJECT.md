# شِفرة · Shifra — Arabic Python 3

A complete **Arabic programming language** (called **Shifra**, written شِفرة) built
on top of a RustPython fork. The project is a "mother-project" containing several
moving parts that share one interpreter core:

| Part | Location | What it is |
| --- | --- | --- |
| The interpreter | `crates/` + `src/` | A fork of the RustPython Python-3 interpreter, wired to accept Shifra (Arabic) source files (`.sf`, `.ar`, `.شفـ`). |
| The language layer | `crates/arabiya/` | The translator/dictionary that turns Shifra text into Python AST-equivalent text before compiling. |
| The language spec | `SHIFRA.md` | The canonical Arabic ↔ Python dictionary (keywords, builtins, methods, escapes, imports) — the source of truth. |
| Desktop editor | `editors/vscode-shifra/` | A VS Code extension: syntax highlighting, RTL-aware custom editor, live diagnostics, running files. |
| Terminal editor | `editors/nvim-shifra/` | A Neovim plugin with RTL rendering, syntax, diagnostics, and snippets. |
| Fonts | `editors/fonts/` | Two monospace Arabic fonts (Thanyah Sans Term + Shifra Naskh Term) bundled with the editors. |
| Mobile app | `android/` | A Gradle-free Android app ("Shifra") that embeds the interpreter as `libshifra.so`, with its own RTL code editor, examples, and output view. |
| GUI demos | `example_projects/shifra_tk/` | Tkinter GUI programs written in Shifra (turtle star, counter app, welcome window), demonstrating Arabic-RTL GUI support in RustPython's Tk binding. |
| Docs/ops | `README.md`, `CONTRIBUTING.md`, `AGENTS.md`, `scripts/` | Build, test and AI-agent operating instructions. |

---

## 1. What is this project, in one paragraph

Shifra is a full Arabic dialect of Python. A program written in Shifra looks like
this (demo.sf, section ⑪):

```shifra
عرف أكبر(أ, ب):
    إذا أ > ب:
        أعد أ
    وإلا:
        أعد ب

طابع = "موجب" إذا ن >= ٠ وإلا "سالب"
```

Every Arabic keyword is spelled by the **meaning** of its Python counterpart
(`عرف` = "define" = `def`, `إذا` = "if" = `if`, `اطبع` = "print", `قائمة` = "list",
etc.). Arabic-Indic digits are accepted everywhere. The preprocessor rewrites this
into ordinary Python and hands it to the RustPython compiler, so **all of Python
3.14 semantics** work unchanged. The project does *not* implement a separate
language engine — it translates to Python 3.14 and reuses CPython's `Lib/` standard
library plus RustPython's VM.

---

## 2. Origins and lineage

- **Upstream:** This is a fork of [RustPython](https://github.com/RustPython/RustPython),
  a Python **3.14.0** interpreter written in Rust (the workspace version is `0.5.0`,
  Rust edition 2024).
- **Local commits** (on branch `main`, on top of upstream):
  1. `84f63d829` — `إضافة الادخال` ("add input")
  2. `f772831e1` — `شغال` ("working")
  3. `7a88f186e` — `Add arabiya library crate with .ar CLI support`
  4. `d130ef6b1` — `Add VS Code extension for Shifra .ar files`
- The current repo is mid-rebrand from the older name `vscode-arabiya` → `vscode-shifra`
  and `.ar` → `.sf` (`.ar` still supported for compatibility).
- **Driving use-case:** allowing Arabic-speaking children/students to learn the
  Python 3 programming model in their own language, on desktop (CLI, VS Code) and
  on an Android phone, without needing to switch keyboard layouts.

---

## 3. Repository layout

```
RustPython/
├── Cargo.toml                # workspace root; rustpython (v0.5.0) executable crate
├── SHIFRA.md                 # ★ the Arabic↔Python dictionary (source of truth)
├── README.md                 # upstream-style readme + Shifra quick start
├── CONTRIBUTING.md, AGENTS.md
├── TKINTER_TURTLE_NOTE.md    # notes on RTL/shaping + rebuilding the released Tk DLL
├── src/                      # the `rustpython` CLI binary
│   ├── main.rs               # thin entry → InterpreterBuilder + run()
│   ├── settings.rs           # args; incl. --check (compile-only), file detection
│   └── lib.rs                # is_shifra_file(), translation before compile
├── crates/
│   ├── arabiya/              # ★ THE language layer (new)
│   │   ├── src/{lib,keywords,builtins,methods,modules,preprocessor}.rs
│   │   └── tests/translation.rs
│   ├── vm/ compiler/ parser/ derive/ stdlib/ wasm/ capi/ pylib/ ...  # upstream crates
│   └── stdlib/src/tkinter.rs # has `mod rtl` (unix): Arabic shaping + fribidi reorder
├── Lib/                      # CPython 3.14 stdlib (unchanged upstream)
├── editors/
│   ├── fonts/                # ThanyahSansTerm.ttf, ShifraNaskhTerm.ttf
│   ├── vscode-shifra/        # the VS Code extension (publishes shifra-language)
│   └── nvim-shifra/          # the Neovim plugin
├── android/                  # the Android app (Gradle-free build)
│   ├── build.sh              # aapt2 + javac + d8 + zipalign + apksigner pipeline
│   ├── AndroidManifest.xml
│   ├── assets/demo.sf        # ~281 lines, 22 numbered sections ([دليل] demo)
│   ├── src/com/shifra/language/{MainActivity,KodeEditor,OutputActivity,SettingsActivity}.java
│   ├── res/                  # layouts, drawables, mipmaps, strings, styles
│   └── dist/                 # built APKs (shifra-0.3.x.apk)  [gitignored]
├── examples/                 # small .sf scripts + Rust embedding demos (package_embed_*)
├── example_projects/shifra_tk/  # Tkinter GUI apps written in Shifra
├── scripts/                  # developer tools (whats_left.py, update_lib, …)
├── extra_tests/snippets/     # pytest suite for the interpreter
└── .github/workflows/        # upstream CI (zizmor-scanned)
```

---

## 4. The interpreter core (fork of RustPython)

- **Workspace crates** (all version `0.5.0`, edition 2024): the main `rustpython`
  binary crate plus `rustpython-vm`, `rustpython-compiler`, `rustpython-parser`,
  `rustpython-derive`, `rustpython-stdlib`, `rustpython-pylib`, `rustpython-wasm`,
  `rustpython-capi`, and the new `rustpython-arabiya` (v0.1.0).
- **Key features** wired into the binary crate:
  - `stdlib`, `importlib`, `encodings`, `stdio`, `threading`
  - `freeze-stdlib` — build the stdlib into the binary (used by the Android app)
  - `tkinter` — enables Tcl/Tk GUI bindings (used on desktop with the bundled RTL
    `tk.pc`; see `TKINTER_TURTLE_NOTE.md`)
  - `ssl-rustls-aws-lc`, `jit`, `sqlite`, `capi`, `flame-it` — optional.
- **CLI entry points** (relevant to Shifra):
  - `rustpython script.sf` — detect Shifra by extension, translate, run.
  - `rustpython --check file.sf` — compile (translate) without executing — used by
    the VS Code/nvim diagnostics features.
  - `rustpython` (no args) — REPL; note: stdin is **not** translated, so piping a
    `.sf` file via stdin fails with `NameError`; you must pass the file path as an
    argument.

---

## 5. The language layer: `crates/arabiya`

This new crate contains all Shifra knowledge. Tables are kept in sync with
`SHIFRA.md` (that file is the canonical spec).

### 5.1 The dictionaries

| File | Entries | What it maps |
| --- | --- | --- |
| `keywords.rs` | **38** | Arabic → Python keyword spellings (`عرف`→`def`, `إذا`→`if`, `وإذا`→`elif`, `وإلا`→`else`, `عدم`→`None`, `صحيح`→`True`, `خاطئ`→`False`, …). |
| `builtins.rs` | **136** | Arabic → Python builtin functions & types (`اطبع`→`print`, `ادخل`→`input`, `نطاق`→`range`, `قائمة`→`list`, `كسري`→`float`, plus all exceptions `خطأ…`→`…Error`). Ships `install_builtins()` (runtime installer used by the embed example). |
| `methods.rs` | **80** | Arabic → Python **object methods** (`.أضف`→`.append`, `.امسح`→`.clear`, `.عد`→`.count`, `.رتب`→`.sort`, `.عكس`→`.reverse`, … — see SHIFRA.md §5). Methods are translated in attribute/call position after a `.`. |
| `modules.rs` | **9** | `MODULES` (2): module names rewritten only in `from … import` position (`من معطيات_الصنوف`→`from dataclasses`, `من تصنيف`→`from typing`); `NAMES` (7): always rewritten names like `الكل__`→`__all__`, `الاسم__`→`__name__`, `__التهيئة__`→`__init__`, `معطيات_الصنف`→`dataclass`, `أي_نوع`→`Any`. |

### 5.2 The preprocessor (`preprocessor.rs`, ~938 lines, 58 unit tests)

`translate(source: &str) -> String` is a character-level Shifra→Python source
rewriter. It:

1. Scans tokens, skipping over string literals and comments (content is preserved
   verbatim, only the surrounding code is rewritten).
2. Rewrites **Arabic keywords** and **builtin identifiers**; unknown identifiers
   (user variables, function names) pass through untouched.
3. Converts **Arabic-Indic** (`٠-٩`, `۰-۹`) and updates digits in code positions.
4. Handles **f-strings**: prefix `مـ` (م + tatweel) = Python `f`; the literal is
   emitted as `f"…"`; plain text kept verbatim; every `{…}` expression is itself
   translated with full Shifra rules; `{{`/`}}` literal braces handled; Latin
   `f`/`F` prefixes also recognized.
5. Handles **one-line `if`/`else`/`elif` statements**: when the body and `:` are on
   the same line, the `وإلا`/`وإذا` is moved onto a new line (Python cannot share a
   physical line with a finished `if` suite). Works nested inside indented blocks.
6. Applies **Shifra escape sequences** inside non-raw string literals:
   `\ج`→`\n` (جديد), `\ط`→`\t` (تبويب), `\ر`→`\r` (رجوع), `\م`→`\\` (مائل),
   `\ص`→`\0` (صفر), `\س`→`\b` (سابق), `\ن`→`\f` (نهاية), `\ع`→`\v` (عمودي),
   `\ت`→`\a` (تنبيه). `\"`/`\'` untouched; raw strings (`r"…"`) keep text verbatim.
7. Distinguishes statement-level `وإلا:` from ternary `… إذا … وإلا …` (checks for
   the following `:`).
8. Keeps a `translate()` entry that is also exposed at runtime, so the embed
   examples and the C-API can reuse it.

### 5.3 Translating "original naming" — the rule behind every word

Words follow a consistent naming scheme (a handful of exceptions are documented in
`SHIFRA.md`):
- keyword/control words — the **meaning**: `إذا`-family (if), `لكل` (for),
  `طالما` (while), `ضمن` (in), `عرف` (def), `صنف` (class), `أعد` (return).
- letter-named things — the **first letter of the Arabic word**: `ج` from جديد
  (newline), `م` from مائل (backslash), `ص` from صفر, etc.
- negation/absence — `عدم` (None), `ليس` (not), `خاطئ` (False), `صحيح` (True).
- exceptions — `خطأ_…` (`خطأ_اسم` = NameError, `خطأ_استيراد` = ImportError, …).

---

## 6. Reading the dictionary: `SHIFRA.md`

`SHIFRA.md` is a browsable Arabic/English spec (read it to answer "what is the
Arabic for X"). Its structure:

1. **Keywords** — the 38-word table above (with notes: `نوع` doubles as builtin
   `type`; `احذف` is the statement form of `del`).
2. **Literals & constants** — `صحيح` (True), `خاطئ` (False), `عدم` (None),
   `غير_منفذ` (NotImplemented), `ثلاث_نقاط` (Ellipsis).
3. **Builtin functions & types** — the 136-entry table (functions + types + the
   exception classes).
4. **Exceptions** — the `خطأ…` family mapped onto Python's exception hierarchy.
5. **Object methods** — 89 documented method spellings (`قسم`, `اقتسم`, `ضم`,
   `مبادلة`, `فرز`, `عكس`, …).
6. **Escape sequences** — the 9-letter table above + the `مـ` f-string prefix + `{{`
   /`}}` literal braces.
7. **Inline conditions** — ternary `… إذا … وإلا …` and one-line
   `إذا … : … وإلا : …`, including all-on-one-line `if/elif/else` chains.
8. **Imports & library names** — `من معطيات_الصنوف استورد …` (dataclasses),
   `تصنيف` (typing), `معطيات_الصنف`/`أي_نوع` (dataclass/Any) plus dunder names.

**Editing the language** (important rule): edit `SHIFRA.md` **first**, then re-sync
the four Rust tables (`keywords.rs`, `builtins.rs`, `methods.rs`, `modules.rs`) *and*
the two editor token lists (VS Code `extension.js` KEYWORDS/BUILTINS + grammar,
nvim `syntax/shifra.vim`), then rebuild `--release` and re-package the extension/APK.

---

## 7. Right-to-left (RTL) text engineering

Arabic must be visually ordered and cursive-join even in monospace contexts. The
project deals with this at several layers:

- **Fonts** (`editors/fonts/`, mirrored into `vscode-shifra/media/`):
  - `ThanyahSansTerm.ttf` (≈246 KB) — a monospace Arabic terminal font used by the
    VS Code RTL editor; Arabic characters join properly at fixed advance widths.
  - `ShifraNaskhTerm.ttf` (≈180 KB) — a Naskh-style monospace Arabic font.
- **Tkinter binding** (`crates/stdlib/src/tkinter.rs`, module `rtl`):
  Tk on X11 has no Unicode bidi layout or Arabic shaping. This module works around
  it in two deterministic steps: (1) a custom **Arabic letter shaper** that turns
  letters into their contextual presentation forms (isolated/initial/medial/final,
  plus lam-alef ligatures), and (2) an ordering pass via the system
  **fribidi** library (linked with `#[link(name = "fribidi")]`) that reorders the
  shaped text to visual order, which Tk then draws correctly. Only compiled on
  unix (Windows/macOS Tk does this natively). `TKINTER_TURTLE_NOTE.md` documents
  how to build the Tcl/Tk DLL that carries this support and how to rebuild the
  released binaries.
- **Android editor** (`KodeEditor.java`) — a custom RTL `EditText`-based editor,
  and the Android activity is `android:supportsRtl="true"`.
- **Neovim** — `rtl.lua` renders the buffer right-to-left (three toggle modes).
- **VS Code** — a Webview-based custom editor (`shifra.rtlEditor`) that renders
  the file right-to-left inside VS Code (the built-in editor keeps the plain-text
  side; see §9).

---

## 8. Desktop CLI

The interpreter binary doubles as a Shifra runner:

```bash
# demo
cargo run --release -- android/assets/demo.sf
# compile-only (used by editors' "check syntax")
cargo run --release -- --check android/assets/demo.sf
# run any .sf/.ar/.شفـ file
cargo run --release -- path/to/prog.sf
```

`examples/package_embed.sf` and `examples/package_embed_arabic.sf` demonstrate
using the `arabiya` crate (and `install_builtins()`) to embed Shifra inside Rust
programs; `examples/برمجة.شفـ`, `examples/atexit_example.sf`,
`examples/call_between_rust_and_python.sf` are small standalone demos.

---

## 9. The VS Code extension (`editors/vscode-shifra`)

**Package:** `shifra-language` v**0.3.8**, publisher `shifra-local`, MIT,
engine `^1.95.0`, `main = extension.js` (~1,750 lines).

**File associations:** `.ar`, `.sf`, `.شفـ` → language id `shifra`.

**Features:**
- **Syntax highlighting** — a TextMate grammar (`syntaxes/shifra.tmLanguage.json`)
  with repositories for: comments, string literals (all prefix combos incl. the
  `مـ` f-string form, raw, triple-quoted, escapes as `constant.character.escape`),
  keywords (including the new `وإذا`), literals (`صحيح|خاطئ|عدم`), function-call
  highlighting, and more.
- **Custom RTL editor** — a Webview custom editor (`shifra.rtlEditor`) that ignores
  VSCode's LTR line model and renders each visual line right-to-left with shaped
  Arabic, a monospace font (default `JetBrains Mono, Thanyah Sans Term, monospace`),
  caret/line numbers, optional line numbers column, configurable tab size and line
  spacing (settings `shifra.rtlEditor.*`). Opens by default for `.ar`/`.sf`/`.شفـ`;
  a plain-text editor can be side-by-side.
- **Live diagnostics** — on save, runs the interpreter `--check` on the file and
  squiggles the failing line; no server needed. Uses `shifra.runtime.*` settings
  (command path or auto-detection of the checkout, terminal name, stderr capture).
- **Symbols & folding** — document symbol provider + folding ranges (from
  indentation and block keywords) for the RTL view.
- **Hover documentation** — hover an Arabic word to see its Python meaning and/or
  usage notes (the `DOCS` table, ~190 entries).
- **Snippets** — `snippets/shifra.code-snippets` (21 snippets): control flow,
  functions, classes, loops, conditionals, f-string, import, with, match, etc.
- **Running** — `shifra.runFile` (F5) or the editor-title run button opens an
  integrated terminal (default name `Shifra`) and runs the current file with the
  detected interpreter. One-key check (`Ctrl+Shift+C`) and format
  (`Ctrl+Shift+I`).
- **VSIX packaging:** `node_modules/.bin/vsce package -o shifra-language-0.3.8.vsix`
  → 12-file package (~230 KB). A previous stale `0.3.7.vsix` is removed after bump.

---

## 10. The Neovim plugin (`editors/nvim-shifra`)

A lightweight Lua plugin (10 files):
- `ftdetect/shifra.vim` — `.ar`/`.sf`/`.شفـ` → `shifra` filetype.
- `syntax/shifra.vim` — syntax groups mirroring the SHIFRA dictionaries
  (keywords incl. ثالث `وإذا`, literals, builtins, methods, strings, escapes,
  comments).
- `ftplugin/shifra.lua` + `lua/shifra/init.lua` — buffer options (soft tabs,
  tabstop), integration wiring (snippets, diagnostics, RTL).
- `lua/shifra/rtl.lua` — right-to-left text rendering: cycles three visual modes
  (LTR off / RTL-on / reverse); Arabic shaping applied; tracked per window.
- `lua/shifra/diagnostics.lua` — runs `--check` (debounced ~650 ms) after changes,
  surfaces errors, supports Vizdoom-Free jumping to errors.
- `lua/shifra/runtime.lua` — finds/reuses the interpreter binary from the project
  or known paths.
- `lua/shifra/snippets.lua` — 21 snippet definitions (mirrors VS Code snippet set).
- `plugin/shifra.vim` — default keymaps (`<leader>Sf` run, etc.).
- `README.md` — install/usage guide.

---

## 11. The Android app (`android/`)

**No Gradle, no Android Studio** — a raw command-line build. It produces
`dist/shifra-<VER>.apk` (currently `shifra-0.3.8.apk`, ~55 MB).

### How it's built (`build.sh`, top to bottom)
1. Cross-compile the interpreter for **arm64-v8a** and **x86_64** with
   `--no-default-features --features stdlib,stdio,threading,importlib,freeze-stdlib`
   (produces binaries that need only `libm`, `libc`, `libdl`). The freezing embeds
   the standard library, so the app has **no Python files** inside.
   (`RUST_BUILD=1` re-runs this step; otherwise the last built binaries are reused.)
2. `aapt2 compile` all `res/*` → resources zip; `aapt2 link` against platform
   **android-35** with the manifest (package `com.shifra.language`, versionCode 2,
   versionName from `$VER`, minSdk 24, targetSdk 34, **no permissions requested**,
   `extractNativeLibs="true"`, `supportsRtl="true"`, `debuggable="true"`).
3. `javac` the four Java sources (with generated `R.java`) → `.class`, then **d8**
   → `classes.dex`.
4. Copy the two `.so` files into `lib/arm64-v8a/` and `lib/x86_64/` (as
   `libshifra.so`), plus `assets/demo.sf`.
5. `zipalign` + `apksigner` with `keystore/debug.keystore` (Android debug key).

### The Java app (`src/com/shifra/language/`)
- **`MainActivity`** (~1,041 lines): an Arabic-language UI.
  - Code editor built on the custom RTL editing surface (`KodeEditor`), with a
    large in-app **template table** (`EXAMPLES`, 7+ programs) and editor
    autocomplete snippets (`EDITOR_SNIPPETS`); menu actions: Run ▸, Save, Open,
    Examples, Settings, Share/Copy.
  - `SNIPPETS`/`EXAMPLES` dialog pickers; first-run migration of legacy preference
    keys (`snippets` → `snippets_full`).
  - Execution: writes the edited script to a temp file, spawns
    `libshifra.so <file>` via `ProcessBuilder` on a background thread, captures
    stdout/stderr, and shows the result in `OutputActivity` (or an error dialog).
  - RTL layout: `android:supportsRtl`, right-aligned editor, RTL-aware menus.
- **`KodeEditor`** (351 lines): the RTL-aware editor view (word wrap, font size,
  monospace Arabic shaping, line numbers, highlighted current line).
- **`OutputActivity`** (173 lines): scrollable output pane with copy/share.
- **`SettingsActivity`** (95 lines): code font family/size, etc., persisted in
  `SharedPreferences`.

### The demo script (`assets/demo.sf`)
~281 lines, 22 numbered sections demos every Shifra feature: variables & Arabic
digits, strings & escapes, **multi-line strings**, list methods, list
comprehensions, string methods, dict methods, set methods, generators (`انتج`),
while/break/continue, if/elif/else incl. one-line + ternary, booleans,
higher-order builtins, type conversions, classes, exceptions, introspection,
`with`/`match`, global/local, async definitions, explicit `del`, and imports.

### On-device verification
```bash
adb install -r android/dist/shifra-0.3.8.apk
APPDIR=$(adb shell pm path com.shifra.language | cut -d: -f2)
BASEDIR=$(adb shell dirname "$APPDIR")
# run a script (pass the path as an argument — stdin is the untranslated REPL!)
adb push prog.sf /data/local/tmp/prog.sf
adb shell "run-as com.shifra.language '$BASEDIR/lib/arm64/libshifra.so' /data/local/tmp/prog.sf"
```

---

## 12. Example projects and demos

- `examples/package_embed.sf` / `package_embed_arabic.sf` — running Shifra code
  inside a Rust binary through the `arabiya` crate.
- `examples/برمجة.شفـ` — a small Shifra "programming" demo (interactive).
- `examples/atexit_example.sf`, `examples/call_between_rust_and_python.sf`.
- `example_projects/shifra_tk/` — Tkinter demos: `عداد_نقرات.sf` (click
  counter), `نجمة_سلحفاة.sf` / `مربع_سلحفاة.sf` (turtle star / square),
  `نافذة_ترحيب.sf` (welcome window) — all with Arabic labels/buttons proving the
  RTL Tk binding (run with `cargo run --release --features tkinter -- example_projects/shifra_tk/عداد_نقرات.sf`).

---

## 13. Tests and quality gates

- **Unit + integration for the language layer:**
  `cargo test -p rustpython-arabiya` → **58 unit tests** in `preprocessor.rs`
  + **12 integration tests** in `crates/arabiya/tests/translation.rs` (all pass).
- **Interpreter:** `cargo test --workspace --exclude rustpython_wasm
  --exclude rustpython-venvlauncher --exclude rustpython-capi`, plus `(cd crates/capi && cargo test)`.
- **Python-level:** `pytest -v` in `extra_tests/`; standard-library tests via
  `cargo run --release -- -m test <module>`.
- **Lint:** `cargo clippy --workspace --all-targets --exclude …` (+ the capi split);
  `ruff` for Python.
- **Git hooks:** `prek install` (pre-commit) must run on every commit; never
  bypass with `--no-verify`. Deliberate Shifra prose inside test data/comments is
  allowed but keep it clearly marked.

---

## 14. Building everything

### Desktop (host, Linux, with tkinter)
```bash
cargo build --release --features tkinter --bin rustpython \
  PKG_CONFIG_PATH=/home/amr/.shifra-tk/lib/pkgconfig   # Tcl/Tk RTL build
cargo run --release -- android/assets/demo.sf
```

### Android cross-compile (needs NDK 28.2.13676358 + static libffi builds)
```bash
# per-architecture Cargo invocations (separate so each cache is valid):
#   aarch64: CC/AR/CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = aarch64-linux-android35-clang / llvm-ar
#            RUSTFLAGS="-C link-arg=-L/tmp/opencode/ffi-aarch64/lib"
#   x86_64:  CC/AR/CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER = x86_64-linux-android35-clang / llvm-ar
#            RUSTFLAGS="-C link-arg=-L/tmp/opencode/ffi-x86_64/lib"
VER=0.3.8 ./android/build.sh    # RUST_BUILD stays unset → reuse binaries
adb install -r android/dist/shifra-0.3.8.apk
```

### VS Code extension
```bash
cd editors/vscode-shifra
node_modules/.bin/vsce package -o shifra-language-0.3.8.vsix
code --install-extension shifra-language-0.3.8.vsix
```

### Neovim plugin
Add `editors/nvim-shifra/` to your runtime path (or a package manager), with the
fonts installed as your terminal's monospace Arabic font.

---

## 15. Design decisions and current limitations

- **Preprocessor, not parser:** Shifra is a source-text rewrite; there is no
  separate grammar. This keeps 100% Python 3.14 semantics but means the translator
  must be careful about strings/comments/f-strings (hence the escaping and
  `{…}`-expression handling) and about one-line statements (hence the line-split
  logic).
- **Method translation needs a context**: identifier rewrites only happen where
  the grammar allows (after `.` in attribute position, in `from … import` module
  position, keyword position, call position). Names that look like a method but
  are user-defined are left alone → this is why method spellings are in their own
  table (`methods.rs`) and take precedence in argument/attribute position.
- **`--check` is compile-only:** diagnostics check syntax/translation; they cannot
  do type/undefined-name checking (that requires execution).
- **stdin is the REPL:** piping `.sf` through stdin runs the untranslated REPL;
  always pass the file path.
- **Android:** no permissions, no network, no Gradle — a fully offline interpreter
  app; only two ABIs (arm64 + x86_64) and one frozen stdlib.
- **Legacy naming:** the extension/plugin still accept the old `.ar` extension for
  compatibility; new canonical extensions are `.sf` and `.شفـ`.
- **Distribution snapshots:** `Shifra-for-friends.zip` on repo root is a *stale*
  pre-rebrand snapshot (Sep 8) and currently out of sync with this tree.

---

## 16. Quick start cheat-sheet

| Want to… | Do |
| --- | --- |
| Run a `.sf` on desktop | `cargo run --release -- prog.sf` |
| Run a Tk GUI demo | `cargo run --release --features tkinter -- example_projects/shifra_tk/عداد_نقرات.sf` (with `PKG_CONFIG_PATH=/home/amr/.shifra-tk/lib/pkgconfig`) |
| Check a file without running | `cargo run --release -- --check prog.sf` |
| Write Arabic code | open any editor extension; keywords/builtins in SHIFRA.md |
| Build the language layer test | `cargo test -p rustpython-arabiya` |
| Build & package the Android app | `VER=0.3.8 ./android/build.sh` → `android/dist/shifra-0.3.8.apk` |
| Package the VS Code extension | `vsce package` in `editors/vscode-shifra/` |
| Find missing/unmapped features | `scripts/whats_left.py` |
| Change the language | edit `SHIFRA.md` → re-sync `crates/arabiya/src/*.rs` + editor token lists → rebuild → re-package |