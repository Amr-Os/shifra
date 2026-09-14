-- Shifra (Arabic Python) support for Neovim.
--
-- A self-contained plugin (no external runtime dependencies) that makes Shifra
-- source files (.sf, .ar, .شفـ) first-class citizens:
--
--   * Filetype detection + syntax highlighting (full Shifra keyword/builtin/
--     exception/method tables, Arabic-Indic and Persian digits).
--   * RTL rendering: right-aligned code with joined Arabic via Neovim
--     'rightleft' + 'arabicshape' (the only pipeline that renders correctly on
--     terminals without shaping, e.g. Ghostty). VTE terminals are switched to
--     explicit mode so they don't re-process the output. Toggle: <leader>tr,
--     <leader>tl, <leader>tg.
--   * Live diagnostics: `shifra --check` on the buffer, debounced.
--   * Run support: <leader>r runs the file in a terminal split.
--   * Self-contained snippets: type عرف/اطبع/إذا... then <C-l>, <Tab>/<s-Tab>
--     jump through placeholders. Mirror of editors/vscode-shifra snippets.
--
-- The interpreter is resolved like the VS Code extension: custom command →
-- project dir / SHIFRA_DIR / RUSTPYTHON_DIR → upward Cargo.toml search →
-- PATH. See lua/shifra/runtime.lua.

local M = {}

local default_config = {
  runtime = {
    -- Full path or command for the Shifra interpreter, e.g. "shifra".
    -- Empty = auto-detect (recommended).
    command = nil,
    -- Shifra checkout root used for auto-detection of
    -- target/{debug,release}/rustpython. Empty = SHIFRA_DIR/RUSTPYTHON_DIR/
    -- upward Cargo.toml search.
    project_dir = nil,
  },
  rtl = {
    -- Apply RTL + Arabic shaping to every shifra buffer on entry.
    auto = true,
    -- One of "nvim", "terminal", "ghostty". "nvim" is the reliable default.
    default_mode = "nvim",
  },
  diagnostics = {
    -- Live syntax checking (rustpython --check), debounced.
    auto = true,
    debounce_ms = 650,
  },
  run = {
    -- Run the current file in a terminal split. false to disable.
    key = "<leader>r",
  },
  snippets = {
    enable = true,
    -- Expansion trigger (insert mode). false to disable the keys.
    expand_key = "<C-l>",
  },
  on_ft = nil, -- optional function(bufnr) called after the defaults apply
}

local config = vim.deepcopy(default_config)

--- Merge user options over the current config (idempotent; safe to call again).
--- @param opts table|nil
function M.setup(opts)
  config = vim.tbl_deep_extend("force", vim.deepcopy(config), opts or {})
  return M
end

--- @private
function M.get_config()
  return config
end

--- Resolve the interpreter once (cached).
--- @return string|nil
function M.executable()
  return require("shifra.runtime").interpreter()
end

--- Buffer-local setup, called by ftplugin/shifra.lua on FileType shifra.
function M.on_ft()
  local bufnr = vim.api.nvim_get_current_buf()
  local rtl = require("shifra.rtl")
  local group = vim.api.nvim_create_augroup("shifra_buffer", { clear = true })

  vim.opt_local.commentstring = "# %s"
  vim.opt_local.shiftwidth = 4
  vim.opt_local.tabstop = 4
  vim.opt_local.expandtab = true
  vim.opt_local.smartindent = true
  vim.opt_local.wrap = true
  vim.opt_local.linebreak = true
  vim.opt_local.foldmethod = "indent"
  vim.opt_local.iskeyword:append("_")

  if config.rtl.auto then
    local mode = config.rtl.default_mode
    if type(rtl[mode]) == "function" then
      rtl[mode]()
    else
      rtl.nvim_rtl()
    end
  end

  -- Re-apply RTL when re-entering the buffer; restore the terminal on leave.
  vim.api.nvim_create_autocmd("BufEnter", {
    group = group,
    buffer = bufnr,
    callback = function()
      if vim.bo[bufnr].filetype == "shifra" then
        rtl.setup()
      end
    end,
  })
  vim.api.nvim_create_autocmd({ "BufLeave", "VimLeavePre" }, {
    group = group,
    buffer = bufnr,
    callback = function()
      if vim.bo[bufnr].filetype == "shifra" then
        rtl.restore()
      end
    end,
  })

  -- Snippets + run + RTL toggles
  local snippets = require("shifra.snippets")
  vim.keymap.set("n", "<leader>tg", rtl.ghostty, { buffer = true, desc = "Shifra RTL: terminal mode (breaks on Ghostty)" })
  vim.keymap.set("n", "<leader>tl", rtl.terminal_bidi, { buffer = true, desc = "Shifra RTL: terminal bidi mode (left-aligned)" })
  vim.keymap.set("n", "<leader>tr", rtl.nvim_rtl, { buffer = true, desc = "Shifra RTL: Neovim mode (right-aligned)" })

  if config.run.key then
    vim.keymap.set("n", config.run.key, function()
      require("shifra.runtime").run()
    end, { buffer = true, desc = "Run current Shifra file" })
  end

  if config.snippets.enable then
    local key = config.snippets.expand_key
    if key then
      vim.keymap.set("i", key, function()
        snippets.expand()
      end, { buffer = true, desc = "Expand Shifra snippet" })
    end
    vim.keymap.set("i", "<Tab>", function()
      if snippets.in_snippet() then
        snippets.jump(1)
      else
        snippets.tab_indent()
      end
    end, { buffer = true, desc = "Jump to next Shifra snippet placeholder" })
    vim.keymap.set("i", "<S-Tab>", function()
      if snippets.in_snippet() then
        snippets.jump(-1)
      end
    end, { buffer = true, desc = "Jump to previous Shifra snippet placeholder" })
  end
  vim.api.nvim_create_autocmd({ "BufLeave", "BufUnload" }, {
    group = group,
    buffer = bufnr,
    callback = function()
      snippets.clear(bufnr)
    end,
  })

  if config.diagnostics.auto then
    pcall(require("shifra.diagnostics").setup, bufnr)
  end

  if type(config.on_ft) == "function" then
    config.on_ft(bufnr)
  end
end

--- Manual syntax re-check of the current buffer.
function M.check()
  require("shifra.diagnostics").setup(vim.api.nvim_get_current_buf())
end

return M