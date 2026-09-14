" Shifra filetype detection
" .sf, .ar, and the Arabic-matthew extension .شفـ
au BufRead,BufNewFile *.sf,*.ar setfiletype shifra
au BufRead,BufNewFile * if expand('%:t') =~# '\.شفـ$' | setfiletype shifra | endif