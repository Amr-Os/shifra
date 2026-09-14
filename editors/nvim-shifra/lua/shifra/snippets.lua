-- Shifra snippets: a tiny, self-contained tab-stop snippet expander.
--
-- VS Code-style behavior without a plugin:
--   * In insert mode, type a trigger (e.g. عرف, إذا, اطبع) then press <C-l>
--     to expand it.
--   * <Tab> jumps to the next placeholder, <s-Tab> to the previous one
--     (placeholders only exist after an expansion, so <Tab> still indents
--     normally everywhere else).
--
-- The snippet bodies and tab stops mirror editors/vscode-shifra's
-- snippets/shifra.code-snippets (upstream).

local M = {}

--- VS Code snippet definitions: prefix -> {body = string[], desc = string}
--- Kept in sync with editors/vscode-shifra/snippets/shifra.code-snippets (upstream).
local SNIPPETS = {
  إذا = { body = { "إذا ${1:شرط}:", "    ${0}" }, desc = "Arabic conditional — if" },
  وإذا = { body = { "وإذا ${1:شرط}:", "    ${0}" }, desc = "else if" },
  وإلا = { body = { "وإلا:", "    ${0}" }, desc = "else" },
  إذاوإلا = { body = { "${1:نتيجة_نعم} إذا ${2:شرط} وإلا ${3:نتيجة_لا}" }, desc = "conditional expression (ternary)" },
  عرف = { body = { "عرف ${1:اسم}(${2:وسائط}):", "    ${0}" }, desc = "Arabic function — def" },
  أعد = { body = { "أعد ${1:قيمة}" }, desc = "return a value" },
  صنف = { body = { "صنف ${1:اسم}${2:(${3:أصل})}:", "    ${0}" }, desc = "Arabic class — class" },
  لامبدا = { body = { "لامبدا ${1:وسائط}: ${2:تعبير}" }, desc = "Arabic lambda — anonymous function" },
  لكل = { body = { "لكل ${1:عنصر} ضمن ${2:نطاق}:", "    ${0}" }, desc = "Arabic for loop" },
  ما_دام = { body = { "ما_دام ${1:شرط}:", "    ${0}" }, desc = "Arabic while loop" },
  استورد = { body = { "استورد ${1:وحدة}" }, desc = "import a module" },
  مناستورد = { body = { "من ${1:وحدة} استورد ${2:اسم}" }, desc = "from ... import ..." },
  طابق = { body = { "طابق ${1:قيمة}:", "    حالة ${2:نمط}:", "        ${0}" }, desc = "Arabic match — pattern matching" },
  حالة = { body = { "حالة ${1:نمط}:", "    ${0}" }, desc = "case branch (inside طابق)" },
  مع = { body = { "مع ${1:مصدر} باسم ${2:اسم}:", "    ${0}" }, desc = "Arabic with — context manager" },
  جرب = { body = { "جرب:", "    ${1}", "التقط ${2:خطأ_قيمة} باسم ${3:خطأ}:", "    ${0}" }, desc = "Arabic exception handling" },
  ختاما = { body = { "ختاما:", "    ${0}" }, desc = "Arabic finally — cleanup block" },
  ارم = { body = { "ارم ${1:خطأ_قيمة}(${2:رسالة})" }, desc = "raise an exception" },
  اطبع = { body = { "اطبع(${1:قيمة})" }, desc = "print to stdout" },
  طول = { body = { "طول(${1:قيمة})" }, desc = "len() — number of items" },
  نطاق = { body = { "نطاق(${1:بداية}, ${2:نهاية}, ${3:خطوة})" }, desc = "range() — numeric sequence" },
}

-- placeholder bookkeeping per buffer: { {n, lnum, col, len}, ... }
-- lnum 0-based, col 0-based byte offset, len in bytes.
local placeholders = {}
local PLR = {} -- current position within placeholders[bufnr]
local PSR = {} -- buffer is mid-snippet

local function buf_clear(bufnr)
  placeholders[bufnr] = nil
  PLR[bufnr] = nil
  PSR[bufnr] = nil
end

--- Text before the cursor on the current line.
--- @return string, number lnum (0-based), number col (0-based byte col)
local function text_before_cursor()
  local lnum = vim.fn.line(".") - 1
  local col = vim.fn.col(".") - 1 -- 0-based byte column
  return string.sub(vim.fn.getline("."), 1, col), lnum, col
end

--- Parse ${n} and ${n:default} tokens on one line.
--- Returns new_text and a list of {n, offset_in_bytes, len_in_bytes, def}.
local function render_line(prefix_off, line)
  local out = {}
  local tabs = {}
  local pos = 1
  while pos <= #line do
    local open = line:find("${", pos, true)
    if not open then
      out[#out + 1] = line:sub(pos)
      break
    end
    if open > pos then
      out[#out + 1] = line:sub(pos, open - 1)
    end
    local close = line:find("}", open, true)
    local inner = line:sub(open + 2, (close or open + 2) - 1)
    local n, def = inner:match("^(%d+):?([^}]*)")
    if n then
      n = tonumber(n)
      if def == "" then
        out[#out + 1] = ""
        tabs[#tabs + 1] = { n = n, offset = #table.concat(out) - 0, len = 0, def = "" }
      else
        out[#out + 1] = def
        tabs[#tabs + 1] = { n = n, offset = #table.concat(out) - #def, len = #def, def = def }
      end
    else
      out[#out + 1] = line:sub(open, close or open)
    end
    pos = (close or open) + 1
  end
  local text = table.concat(out)
  for _, t_ in ipairs(tabs) do
    t_.offset = t_.offset + prefix_off
  end
  return text, tabs
end

--- Is the character ending at byte pos-1 (1-based) a keyword char?
--- Arabic identifiers are multibyte, so skip continuation bytes back to the
--- leading byte before classifying.
local function keyword_before(before, pos)
  local i = pos - 1
  if i < 1 then
    return false
  end
  local b = string.byte(before, i)
  while b and b >= 0x80 and b <= 0xBF and i > 1 do
    i = i - 1
    b = string.byte(before, i)
  end
  if not b then
    return false
  end
  return b == 95 or (b >= 48 and b <= 57) or (b >= 65 and b <= 90) or (b >= 97 and b <= 122) or b >= 0xC0
end

--- Expand the snippet under the cursor. Called from insert mode.
--- @return boolean whether a snippet was expanded
function M.expand()
  if PSR[vim.api.nvim_get_current_buf()] then
    return false
  end
  local before, lnum, col = text_before_cursor()
  if before == "" then
    return false
  end

  -- find the longest matching trigger at a word boundary
  local trig, data
  for prefix, d in pairs(SNIPPETS) do
    local pos = #before - #prefix + 1
    if pos >= 1 and string.sub(before, pos) == prefix then
      local boundary = pos == 1 or not keyword_before(before, pos)
      if boundary and (not trig or #prefix > #trig) then
        trig = prefix
        data = d
      end
    end
  end
  if not trig then
    return false
  end

  -- indentation of the current line (bytes before the trigger)
  local line_text = vim.fn.getline(".")
  local indent_off = line_text:find("[^ \t]") or (#line_text + 1)
  local indent = string.sub(line_text, 1, indent_off - 1)
  local trig_start = col - #trig

  local bufnr = vim.api.nvim_get_current_buf()
  buf_clear(bufnr)

  -- render body, collecting placeholders
  local tabs = {}
  local new_lines = {}
  local lnum0 = lnum
  for i, bl in ipairs(data.body) do
    local prefix_off = 0
    local btext
    if i == 1 then
      btext, tabs = render_line(0, bl)
    else
      prefix_off = #indent
      btext, tabs = render_line(prefix_off, bl)
    end
    local final_line = (i == 1) and btext or (indent .. btext)
    new_lines[#new_lines + 1] = final_line
    for _, t_ in ipairs(tabs) do
      placeholders[bufnr] = placeholders[bufnr] or {}
      local pc = t_.offset
      local plnum = lnum0 + (i - 1)
      local pcol = (i == 1) and (trig_start + pc) or pc
      table.insert(placeholders[bufnr], { n = t_.n, lnum = plnum, col = pcol, len = t_.len })
    end
  end

  -- replace the trigger with the first body line, insert extra lines after
  vim.api.nvim_buf_set_text(bufnr, lnum0, trig_start, lnum0, col, { new_lines[1] })
  if #new_lines > 1 then
    local tail = { "" }
    for i = 2, #new_lines do
      tail[#tail + 1] = new_lines[i]
    end
    vim.api.nvim_buf_set_text(
      bufnr,
      lnum0,
      trig_start + #new_lines[1],
      lnum0,
      trig_start + #new_lines[1],
      tail
    )
  end

  -- move to the first placeholder (lowest n > 0; n==0 is the final exit)
  local function sort_key(p)
    return p.n == 0 and math.huge or p.n
  end
  local order = vim.deepcopy(placeholders[bufnr] or {})
  table.sort(order, function(a, b)
    return sort_key(a) < sort_key(b)
  end)
  if order[1] and order[1].n > 0 then
    PLR[bufnr] = 1
    PSR[bufnr] = true
    local target = order[1]
    vim.api.nvim_win_set_cursor(0, { target.lnum + 1, target.col + target.len })
  end
  return true
end

--- Jump to the next/previous placeholder. Returns true if it moved.
--- @param dir 1 or -1
function M.jump(dir)
  local bufnr = vim.api.nvim_get_current_buf()
  local pls = placeholders[bufnr]
  if not pls or not PSR[bufnr] then
    return false
  end
  local order = vim.deepcopy(pls)
  table.sort(order, function(a, b)
    local sa = a.n == 0 and math.huge or a.n
    local sb = b.n == 0 and math.huge or b.n
    return sa < sb
  end)
  local cur = PLR[bufnr] or 1
  cur = cur + (dir or 1)
  if cur < 1 then
    cur = 1
  end
  local target = order[cur]
  if not target then
    -- done: placeholder cycle complete
    PSR[bufnr] = false
    buf_clear(bufnr)
    return true
  end
  PLR[bufnr] = cur
  vim.api.nvim_win_set_cursor(0, { target.lnum + 1, target.col + target.len })
  return true
end

--- Whether the current buffer is mid-snippet (used to guard <Tab>).
function M.in_snippet()
  local bufnr = vim.api.nvim_get_current_buf()
  return PSR[bufnr] == true
end

--- Clear state for a buffer (call on BufLeave/BufDelete).
function M.clear(bufnr)
  buf_clear(bufnr or vim.api.nvim_get_current_buf())
end

--- <Tab> fallback: not inside a snippet, so mimic default indent behavior
--- (expandtab → spaces up to the next shiftwidth boundary). Works from
--- insert mode without remapping.
function M.tab_indent()
  local row = vim.fn.line(".") - 1
  local col = vim.fn.col(".") - 1
  local sw = vim.o.shiftwidth
  if sw < 1 then
    sw = vim.o.tabstop
  end
  local n = sw - (col % sw)
  if n < 1 then
    n = sw
  end
  local spaces = string.rep(" ", n)
  vim.api.nvim_buf_set_text(0, row, col, row, col, { spaces })
  vim.fn.cursor(row + 1, col + n + 1)
end

M.SNIPPETS = SNIPPETS

return M