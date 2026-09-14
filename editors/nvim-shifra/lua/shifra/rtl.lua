-- Shifra RTL rendering module.
--
-- Rendering model (empirically determined):
--
-- * Ghostty (mainline, incl. the 1.3.0-dev tip) force-LTRs everything and has
--   no working UAX#9. Its shaper cannot join canonical Arabic text. So Neovim
--   MUST do the shaping + RTL layout; sending logical text to Ghostty yields
--   left-aligned, disconnected, unreadable output.
--
-- * Neovim 'arabicshape' produces real Arabic presentation-form glyphs
--   (U+FE70..U+FEFF), which terminals render directly. This is the reliable
--   way to get joined, readable Arabic.
--
-- * VTE terminals (GNOME Terminal): default "implicit" mode re-processes
--   Neovim output, splitting letters. VTE's "\27[8l" escape switches to
--   "explicit" mode (terminal lays chars linearly, no BiDi). It is sent
--   unconditionally because GNOME Terminal does not export $VTE_VERSION;
--   terminals that ignore the escape are unaffected.
--
-- Default mode for Shifra buffers: nvim_rtl().

local M = {}

local function bidi_mode(mode)
  -- mode: "l" (explicit, terminal does no BiDi) or "h" (implicit, terminal BiDi)
  pcall(vim.api.nvim_chan_send, 1, "\27[" .. mode)
end

-- Neovim does shaping + RTL layout; terminal is in explicit mode.
function M.nvim_rtl()
  bidi_mode("l")
  vim.opt_local.termbidi = false
  vim.opt_local.rightleft = true
  vim.opt_local.arabicshape = true
end

-- Terminal-driven (generic): keep Neovim LTR, let the terminal work.
-- Only useful in VTE-style terminals with Gehzi implicit mode + their own BiDi.
-- Ghostty shows disconnected, LTR, unreadable Arabic -- do NOT use.
function M.terminal_bidi()
  bidi_mode("h")
  vim.opt_local.rightleft = false
  vim.opt_local.arabicshape = false
  vim.opt_local.termbidi = true
end

-- Alias kept for the <leader>tg binding; same caveat as terminal_bidi().
function M.ghostty()
  M.terminal_bidi()
end

-- Restore the terminal to a normal state when done with a shifra buffer.
function M.restore()
  bidi_mode("h")
  vim.opt_local.rightleft = false
  vim.opt_local.termbidi = false
  vim.opt_local.arabicshape = false
end

-- Default mode for shifra buffers.
function M.setup()
  M.nvim_rtl()
end

return M