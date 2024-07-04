# Nvim traveller port
This is a rust port of the nivm-traveller plugin which was written in Lua. Built using the `neo-api-rs` library for this project. 

> [!NOTE]
> This is in active development so sometimes the `neo-api-rs` crate will point to my own location but you can toggle the comment in Cargo.toml to apply the github version (https://github.com/norlock/neo-api-rs)

## Roadmap
- [ ] Find a good way how to deal with autocmds together with async

## Startup (dev settings)
- Add to plugins (e.g. lazy): 

```lua 
	{
		"norlock/nvim-traveller-rs",
		build = "./prepare.sh"
	},
```

- Add keymap: 

```lua
local nvim_traveller = require('nvim-traveller-rs')

vim.keymap.set('n', '<leader>i', nvim_traveller.open_navigation, {})
```
