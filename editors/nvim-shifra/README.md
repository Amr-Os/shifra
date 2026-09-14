# shifra.nvim

Neovim support for **Shifra** (شِفرة) — Arabic Python, a Python 3 interpreter whose
keywords and standard library are spelled in Arabic. This plugin makes `.sf`,
`.ar`, and `.شفـ` source files first-class citizens in Neovim without any
external runtime dependencies.

## Features

- **Filetype detection** for `.sf`, `.ar`, and the Arabic-mantissa extension `.شفـ`.
- **Syntax highlighting** — the complete Shifra dictionary: keywords, builtins,
  types, constants, exceptions, and method names after a dot (`.أضف`, `.اخرج`,
  `.اجلب`, `.قسم`, …), including Arabic-Indic (٠-٩) and Persian (۰-۹) digits.
- **RTL rendering** — Arabic code is right-aligned with the letters joined,
  using Neovim's own `rightleft` + `arabicshape` pipeline (the only reliable way
  on terminals without BiDi, e.g. Ghostty). VTE terminals (GNOME Terminal) are
  switched to explicit mode via `\e[8l` so they don't re-process the output.
  Three modes, toggleable per buffer:
  - `<leader>tr` — Neovim shaping (default, recommended)
  - `<leader>tl` — terminal BiDi (VTE-style terminals only)
  - `<leader>tg` — ghostty-native: **broken upstream**, do not use
- **Live diagnostics** — every buffer is syntax-checked as you type with
  `rustpython --check` (debounced 650 ms), reported via `vim.diagnostic`.
  `:ShifraCheck` re-checks the current buffer immediately.
- **Run support** — `<leader>r` runs the file in a terminal split.
- **Self-contained snippets** — type a trigger (`عرف`, `إذا`, `اطبع`, `لكل`,
  `طابق`, …) then press `<C-l>` to expand; `<Tab>`/`<S-Tab>` jump between
  placeholders. No snippet plugin required.

## Requirements

- Neovim ≥ 0.9 (uses `vim.uv`, `nvim_create_namespace`, `jobstart`, `vim.diagnostic`).
- The Shifra interpreter — a built `rustpython` binary (pointer below). The
  plugin auto-detects it, so most users don't need to configure anything.
- A terminal that displays Arabic correctly. For Ghostty, pair it with the
  bundled **Thmanyah Sans Term** (`ThmanyahSansTerm.ttf` in this repository) or
  any Arabic-capable font as your font fallback; Neovim's shaping produces the
  presentation-form glyphs that the terminal then renders.

### Getting the interpreter

Shifra is a Python 3 interpreter written in Rust:

```bash
git clone https://github.com/RustPython/RustPython   # the Shifra fork lives in this checkout
cd RustPython
cargo build --release
# the binary is now target/release/shifra
```

## Installation

### lazy.nvim

```lua
{
  "dir" = "/path/to/Shifra/checkout/editors/nvim-shifra",
  opts = {
    -- runtime = { command = "shifra" },              -- or point at a binary
    -- runtime = { project_dir = "/path/to/Shifra" }, -- or at the checkout
  },
}
```

### vim-plug

```vim
Plug "/path/to/Shifra/checkout/editors/nvim-shifra"
lua require("shifra").setup({ runtime = { project_dir = "/path/to/Shifra" } })
```

### packer.nvim

```lua
use { "/path/to/Shifra/checkout/editors/nvim-shifra", config = function()
  require("shifra").setup({ runtime = { project_dir = "/path/to/Shifra" } })
end }
```

## Configuration

`require("shifra").setup(opts)` is idempotent — you can call it with your full
config in your plugin manager, or call it again later. Defaults:

```lua
require("shifra").setup({
  runtime = {
    command = nil,     -- full path or command for the interpreter
    project_dir = nil, -- Shifra checkout root (auto-detects target/{debug,release}/rustpython)
  },
  rtl = {
    auto = true,               -- apply RTL to shifra buffers on entry
    default_mode = "nvim",     -- "nvim" | "terminal" | "ghostty"
  },
  diagnostics = {
    auto = true,
    debounce_ms = 650,
  },
  run = {
    key = "<leader>r",         -- false to disable
  },
  snippets = {
    enable = true,
    expand_key = "<C-l>",      -- false to disable the keys
  },
  on_ft = nil,                 -- function(bufnr) extra per-buffer setup
})
```

### Interpreter resolution order

1. `runtime.command`
2. `runtime.project_dir` → `target/release/shifra` → `target/debug/shifra`
3. `$SHIFRA_DIR`, then `$RUSTPYTHON_DIR`
4. Walking up the current directory looking for a Shifra `Cargo.toml`
5. `shifra` or `rustpython` on `PATH`

Call `require("shifra.runtime").refresh()` after rebuilding the interpreter if
the resolver cached an old path.

## Mappings (buffer-local, only on .sf/.ar/.شفـ)

| Key | Action |
| --- | --- |
| `<leader>r` | Run the current file in a terminal split |
| `<leader>tr` | RTL: Neovim shaping (default) |
| `<leader>tl` | RTL: terminal BiDi |
| `<leader>tg` | RTL: ghostty-native (broken upstream) |
| `<C-l>` | Expand the snippet under the cursor |
| `<Tab>` / `<S-Tab>` | Next / previous snippet placeholder |

## Terminal font tips

A proportionally-spaced terminal font breaks the Neovim grid, so use a
monospace-compatible Arabic font as the fallback. This repo ships two
monospace-tuned Arabic faces in `editors/fonts/`: `ThmanyahSansTerm.ttf`
(monospace-tuned Thmanyah Sans Medium) and `ShifraNaskhTerm.ttf` (monospace
Noto Naskh Arabic). Install to `~/.fonts`, then add the family as a fallback
in your terminal. For Ghostty:

```
font-family = "JetBrains Mono"
font-family = "Thmanyah Sans Term"
```

The plugin only controls the editor; the terminal owns font size and fallback.