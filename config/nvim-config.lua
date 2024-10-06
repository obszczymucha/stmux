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

if is_wsl then
  -- Alacritty doesn't want to send Ctrl+Alt, so the only way is to use AHK.
  -- AHK is sending ^[[1;5R for <C-A-j> and ^[[1;5S for <C-A-k>.
  -- These map to <C-F3> (F27) and <C-F4> (F28) respectively.
  vim.keymap.set( "n", "<F27>", "<cmd>m .+1<CR>==" )
  vim.keymap.set( "n", "<F28>", "<cmd>m .-2<CR>==" )
  vim.keymap.set( "i", "<F27>", "<Esc><cmd>m .+1<CR>==gi" )
  vim.keymap.set( "i", "<F28>", "<Esc><cmd>m .-2<CR>==gi" )
  vim.keymap.set( "v", "<F27>", ":m '>+1<CR>gv=gv" )
  vim.keymap.set( "v", "<F28>", ":m '<-2<CR>gv=gv" )
else
  vim.keymap.set( "n", "<C-A-j>", "<cmd>m .+1<CR>==" )
  vim.keymap.set( "n", "<C-A-k>", "<cmd>m .-2<CR>==" )
  vim.keymap.set( "i", "<C-A-j>", "<Esc><cmd>m .+1<CR>==gi" )
  vim.keymap.set( "i", "<C-A-k>", "<Esc><cmd>m .-2<CR>==gi" )
  vim.keymap.set( "v", "<C-A-j>", ":m '>+1<CR>gv=gv" )
  vim.keymap.set( "v", "<C-A-k>", ":m '<-2<CR>gv=gv" )
end
