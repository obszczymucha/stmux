vim.opt.laststatus = 0
vim.opt.cmdheight = 0
vim.opt.showtabline = 0
vim.opt.number = true
vim.opt.numberwidth = 3
vim.opt.fillchars = { eob = " " }
vim.opt.termguicolors = true
vim.opt.cursorline = true
vim.opt.cursorlineopt = "line,number"
vim.opt.guicursor = 'n-c:block,i:block-iCursor,r:block-rCursor,v:block-vCursor'

vim.cmd( "hi Normal guibg=#101010" )
vim.cmd( "hi CursorLine guibg=#3a2943" )

---@diagnostic disable-next-line: undefined-field
local release = vim.loop.os_uname().release
---@diagnostic disable-next-line: lowercase-global
is_wsl = release:match( "microsoft" ) and true or release:match( "WSL" ) and true or false

vim.keymap.set( "n", "q", ":wq!<CR>", { silent = true } )
vim.keymap.set( "n", "<Esc>", ":wq!<CR>", { silent = true } )
vim.keymap.set( "n", "<A-q>", ":wq!<CR>", { silent = true } )
vim.keymap.set( "n", "<A-j>", "j", { silent = true } )
vim.keymap.set( "n", "<A-k>", "k", { silent = true } )
vim.keymap.set( "n", ":", function() end, { silent = true } )
vim.keymap.set( "n", "`", function() end, { silent = true } )

local function safe_line_up()
  if vim.fn.line( '.' ) > 1 then
    vim.cmd( "silent! m .-2" )
    vim.cmd( "silent! normal! ==" )
  end
end

local function safe_line_down()
  if vim.fn.line( '.' ) < vim.fn.line( '$' ) then
    vim.cmd( "silent! m .+1" )
    vim.cmd( "silent! normal! ==" )
  end
end

if is_wsl then
  -- Alacritty doesn't want to send Ctrl+Alt, so the only way is to use AHK.
  -- AHK is sending ^[[1;5R for <C-A-j> and ^[[1;5S for <C-A-k>.
  -- These map to <C-F3> (F27) and <C-F4> (F28) respectively.
  vim.keymap.set( "n", "<F27>", safe_line_down )
  vim.keymap.set( "n", "<F28>", safe_line_up )
else
  vim.keymap.set( "n", "<C-A-j>", safe_line_down )
  vim.keymap.set( "n", "<C-A-k>", safe_line_up )
end
