-- Interpreter resolution + run / check helpers.
--
-- Resolution mirrors editors/vscode-shifra's run configuration, in order:
--   1. setup({ runtime = { command = "..." } }) -- explicit
--   2. setup({ runtime = { project_dir = "..." } }) -- checkout root
--   3. $SHIFRA_DIR, $RUSTPYTHON_DIR
--   4. walk up from the current working directory for a Shifra Cargo.toml
--   5. `shifra` / `rustpython` on PATH
--
-- The candidate binaries are <root>/target/{release,debug}/rustpython.

local M = {}

local cached = false
local resolved = nil

local RELEASE = "target/release/shifra"
local DEBUG = "target/debug/shifra"

local function find_in_root(root)
  for _, suffix in ipairs({ RELEASE, DEBUG }) do
    local cand = root .. "/" .. suffix
    if vim.fn.filereadable(cand) == 1 and vim.fn.executable(cand) == 1 then
      return cand
    end
  end
end

local function search_upward()
  local dir = vim.fn.getcwd()
  while dir and dir ~= "/" and dir ~= "" do
    local cargo = dir .. "/Cargo.toml"
    if vim.fn.filereadable(cargo) == 1 then
      for _, line in ipairs(vim.fn.readfile(cargo)) do
        if line:match("^%s*name%s*=%s*%[\"']?shifra%[\"']?%s*$") then
          return dir
        end
      end
    end
    dir = vim.fn.fnamemodify(dir, ":h")
  end
end

--- Resolve the interpreter path/command, caching the result.
--- @return string|nil
function M.interpreter()
  if cached then
    return resolved
  end
  local config = require("shifra").get_config()
  local cmd = config.runtime.command
  if cmd and cmd ~= "" then
    resolved = cmd
  else
    local roots = {}
    if config.runtime.project_dir and config.runtime.project_dir ~= "" then
      roots[#roots + 1] = config.runtime.project_dir
    end
    for _, env in ipairs({ "SHIFRA_DIR", "RUSTPYTHON_DIR" }) do
      local v = vim.env[env]
      if v and v ~= "" then
        roots[#roots + 1] = v
      end
    end
    roots[#roots + 1] = search_upward()
    for _, root in ipairs(roots) do
      local found = root and find_in_root(root)
      if found then
        resolved = found
        break
      end
    end
    if not resolved then
      for _, name in ipairs({ "shifra", "rustpython" }) do
        if vim.fn.executable(name) == 1 then
          resolved = name
          break
        end
      end
    end
  end
  cached = true
  return resolved
end

--- Forget the cached interpreter (e.g. after a fresh RustPython build).
function M.refresh()
  cached = false
  resolved = nil
end

--- Run the current Shifra buffer in a bottom terminal split.
--- Arabic output is shaped (joined) while staying LTR so it is readable.
function M.run()
  local interp = M.interpreter()
  if not interp then
    vim.notify("Shifra: interpreter not found. Set shifra.runtime.command or project_dir.", vim.log.levels.WARN)
    return
  end
  local file = vim.fn.expand("%:p")
  if file == "" then
    vim.notify("Shifra: no file to run", vim.log.levels.WARN)
    return
  end
  local cmd = vim.fn.shellescape(interp) .. " " .. vim.fn.shellescape(file)
  vim.cmd("terminal " .. cmd)
  vim.opt_local.rightleft = true
  vim.opt_local.arabicshape = true
  vim.opt_local.termbidi = false
end

return M