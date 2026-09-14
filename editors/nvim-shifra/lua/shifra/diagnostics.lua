-- Live syntax diagnostics for Shifra buffers using the interpreter's
-- compile-only check (`rustpython --check file`), mirroring the behavior of
-- the VS Code extension's Problems panel.
--
-- The checker runs on the CURRENT buffer content (written to a temp .sf file)
-- so errors highlight as you type, debounced to keep it cheap.

local ns = vim.api.nvim_create_namespace("shifra")
local M = {}

--- Parse `shifra --check` stderr into {line, message} (1-based) or nil.
--- @param text string
local function parse_error(text)
  local line
  for match in text:gmatch("line (%d+)") do
    line = tonumber(match)
  end
  if not line then
    return nil
  end
  local last = ""
  for l in text:gmatch("[^\r\n]+") do
    local trimmed = l:gsub("%s+$", "")
    if trimmed ~= "" then
      last = trimmed
    end
  end
  return { line = line, message = last }
end

local function set_diagnostic(bufnr, err)
  if not vim.api.nvim_buf_is_valid(bufnr) then
    return
  end
  if not err then
    vim.diagnostic.reset(ns, bufnr)
    return
  end
  local lnum = err.line - 1
  local lcount = vim.api.nvim_buf_line_count(bufnr)
  if lnum < 0 or lnum >= lcount then
    return
  end
  local col = #vim.api.nvim_buf_get_lines(bufnr, lnum, lnum + 1, false)[1] or 0
  vim.diagnostic.set(ns, bufnr, {
    {
      lnum = lnum,
      col = 0,
      end_lnum = lnum,
      end_col = col,
      severity = vim.diagnostic.severity.ERROR,
      source = "Shifra",
      message = err.message,
    },
  })
end

--- Run the checker on the current buffer content.
--- @param bufnr number
local function run_check(bufnr)
  if not vim.api.nvim_buf_is_valid(bufnr) then
    return
  end
  local interp = require("shifra.runtime").interpreter()
  if not interp then
    return
  end
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  local tmp = vim.fn.tempname() .. ".sf"
  vim.fn.writefile(lines, tmp)
  local stderr = {}
  local stdout = {}
  vim.fn.jobstart({ interp, "--check", tmp }, {
    stdout_buffered = true,
    stderr_buffered = true,
    on_stdout = function(_, data)
      if data then
        vim.list_extend(stdout, data)
      end
    end,
    on_stderr = function(_, data)
      if data then
        vim.list_extend(stderr, data)
      end
    end,
    on_exit = function()
      if not vim.api.nvim_buf_is_valid(bufnr) then
        return
      end
      local err = parse_error(table.concat(stderr, "\n") .. "\n" .. table.concat(stdout, "\n"))
      set_diagnostic(bufnr, err)
    end,
  })
end

--- Set up debounced live diagnostics for a Shifra buffer.
--- @param bufnr number|nil
function M.setup(bufnr)
  bufnr = bufnr or vim.api.nvim_get_current_buf()
  if vim.b[bufnr].shifra_diag_active then
    return
  end
  vim.b[bufnr].shifra_diag_active = true

  local debounce_ms = require("shifra").get_config().diagnostics.debounce_ms
  local timer = vim.uv.new_timer()
  local function schedule()
    if not vim.api.nvim_buf_is_valid(bufnr) then
      return
    end
    timer:stop()
    timer:start(debounce_ms, 0, function()
      vim.schedule(function()
        run_check(bufnr)
      end)
    end)
  end

  local group = vim.api.nvim_create_augroup("shifra_diagnostics_" .. bufnr, { clear = true })
  vim.api.nvim_create_autocmd({ "TextChanged", "TextChangedI", "InsertLeave", "BufWritePost" }, {
    buffer = bufnr,
    group = group,
    callback = schedule,
  })
  vim.api.nvim_create_autocmd({ "BufUnload" }, {
    buffer = bufnr,
    group = group,
    callback = function()
      timer:stop()
      timer:close()
    end,
  })

  schedule()
end

return M