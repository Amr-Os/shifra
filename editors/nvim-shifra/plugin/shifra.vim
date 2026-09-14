" Shifra (Arabic Python) support for Neovim.
" Loads the default configuration; call `require('shifra').setup(opt)`
" beforehand in your init for custom options (see lua/shifra/init.lua).
if exists('g:loaded_shifra')
  finish
endif
let g:loaded_shifra = 1

lua require('shifra').setup()